use {
    crate::{
        db, pandoc,
        parser::{parser, UnitRefSearch, TAG_ID_ESCAPED_RE},
    },
    anyhow::{Context as AnyhowContext, Result},
    html5ever::{local_name, namespace_url, ns, QualName},
    kuchiki,
    kuchiki::{iter::NodeIterator, traits::TendrilSink, Attribute, ExpandedName, NodeRef},
    log, rusqlite as sql,
    std::{
        cell::RefCell,
        fs,
        io::{Cursor, Read, Seek, Write},
        path,
    },
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error parsing html during linkification: {0}")]
    ParsingHtml(String),
}

/// `file_via_pandoc(&conn, &path, true)` will linkify the content of the file
/// at `path` as per [linkify_spec_html].
///
/// `gfm` determines wheter we are targeting GitHub Flavored markdown
/// compability and inherent the resulting limitations
pub fn file_via_pandoc(conn: &sql::Connection, path: &path::Path, gfm: bool) -> Result<()> {
    log::debug!("linkifying file {:?}", path);
    if gfm {
        log::debug!("adapting output for github flavored markdown");
    }
    let html = pandoc::parse_file(path)?;
    let new_html = linkify_spec_string(Some(conn), &html, gfm)
        .with_context(|| format!("linkifying file {}", path.display()))?;
    let pandoc_md = pandoc::html_to_markdown(&new_html)?;

    // Adjustments to the pandoc generated markdown
    let adjusted = {
        if gfm {
            gfm_anchorify(&pandoc_md)
        } else {
            pandoc_md.replace("[\\|", "[|").replace("\\|]", "|]")
        }
    };

    {
        let mut f = fs::File::create(&path)?;
        let _ = f.write_all(adjusted.as_bytes())?;
    }
    Ok(())
}

fn gfm_anchorify(md: &str) -> String {
    TAG_ID_ESCAPED_RE
        .replace_all(&md, r#"<span id="$tag">|$tag|</span>"#)
        .to_string()
}

/// As [linkify_spec_html], but with a `String` as input and output.
pub fn linkify_spec_string(
    conn: Option<&sql::Connection>,
    html: &str,
    gfm: bool,
) -> Result<String> {
    let mut buff = Cursor::new(Vec::new());
    linkify_spec_html(conn, &mut html.as_bytes(), &mut buff, gfm)?;
    let res = buff.into_inner();
    let new_html = std::str::from_utf8(&res)?;
    Ok(new_html.into())
}

/// As [linkify_spec_html], but reading from and writing to the file at the
/// given `path`.
pub fn linkify_spec_file(conn: Option<&sql::Connection>, path: &path::Path) -> Result<()> {
    let mut buff = Cursor::new(Vec::new());
    {
        let mut file_in = fs::File::open(path)?;
        linkify_spec_html(conn, &mut file_in, &mut buff, false)?;
    }
    {
        let mut file_out = fs::File::create(path)?;
        file_out.write_all(&buff.into_inner())?;
    };
    Ok(())
}

/// `linkify_spec_html(reader, writer)` transfers the seralized html data from
/// `reader` into `writer`, transforming logical unit tag definitions and
/// references as follows:
///
/// - Tag definitions `<dt>|FOO.1::BAR.1|</dt>` are transformed into
///   `<dt>|FOO.1::BAR.1|<a id="FOO.1::BAR.1"></a></dt>`
/// - Tag references `[FOO.1::BAR.1]` are transformed into
///   `<a href="path/to/file#FOO.1::BAR.1">FOO.1::BAR.1</a>`
// TODO Add conn so we can lookup logical unit references
// TODO Use lazy static for css selector generation
pub fn linkify_spec_html<R: Read, W: Write + Seek>(
    conn: Option<&sql::Connection>,
    reader: &mut R,
    writer: &mut W,
    gfm: bool,
) -> Result<()> {
    let doc = kuchiki::parse_html().from_utf8().read_from(reader)?;

    if !gfm {
        // Anchorification
        let terms = doc
            .select("dt")
            .map_err(|()| Error::ParsingHtml("selecting dt elements".into()))?;
        for term in terms {
            anchorify_tag_def_term(term)?;
        }
    }

    // Linkification
    let texts = doc.descendants().text_nodes();

    // All the nodes we'll need to remove after linkifying their content
    // We have to do this after we've processed all the descendents, because
    // if detach each node right after linkifying it, we lose the pointer
    // to the next node in the tree.
    let mut nodes_to_detatch: Vec<kuchiki::NodeDataRef<RefCell<String>>> = Vec::new();
    for text in texts {
        if linkify_tag_refs(conn, &text)? {
            nodes_to_detatch.push(text);
        }
    }

    // Remove all those nodes which contained unit refs, since we've now
    // duplicated them
    for node in nodes_to_detatch {
        node.as_node().detach()
    }

    doc.serialize(writer)?;
    writer.flush()?;
    Ok(())
}

// TODO Switch to using a populated in-memmory sqlite db for testing?
// Use without a `conn` is only intended for unit testing purposes
// Returns `Ok(true)` if the given text node contains a logical unit reference,
// in which case it will linkify the reference, and duplicate all the contents
// of the node, inserting it ahead of the original node.
// NOTE: We need to clean up these nodes separately. See `nodes_to_detach` above.
fn linkify_tag_refs(
    conn: Option<&sql::Connection>,
    node: &kuchiki::NodeDataRef<RefCell<String>>,
) -> Result<bool> {
    let mut unit_ref_found = false;
    if let Some(text_node) = node.as_node().as_text() {
        let text = &text_node.borrow();
        // We want to extract unlinked logical unit references from a chunk of
        // text. To do that we parse the chunk of text into an array of items
        // that differentiate plain text from unit references.
        match parser::find_logical_unit_refs(text)? {
            None => (), // Chunk contains no unit refs, so leave it unchanged
            Some(parts) => {
                for part in parts.iter() {
                    let new_node = match part {
                        // Chunks of plain text are added back as new text nodes
                        UnitRefSearch::Text(t) => NodeRef::new_text(t),
                        // Tag refs are converted into link element nodes
                        UnitRefSearch::Ref(tag) => {
                            unit_ref_found = true;
                            let url = match conn {
                                None => tag_to_id_ref(tag), // For unit testing
                                Some(c) => db::unit::get_path(c, &tag)?,
                            };
                            new_link(url, tag.into())
                        }
                    };
                    // Each piece of the text chunk is inserted before the
                    // original text node. As we do this repeatedly, it stacks
                    // the new nodes on top of the original text node.
                    node.as_node().insert_before(new_node);
                }
            }
        }
    };
    Ok(unit_ref_found)
}

fn tag_to_id_ref(tag: &str) -> String {
    let mut id_ref = "#".to_string();
    id_ref.push_str(&tag);
    id_ref
}

// Wrap a unit tag def term in an anchor, if it is "naked" (see [as_naked_unit_tag])
fn anchorify_tag_def_term(term: kuchiki::NodeDataRef<kuchiki::ElementData>) -> Result<()> {
    if let Some(tag) = as_naked_unit_tag(term.as_node()) {
        // Create a new span element to anchor the tag
        let anchor = new_anchor_span(tag);
        // Retreive the child of the dt element
        let child = term
            .as_node()
            .first_child()
            .ok_or_else(|| Error::ParsingHtml("could not get child of dt element".into()))?;
        // Move the child into the anchor
        anchor.append(child);
        // Put the anchor into the dt element
        term.as_node().append(anchor);
    };
    Ok(())
}

// A unit tag is "naked" if it is not wrapped in an anchoring element
// This returns just the textual tag that is thus exposed.
fn as_naked_unit_tag(node: &kuchiki::Node) -> Option<String> {
    match node.first_child() {
        None => None,
        Some(child) => {
            if is_anchor_element(&child) {
                None
            } else {
                match child.as_text() {
                    None => as_naked_unit_tag(&child),
                    Some(t) => {
                        let text = t.borrow();
                        parser::logical_unit_definiendum(&text).ok()
                    }
                }
            }
        }
    }
}

fn is_anchor_element(node: &kuchiki::Node) -> bool {
    match node.as_element() {
        None => false,
        Some(el) => {
            let attrs = el.attributes.borrow();
            attrs.contains("id")
        }
    }
}

// Create a <span id=`tag`> element
fn new_anchor_span(tag: String) -> kuchiki::NodeRef {
    kuchiki::NodeRef::new_element(
        QualName::new(None, ns!(html), local_name!("span")),
        Some((
            ExpandedName::new("", "id"),
            Attribute {
                prefix: None,
                // Give it the tag's ID as its id attribute
                value: tag,
            },
        )),
    )
}

// Create an <a href=`url`> element
fn new_link(url: String, tag: String) -> kuchiki::NodeRef {
    let link_element = kuchiki::NodeRef::new_element(
        QualName::new(None, ns!(html), local_name!("a")),
        Some((
            ExpandedName::new("", "href"),
            Attribute {
                prefix: None,
                // Give it the tag's ID as its id attribute
                value: url,
            },
        )),
    );
    link_element.append(NodeRef::new_text(tag));
    link_element
}

#[cfg(test)]
mod test {
    use super::*;

    fn wrap_as_doc(html: &str) -> String {
        let mut wrapped = "<html><head></head><body>".to_owned();
        wrapped.push_str(&html);
        wrapped.push_str("</body></html>");
        wrapped
    }

    fn assert_html_transformation(input: &str, expected_output: &str) {
        let input = wrap_as_doc(input);
        let expected = wrap_as_doc(expected_output);
        let actual = linkify_spec_string(None, &input.to_string(), false).unwrap();
        println!("Expected:\n{}", expected);
        println!("Actual:\n{}", actual);
        assert_eq!(actual, expected);
    }

    #[test]
    fn can_add_anchors_to_spec() {
        let html = r#"
<dl>
  <dt>|FOO.1::BAR.1|</dt>
  <dd>Bam bip blop.</dd>
</dl>
"#;
        let expected = r#"
<dl>
  <dt><span id="FOO.1::BAR.1">|FOO.1::BAR.1|</span></dt>
  <dd>Bam bip blop.</dd>
</dl>
"#;
        assert_html_transformation(html, expected)
    }

    #[test]
    fn can_add_anchors_to_wrapped_elements() {
        let html = r#"
<dl>
  <dt><em>|FOO.1::BIZ.1|</em></dt>
  <dd>Bam bip blop.</dd>
</dl>
"#;
        let expected = r#"
<dl>
  <dt><span id="FOO.1::BIZ.1"><em>|FOO.1::BIZ.1|</em></span></dt>
  <dd>Bam bip blop.</dd>
</dl>
"#;
        assert_html_transformation(html, expected)
    }

    #[test]
    fn does_not_add_redundant_anchors() {
        let html = r#"
<dl>
  <dt><a id="FOO.1::BAZ.1">|FOO.1::BAZ.1|</a></dt>
  <dd>Bam bip blop.</dd>
  <dt><span id="FOO.1::BAZ.1">|FOO.1::BAZ.1|</span></dt>
  <dd>Bam bip blop.</dd>
  <dt><em id="FOO.1::BOZ.1">|FOO.1::BOZ.1|</em></dt>
  <dd>Bam bip blop.</dd>
</dl>
"#;
        // Since all the <dt>'s are already anchored, the
        // transformation should be a no-op.
        assert_html_transformation(html, html)
    }

    #[test]
    fn can_linkify_refs_in_paragraph() {
        let html = r#"
<p>
Here is some text, and here is a ref [FOO.1::BAR.1] and here is more
text and then another ref [FOO.1].
</p>
"#;
        let expected = r##"
<p>
Here is some text, and here is a ref <a href="#FOO.1::BAR.1">FOO.1::BAR.1</a> and here is more
text and then another ref <a href="#FOO.1">FOO.1</a>.
</p>
"##;
        assert_html_transformation(html, expected)
    }

    // Adresses a bug where deletion of the current text node was aborting
    // iteration through thre reamining text nodes.
    #[test]
    fn can_linkify_refs_accross_multiple_paragraphs() {
        let html = r#"
<p>
Here is some text, and here is a ref [FOO.1::BAR.1] and here is more
text and then another ref [FOO.1].
</p>

<p>
Here is some text, and here is a ref [FOO.1::BIR.1] and here is more
text and then another ref [FOP.1].
</p>
"#;
        let expected = r##"
<p>
Here is some text, and here is a ref <a href="#FOO.1::BAR.1">FOO.1::BAR.1</a> and here is more
text and then another ref <a href="#FOO.1">FOO.1</a>.
</p>

<p>
Here is some text, and here is a ref <a href="#FOO.1::BIR.1">FOO.1::BIR.1</a> and here is more
text and then another ref <a href="#FOP.1">FOP.1</a>.
</p>
"##;
        assert_html_transformation(html, expected)
    }

    #[test]
    fn can_linkify_refs_in_dd_elements() {
        let html = r#"
<dl>
<dt>|FOO.1::BAR.1|</dt>
<dd>
Here is some text, and here is a ref [FOO.1::BAR.1] and here is more
text and then another ref [FOO.1].
</dd>
</dl>
"#;
        let expected = r##"
<dl>
<dt><span id="FOO.1::BAR.1">|FOO.1::BAR.1|</span></dt>
<dd>
Here is some text, and here is a ref <a href="#FOO.1::BAR.1">FOO.1::BAR.1</a> and here is more
text and then another ref <a href="#FOO.1">FOO.1</a>.
</dd>
</dl>
"##;
        assert_html_transformation(html, expected)
    }

    #[test]
    fn gfm_md_compatible_anchoring() {
        // This is testing the ad-hoc post-processing we do on the pandoc
        // generated markdown (that's where the escaped `|`s come from)
        let actual = gfm_anchorify(
            r#"
\|FOO.1\|
: Some stuff

\|FOO.1::BAR.1\|
: Some other stuff
"#,
        );

        let expected = r#"
<span id="FOO.1">|FOO.1|</span>
: Some stuff

<span id="FOO.1::BAR.1">|FOO.1::BAR.1|</span>
: Some other stuff
"#
        .to_string();

        assert_eq!(actual, expected)
    }
}

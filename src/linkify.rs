use {
    crate::{
        db, pandoc,
        parser::{parser, UnitRefSearch},
    },
    anyhow::{Context as AnyhowContext, Result},
    html5ever::{local_name, namespace_url, ns, QualName},
    // lol_html::{element, rewrite_str, RewriteStrSettings},
    kuchiki,
    kuchiki::{iter::NodeIterator, traits::TendrilSink, Attribute, ExpandedName, NodeRef},
    rusqlite as sql,
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

pub fn file_via_pandoc(conn: &sql::Connection, path: &path::Path) -> Result<()> {
    let html = pandoc::parse_file(path)?;
    let new_html = linkify_spec_string(Some(conn), &html)
        .with_context(|| format!("linkifying file {}", path.display()))?;
    pandoc::write_file(&new_html, path)?;

    // FIXME
    // We unescape the pipes for aesthetic reasons.
    // This won't be needed once we're using pulldown-cmark.
    let escaped = {
        let mut f = fs::File::open(&path)?;
        let mut data = String::new();
        f.read_to_string(&mut data)?;
        data
    };

    let unescaped = escaped.replace("[\\|", "[|").replace("\\|]", "|]");

    {
        let mut f = fs::File::create(&path)?;
        let _ = f.write_all(unescaped.as_bytes())?;
    }
    Ok(())
}

/// As [linkify_spec_html], but with a `String` as input and output.
pub fn linkify_spec_string(conn: Option<&sql::Connection>, html: &str) -> Result<String> {
    let mut buff = Cursor::new(Vec::new());
    linkify_spec_html(conn, &mut html.as_bytes(), &mut buff)?;
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
        linkify_spec_html(conn, &mut file_in, &mut buff)?;
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
pub fn linkify_spec_html<R: Read, W: Write + Seek>(
    conn: Option<&sql::Connection>,
    reader: &mut R,
    writer: &mut W,
) -> Result<()> {
    let doc = kuchiki::parse_html().from_utf8().read_from(reader)?;

    // Anchorification
    let terms = doc
        .select("dt")
        .map_err(|()| Error::ParsingHtml("selecting dt elements".into()))?;
    for term in terms {
        anchorify_tag_def_term(term)?;
    }

    // Linkification
    let texts = doc.descendants().text_nodes();
    for text in texts {
        linkify_tag_refs(conn, text)?;
    }

    doc.serialize(writer)?;
    writer.flush()?;
    Ok(())
}

// TODO Switch to using a populated in-memmory sqlite db for testing?
// Use without a `conn` is only intended for unit testing purposes
fn linkify_tag_refs(
    conn: Option<&sql::Connection>,
    node: kuchiki::NodeDataRef<RefCell<String>>,
) -> Result<()> {
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
                // After all the new nodes have been stacked on top of the
                // original one, we remove the original.
                node.as_node().detach()
            }
        }
    };
    Ok(())
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
        let actual = linkify_spec_string(None, &input.to_string()).unwrap();
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
}

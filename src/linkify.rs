use {
    crate::parser::parser,
    anyhow::Result,
    html5ever::{local_name, namespace_url, ns, QualName},
    // lol_html::{element, rewrite_str, RewriteStrSettings},
    kuchiki,
    kuchiki::{iter::NodeIterator, traits::TendrilSink, Attribute, ExpandedName},
    std::{
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

/// As [linkify_spec_html], but with a `String` as input and output.
pub fn linkify_spec_string(html: &String) -> Result<String> {
    let mut buff = Cursor::new(Vec::new());
    linkify_spec_html(&mut html.as_bytes(), &mut buff)?;
    let res = buff.into_inner();
    let new_html = std::str::from_utf8(&res)?;
    Ok(new_html.into())
}

/// As [linkify_spec_html], but reading from and writing to the file at the
/// given `path`.
pub fn linkify_spec_file(path: &path::Path) -> Result<()> {
    let mut buff = Cursor::new(Vec::new());
    {
        let mut file_in = fs::File::open(path)?;
        linkify_spec_html(&mut file_in, &mut buff)?;
    }
    let res = {
        let mut file_out = fs::File::create(path)?;
        file_out.write_all(&mut buff.into_inner())?;
    };
    Ok(res)
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
pub fn linkify_spec_html<R: Read, W: Write + Seek>(reader: &mut R, writer: &mut W) -> Result<()> {
    let doc = kuchiki::parse_html().from_utf8().read_from(reader)?;
    let terms = doc
        .select("dt")
        .map_err(|()| Error::ParsingHtml("selecting dt elements".into()))?;
    for term in terms {
        anchorify_dt(term)?;
    }
    let res = doc.serialize(writer)?;
    writer.flush()?;
    Ok(res)
}

fn anchorify_dt(term: kuchiki::NodeDataRef<kuchiki::ElementData>) -> Result<()> {
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
                        parser::logical_unit_definiendum(&text)
                            .ok()
                            .map(|tag| tag.trim_matches('|').into())
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

// Create a a <span> element with the id attribute set to `tag`
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
        let actual = linkify_spec_string(&input.to_string()).unwrap();
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
    fn scratch() {
        let html = r#"<html><head></head><body>
<dl>
  <dt>|FOO::1.BAR::1|</dt>
  <dd>Bam bip blop.</dd>
  <dt><a id="FOO::1.BAZ::1">|FOO::1.BAZ::1|</a></dt>
  <dd>Bam bip <a href="foo">link</a> blop.</dd>
</dl>

</body></html>"#
            .to_string();

        let expected = r#"<html><head></head><body>
<dl>
  <dt>|FOO::1.BAR::1|<a id="FOO::1.BAR::1"></a></dt>
  <dd>Bam bip blop.</dd>
  <dt><a id="FOO::1.BAZ::1">|FOO::1.BAZ::1|</a></dt>
  <dd>Bam bip blop.</dd>
</dl>

</body></html>"#
            .to_string();

        let doc = kuchiki::parse_html().one(html).descendants();
        let nodes = doc.text_nodes();
        for node in nodes {
            if let Some(text_ref) = node.as_node().as_text() {
                let t = text_ref.borrow();
                println!("{}", t);
            }
        }

        assert_eq!(expected, "".to_string());
    }
}

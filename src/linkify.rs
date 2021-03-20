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
    #[error("Error parsing html during linkification")]
    ParsingHtml,
}

fn as_naked_unit_tag(node: &kuchiki::Node) -> Option<String> {
    match node.first_child() {
        None => None,
        Some(child) => {
            if let Some(el) = node.as_element() {
                if let local_name!("a") = el.name.local {
                    // Bail if the node is already anchored
                    return None;
                };
            };
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
    let terms = doc.select("dt").map_err(|()| Error::ParsingHtml)?;
    for term in terms {
        if let Some(tag) = as_naked_unit_tag(term.as_node()) {
            let anchor = kuchiki::NodeRef::new_element(
                QualName::new(None, ns!(html), local_name!("a")),
                Some((
                    ExpandedName::new("", "id"),
                    Attribute {
                        prefix: None,
                        value: tag,
                    },
                )),
            );
            term.as_node().append(anchor)
        }
    }
    let res = doc.serialize(writer)?;
    writer.flush()?;
    Ok(res)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_anchor_spec_html() {
        let html = r#"<html><head></head><body>
<dl>
  <dt>|FOO::1.BAR::1|</dt>
  <dd>Bam bip blop.</dd>
  <dt><a id="FOO::1.BAZ::1">|FOO::1.BAZ::1|</a></dt>
  <dd>Bam bip blop.</dd>
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

        assert_eq!(expected, linkify_spec_string(&html).unwrap());
    }

    #[test]
    fn can_linkify_spec_html() {
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

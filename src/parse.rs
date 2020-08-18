// use super::logical_unit::LogicalUnit;
use crate::logical_unit::{Kind, LogicalUnit};
use crate::pandoc;
use pandoc_ast::{Block, Inline, Pandoc, QuoteType};
use std::path::Path;

pub fn run(path: &Path) {
    // TODO Error handling
    file(path).map(render).expect("failed to parse")
}

pub fn render(lus: Vec<LogicalUnit>) -> () {
    for lu in lus {
        println!("{}", lu)
    }
}

pub fn file(path: &Path) -> Result<Vec<LogicalUnit>, String> {
    pandoc::parse_file(path).map(|ast| parse_ast(Some(path), ast))
}

pub fn string(s: String) -> Result<Vec<LogicalUnit>, String> {
    pandoc::parse_string(s).map(|ast| parse_ast(None, ast))
}

fn parse_ast(path: Option<&Path>, ast: Pandoc) -> Vec<LogicalUnit> {
    ast.blocks
        .iter()
        .filter_map(|b| match b {
            Block::DefinitionList(dl) => {
                let logical_units = logical_units_of_deflist(path, dl);
                Some(logical_units)
            }
            _ => None,
        })
        .flatten()
        .collect()
}

peg::parser! {
    grammar parser() for str {
        pub rule definiendum() -> String =
            // "|" d:$([_]+) "|"
            "|" d:$(['a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' | ':']+) "|"
        { d.to_string() }
    }
}

fn logical_units_of_deflist(
    path: Option<&Path>,
    deflist: &Vec<(Vec<Inline>, Vec<Vec<Block>>)>,
) -> Vec<LogicalUnit> {
    // TODO Infer from file type?
    deflist
        .iter()
        .filter_map(|(tags, blocks)| {
            logical_unit_definiendum(tags).and_then(|id| {
                let kind = Kind::Requirement;
                let contents = pandoc_blocks_list_to_string(blocks);
                // TODO Handle error instead of making `ok`?
                match LogicalUnit::new(path, kind, id, contents) {
                    Ok(lu) => Some(lu),
                    Err(err) => {
                        // TODO Replace with logging
                        println!("Error: {:?}", err);
                        None
                    }
                }
            })
        })
        .collect()
}

fn logical_unit_definiendum(tags: &Vec<Inline>) -> Option<String> {
    match &tags[..] {
        // Only defininiendum's with a single inline element are taken to be
        // logical unit defs
        [lu] => match lu {
            Inline::Str(s) => parser::definiendum(&s).ok(),
            Inline::Emph(v) => logical_unit_definiendum(&v),
            Inline::Strong(v) => logical_unit_definiendum(&v),
            Inline::Link(_, v, _) => logical_unit_definiendum(&v),
            // TODO Are we sure we don't want support any other variants?
            _ => None,
        },
        _ => None,
    }
}

fn pandoc_inlines_to_string(inlines: &[pandoc_ast::Inline]) -> String {
    inlines
        .iter()
        .map(pandoc_inline_to_string)
        .collect::<Vec<String>>()
        .join("")
}

fn pandoc_inline_to_string(i: &pandoc_ast::Inline) -> String {
    match i {
        Inline::Str(s) => s.clone(),
        Inline::Emph(v) => format!("*{}*", pandoc_inlines_to_string(v),),
        Inline::Strong(v) => format!("**{}**", pandoc_inlines_to_string(v),),
        Inline::Space => " ".to_string(),
        Inline::SoftBreak => "\n".to_string(),
        Inline::LineBreak => "\\\n".to_string(),
        Inline::Quoted(t, v) => match t {
            QuoteType::SingleQuote => format!("'{}'", pandoc_inlines_to_string(v)),
            QuoteType::DoubleQuote => format!("\"{}\"", pandoc_inlines_to_string(v)),
        },
        Inline::Link(_, v, (url, _)) => format!("[{}]({})", pandoc_inlines_to_string(v), url,),
        Inline::Image(_, v, (url, _)) => format!("![{}]({})", pandoc_inlines_to_string(v), url,),
        Inline::Code(_, s) => format!("`{}`", s),
        _ => "TODO".to_string(),
    }
}

fn pandoc_blocks_list_to_string(blocks_list: &[Vec<Block>]) -> String {
    blocks_list
        .iter()
        .map(pandoc_blocks_to_string)
        .collect::<Vec<String>>()
        .join("\n\n")
}

#[allow(clippy::ptr_arg)]
fn pandoc_blocks_to_string(blocks: &Vec<Block>) -> String {
    blocks
        .iter()
        .map(pandoc_block_to_string)
        .collect::<Vec<String>>()
        .join("\n\n")
}

fn pandoc_block_to_string(b: &Block) -> String {
    match b {
        pandoc_ast::Block::Plain(v) => pandoc_inlines_to_string(v),
        pandoc_ast::Block::Para(v) => pandoc_inlines_to_string(v),
        _ => "TODO".to_string(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_parse_defininiendum() {
        assert_eq!(
            Ok("FOO.1::BAR.1::BAZ.1".to_string()),
            parser::definiendum(&"|FOO.1::BAR.1::BAZ.1|")
        )
    }

    #[test]
    fn can_parse_logical_unit() {
        let spec = "
|FOO.1::BAR.1|
:  Biz baz blam.

|FOO.1::BAZ.1|
:  Pop crink splot.
";
        let expected = Ok(vec![
            LogicalUnit::new(
                None,
                Kind::Requirement,
                "FOO.1::BAR.1".to_string(),
                "Biz baz blam.".to_string(),
            )
            .unwrap(),
            LogicalUnit::new(
                None,
                Kind::Requirement,
                "FOO.1::BAZ.1".to_string(),
                "Pop crink splot.".to_string(),
            )
            .unwrap(),
        ]);
        let result = string(spec.to_string());
        assert_eq!(expected, result)
    }
}

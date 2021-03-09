//!
//! Interface to the pandoc CLI
//!

use {
    anyhow::{Context, Result},
    pandoc_ast::{Block, Inline, Pandoc, QuoteType},
    std::{
        convert::TryInto,
        io,
        io::{Read, Write},
        path::Path,
        process::{Command, ExitStatus, Stdio},
    },
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Deserialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Error invoking pandoc: {0}")]
    PandocInvocation(#[from] io::Error),

    #[error("Pandoc failed with status {0} and stderr: {1}")]
    PandocFailure(ExitStatus, String),

    #[error("Error processing data from pandoc: {0}")]
    PandocData(String),

    #[error("Could not convert given path to string")]
    Path,
}

static PANDOC: &str = "pandoc";
static ARGS: &[&str] = &["--standalone", "--from", "markdown-smart", "--to", "json"];

/// # Running the pandoc executable

fn pandoc_from_bytes(b: &[u8]) -> Result<Pandoc> {
    match b[..] {
        [] => Err(Error::PandocData("no data received from pandoc".into()).into()),
        _ => serde_json::from_str(String::from_utf8_lossy(b).as_ref())
            .map_err(|e: serde_json::Error| -> Error { Error::Serialization(e) })
            .with_context(|| {
                format!(
                    "deserializing pandoc JSON output: {}",
                    String::from_utf8_lossy(b)
                )
            }),
    }
}

/// Returns an [`Ok`] [`Pandoc`] value if the string can be parsed into the
/// pandoc AST, otherwise returns an [`Err`] with a string explaining the
/// failure.
pub fn parse_string(s: &str) -> Result<Pandoc> {
    let process = Command::new(PANDOC)
        .args(ARGS)
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .map_err(Error::PandocInvocation)?;

    // TODO
    process
        .stdin
        .unwrap()
        .write_all(s.as_bytes())
        .map_err(Error::PandocInvocation)?;

    let mut bytes = Vec::new();
    process
        .stdout
        .ok_or_else(|| Error::PandocData("trying to read from stdout".into()))
        .and_then(|mut c| c.read_to_end(&mut bytes).map_err(Error::PandocInvocation))?;

    pandoc_from_bytes(&bytes)
}

// TODO Use this to support more integrated recovery of nested markdown
/// Renders a pandoc AST into a markdown string
// pub fn render_ast(p: &Pandoc) -> std::result::Result<String, String> {
//     let process = Command::new(PANDOC)
//         .args(&["--from", "native", "--to", "markdown"])
//         .stdout(Stdio::piped())
//         .spawn()
//         .map_err(|_| "spawning panodc process".to_string())?;

//     let s = serde_json::to_string(p).map_err(|_| "serializing to json")?;

//     process
//         .stdin
//         .unwrap()
//         .write_all(s.as_bytes())
//         .map_err(|_| "writing to pandoc process")?;

//     let mut bytes = Vec::new();
//     process
//         .stdout
//         .ok_or("receiving pandoc process output")
//         .and_then(|mut c| {
//             c.read_to_end(&mut bytes)
//                 .or(Err("reading from pandoc process"))
//         })?;

//     std::str::from_utf8(&bytes)
//         .map_err(|_| "decoding markdown string".to_string())
//         .map(str::to_string)
// }

pub fn parse_file(path: &Path) -> Result<Pandoc> {
    let source = path.to_str().ok_or(Error::Path)?;

    let output = Command::new(PANDOC)
        .args(ARGS)
        .arg(source)
        .output()
        .map_err(Error::PandocInvocation)?;

    if output.status.success() {
        pandoc_from_bytes(&output.stdout)
    } else {
        Err(Error::PandocFailure(
            output.status,
            String::from_utf8_lossy(&output.stderr).into(),
        )
        .into())
    }
}

/// # Parsing the pandoc AST
#[allow(clippy::ptr_arg)]
pub fn inlines_to_string(inlines: &Vec<Inline>) -> String {
    inlines
        .iter()
        .map(inline_to_string)
        .collect::<Vec<String>>()
        .join("")
}

pub fn inline_to_string(i: &pandoc_ast::Inline) -> String {
    match i {
        Inline::Str(s) => s.clone(),
        Inline::Emph(v) => format!("*{}*", inlines_to_string(v),),
        Inline::Strong(v) => format!("**{}**", inlines_to_string(v),),
        Inline::Space => " ".to_string(),
        Inline::SoftBreak => "\n".to_string(),
        Inline::LineBreak => "\\n".to_string(),
        Inline::Quoted(t, v) => match t {
            QuoteType::SingleQuote => format!("'{}'", inlines_to_string(v)),
            QuoteType::DoubleQuote => format!("\"{}\"", inlines_to_string(v)),
        },
        Inline::Link(_, v, (url, _)) => format!("[{}]({})", inlines_to_string(v), url,),
        Inline::Image(_, v, (url, _)) => format!("![{}]({})", inlines_to_string(v), url,),
        Inline::Code(_, s) => format!("`{}`", s),
        Inline::Underline(il) => inlines_to_string(il),
        Inline::Strikeout(il) => format!("~~{}~~", inlines_to_string(il)),
        Inline::Superscript(il) => format!("^{}^", inlines_to_string(il)),
        Inline::Subscript(il) => format!("~{}~", inlines_to_string(il)),
        Inline::SmallCaps(il) => format!("[{}]{{.smallcaps}}", inlines_to_string(il)),
        Inline::Cite(_, il) => inlines_to_string(il),
        Inline::Math(_, s) => s.clone(),
        Inline::RawInline(_, s) => s.clone(),
        Inline::Note(bl) => blocks_to_string(bl),
        Inline::Span(_, il) => inlines_to_string(il),
    }
}

pub fn blocks_list_to_string(blocks_list: &[Vec<Block>]) -> String {
    blocks_list
        .iter()
        .map(blocks_to_string)
        .collect::<Vec<String>>()
        .join("\n\n")
}

#[allow(clippy::ptr_arg)]
pub fn blocks_to_string(blocks: &Vec<Block>) -> String {
    blocks
        .iter()
        .map(block_to_string)
        .collect::<Vec<String>>()
        .join("\n\n")
}

#[allow(clippy::ptr_arg)]
fn line_blocks_to_string(lines: &Vec<Vec<Inline>>) -> String {
    lines
        .iter()
        .map(inlines_to_string)
        .collect::<Vec<String>>()
        .join("\n\n")
}

#[allow(clippy::ptr_arg)]
fn block_quote_to_string(blocks: &Vec<Block>) -> String {
    blocks
        .iter()
        .map(block_to_string)
        .map(|b| format!("> {}", b))
        .collect::<Vec<String>>()
        .join("\n")
}

#[allow(clippy::ptr_arg)]
fn list_to_string(prefix: &str, blocks: &Vec<Vec<Block>>) -> String {
    blocks
        .iter()
        .map(blocks_to_string)
        .map(|b| format!("{} {}", prefix, b))
        .collect::<Vec<String>>()
        .join("\n")
}

#[allow(clippy::ptr_arg)]
fn header_to_string(level: &i64, content: &Vec<Inline>) -> String {
    // Pandoc would violate its own spec if it allowed more than 6 heading levels
    // so we should be safe to unwrap here.
    let hashes = "#".repeat((*level).try_into().unwrap());
    format!("{} {}", hashes, inlines_to_string(content))
}

pub fn block_to_string(b: &Block) -> String {
    match b {
        Block::Plain(v) => inlines_to_string(v),
        Block::Para(v) => inlines_to_string(v),
        Block::LineBlock(v) => line_blocks_to_string(v),
        Block::CodeBlock(_, s) => format!("```\n{}\n```", s),
        Block::RawBlock(_, s) => s.into(),
        Block::BlockQuote(v) => block_quote_to_string(v),
        Block::OrderedList(_, v) => list_to_string("1.", v),
        Block::BulletList(v) => list_to_string("-", v),
        Block::Header(level, _, v) => header_to_string(level, v),
        Block::HorizontalRule => "---".into(),
        Block::Null => "".into(),
        // Currently unspported values
        Block::Div(_, _) => "<!-- WARNING: nested div omitted -->".into(),
        Block::DefinitionList(_) => "<!-- WARNING: nested definition lists omitted -->".into(),
        Block::Table(_, _, _, _, _, _) => "<!-- WARNING: nested table omitted -->".into(),
    }
}

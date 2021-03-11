//!
//! Interface to the pandoc CLI
//!

use {
    anyhow::Result,
    html2md,
    itertools::Itertools,
    scraper,
    std::{
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

    #[error("Processing data from pandoc: {0}")]
    PandocData(String),

    #[error("Could not convert given path to string")]
    Path,

    #[error("Parsing definition list. Encountered {0} while parsing `{1}`")]
    DefinitionListParsing(String, String),
}

static PANDOC: &str = "pandoc";
static ARGS: &[&str] = &["--from", "markdown", "--to", "html"];

/// # Running the pandoc executable

fn html_from_md_bytes(b: &[u8]) -> Result<scraper::Html> {
    match b[..] {
        [] => Err(Error::PandocData("no data received from pandoc".into()).into()),
        _ => Ok(scraper::Html::parse_fragment(
            String::from_utf8_lossy(b).as_ref(),
        )),
    }
}

/// Returns an [`Ok`] [`Pandoc`] value if the string can be parsed into the
/// pandoc AST, otherwise returns an [`Err`] with a string explaining the
/// failure.
fn parse_string(s: &str) -> Result<scraper::Html> {
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

    html_from_md_bytes(&bytes)
}

fn parse_file(path: &Path) -> Result<scraper::Html> {
    let source = path.to_str().ok_or(Error::Path)?;

    let output = Command::new(PANDOC)
        .args(ARGS)
        .arg(source)
        .output()
        .map_err(Error::PandocInvocation)?;

    if output.status.success() {
        html_from_md_bytes(&output.stdout)
    } else {
        Err(Error::PandocFailure(
            output.status,
            String::from_utf8_lossy(&output.stderr).into(),
        )
        .into())
    }
}

fn is_dt(element: &scraper::ElementRef) -> bool {
    element.value().name() == "dt"
}

fn is_dd(element: &scraper::ElementRef) -> bool {
    element.value().name() == "dd"
}

fn def_parsing_err(msg: &str, element: scraper::ElementRef) -> anyhow::Error {
    Error::DefinitionListParsing(msg.to_string(), element.html()).into()
}

fn definitions_from_html(html: scraper::Html) -> Result<Vec<(String, String)>> {
    // Beware, yucky imperative programming ahead :(
    let mut defs = Vec::new();

    // These unwraps should only fail if invalid selectors are constructed,
    // but this is effectively a constant we know at build time and exercise in
    // our tests
    let def_lists = scraper::Selector::parse("dl").unwrap();
    // Either a def term or a definition
    let def_item = scraper::Selector::parse("dt, dd").unwrap();

    for def_list in html.select(&def_lists) {
        // Group dt and dd elems together
        let grouped_elements = def_list
            .select(&def_item)
            .into_iter()
            .group_by(|el| el.value().name());
        // Arrange
        let definitions = grouped_elements.into_iter().tuples();
        for ((term_tag, terms_group), (defs_tag, defs_group)) in definitions {
            if !(term_tag == "dt" && defs_tag == "dd") {
                // This generally shouldn't occur, since a definition list without a
                // leading dt can't appear in markdown. But it could arise from
                // HTML embedded in a markdown doc.
                return Err(def_parsing_err("invalid tags on def list groups", def_list));
            }

            let terms: Vec<scraper::ElementRef> = terms_group.collect();

            let tag = match terms[..] {
                [term] => term.inner_html(),
                [] => return Err(def_parsing_err("no definition terms", def_list)),
                _ => {
                    return Err(def_parsing_err(
                        "multiple definition terms (not yet supported)",
                        def_list,
                    ))
                }
            };

            let content = defs_group
                .map(|el| html2md::parse_html(&el.html()))
                .join("\n\n");

            defs.push((tag, content));
        }
    }
    Ok(defs)
}

pub fn definitions_from_file(path: &Path) -> Result<Vec<(String, String)>> {
    let html = parse_file(path)?;
    definitions_from_html(html)
}

pub fn definitions_from_string(s: &str) -> Result<Vec<(String, String)>> {
    let html = parse_string(s)?;
    definitions_from_html(html)
}

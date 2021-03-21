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

fn html_from_md_bytes(b: &[u8]) -> Result<String> {
    match b[..] {
        [] => Err(Error::PandocData("no data received from pandoc".into()).into()),
        _ => Ok(String::from_utf8_lossy(b).into()),
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

    html_from_md_bytes(&bytes).map(|s| scraper::Html::parse_fragment(&s))
}

/// `parse_file(path)` parses the markdown file at `path` into an html string
pub fn parse_file(path: &Path) -> Result<String> {
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

/// `html_to_markdown(html, path)` write the `html` to the file at `path`
/// as markdown.
pub fn html_to_markdown(html: &str) -> Result<String> {
    let process = Command::new(PANDOC)
        .args(&[
            "--from",
            "html-native_divs-native_spans",
            "--to",
            "markdown",
            "--reference-links",
        ])
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .map_err(Error::PandocInvocation)?;

    process
        .stdin
        .unwrap()
        .write_all(html.as_bytes())
        .map_err(Error::PandocInvocation)?;

    let mut bytes = Vec::new();
    process
        .stdout
        .ok_or_else(|| Error::PandocData("trying to read from stdout".into()))
        .and_then(|mut c| c.read_to_end(&mut bytes).map_err(Error::PandocInvocation))?;

    Ok(String::from_utf8_lossy(&bytes).to_string())
}

fn parse_file_to_scraper(path: &Path) -> Result<scraper::Html> {
    parse_file(path).map(|s| scraper::Html::parse_fragment(&s))
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

// Get the first child and wrap it back up into an ElementRef (if it exists)
fn get_child_element(el: scraper::ElementRef) -> Option<scraper::ElementRef> {
    el.first_child().and_then(scraper::ElementRef::wrap)
}

// Get the inner html after applying the selection
fn select_inner_html(el: scraper::ElementRef, selector: &scraper::Selector) -> Option<String> {
    el.select(selector).next().map(|e| e.inner_html())
}

// Finds a logical unit tag from a def list term, even if it's wrapped in
// (possibly nested) inline HTML tags.
fn tag_of_term(term: scraper::ElementRef) -> Option<String> {
    match get_child_element(term) {
        None => Some(term.inner_html()),
        Some(child) => tag_of_term(child),
    }
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

            // TODO Can we dispense with tese imperative returns?
            let tag = match terms[..] {
                [term] => {
                    if let Some(tag) = tag_of_term(term) {
                        tag
                    } else {
                        return Err(def_parsing_err(
                            "could not parse html of definition term",
                            def_list,
                        ));
                    }
                }
                // This should be impossbile
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
    let html = parse_file_to_scraper(path)?;
    definitions_from_html(html)
}

pub fn definitions_from_string(s: &str) -> Result<Vec<(String, String)>> {
    let html = parse_string(s)?;
    definitions_from_html(html)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn find_def_list_with_naked_term() {
        let input = r#"
|FOO.1::BAR.1|
:  Biz baz blam.
"#;
        let actual = definitions_from_string(input).unwrap();
        let expected = vec![("|FOO.1::BAR.1|".into(), "Biz baz blam.".into())];
        assert_eq!(actual, expected)
    }

    fn find_def_list_with_wrapped_term() {
        let input = r#"
<a id="FOO.1::BAR.1">|FOO.1::BAR.1|</a>
:  Biz baz blam.
"#;
        let actual = definitions_from_string(input).unwrap();
        let expected = vec![("|FOO.1::BAR.1|".into(), "Biz baz blam.".into())];
        assert_eq!(actual, expected)
    }

    fn find_def_list_with_nestd_wrapped_term() {
        let input = r#"
<a id="FOO.1::BAR.1"><em>|FOO.1::BAR.1|</em></a>
:  Biz baz blam.
"#;
        let actual = definitions_from_string(input).unwrap();
        let expected = vec![("|FOO.1::BAR.1|".into(), "Biz baz blam.".into())];
        assert_eq!(actual, expected)
    }
}

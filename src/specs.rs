//!
//! Project specifications-related functionality.
//!

use super::{Error, LogicalUnit, LogicalUnitID, ProjectSourceFile, Result};
use failure::Fail;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Fail)]
pub enum SpecificationParseError {
    #[fail(display = "duplicate logical unit found with ID: {}", _0)]
    DuplicateLogicalUnit(LogicalUnitID),
    #[fail(display = "Pandoc execution failed: {}", _0)]
    PandocError(String),
    #[fail(display = "failed to parse Pandoc AST: {}", _0)]
    PandocASTParseError(String),
}

/// Project specifications are comprised of logical units, mapped to their IDs.
#[derive(Debug)]
pub struct ProjectSpecifications(HashMap<LogicalUnitID, LogicalUnit>);

impl ProjectSpecifications {
    /// Creates an empty set of project specifications.
    pub fn new() -> ProjectSpecifications {
        ProjectSpecifications(HashMap::new())
    }

    /// Attempts to load project specifications from the given project source
    /// file. We assume this file is written in Themis Tracer-compatible
    /// Markdown.
    pub fn parse_from_source_file(f: &ProjectSourceFile) -> Result<ProjectSpecifications> {
        Ok(ProjectSpecifications(
            parse_file_with_pandoc(PathBuf::from(f.filename.clone()).as_path())?
                .blocks
                .iter()
                // convert definition lists to hashmaps mapping logical unit IDs
                // to logical units
                .filter_map(|b| match b {
                    pandoc_ast::Block::DefinitionList(dl) => Some(pandoc_deflist_to_specs(f, dl)),
                    _ => None,
                })
                .fold(Ok(HashMap::new()), |acc_res, m_res| match acc_res {
                    Ok(acc) => match m_res {
                        Ok(res) => merge_project_lu_maps(acc, res),
                        // if the map operation resulted in an error, bubble it
                        // up by replacing the accumulator
                        Err(e) => Err(e),
                    },
                    // bubble up the accumulated error and skip over any further
                    // operations
                    Err(e) => Err(e),
                })?,
        ))
    }
}

impl std::default::Default for ProjectSpecifications {
    fn default() -> Self {
        ProjectSpecifications::new()
    }
}

fn parse_file_with_pandoc(path: &Path) -> Result<pandoc_ast::Pandoc> {
    let output = Command::new("pandoc")
        .arg("-s")
        .arg(path.to_str().ok_or_else(|| {
            Error::InternalError(format!("path {:?} contains non-Unicode characters", path))
        })?)
        .arg("-t")
        .arg("json")
        .output()
        .map_err(|e| {
            Error::SpecificationParseError(SpecificationParseError::PandocError(e.to_string()))
        })?;
    if let Some(code) = output.status.code() {
        if code != 0 {
            return Err(Error::SpecificationParseError(
                SpecificationParseError::PandocError(format!("Pandoc exited with code {}", code)),
            ));
        }
    }
    serde_json::from_str(String::from_utf8_lossy(&output.stdout).as_ref()).map_err(|e| {
        Error::SpecificationParseError(SpecificationParseError::PandocASTParseError(e.to_string()))
    })
}

fn pandoc_deflist_to_specs(
    f: &ProjectSourceFile,
    dl: &Vec<(Vec<pandoc_ast::Inline>, Vec<Vec<pandoc_ast::Block>>)>,
) -> Result<HashMap<LogicalUnitID, LogicalUnit>> {
    let mut lu_map = HashMap::<LogicalUnitID, LogicalUnit>::new();
    for def_pair in dl {
        let (tags, contents) = def_pair;
        // we're only interested in the first definition
        if let Some(tag) = tags.first() {
            let tag_str = pandoc_inline_to_string(tag);
            if tag_str.starts_with("|") && tag_str.ends_with("|") {
                let luid = LogicalUnitID::from_str(tag_str.trim_matches('|').as_ref())?;
                lu_map.insert(
                    luid.clone(),
                    LogicalUnit {
                        source_file: (*f).clone(),
                        id: luid,
                        desc: pandoc_blocks_list_to_string(contents),
                    },
                );
            }
        }
    }
    Ok(lu_map)
}

fn pandoc_inlines_to_string(inlines: &Vec<pandoc_ast::Inline>) -> String {
    inlines
        .iter()
        .map(pandoc_inline_to_string)
        .collect::<Vec<String>>()
        .join("")
}

fn pandoc_inline_to_string(i: &pandoc_ast::Inline) -> String {
    match i {
        pandoc_ast::Inline::Str(s) => s.clone(),
        pandoc_ast::Inline::Emph(v) => format!("*{}*", pandoc_inlines_to_string(v),),
        pandoc_ast::Inline::Strong(v) => format!("**{}**", pandoc_inlines_to_string(v),),
        pandoc_ast::Inline::Space => " ".to_string(),
        pandoc_ast::Inline::SoftBreak => "\n".to_string(),
        pandoc_ast::Inline::LineBreak => "\\\n".to_string(),
        pandoc_ast::Inline::Link(_, v, (url, _)) => {
            format!("[{}]({})", pandoc_inlines_to_string(v), url,)
        }
        pandoc_ast::Inline::Image(_, v, (url, _)) => {
            format!("![{}]({})", pandoc_inlines_to_string(v), url,)
        }
        _ => "".to_string(),
    }
}

fn pandoc_blocks_list_to_string(blocks_list: &Vec<Vec<pandoc_ast::Block>>) -> String {
    blocks_list
        .iter()
        .map(pandoc_blocks_to_string)
        .collect::<Vec<String>>()
        .join("\n\n")
}

fn pandoc_blocks_to_string(blocks: &Vec<pandoc_ast::Block>) -> String {
    blocks
        .iter()
        .map(pandoc_block_to_string)
        .collect::<Vec<String>>()
        .join("\n\n")
}

fn pandoc_block_to_string(b: &pandoc_ast::Block) -> String {
    match b {
        pandoc_ast::Block::Plain(v) => pandoc_inlines_to_string(v),
        pandoc_ast::Block::Para(v) => pandoc_inlines_to_string(v),
        _ => "".to_string(),
    }
}

// TODO: Fix this horrifically inefficient mess.
fn merge_project_lu_maps(
    dest: HashMap<LogicalUnitID, LogicalUnit>,
    src: HashMap<LogicalUnitID, LogicalUnit>,
) -> Result<HashMap<LogicalUnitID, LogicalUnit>> {
    let mut r = dest
        .iter()
        .map(|(k, v)| ((*k).clone(), (*v).clone()))
        .collect::<HashMap<LogicalUnitID, LogicalUnit>>();
    for (luid, lu) in src.iter() {
        // we can't allow duplicate logical unit IDs
        if dest.contains_key(luid) {
            return Err(Error::SpecificationParseError(
                SpecificationParseError::DuplicateLogicalUnit((*luid).clone()),
            ));
        }
        r.insert((*luid).clone(), (*lu).clone());
    }
    Ok(r)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{ProjectSourceFile, ProjectSourceFileKind};
    use std::fs::File;
    use std::io::Write;
    use textwrap::dedent;

    const SIMPLE_SPEC: &str = r#"
    # Specification

    |SPEC-HELLO.1|
    :   When executed, the program must print out the text "Hello world!"
    "#;

    const MULTI_UNIT_SPEC: &str = r#"
    # Specification

    |SPEC-INPUT.1|
    :   When executed, the program must print the text: "Hello! What's your name?",
        and allow the user to input their name.
    
    |SPEC-HELLO.2|
    :   Once the user's name has been obtained, the program must print out the text
        "Hello {name}!", where `{name}` must be replaced by the name obtained in
        [SPEC-INPUT.1].
    "#;

    #[test]
    fn test_simple_spec_parsing() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("simple-spec.md");
        let mut spec_file = File::create(&file_path).unwrap();
        spec_file.write_all(dedent(SIMPLE_SPEC).as_bytes()).unwrap();
        drop(spec_file);

        let spec = ProjectSpecifications::parse_from_source_file(&ProjectSourceFile {
            filename: file_path.to_str().unwrap().to_string(),
            kind: ProjectSourceFileKind::Spec,
        })
        .unwrap();
        assert_eq!(
            1,
            spec.0.len(),
            "we expect a single logical unit in the specification"
        );

        assert!(
            spec.0
                .contains_key(&LogicalUnitID::from_str("SPEC-HELLO.1").unwrap()),
            "we expect a logical unit named SPEC-HELLO.1 in the specification"
        );

        // clean up temporary directory
        tmp_dir.close().unwrap();
    }

    #[test]
    fn test_multi_unit_spec_parsing() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let file_path = tmp_dir.path().join("multi-unit-spec.md");
        let mut spec_file = File::create(&file_path).unwrap();
        spec_file
            .write_all(dedent(MULTI_UNIT_SPEC).as_bytes())
            .unwrap();
        drop(spec_file);

        let spec = ProjectSpecifications::parse_from_source_file(&ProjectSourceFile {
            filename: file_path.to_str().unwrap().to_string(),
            kind: ProjectSourceFileKind::Spec,
        })
        .unwrap();
        assert_eq!(
            2,
            spec.0.len(),
            "we expect two logical units in the specification"
        );

        assert!(
            spec.0
                .contains_key(&LogicalUnitID::from_str("SPEC-INPUT.1").unwrap()),
            "we expect a logical unit named SPEC-INPUT.1 in the specification"
        );
        assert!(
            spec.0
                .contains_key(&LogicalUnitID::from_str("SPEC-HELLO.2").unwrap()),
            "we expect a logical unit named SPEC-HELLO.2 in the specification"
        );

        // clean up temporary directory
        tmp_dir.close().unwrap();
    }
}

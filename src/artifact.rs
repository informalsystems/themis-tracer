use {
    crate::{
        logical_unit::{Kind, LogicalUnit},
        pandoc, util,
    },
    anyhow::Result,
    pandoc_ast::{Block, Inline, Pandoc},
    std::{
        collections::HashSet,
        fmt,
        path::{Path, PathBuf},
    },
    thiserror::Error,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Artifact {
    pub source: Option<PathBuf>,
    pub logical_units: HashSet<LogicalUnit>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("parsing artifact {0}")]
    ParsingArtifact(PathBuf),

    #[error("parsing artifact from string {0}")]
    ParsingString(String),
}

impl Artifact {
    pub fn new(source: Option<PathBuf>, logical_units: HashSet<LogicalUnit>) -> Artifact {
        Artifact {
            source,
            logical_units,
        }
    }

    /// Parse the file `path` into an artifact
    pub fn from_file(path: &Path) -> Result<Artifact> {
        pandoc::parse_file(path)
            .map(|ast| parse_ast(Some(path), ast))
            .map(|lus| Artifact::new(Some(path.to_owned()), lus.iter().cloned().collect()))
            .map_err(|_| Error::ParsingArtifact(PathBuf::from(path)).into())
    }

    /// Parse the string `s` into an artifact with no source
    pub fn from_string(s: &str) -> Result<Artifact> {
        pandoc::parse_string(s)
            .map(|ast| parse_ast(None, ast))
            .map(|lus| Artifact::new(None, lus.iter().cloned().collect()))
            .map_err(|_| Error::ParsingString(s.into()).into())
    }
}

impl fmt::Display for Artifact {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Artifact from source '{:?}' content: {:?}",
            self.source, self.logical_units
        )
    }
}

// Parse logical units out of the pandoc AST.
fn parse_ast(path: Option<&Path>, ast: Pandoc) -> HashSet<LogicalUnit> {
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

// Given the `pandoc_ast` representation of a description list,
// this finds any items that are valid logical units.
fn logical_units_of_deflist(
    path: Option<&Path>,
    deflist: &[(Vec<Inline>, Vec<Vec<Block>>)],
) -> Vec<LogicalUnit> {
    // TODO Infer from file type?
    deflist
        .iter()
        .filter_map(|(tags, blocks)| {
            logical_unit_definiendum(tags).and_then(|id| {
                let kind = Kind::Requirement;
                let contents = pandoc::blocks_list_to_string(blocks);
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

// Is `Some(s)` if `s` can be a logical unit ID enclosed in pipes.
fn logical_unit_definiendum(tags: &[Inline]) -> Option<String> {
    match tags {
        // Only defininiendum's with a single inline element are taken to be
        // logical unit defs
        [lu] => match lu {
            Inline::Str(s) => util::parser::logical_unit_definiendum(&s).ok(),
            Inline::Emph(v) => logical_unit_definiendum(&v),
            Inline::Strong(v) => logical_unit_definiendum(&v),
            Inline::Link(_, v, _) => logical_unit_definiendum(&v),
            // TODO Are we sure we don't want support any other variants?
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_parse_logical_unit() {
        let spec = "
|FOO.1::BAR.1|
:  Biz baz blam.

|FOO.1::BAZ.1|
:  Pop crink splot.

[|FOO.1::BOP.1|](./url)
:  Can parse URLs
";

        let logical_units: HashSet<LogicalUnit> = vec![
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
            LogicalUnit::new(
                None,
                Kind::Requirement,
                "FOO.1::BOP.1".to_string(),
                "Can parse URLs".to_string(),
            )
            .unwrap(),
        ]
        .iter()
        .cloned()
        .collect();

        let expected: Artifact = Artifact::new(None, logical_units);
        let result = Artifact::from_string(&spec).unwrap();
        assert_eq!(expected, result)
    }
}

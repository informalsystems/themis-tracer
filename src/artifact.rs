use {
    crate::{
        logical_unit::{Kind, LogicalUnit},
        pandoc,
        repo::Repo,
        util,
    },
    anyhow::{Context, Result},
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
    #[error("parsing artifact {0}: {1}")]
    ParsingArtifact(PathBuf, serde_json::Error),
}

impl Artifact {
    pub fn new(source: Option<PathBuf>, logical_units: HashSet<LogicalUnit>) -> Artifact {
        Artifact {
            source,
            logical_units,
        }
    }

    /// Parse the file `path` into an artifact
    pub fn from_file(repo: Option<Repo>, path: &Path) -> Result<Artifact> {
        pandoc::definitions_from_file(path)
            .map(|defs| logical_units_of_defs(repo, Some(path), &defs))
            .map(|lus| Artifact::new(Some(path.to_owned()), lus.iter().cloned().collect()))
            .with_context(|| {
                format!(
                    "while parsing artifact {}",
                    path.as_os_str().to_str().unwrap_or("<cannot render path>")
                )
            })
    }

    /// Parse the string `s` into an artifact with no source
    pub fn from_string(s: &str) -> Result<Artifact> {
        pandoc::definitions_from_string(s)
            .map(|defs| logical_units_of_defs(None, None, &defs))
            .map(|lus| Artifact::new(None, lus.iter().cloned().collect()))
            .with_context(|| format!("parsing artifact from string {}", s))
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

// Given the `pandoc_ast` representation of a description list,
// this finds any items that are valid logical units.
fn logical_units_of_defs(
    repo: Option<Repo>,
    file: Option<&Path>,
    defs: &[(String, String)],
) -> Vec<LogicalUnit> {
    // TODO Infer from file type?
    defs.iter()
        .filter_map(|(tags, content)| {
            util::parser::logical_unit_definiendum(tags)
                .ok()
                .and_then(|id| {
                    // TODO Determine kind from file type
                    let kind = Kind::Requirement;
                    // TODO Determine line
                    match LogicalUnit::new(repo.clone(), file, None, kind, id, content.clone()) {
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

|FOO.1::BOP.1|
:  Can parse URLs
";

        let logical_units: HashSet<LogicalUnit> = vec![
            LogicalUnit::new(
                None,
                None,
                None,
                Kind::Requirement,
                "FOO.1::BAR.1".to_string(),
                "Biz baz blam.".to_string(),
            )
            .unwrap(),
            LogicalUnit::new(
                None,
                None,
                None,
                Kind::Requirement,
                "FOO.1::BAZ.1".to_string(),
                "Pop crink splot.".to_string(),
            )
            .unwrap(),
            LogicalUnit::new(
                None,
                None,
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
        let result = Artifact::from_string(&spec);
        println!("{:?}", result);
        assert_eq!(expected, result.unwrap())
    }
}

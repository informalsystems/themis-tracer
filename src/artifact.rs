use {
    crate::{
        logical_unit::{Kind, LogicalUnit},
        pandoc,
        parser::parser,
        repo::Repo,
    },
    anyhow::{Context, Result},
    std::{
        collections::HashSet,
        convert::TryFrom,
        fmt,
        fs::File,
        io,
        io::BufRead, // modularity is awkward in rust
        path::{Path, PathBuf},
    },
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("parsing artifact {0}: {1}")]
    ParsingArtifact(PathBuf, serde_json::Error),
    #[error("no source file defined file for {0} files")]
    SourceFileExt(String),
    #[error("cannot determine source file for {0}, because it has no extension")]
    SourceFileNoExt(PathBuf),
}

enum SourceFileKind {
    Markdown,
    Rust,
}

struct SourceFile<'a> {
    kind: SourceFileKind,
    path: &'a Path,
    repo: Option<&'a Repo>,
}

impl<'a> SourceFile<'a> {
    fn new(kind: SourceFileKind, path: &'a Path) -> Self {
        Self {
            kind,
            path,
            repo: None,
        }
    }

    fn set_repo(&mut self, repo: Option<&'a Repo>) {
        self.repo = repo
    }

    fn units(&self) -> Result<HashSet<LogicalUnit>> {
        match self.kind {
            SourceFileKind::Markdown => self.units_of_md(),
            SourceFileKind::Rust => self.units_of_rs(),
        }
    }

    fn units_of_md(&self) -> Result<HashSet<LogicalUnit>> {
        pandoc::definitions_from_file(self.path)
            .map(|defs| {
                logical_units_of_defs(self.repo.cloned(), Some(self.path), &defs)
                    .iter()
                    .cloned()
                    .collect()
            })
            .with_context(|| {
                format!(
                    "while parsing artifact {}",
                    self.path
                        .as_os_str()
                        .to_str()
                        .unwrap_or("<cannot render path>")
                )
            })
    }

    fn units_of_rs(&self) -> Result<HashSet<LogicalUnit>> {
        let file = File::open(self.path)?;
        let units: HashSet<LogicalUnit> = io::BufReader::new(file)
            .lines()
            .map(|l| l.unwrap()) // FIXME remove unwrap
            .enumerate()
            .flat_map(|line| self.unit_of_rs_line(line))
            .collect();
        Ok(units)
    }

    fn unit_of_rs_line(&self, (_n, _line): (usize, String)) -> Option<LogicalUnit> {
        panic!("TODO")
    }
}

impl TryFrom<&Path> for SourceFileKind {
    type Error = Error;

    fn try_from(p: &Path) -> std::result::Result<Self, Self::Error> {
        if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            match ext {
                "md" => Ok(SourceFileKind::Markdown),
                "rs" => Ok(SourceFileKind::Rust),
                _ => Err(Error::SourceFileExt(ext.to_string())),
            }
        } else {
            Err(Error::SourceFileNoExt(p.to_path_buf()))
        }
    }
}

impl<'a> TryFrom<&'a Path> for SourceFile<'a> {
    type Error = Error;

    fn try_from(p: &'a Path) -> std::result::Result<Self, Self::Error> {
        let kind = SourceFileKind::try_from(p)?;
        Ok(SourceFile::new(kind, p))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Artifact {
    pub path: Option<PathBuf>,
    pub logical_units: HashSet<LogicalUnit>,
}

impl Artifact {
    pub fn new(path: Option<PathBuf>, logical_units: HashSet<LogicalUnit>) -> Artifact {
        Artifact {
            path,
            logical_units,
        }
    }

    /// Parse the file `path` into an artifact
    pub fn from_file(repo: Option<&Repo>, path: &Path) -> Result<Artifact> {
        let mut source_file = SourceFile::try_from(path)?;
        source_file.set_repo(repo);
        let units = source_file.units()?;
        Ok(Artifact::new(Some(path.to_owned()), units))
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
            self.path, self.logical_units
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
            parser::logical_unit_definiendum(tags).ok().and_then(|id| {
                // TODO Determine kind from file type
                let kind = Kind::Requirement;
                // TODO Determine line
                match LogicalUnit::new(repo.clone(), file, None, kind, &id, &content) {
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
                "FOO.1::BAR.1",
                "Biz baz blam.",
            )
            .unwrap(),
            LogicalUnit::new(
                None,
                None,
                None,
                Kind::Requirement,
                "FOO.1::BAZ.1",
                "Pop crink splot.",
            )
            .unwrap(),
            LogicalUnit::new(
                None,
                None,
                None,
                Kind::Requirement,
                "FOO.1::BOP.1",
                "Can parse URLs",
            )
            .unwrap(),
        ]
        .iter()
        .cloned()
        .collect();

        let expected: Artifact = Artifact::new(None, logical_units);
        let actual = Artifact::from_string(&spec);
        assert_eq!(actual.unwrap(), expected)
    }

    #[test]
    fn can_parse_logical_unit_within_anchor() {
        let spec = r#"
<a id="TAG.1::IN-TARGET.1">|TAG.1::IN-ANCHOR-TAG.1|</a>
: We can parse tags in an anchor html element.
"#;
        let logical_units: HashSet<LogicalUnit> = vec![LogicalUnit::new(
            None,
            None,
            None,
            Kind::Requirement,
            "TAG.1::IN-ANCHOR-TAG.1".into(),
            "We can parse tags in an anchor html element.".into(),
        )
        .unwrap()]
        .iter()
        .cloned()
        .collect();

        let expected = Artifact::new(None, logical_units);
        let actual = Artifact::from_string(&spec);
        assert_eq!(actual.unwrap(), expected)
    }

    fn can_parse_logical_unit_preceding_anchor() {
        let spec = r#"
|TAG.1::IN-ANCHOR-TAG.1|<a id="TAG.1::IN-TARGET.1"></a>
: We can parse tags in an anchor html element.
"#;
        let logical_units: HashSet<LogicalUnit> = vec![LogicalUnit::new(
            None,
            None,
            None,
            Kind::Requirement,
            "TAG.1::IN-ANCHOR-TAG.1".into(),
            "We can parse tags in an anchor html element.".into(),
        )
        .unwrap()]
        .iter()
        .cloned()
        .collect();

        let expected = Artifact::new(None, logical_units);
        let actual = Artifact::from_string(&spec);
        assert_eq!(actual.unwrap(), expected)
    }
}

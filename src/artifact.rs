use {
    crate::{
        logical_unit::{Kind, LogicalUnit},
        pandoc,
        parser::{parser, TAG_ID_RE},
        repo::Repo,
    },
    anyhow::{Context, Result},
    log,
    std::{
        collections::HashSet,
        convert::{TryFrom, TryInto},
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
            SourceFileKind::Rust => self.units_of_src(),
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

    fn units_of_src(&self) -> Result<HashSet<LogicalUnit>> {
        let mut file = File::open(self.path)?;
        self.units_of_src_reader(&mut file)
    }

    fn units_of_src_reader(&self, reader: &mut impl io::Read) -> Result<HashSet<LogicalUnit>> {
        let units: HashSet<LogicalUnit> = io::BufReader::new(reader)
            .lines()
            .collect::<io::Result<Vec<String>>>()? // Fail if we errored on any read
            .iter() // TODO avoid an intermediate collection?
            .enumerate() // Get the line numbers
            .flat_map(|line| self.unit_of_src_line(line))
            .collect();
        Ok(units)
    }

    fn unit_of_src_line(&self, (n, line): (usize, &String)) -> Option<LogicalUnit> {
        let id = TAG_ID_RE
            .captures(line)
            .and_then(|c| c.name("tag"))?
            .as_str();
        let content = "";
        if let Ok(unit) = LogicalUnit::new(
            self.repo.cloned(),
            Some(self.path.clone()),
            Some(n.try_into().unwrap()),
            Kind::Implementation,
            id,
            content,
        ) {
            Some(unit)
        } else {
            log::error!("unable to parse unit ID {}", id);
            None
        }
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

// For testing purposes
impl<'a> From<SourceFileKind> for SourceFile<'a> {
    fn from(kind: SourceFileKind) -> SourceFile<'a> {
        SourceFile::new(kind, Path::new("/"))
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

    // FIXME we can't currently parse out units followed by anchor
    #[ignore]
    #[test]
    fn can_parse_logical_unit_preceding_anchor() {
        let spec = r#"
|TAG.1::IN-ANCHOR-TAG.1| <a id="TAG.1::IN-TARGET.1"></a>
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

    #[test]
    fn can_prase_logical_units_from_rs_file() {
        let src = r#"
// 1
// 2
// 3
///4 |FOO.1::BAR.2::BAZ.1|
fn some_fun() {}
// 6

///8 |FLO.1::BOA.2::BOZ.1|
fn some_other_fun() {}
"#;
        let expected: HashSet<LogicalUnit> = vec![
            LogicalUnit::new(
                None,
                Some(Path::new("/")),
                Some(4),
                Kind::Implementation,
                "FOO.1::BAR.2::BAZ.1",
                "",
            )
            .unwrap(),
            LogicalUnit::new(
                None,
                Some(Path::new("/")),
                Some(8),
                Kind::Implementation,
                "FLO.1::BOA.2::BOZ.1",
                "",
            )
            .unwrap(),
        ]
        .iter()
        .cloned()
        .collect();

        let mut reader = io::Cursor::new(src);
        let actual = SourceFile::from(SourceFileKind::Rust)
            .units_of_src_reader(&mut reader)
            .unwrap();

        assert_eq!(actual, expected);
    }
}

use {
    crate::{repo::Repo, util},
    serde::{de, Deserialize, Deserializer, Serialize, Serializer},
    std::{
        fmt,
        path::{Path, PathBuf},
    },
};

///  |TRC-TAG.1::SYNTAX.1::IMPL.1|
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id {
    parts: Vec<(String, u32)>,
}

impl Id {
    pub fn new(s: &str) -> Result<Id, String> {
        let parts = util::parser::logical_unit_id(s).map_err(|_| "parsing id")?;
        Ok(Id { parts })
    }

    fn parts(&self) -> Vec<(String, u32)> {
        self.parts.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Kind {
    Requirement,
    Model,
    Implementation,
    Verification,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LogicalUnit {
    pub id: Id,
    pub kind: Kind,
    pub repo: Option<Repo>,
    pub file: Option<PathBuf>,
    pub line: Option<u64>,
    pub content: String,
    /// Logical units that are referred to in the content of this one
    pub references: Vec<Id>, // TODO
}

impl LogicalUnit {
    pub fn new(
        repo: Option<Repo>,
        file: Option<&Path>,
        line: Option<u64>,
        kind: Kind,
        id: String,
        content: String,
    ) -> Result<LogicalUnit, String> {
        let id = Id::new(&id)?;
        let file = file.map(|f| f.to_owned());
        let references = references_of_content(&content);
        Ok(LogicalUnit {
            id,
            kind,
            repo,
            file,
            line,
            content,
            references,
        })
    }

    /// `unit.synopsis()` is a tripplet `(tag, content, path)` with
    ///
    /// - the `unit`'s `tag`
    /// - the `path` of the `unit`'s `` repo
    /// - the `unit`'s `content`
    pub fn synopsis(&self) -> (String, String, String) {
        let tag = self.id.to_string();
        let content = self.content.replace("\n", " ");
        let repo = self.repo.clone().map_or("".into(), |r| r.path_as_string());
        (tag, repo, content)
    }
}

// TODO
fn references_of_content(_s: &str) -> Vec<Id> {
    vec![]
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl From<Id> for String {
    fn from(id: Id) -> Self {
        id.to_string()
    }
}

struct IdVisitor;

impl<'de> de::Visitor<'de> for IdVisitor {
    type Value = Id;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string that is a valid logical unit ID")
    }

    fn visit_str<E>(self, value: &str) -> Result<Id, E>
    where
        E: de::Error,
    {
        Id::new(value).map_err(|_| E::custom(format!("Invalid logical unit tag: {}", value)))
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Id, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(IdVisitor)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.parts()
                .iter()
                .map(|(tag, version)| format!("{}.{}", tag, version))
                .collect::<Vec<String>>()
                .join("::")
        )
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for LogicalUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = self
            .file
            .clone()
            .map(|p| p.as_path().display().to_string())
            .unwrap_or_else(|| "None".into());
        let repo = self
            .repo
            .clone()
            .map(|r| r.to_string())
            .unwrap_or_else(|| "None".into());
        write!(
            f,
            "LOGICAL-UNIT{{repo: {}, file: {}, id: {}, kind: {}, content: \"{}\"}}",
            repo, file, self.id, self.kind, self.content
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;

    #[test]
    fn json_serialize_id() {
        let id = Id::new("FOO.1::BAR.2::BAZ.3").unwrap();
        let actual = serde_json::to_string(&id).unwrap();
        let expected = "\"FOO.1::BAR.2::BAZ.3\"";
        assert_eq!(actual, expected);
    }

    #[test]
    fn json_deserialize_id() {
        let actual: Id = serde_json::from_str(&"\"FOO.1::BAR.2::BAZ.3\"").unwrap();
        let expected = Id::new("FOO.1::BAR.2::BAZ.3").unwrap();
        assert_eq!(actual, expected);
    }

    // TODO Add unit tests for serde
    // #[test]
    // fn csv_serialize_unit() {
    //     let actual = LogicalUnit::new(
    //         None,
    //         None,
    //         None,
    //         Kind::Requirement,
    //         "FOO.1".into(),
    //         "".into(),
    //     );
    //     let expected = ",,,Requirement,FOO1,,";
    //     assert_eq!(actual, expected);
    // }
}

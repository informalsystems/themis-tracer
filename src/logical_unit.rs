use crate::util;
use serde::{Serialize, Serializer};
use std::fmt;
use std::path::Path;

/// # Implements
///
/// * [TRC-TAG.1::SYNTAX.1]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Hash)]
pub enum Kind {
    Requirement,
    Model,
    Implementation,
    Test,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Hash)]
pub struct LogicalUnit {
    pub id: Id,
    pub kind: Kind,
    pub source_file: Option<String>,
    pub content: String,
    /// Logical units that are referred to in the content of this one
    #[serde(skip)]
    pub references: Vec<Id>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

impl LogicalUnit {
    pub fn new(
        path: Option<&Path>,
        kind: Kind,
        id: String,
        content: String,
    ) -> Result<LogicalUnit, String> {
        let id = Id::new(&id)?;
        let source_file = if let Some(p) = path {
            Some(p.to_str().ok_or("logical unit source file")?.to_owned())
        } else {
            None
        };
        let references = references_of_content(&content);
        Ok(LogicalUnit {
            id,
            kind,
            content,
            references,
            source_file,
            column: None, // TODO
            line: None,   // TODO
        })
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
        let source = self.source_file.clone().unwrap_or_else(|| "".to_string());
        write!(f, "{} {} {} <{}>", source, self.id, self.kind, self.content)
    }
}

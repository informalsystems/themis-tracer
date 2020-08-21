// use super::luid::LogicalUnitID;
use serde::{Serialize, Serializer};
use std::fmt;
use std::path::Path;

// Private Id type for type safe internal representation of Ids
// Ensures an id field cannot be created without going through the
// constructors that enforce validation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct Id {
    parts: Vec<(String, u32)>,
}

impl Id {
    pub fn new(s: &str) -> Result<Id, String> {
        let parts = parser::id(s).map_err(|_| "parsing id")?;
        Ok(Id { parts })
    }

    fn parts(&self) -> Vec<(String, u32)> {
        self.parts.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize)]
pub enum Kind {
    Requirement,
    Model,
    Implementation,
    Test,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Serialize)]
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
        p: Option<&Path>,
        kind: Kind,
        id: String,
        content: String,
    ) -> Result<LogicalUnit, String> {
        let id = Id::new(&id)?;
        let source_file = if let Some(p) = p {
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

peg::parser! {
    grammar parser() for str {
        pub rule id() -> Vec<(String, u32)> =
            id:(part() ** "::")
        { id }

        rule letter() -> String =
            l:$(['a'..='z' | 'A'..='Z' ])
        { l.to_string() }

        rule tag() -> String =
            t:$((letter() / ['_']) (letter() / ['0'..='9' | '-' | '_'])+)
        { t.to_string() }

        rule version() -> u32 =
            v:$(['1'..='9'] ['0'..='9']*)
        { v.parse().unwrap() }

        rule part() -> (String, u32) =
            t:tag() "."  v:version()
        { (t, v) }
    }
}

#[cfg(test)]
mod test_parser {
    use super::*;
    #[test]
    fn test_id() {
        assert_eq!(
            parser::id("FOO.1::BAR-BAZ.2::BING.3"),
            Ok(vec![
                ("FOO".to_string(), 1),
                ("BAR-BAZ".to_string(), 2),
                ("BING".to_string(), 3)
            ])
        )
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

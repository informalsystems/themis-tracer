// use super::luid::LogicalUnitID;
use std::fmt;
use std::path::Path;

// Private Id type for type safe internal representation of Ids
// Ensures an id field cannot be created without going through the
// constructors that enforce validation.
#[derive(Debug, Clone, PartialEq)]
struct Id_(Vec<(String, u32)>);

// The publically available type alias, allows clients to
// consume `Id`s as transpranet vecs of tupples.
pub type Id = Vec<(String, u32)>;

peg::parser! {
    grammar parser() for str {
        pub rule id() -> Id =
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

fn id_of_string(s: String) -> Result<Id_, String> {
    parser::id(&s)
        .map(Id_)
        .map_err(|_| "parsing id".to_string())
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Requirement,
    Model,
    Implementation,
    Test,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogicalUnit {
    id: Id_,
    pub kind: Kind,
    pub source_file: Option<String>,
    pub content: String,
    /// Logical units referred to
    pub references: Vec<Id>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

// TODO
fn references_of_content(_s: &str) -> Vec<Id> {
    vec![]
}

impl LogicalUnit {
    pub fn new(
        p: Option<&Path>,
        kind: Kind,
        id: String,
        content: String,
    ) -> Result<LogicalUnit, String> {
        let id = id_of_string(id)?;
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

    pub fn get_id(&self) -> Id {
        self.id.0.clone()
    }
}

impl fmt::Display for Id_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
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

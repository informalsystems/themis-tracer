//!
//! Logical unit ID management.
//!

use crate::{Error, Result};

peg::parser! {
    grammar luid_parser() for str {
        pub rule luid() -> LogicalUnitId
            = p0:luid_part() p:(luid_suffix_part()*) {
                let mut v = vec![p0];
                v.extend(p.iter().cloned());
                LogicalUnitId(v)
            }

        rule luid_part() -> LogicalUnitIdPart
            = tag:$(['a'..='z' | 'A'..='Z' | '_'] ['a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_']+) "." version:$(['1'..='9'] ['0'..='9']*) {
                LogicalUnitIdPart{ tag: String::from(tag), version: version.parse().unwrap() }
            }

        rule luid_suffix_part() -> LogicalUnitIdPart
            = "::" p:luid_part() { p }
    }
}

/// A fully qualified logical unit ID, e.g. "TRC-TAG.1::SYNTAX.1".
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct LogicalUnitId(Vec<LogicalUnitIdPart>);

/// A single part of a logical unit's ID, e.g. "TRC-REF.1".
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct LogicalUnitIdPart {
    /// The tag component of the logical unit ID part, e.g. "TRC-REF".
    pub tag: String,
    /// The version number associated with the logical unit ID part.
    pub version: u32,
}

impl LogicalUnitId {
    pub fn from_parts(parts: Vec<LogicalUnitIdPart>) -> LogicalUnitId {
        LogicalUnitId(parts)
    }
}

impl std::fmt::Display for LogicalUnitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|p| format!("{}", p))
                .collect::<Vec<String>>()
                .join("::")
        )
    }
}

impl std::str::FromStr for LogicalUnitId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        luid_parser::luid(s).map_err(|e| Error::LogicalUnitParseError(e.to_string()))
    }
}

impl std::fmt::Display for LogicalUnitIdPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.tag, self.version)
    }
}

impl Clone for LogicalUnitIdPart {
    fn clone(&self) -> Self {
        LogicalUnitIdPart {
            tag: self.tag.clone(),
            version: self.version,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_simple_luid_parsing() {
        assert_eq!(
            LogicalUnitId(vec![LogicalUnitIdPart {
                tag: "SPEC-INPUT".to_string(),
                version: 1
            },]),
            LogicalUnitId::from_str("SPEC-INPUT.1").unwrap(),
        );
    }

    #[test]
    fn test_complex_luid_parsing() {
        assert_eq!(
            LogicalUnitId(vec![
                LogicalUnitIdPart {
                    tag: "SPEC-INPUT".to_string(),
                    version: 1
                },
                LogicalUnitIdPart {
                    tag: "HELLO-WORLD".to_string(),
                    version: 2
                },
                LogicalUnitIdPart {
                    tag: "TO-SOMEONE".to_string(),
                    version: 3
                },
            ]),
            LogicalUnitId::from_str("SPEC-INPUT.1::HELLO-WORLD.2::TO-SOMEONE.3").unwrap(),
        );
    }
}

//!
//! Logical unit ID management.
//!

use crate::{Error, Result};

peg::parser! {
    grammar luid_parser() for str {
        pub rule luid() -> LogicalUnitID
            = p0:luid_part() p:(luid_suffix_part()*) {
                let mut v = vec![p0];
                v.extend(p.iter().cloned());
                LogicalUnitID(v)
            }

        rule luid_part() -> LogicalUnitIDPart
            = tag:$(['a'..='z' | 'A'..='Z' | '_'] ['a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_']+) "." version:$(['1'..='9'] ['0'..='9']*) {
                LogicalUnitIDPart{ tag: String::from(tag), version: version.parse().unwrap() }
            }

        rule luid_suffix_part() -> LogicalUnitIDPart
            = "::" p:luid_part() { p }
    }
}

/// A fully qualified logical unit ID, e.g. "TRC-TAG.1::SYNTAX.1".
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct LogicalUnitID(Vec<LogicalUnitIDPart>);

/// A single part of a logical unit's ID, e.g. "TRC-REF.1".
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct LogicalUnitIDPart {
    /// The tag component of the logical unit ID part, e.g. "TRC-REF".
    pub tag: String,
    /// The version number associated with the logical unit ID part.
    pub version: u32,
}

impl LogicalUnitID {
    pub fn from_parts(parts: Vec<LogicalUnitIDPart>) -> LogicalUnitID {
        LogicalUnitID(parts)
    }
}

impl std::fmt::Display for LogicalUnitID {
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

impl std::str::FromStr for LogicalUnitID {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        luid_parser::luid(s).map_err(|e| Error::LogicalUnitParseError(e.to_string()))
    }
}

impl std::fmt::Display for LogicalUnitIDPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.tag, self.version)
    }
}

impl Clone for LogicalUnitIDPart {
    fn clone(&self) -> Self {
        LogicalUnitIDPart {
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
            LogicalUnitID(vec![LogicalUnitIDPart {
                tag: "SPEC-INPUT".to_string(),
                version: 1
            },]),
            LogicalUnitID::from_str("SPEC-INPUT.1").unwrap(),
        );
    }

    #[test]
    fn test_complex_luid_parsing() {
        assert_eq!(
            LogicalUnitID(vec![
                LogicalUnitIDPart {
                    tag: "SPEC-INPUT".to_string(),
                    version: 1
                },
                LogicalUnitIDPart {
                    tag: "HELLO-WORLD".to_string(),
                    version: 2
                },
                LogicalUnitIDPart {
                    tag: "TO-SOMEONE".to_string(),
                    version: 3
                },
            ]),
            LogicalUnitID::from_str("SPEC-INPUT.1::HELLO-WORLD.2::TO-SOMEONE.3").unwrap(),
        );
    }
}

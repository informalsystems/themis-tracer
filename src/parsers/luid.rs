//!
//! Logical unit ID parsing.
//!

use crate::{Error, LogicalUnitID, LogicalUnitIDPart, Result};

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

pub fn parse_luid(s: &str) -> Result<LogicalUnitID> {
    luid_parser::luid(s).map_err(|e| Error::LogicalUnitParseError(e.to_string()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple_luid_parsing() {
        assert_eq!(
            LogicalUnitID(vec![LogicalUnitIDPart {
                tag: "SPEC-INPUT".to_string(),
                version: 1
            },]),
            parse_luid("SPEC-INPUT.1").unwrap(),
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
            parse_luid("SPEC-INPUT.1::HELLO-WORLD.2::TO-SOMEONE.3").unwrap(),
        );
    }
}

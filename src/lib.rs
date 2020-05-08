//!
//! Themis Tracer library interface.
//!

use std::collections::HashMap;
use failure::Fail;
use log::{info, debug, error};

mod parsers;

/// Themis Tracer's general result type.
pub type Result<T> = std::result::Result<T, Error>;

/// All possible errors that Themis Tracer can generate.
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "failed to parse logical unit: {}", _0)]
    LogicalUnitParseError(String),
}

/// A project is a collection of logical units and their implementations.
#[derive(Debug)]
pub struct Project {
    /// The supplied configuration for the project.
    pub config: ProjectConfig,
    /// A list of all of the discovered source files after processing the
    /// project configuration.
    pub source_files: Vec<ProjectSourceFile>,
    /// A mapping from fully qualified logical unit IDs to their respective
    /// logical units.
    pub logical_units: HashMap<String, LogicalUnit>,
    /// A mapping from fully qualified logical unit IDs to their respective
    /// implementations (where you could have more than one implementation
    /// referencing a logical unit).
    pub impl_units: HashMap<String, Vec<ImplUnit>>,
}

#[derive(Debug)]
pub struct ProjectSourceFile {
    /// The full path to the source file.
    pub filename: String,
    /// What kind of source file is this?
    pub kind: ProjectSourceFileKind,
}

/// The different kinds of project source files that we can process.
#[derive(Debug)]
pub enum ProjectSourceFileKind {
    /// Specifications can only contain logical units.
    Spec,
    /// Implementations can only contain implementation units.
    Impl,
    /// Dual specification/implementation files can contain both specifications
    /// and their implementations.
    DualSpecImpl,
}

#[derive(Debug)]
pub struct ProjectConfig {
    /// A descriptive name for this project.
    pub name: String,
    /// Where to find the various different components of the project.
    pub source_refs: Vec<ProjectFilesRef>,
}

#[derive(Debug)]
pub struct ProjectFilesRef {
    /// The source of this collection of files. This can be a local filesystem
    /// path, or a Git repository.
    pub source: String,
    /// An optional path glob to match specific files within this collection of
    /// files. If not specified, specifications match against "**/*.md" and
    /// implementations match against "src/**/*.rs".
    pub path: Option<String>,
}

/// A fully qualified logical unit ID, e.g. "TRC-TAG.1::SYNTAX.1".
#[derive(Debug, Eq, PartialEq)]
pub struct LogicalUnitID(Vec<LogicalUnitIDPart>);

/// A single part of a logical unit's ID, e.g. "TRC-REF.1".
#[derive(Debug, Eq, PartialEq)]
pub struct LogicalUnitIDPart {
    /// The tag component of the logical unit ID part, e.g. "TRC-REF".
    pub tag: String,
    /// The version number associated with the logical unit ID part.
    pub version: u32,
}

/// A logical (or conceptual) unit in our specification.
#[derive(Debug)]
pub struct LogicalUnit {
    /// The index of the source file (in the project files list) from which this
    /// logical unit was parsed. An index is used here to minimize storage
    /// space.
    pub source_file: u32,
    /// This logical unit's fully qualified ID.
    pub id: LogicalUnitID,
    /// The human-readable description of the logical unit. In future we should
    /// consider using the AST here instead of a plain string.
    pub desc: String,
}

/// An implementation unit for a specific logical unit.
#[derive(Debug)]
pub struct ImplUnit {
    /// The index of the source file (in the project files list) from which this
    /// implementation unit was parsed. An index is used here to minimize
    /// storage space.
    pub source_file: u32,
    /// The logical unit to which this implementation unit refers.
    pub id: LogicalUnitID,
    /// An optional line number for the code that makes up this implementation
    /// of the logical unit. This line number is present for references attached
    /// to specific methods or structs, whereas the line number will not be
    /// present for references attached to entire files (e.g. by way of inner
    /// line/block comments at the beginning of a Rust source file).
    pub line_no: Option<u32>,
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
        parsers::luid::parse_luid(s)
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
    use textwrap::dedent;

    const SIMPLE_SPEC: &str = r#"
    # Specification

    |SPEC-HELLO.1|
    :   When executed, the program must print out the text "Hello world!"
    "#;

    const MULTI_UNIT_SPEC: &str = r#"
    # Specification

    |SPEC-INPUT.1|
    :   When executed, the program must print the text: "Hello! What's your name?",
        and allow the user to input their name.
    
    |SPEC-HELLO.2|
    :   Once the user's name has been obtained, the program must print out the text
        "Hello {name}!", where `{name}` must be replaced by the name obtained in
        [SPEC-INPUT.1].
    "#;

    fn test_simple_spec_parsing() {}
}

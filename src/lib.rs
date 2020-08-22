//!
//! Themis Tracer library interface.
//!

pub mod cmd;
pub mod logical_unit;
mod luid;
mod pandoc;
mod specs;
mod util;

use failure::Fail;
use std::collections::HashMap;

pub use luid::{LogicalUnitID, LogicalUnitIDPart};
pub use specs::{ProjectSpecifications, SpecificationParseError};

/// Themis Tracer's general result type.
pub type Result<T> = std::result::Result<T, Error>;

/// All possible errors that Themis Tracer can generate.
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "failed to parse logical unit: {}", _0)]
    LogicalUnitParseError(String),
    #[fail(display = "failed to parse specification: {}", _0)]
    SpecificationParseError(SpecificationParseError),
    #[fail(display = "internal error: {}", _0)]
    InternalError(String),
}

/// A project is a collection of specifications and their implementations. We
/// separate the two because they are independent of each other, and the purpose
/// of this program (and this structure) is to validate the relationships
/// between the two.
#[derive(Debug)]
pub struct Project {
    /// The supplied configuration for the project.
    pub config: ProjectConfig,
    /// The specifications for this project.
    pub specifications: ProjectSpecifications,
    /// The implementation of this project's specifications, which can consist
    /// of many different files spread across many repositories.
    pub implementation: ProjectImplementation,
}

#[derive(Debug)]
pub struct ProjectImplementation {
    /// Implementations attached to specific logical units.
    pub lu_impls: HashMap<LogicalUnitID, Vec<ImplUnit>>,
    /// Dangling implementations without reference to specific logical units.
    pub dangling_impls: Vec<ImplUnit>,
}

#[derive(Debug, Clone)]
pub struct ProjectSourceFile {
    /// The full path to the source file.
    pub filename: String,
    /// What kind of source file is this?
    pub kind: ProjectSourceFileKind,
}

/// The different kinds of project source files that we can process.
#[derive(Debug, Clone)]
pub enum ProjectSourceFileKind {
    /// Specifications can only contain logical units.
    Spec,
    /// Implementations can only contain implementation units.
    Impl,
    /// Dual specification/implementation files can contain both specifications
    /// and their implementations.
    DualSpecImpl,
}

/// High-level configuration for a project.
#[derive(Debug)]
pub struct ProjectConfig {
    /// A short, human-readable name for this project.
    pub name: String,
    /// Where to find the various different components of the project.
    pub components: Vec<ProjectComponentRef>,
}

/// A reference to a component of a project, which is effectively just a named
/// collection of files. A component can contain both specifications and
/// implementations.
#[derive(Debug)]
pub struct ProjectComponentRef {
    /// A short, human-readable description of this component of a project.
    pub name: String,
    /// The source of this collection of files. This can be a local filesystem
    /// path, or a Git repository. The ideal is to have the source specified as
    /// a remote Git repository, and then a local mapping from that remote Git
    /// repo to a local folder on your local machine.
    pub source: String,
    /// An optional path glob to match specific files within this collection of
    /// files. If not specified, all visible folders will be scanned for both
    /// specifications and source code files.
    pub path: Option<String>,
}

/// A logical (or conceptual) unit from our specifications.
#[derive(Debug, Clone)]
pub struct LogicalUnit {
    /// The source file from which this logical unit was extracted.
    pub source_file: ProjectSourceFile,
    /// This logical unit's fully qualified ID.
    pub id: LogicalUnitID,
    /// The human-readable description of the logical unit. In future we should
    /// consider using the AST here instead of a plain string.
    pub desc: String,
}

/// An implementation unit from code.
#[derive(Debug)]
pub struct ImplUnit {
    /// The source file from which this implementation unit was extracted.
    pub source_file: ProjectSourceFile,
    /// The logical unit to which this implementation unit refers, if any.
    pub id: Option<LogicalUnitID>,
    /// An optional line number for the code that makes up this implementation
    /// of the logical unit. This line number is present for references attached
    /// to specific methods or structs, whereas the line number will not be
    /// present for references attached to entire files (e.g. by way of inner
    /// line/block comments at the beginning of a Rust source file).
    pub line_no: Option<u32>,
    /// The kind of visibility this implementation unit has from the perspective
    /// of an external user of the project as a whole.
    pub visibility: ImplUnitVisibility,
}

/// The visibility of some specific piece of implementation code. Right now we
/// just support public or private visibilities, but we may want to consider
/// more nuanced visibilities in future.
#[derive(Debug)]
pub enum ImplUnitVisibility {
    Public,
    Private,
}

impl Project {
    pub fn new(config: ProjectConfig) -> Result<Project> {
        let source_files = config.discover_source_files()?;
        let (specifications, implementation) = Project::parse(&source_files)?;
        Ok(Project {
            config,
            specifications,
            implementation,
        })
    }

    fn parse(
        _source_files: &[ProjectSourceFile],
    ) -> Result<(ProjectSpecifications, ProjectImplementation)> {
        let lu_impls = HashMap::<LogicalUnitID, Vec<ImplUnit>>::new();
        let dangling_impls = Vec::<ImplUnit>::new();
        // TODO: Parse specifications and implementations to build the above variables
        Ok((
            ProjectSpecifications::new(),
            ProjectImplementation {
                lu_impls,
                dangling_impls,
            },
        ))
    }
}

impl ProjectConfig {
    /// Uses the project configuration to scan all file references to discover
    /// project source files.
    pub fn discover_source_files(&self) -> Result<Vec<ProjectSourceFile>> {
        let source_files = Vec::<ProjectSourceFile>::new();
        // TODO: Scan the configured components' sources
        Ok(source_files)
    }
}

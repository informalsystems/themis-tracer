//! CLI specification
use crate::cmd;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "tracer")]
/// Weaving together the context for critical systems
pub enum Cmd {
    /// Parse logical units out of a spec
    Parse {
        /// The file or directory to parse
        #[structopt(parse(from_os_str))]
        path: PathBuf,
        #[structopt(short, long, default_value, parse(try_from_str))]
        format: cmd::format::Format,
    },

    /// Linkify a markdown file
    ///
    /// This command anchors logical unit definitions and links logical unit
    /// references in a markdown file, modifying the file in place.
    Linkify {
        /// Paths to the the files to linkify
        paths: Vec<PathBuf>,
    },

    /// Update the curren context
    ///
    /// Update the logical units for the current context by rescanning all
    /// assocaited repositories.
    Sync {},

    /// Initialize tracer
    ///
    /// Defaults to initializing in your home directory. Set `TRACER_HOME` to
    /// override.
    Init {},

    /// Manage contexts
    Context(Context),

    /// Manage repositories
    Repo(Repo),

    /// Manage logical units
    Unit(Unit),

    /// Generate a dot graph of the current context
    Graph {
        /// Format can be dot or svg
        #[structopt(short, long, default_value, parse(try_from_str))]
        format: cmd::format::dot::Format,
    },
}

#[derive(Debug, StructOpt)]
pub struct Context {
    #[structopt(subcommand)]
    pub cmd: ContextCmd,
}

#[derive(Debug, StructOpt)]
pub enum ContextCmd {
    /// Createa a new context
    New {
        /// The name of the context
        name: String,
    },

    /// List all available contexts
    List {},

    /// Switch to a different context
    Switch {
        /// The name of the context to switch to
        name: String,
    },
}

#[derive(Debug, StructOpt)]
pub struct Repo {
    #[structopt(subcommand)]
    pub cmd: RepoCmd,
}

#[derive(Debug, StructOpt)]
pub enum RepoCmd {
    /// List all the repos registered to the current context
    List {},

    /// Add a repoistory to the current context
    Add {
        /// The path to the repo to be added
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
}

#[derive(Debug, StructOpt)]
pub struct Unit {
    #[structopt(subcommand)]
    pub cmd: UnitCmd,
}

#[derive(Debug, StructOpt)]
pub enum UnitCmd {
    /// List the specs registered to the current context
    List {
        // TODO
        // Search criteria to filter out listed spec results
        // filter: Option<String>,
        /// Serialization to use when listing the units.
        ///
        /// When absent, the output is a tab delimited readible with a synopsis
        /// of each unit, optimized for human readability.
        #[structopt(short, long, parse(try_from_str))]
        format: Option<cmd::format::Format>,
    },

    Show {
        /// The tag of the unit to display
        tag: String,

        /// Serialization to use when showing the unit units.
        ///
        /// When absent, the output is a tab delimited view of the unit,
        /// optimized for human readability.
        #[structopt(short, long, parse(try_from_str))]
        format: Option<cmd::format::Format>,
    },
}

// FIXME
fn unimplemented() -> Result<()> {
    Err(anyhow!("{}", "Not yet implemented!"))
}

pub fn run() -> Result<()> {
    let opt = Cmd::from_args();
    match opt {
        Cmd::Context(opt) => cmd::context::run(opt),
        Cmd::Init {} => cmd::init::run(),
        Cmd::Linkify { paths } => cmd::linkify::run(&paths),
        Cmd::Parse { path, format } => cmd::parse::run(&path, format),
        Cmd::Repo(opt) => cmd::repo::run(opt),
        Cmd::Sync {} => cmd::sync::run(),
        Cmd::Unit(opt) => cmd::unit::run(opt),
        Cmd::Graph { format } => cmd::graph::run(format),
    }
}

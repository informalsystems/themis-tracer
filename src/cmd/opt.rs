//! CLI specification
use crate::cmd;
use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "kontxt")]
/// Weaving together the context for critical systems
pub enum Cmd {
    /// Initialize kontxt
    ///
    /// Defaults to initializing in your home directory. Set `TRACER_HOME` to
    /// override.
    Init {},

    /// Manage contexts
    #[structopt(flatten)]
    Context(Context),

    /// File operations
    File(File),

    /// Repository management
    Repo(Repo),

    /// Logical unit management
    Unit(Unit),

    /// Context views and reports
    Generate(Generate),
}

#[derive(Debug, StructOpt)]
pub enum Context {
    /// Context creation
    New {
        /// The name of the context
        name: String,
    },
    /// Context synchronization
    ///
    /// Update the logical units for the current context by rescanning all
    /// assocaited repositories.
    Sync {},

    /// Context listing
    List {},

    /// Context switching
    Switch {
        /// The name of the context to switch to
        name: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum File {
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
}

#[derive(Debug, StructOpt)]
pub enum Repo {
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
pub enum Unit {
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

#[derive(Debug, StructOpt)]
pub enum Generate {
    /// Generate a dot graph of the current context
    Graph {
        /// Format can be dot or svg
        #[structopt(short, long, default_value, parse(try_from_str))]
        format: cmd::format::dot::Format,
    },

    /// Generate an HTML site summarizing the current context
    Site {},
}

pub fn run() -> Result<()> {
    let opt = Cmd::from_args();

    cmd::init::ensured()?;

    match opt {
        Cmd::Init {} => cmd::init::run(),
        Cmd::Context(ctxt) => cmd::context::run(ctxt),
        Cmd::Repo(opt) => cmd::repo::run(opt),
        Cmd::Unit(opt) => cmd::unit::run(opt),
        Cmd::File(file) => cmd::file::run(file),
        // TODO Clean up
        Cmd::Generate(Generate::Graph { format }) => cmd::graph::run(format),
        Cmd::Generate(Generate::Site {}) => cmd::site::run(),
    }
}

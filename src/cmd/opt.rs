//! CLI specification
use crate::cmd;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "tracer",
    about = "Spining threads of signifcance for the context of critical systems"
)]
pub enum Cmd {
    /// Parse logical units out of a spec
    #[structopt(name = "parse")]
    Parse {
        /// The file or directory to parse
        #[structopt(parse(from_os_str))]
        path: PathBuf,
        #[structopt(short, long, default_value, parse(try_from_str))]
        format: cmd::parse::Format,
    },

    /// List registered specs.
    #[structopt(name = "list")]
    List {
        /// Search criteria to filter out listed spec results
        filter: Option<String>,
    },

    /// Register specs
    #[structopt(name = "add")]
    Add {
        /// The path to load sepcs from (will recursce into all sudirectories)
        #[structopt(parse(from_os_str))]
        project: Option<PathBuf>,
    },

    /// Update the spec DB for the current project with all specs from registered sources
    #[structopt(name = "sync")]
    Sync {
        /// The project whose db should be updated
        #[structopt(parse(from_os_str))]
        project: Option<PathBuf>,
    },

    /// Initialize tracer
    ///
    /// Defaults to initializing in your home directory. Set `TRACER_HOME` to
    /// override.
    #[structopt(name = "init")]
    Init {},

    /// Manage contexts
    Context(Context),
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
}

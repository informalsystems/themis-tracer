//! CLI specification
use crate::cmd;
use anyhow::{anyhow, Result};
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

    /// Manage repositories
    #[structopt(name = "repo")]
    Repo(Repo),
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

// FIXME
fn unimplemented() -> Result<()> {
    Err(anyhow!("{}", "Not yet implemented!"))
}

pub fn run() -> Result<()> {
    let opt = Cmd::from_args();
    match opt {
        Cmd::Init {} => cmd::init::run(),
        Cmd::Context(opt) => cmd::context::run(opt),
        Cmd::Repo(opt) => cmd::repo::run(opt),
        Cmd::Parse { path, format } => cmd::parse::run(&path, format),
        Cmd::Sync { project: _ } => unimplemented(),
        Cmd::List { filter: _ } => unimplemented(),
    }
}

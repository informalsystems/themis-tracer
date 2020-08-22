use std::path::PathBuf;
use structopt::StructOpt;
use tracer::cmd;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "themis-tracer",
    about = "Requirements traceability made easier"
)]
enum Opt {
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
}

// FIXME
fn unimplemented() -> Result<(), String> {
    Err("Not yet implemented!".to_string())
}

fn main() -> Result<(), String> {
    let opt = Opt::from_args();
    match opt {
        Opt::Parse { path, format } => cmd::parse::run(&path, format),
        Opt::List { filter: _ } => unimplemented(),
        Opt::Add { project: _ } => unimplemented(),
        Opt::Sync { project: _ } => unimplemented(),
    }
}

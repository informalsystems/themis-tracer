use structopt::StructOpt;
use tracer::{cmd, cmd::opt::Cmd};

// FIXME
fn unimplemented() -> Result<(), String> {
    Err("Not yet implemented!".to_string())
}

// TODO Replace String with error type
fn main() -> Result<(), String> {
    let opt = Cmd::from_args();
    match opt {
        Cmd::Init {} => cmd::init::run(),
        Cmd::Parse { path, format } => cmd::parse::run(&path, format),
        Cmd::List { filter: _ } => unimplemented(),
        Cmd::Add { project: _ } => unimplemented(),
        Cmd::Sync { project: _ } => unimplemented(),
        Cmd::Context(opt) => cmd::context::run(opt),
    }
}

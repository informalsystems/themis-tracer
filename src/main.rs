use {
    anyhow::{anyhow, Result},
    structopt::StructOpt,
    tracer::{cmd, cmd::opt::Cmd},
};

// FIXME
fn unimplemented() -> Result<()> {
    Err(anyhow!("{}", "Not yet implemented!"))
}

// TODO Replace String with error type
fn main() -> Result<()> {
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

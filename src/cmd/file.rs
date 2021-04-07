use {
    crate::{cmd, cmd::opt},
    anyhow::Result,
};

pub fn run(cmd: opt::File) -> Result<()> {
    match cmd {
        opt::File::Linkify { paths } => cmd::linkify::run(&paths),
        opt::File::Parse { path, format } => cmd::parse::run(&path, format),
    }
}

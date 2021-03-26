//! Interface to the dot executable

use {
    anyhow::Result,
    std::{
        io,
        io::Write,
        process::{Command, Stdio},
    },
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invoking dot")]
    Invocation(#[from] io::Error),
    #[error("Parsing destination path")]
    Path,
}

static DOT: &str = "dot";

pub(crate) fn to_svg(graph: &str) -> Result<()> {
    let process = Command::new(DOT)
        .args(&["-T", "svg"])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(Error::Invocation)?;

    let mut stdin = process.stdin.unwrap();
    stdin.write_all(graph.as_bytes())?;
    Ok(())
}

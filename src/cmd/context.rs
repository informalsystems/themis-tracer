use crate::cmd::{opt, opt::ContextCmd as Cmd};
// use crate::context::Context;

// FIXME
#[allow(clippy::unnecessary_wraps)]
fn new(_name: String) -> Result<(), String> {
    Ok(())
}

pub fn run(opt: opt::Context) -> Result<(), String> {
    match opt.cmd {
        Cmd::New { name } => new(name),
        Cmd::List {} => Ok(()),
    }
}

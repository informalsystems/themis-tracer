use crate::{
    cmd::{init, opt, opt::ContextCmd as Cmd},
    context::Context,
};
// use crate::context::Context;

// FIXME
#[allow(clippy::unnecessary_wraps)]
fn new(name: String) -> Result<(), String> {
    Context::new(&name).map(|_| ())?;
    println!("Created the new context {}", name);
    Ok(())
}

pub fn run(opt: opt::Context) -> Result<(), String> {
    init::ensured()?;
    match opt.cmd {
        Cmd::New { name } => new(name),
        Cmd::List {} => Ok(()),
    }
}

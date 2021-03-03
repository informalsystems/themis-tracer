use {
    crate::{
        cmd::{init, opt, opt::ContextCmd as Cmd},
        context::Context,
        db,
    },
    anyhow::{Context as AnyhowContext, Result},
};
// use crate::context::Context;

// FIXME
#[allow(clippy::unnecessary_wraps)]
fn new(name: String) -> Result<()> {
    let conn = db::connection()?;
    db::context::add(&conn, Context::new(&name))?;
    println!("Created the context `{}`", name);
    Ok(())
}

fn list() -> Result<()> {
    let conn = db::connection()?;
    let mut ctxs: Vec<String> = db::context::get_all(&conn)?
        .iter()
        .map(|c| c.name.clone())
        .collect();
    ctxs.sort();
    for ctx in ctxs {
        println!("  {}", ctx)
    }

    Ok(())
}

pub fn run(opt: opt::Context) -> Result<()> {
    init::ensured().context("Running `context` subcommand")?;
    match opt.cmd {
        Cmd::New { name } => new(name),
        Cmd::List {} => list(),
    }
}

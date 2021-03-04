use {
    crate::{
        cmd::{init, opt, opt::ContextCmd as Cmd},
        context::Context,
        db,
    },
    anyhow::{Context as AnyhowContext, Result},
    thiserror::Error,
};
// use crate::context::Context;

#[derive(Error, Debug)]
pub enum Error {
    #[error("A context named {0} already exists")]
    ContextExists(String),
}

// FIXME
#[allow(clippy::unnecessary_wraps)]
fn new(name: String) -> Result<()> {
    let conn = db::connection()?;
    match db::context::add(&conn, Context::new(name.clone())) {
        Ok(()) => Ok(()),
        Err(err) => {
            if err
                .to_string()
                .contains("UNIQUE constraint failed: context.name")
            {
                Err(Error::ContextExists(name).into())
            } else {
                Err(err)
            }
        }
    }
}

fn list() -> Result<()> {
    let conn = db::connection()?;
    let current_ctx = db::context::current(&conn)?;
    let mut ctxs: Vec<Context> = db::context::get_all(&conn)?;
    ctxs.sort();

    // TODO Cleanup
    if let Some(current) = current_ctx {
        for ctx in ctxs {
            if ctx == current {
                println!("* {}", ctx)
            } else {
                println!("  {}", ctx)
            }
        }
    } else {
        for ctx in ctxs {
            println!("  {}", ctx)
        }
    }
    Ok(())
}

fn switch(name: String) -> Result<()> {
    let conn = db::connection()?;
    db::context::set(&conn, name)
}

pub fn run(opt: opt::Context) -> Result<()> {
    init::ensured().context("Running `context` subcommand")?;
    match opt.cmd {
        Cmd::New { name } => new(name),
        Cmd::List {} => list(),
        Cmd::Switch { name } => switch(name),
    }
}

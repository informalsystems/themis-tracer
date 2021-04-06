use {
    crate::{cmd, context::Context, db},
    anyhow::Result,
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
    // TODO Clean up to remove pointless match?
    match db::context::add(&conn, Context::new(name.clone())) {
        Ok(()) => db::context::set(&conn, name),
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

pub fn run(ctx: cmd::opt::Context) -> Result<()> {
    match ctx {
        cmd::opt::Context::Init {} => cmd::init::run(),
        cmd::opt::Context::Sync {} => cmd::sync::run(),
        cmd::opt::Context::New { name } => new(name),
        cmd::opt::Context::List {} => list(),
        cmd::opt::Context::Switch { name } => switch(name),
    }
}

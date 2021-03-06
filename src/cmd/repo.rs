use {
    crate::{cmd::opt, db, repo::Repo},
    anyhow::Result,
    std::{fs, path::PathBuf},
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("No repo found at path {0}")]
    RepoNotFound(PathBuf),
}

fn list() -> Result<()> {
    let conn = db::connection()?;
    let mut repos: Vec<Repo> = db::repo::get_all_in_context(&conn)?;
    repos.sort();

    for ctx in repos {
        println!("  {}", ctx)
    }

    Ok(())
}

// TODO Add support for setting default branch and upstream
fn add(path: PathBuf) -> Result<()> {
    let path = fs::canonicalize(path)?;
    if !path.exists() {
        Err(Error::RepoNotFound(path).into())
    } else {
        let conn = db::connection()?;
        let repo = Repo::new_local(path);
        db::repo::add(&conn, &repo)
    }
}

pub fn run(opt: opt::Repo) -> Result<()> {
    match opt.cmd {
        opt::RepoCmd::List {} => list(),
        opt::RepoCmd::Add { path } => add(path),
    }
}

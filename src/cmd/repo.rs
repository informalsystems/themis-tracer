use {
    crate::{artifact::Artifact, cmd::opt, db, repo::Repo},
    anyhow::Result,
    glob::glob,
    rusqlite as sql,
    std::{
        env, fs,
        path::{Path, PathBuf},
    },
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("No repo found at path {0}")]
    RepoNotFound(PathBuf),
    #[error("The repo {0} is already registered in the current context")]
    RepoExists(Repo),
}

fn list() -> Result<()> {
    let conn = db::connection()?;
    let mut repos: Vec<Repo> = db::repo::get_all_in_context(&conn)?;
    repos.sort();

    for repo in repos {
        println!("  {}", repo)
    }

    Ok(())
}

fn load_units_from_file(conn: &sql::Connection, repo: &Repo, path: &Path) -> Result<()> {
    Artifact::from_file(Some(repo.clone()), path)?
        .logical_units
        .iter()
        .try_for_each(|unit| db::unit::add(conn, repo, unit))
}

fn load_units_from_repo(conn: &sql::Connection, repo: &Repo) -> Result<()> {
    env::set_current_dir(repo.path())?;
    // TODO Support more than just MD files
    for path in glob("**/*.md")? {
        let path = path?;
        load_units_from_file(conn, repo, &(path))?
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
        match db::repo::add(&conn, &repo) {
            // You'd think I could use a `map_err` here, but I can't for a
            // reason I don't want to burn time unraveling at the moment.
            Ok(()) => Ok(()),
            Err(err) => {
                if err
                    .to_string()
                    .contains("UNIQUE constraint failed: repo.path")
                {
                    Err(Error::RepoExists(repo.clone()).into())
                } else {
                    Err(err)
                }
            }
        }?;
        load_units_from_repo(&conn, &repo)
    }
}

pub fn run(opt: opt::Repo) -> Result<()> {
    match opt.cmd {
        opt::RepoCmd::List {} => list(),
        opt::RepoCmd::Add { path } => add(path),
    }
}

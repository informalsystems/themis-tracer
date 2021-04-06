use {
    crate::{artifact::Artifact, cmd::opt, db, graph, locations, repo::Repo},
    anyhow::Result,
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
    Artifact::from_file(Some(repo), path)?
        .logical_units
        .iter()
        .try_for_each(|unit| db::unit::add(conn, repo, unit))
}

pub fn load_units_from_repo(conn: &sql::Connection, repo: &Repo) -> Result<()> {
    env::set_current_dir(repo.path())?;
    for path in locations::find_all_supported_source_files(&repo.path())? {
        let path = path.strip_prefix(repo.path())?;
        load_units_from_file(conn, repo, &path)?
    }
    // Build the graph to check for orphan units
    let units = db::unit::get_all_in_context(&conn)?;
    graph::of_units(&units);
    Ok(())
}

// TODO Add support for setting default branch and upstream
fn add(path: PathBuf) -> Result<()> {
    let path = fs::canonicalize(path)?;
    if !path.exists() {
        Err(Error::RepoNotFound(path).into())
    } else {
        let conn = db::connection()?;
        let repo = Repo::new_local(path)?;
        match db::repo::add(&conn, &repo) {
            // You'd think I could use a `map_err` here, but I can't for a
            // reason I don't want to burn time unraveling at the moment.
            Ok(()) => Ok(()),
            Err(err) => {
                if err
                    .to_string()
                    .contains("UNIQUE constraint failed: context_repo.context, context_repo.repo")
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

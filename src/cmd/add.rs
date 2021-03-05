use {
    crate::{db, repo::Repo},
    anyhow::Result,
    std::fs,
    std::path::PathBuf,
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("No repo found at path {0}")]
    RepoNotFound(PathBuf),
}

// TODO Add support for setting default branch and upstream
pub fn run(path: PathBuf) -> Result<()> {
    let path = fs::canonicalize(path)?;
    if !path.exists() {
        Err(Error::RepoNotFound(path).into())
    } else {
        let conn = db::connection()?;
        let repo = Repo::new_local(path);
        db::repo::add(&conn, &repo)
    }
}

use {
    crate::{db, repo::Repo},
    anyhow::Result,
    // std::path::PathBuf,
    // thiserror::Error,
};

pub fn run() -> Result<()> {
    let conn = db::connection()?;
    let mut repos: Vec<Repo> = db::repo::get_all_in_context(&conn)?;
    repos.sort();

    for ctx in repos {
        println!("  {}", ctx)
    }

    Ok(())
}

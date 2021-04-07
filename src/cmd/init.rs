//! Initialize new contexts

use {
    crate::{db, locations},
    anyhow::Result,
    rusqlite as sql,
    std::{fs, path::PathBuf},
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum InitError {
    #[error("Cannot initialize tracer home dir {0}")]
    Home(String),
    #[error("Already initialized in {0}")]
    Exists(PathBuf),
}

/// Returns `Ok(conn)` if the db connection `conn` can be iniated with the app's
/// db. If needed, the config dir is created.
pub(super) fn ensured() -> Result<sql::Connection> {
    let dir = locations::tracer_dir()?;
    if dir.exists() {
        // If the directory exists, we assume proper initialization
        let conn = db::connection()?;
        Ok(conn)
    } else {
        fs::create_dir_all(dir.clone()).map_err(|e| InitError::Home(e.to_string()))?;
        let location = dir
            .into_os_string()
            .into_string()
            .unwrap_or_else(|_| "<cannot be displayed>".into());
        let conn = db::connection()?;
        db::init(&conn)?;
        println!("Initialized into {}", location);
        Ok(conn)
    }
}

/// Run the tool initializion program
pub fn run() -> Result<()> {
    let _ = ensured()?;
    Ok(())
}

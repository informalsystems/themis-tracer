//! Constants and derived values recording location in computer space.
//! Dirs, URLs, etc.

use {
    crate::envvar,
    anyhow::{Context, Result},
    std::path::PathBuf,
    thiserror::Error,
};

#[derive(Error, Debug)]
enum Error {
    #[error("Cannot identify home directory")]
    NoHome,
}

pub const WORKSITE_FILE_NAME: &str = ".tracer";
pub const CONTEXTS_DIR_NAME: &str = "contexts";

// TODO DOcument TRACER_HOME var
/// The directory used as HOME for tracer
pub fn tracer_home() -> Option<PathBuf> {
    match std::env::var_os(envvar::TRACER_HOME) {
        None => home::home_dir(),
        Some(dir) => Some(PathBuf::from(dir)),
    }
}

/// The directory used for storing local data
pub fn tracer_dir() -> Result<PathBuf> {
    let mut path = tracer_home()
        .ok_or_else(|| Error::NoHome)
        .context("looking from $HOME or a value set by $TRACER_HOME")?;
    path.push(WORKSITE_FILE_NAME);
    Ok(path)
}

/// Directory in which contexts are stored
pub fn contexts_dir() -> Result<PathBuf> {
    let mut path = tracer_dir()?;
    path.push(CONTEXTS_DIR_NAME);
    Ok(path)
}

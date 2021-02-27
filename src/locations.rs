//! Constants and derived values recording location in computer space.
//! Dirs, URLs, etc.

use crate::envvar;
use std::path::PathBuf;

pub const WORKSITE_FILE_NAME: &str = ".tracer";
pub const CONTEXTS_DIR_NAME: &str = "contexts";

/// The directory used as HOME for tracer
pub fn tracer_home() -> Option<PathBuf> {
    match std::env::var_os(envvar::TRACER_HOME) {
        None => home::home_dir(),
        Some(dir) => Some(PathBuf::from(dir)),
    }
}

/// The directory used for storing local data
pub fn tracer_dir() -> Result<PathBuf, String> {
    let mut path = tracer_home().ok_or_else(|| "Cannot identify home directory".to_string())?;
    path.push(WORKSITE_FILE_NAME);
    Ok(path)
}

/// Directory in which contexts are stored
pub fn contexts_dir() -> Result<PathBuf, String> {
    let mut path = tracer_dir()?;
    path.push(CONTEXTS_DIR_NAME);
    Ok(path)
}

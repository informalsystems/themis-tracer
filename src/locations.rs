//! Constants and derived values recording location in computer space.
//! Dirs, URLs, etc.

use crate::envvar;
use std::path::PathBuf;

pub const WORKSITE_FILE_NAME: &str = ".tracer";

/// The directory used as HOME for tracer
pub fn tracer_home() -> Option<PathBuf> {
    match std::env::var_os(envvar::TRACER_HOME) {
        None => home::home_dir(),
        Some(dir) => Some(PathBuf::from(dir)),
    }
}

/// The directory used for storing local data
pub fn tracer_dir() -> Result<PathBuf, String> {
    let mut home_dir = tracer_home().ok_or_else(|| "Cannot identify home directory".to_string())?;
    home_dir.push(WORKSITE_FILE_NAME);
    Ok(home_dir)
}

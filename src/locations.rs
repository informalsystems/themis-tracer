//! Constants and derived values recording location in computer space.
//! Dirs, URLs, etc.

use {
    crate::{artifact, envvar},
    anyhow::{Context, Result},
    std::{
        convert::TryFrom,
        path::{Path, PathBuf},
    },
    thiserror::Error,
    walkdir,
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
        .ok_or(Error::NoHome)
        .context("looking from $HOME or a value set by $TRACER_HOME")?;
    path.push(WORKSITE_FILE_NAME);
    Ok(path)
}

pub fn tracer_db() -> Result<PathBuf> {
    tracer_dir().map(|p| p.join("tracer.db"))
}

/// Directory in which contexts are stored
pub fn contexts_dir() -> Result<PathBuf> {
    let mut path = tracer_dir()?;
    path.push(CONTEXTS_DIR_NAME);
    Ok(path)
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn source_file_is_supported(entry: &walkdir::DirEntry) -> bool {
    !is_hidden(entry) && artifact::SourceFileKind::try_from(entry.path()).is_ok()
}

pub fn find_all_supported_source_files(p: &Path) -> Result<Vec<PathBuf>> {
    // FIXME Avoid the tripper into iter
    Ok(walkdir::WalkDir::new(p)
        .into_iter()
        .collect::<walkdir::Result<Vec<walkdir::DirEntry>>>()? // Bail if we hit any errors
        .into_iter()
        .filter(source_file_is_supported)
        .into_iter()
        .map(|e| e.into_path())
        .collect())
}

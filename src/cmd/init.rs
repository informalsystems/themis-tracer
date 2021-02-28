//! Initialize new contexts

use {
    crate::locations,
    anyhow::Result,
    // sled,
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

/// Returns `Ok(Some(path))` if already inited to `path`, `Ok(None)` if
/// initalized on call, and `Err(msg)` if something goes wrong during
/// initialization.
pub(super) fn ensured() -> Result<Option<PathBuf>> {
    let dir = locations::tracer_dir()?;
    if dir.exists() {
        // If the directory exists, we assume proper initialization
        Ok(Some(dir))
    } else {
        let contexts_dir = locations::contexts_dir()?;
        fs::create_dir_all(contexts_dir).map_err(|e| InitError::Home(e.to_string()))?;
        match dir.into_os_string().into_string() {
            Ok(fname) => println!("Initialized tracer to {}", fname),
            Err(_) => {
                // TODO output warning
                println!("Initialization succeeded but the home location cannot be dispalyed.")
            }
        }
        Ok(None)
    }
}

// TODO Cleanup
/// Run the tool initializion program
pub fn run() -> Result<()> {
    if let Some(dir) = ensured()? {
        Err(InitError::Exists(dir).into())
    } else {
        Ok(())
    }
}

//! Initialize new contexts

use crate::locations;
use std::{fs, path::PathBuf};

/// Returns `Ok(Some(path))` if already inited to `path`, `Ok(None)` if
/// initalized on call, and `Err(msg)` if something goes wrong during
/// initialization.
pub(super) fn ensured() -> Result<Option<PathBuf>, String> {
    let dir = locations::tracer_dir()?;
    if dir.exists() {
        Ok(Some(dir))
    } else {
        let contexts_dir = locations::contexts_dir()?;
        fs::create_dir_all(contexts_dir).map_err(|e| format!("{:?}", e))?;
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
pub fn run() -> Result<(), String> {
    if let Some(dir) = ensured()? {
        let msg = match dir.into_os_string().into_string() {
            Err(_) => "Tracer is already initialized. But the the location cannot be displayed."
                .to_string(),
            Ok(dirname) => format!("Tracer is already initialized in {}", dirname),
        };
        Err(msg)
    } else {
        Ok(())
    }
}

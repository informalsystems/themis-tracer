//! Initialize new contexts

use std::fs;
use std::path::PathBuf;

use crate::constants::CONFIG_FILE_NAME;

/// Run the context initulize program
pub fn run(name: PathBuf) -> Result<(), String> {
    if name.exists() {
        let msg = format!("A file or directory named {:?} already exists", name);
        Err(msg)
    } else {
        fs::create_dir_all(name.clone()).map_err(|e| format!("{:?}", e))?;
        fs::File::create(name.join(CONFIG_FILE_NAME)).map_err(|e| e.to_string())?;
        match name.into_os_string().into_string() {
            Ok(fname) => println!("Initialized a new context in {}", fname),
            Err(_) => println!("Initialized a new context"),
        }
        Ok(())
    }
}

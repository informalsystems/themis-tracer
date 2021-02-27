//! Initialize new contexts

use crate::locations;
use std::fs;

// TODO Cleanup
/// Run the tool initializion program
pub fn run() -> Result<(), String> {
    let tracer_dir = locations::tracer_dir()?;
    if tracer_dir.exists() {
        let msg = match tracer_dir.into_os_string().into_string() {
            Err(_) => "Tracer is already initialized. But the the location cannot be displayed."
                .to_string(),
            Ok(dirname) => format!("Tracer is already initialized in {}", dirname),
        };
        Err(msg)
    } else {
        let contexts_dir = locations::contexts_dir()?;
        fs::create_dir_all(contexts_dir).map_err(|e| format!("{:?}", e))?;
        match tracer_dir.into_os_string().into_string() {
            Ok(fname) => println!("Initialized tracer to {}", fname),
            Err(_) => {
                // TODO output warning
                println!("Initialization succeeded but the home location cannot be dispalyed.")
            }
        }
        Ok(())
    }
}

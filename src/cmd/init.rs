//! Initialize new contexts

use crate::locations;
use std::fs;

// TODO Cleanup
/// Run the tool initializion program
pub fn run() -> Result<(), String> {
    let whorl_dir = locations::whorl_dir()?;
    if whorl_dir.exists() {
        let msg = match whorl_dir.into_os_string().into_string() {
            Err(_) => "Whorl is already initialized. But the the location cannot be displayed."
                .to_string(),
            Ok(dirname) => format!("Whorl is already initialized in {}", dirname),
        };
        Err(msg)
    } else {
        let contexts_dir = whorl_dir.join("contexts");
        fs::create_dir_all(contexts_dir).map_err(|e| format!("{:?}", e))?;
        match whorl_dir.into_os_string().into_string() {
            Ok(fname) => println!("Initialized whorl to {}", fname),
            Err(_) => {
                // TODO output warning
                println!("Initialization succeeded but the home location cannot be dispalyed.")
            }
        }
        Ok(())
    }
}

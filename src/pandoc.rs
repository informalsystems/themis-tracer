//!
//! Interface to the pandoc CLI
//!

use pandoc_ast::Pandoc;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

static PANDOC: &str = "pandoc";
static ARGS: &'static [&str] = &["--standalone", "--from", "markdown-smart", "--to", "json"];

// TODO
type Result = std::result::Result<Pandoc, String>;

fn pandoc_from_bytes(b: &[u8]) -> Result {
    serde_json::from_str(String::from_utf8_lossy(b).as_ref()).map_err(|_| "josn error".to_string())
}

/// Returns an [`Ok`] [`Pandoc`] value if the string can be parsed into the
/// pandoc AST, otherwise returns an [`Err`] with a string explaining the
/// failure.
pub fn parse_string(s: String) -> Result {
    let process = Command::new(PANDOC)
        .args(ARGS)
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|_| "spawning panodc process".to_string())?;

    // TODO
    process
        .stdin
        .unwrap()
        .write_all(s.as_bytes())
        .map_err(|_| "writing to pandoc process")?;

    let mut bytes = Vec::new();
    process
        .stdout
        .ok_or("receiving pandoc process output")
        .and_then(|mut c| {
            c.read_to_end(&mut bytes)
                .or(Err("reading from pandoc process"))
        })?;

    pandoc_from_bytes(&bytes)
}

/// Renders a pandoc AST into a markdown string
// pub fn render_ast(p: &Pandoc) -> std::result::Result<String, String> {
//     let process = Command::new(PANDOC)
//         .args(&["--from", "native", "--to", "markdown"])
//         .stdout(Stdio::piped())
//         .spawn()
//         .map_err(|_| "spawning panodc process".to_string())?;

//     let s = serde_json::to_string(p).map_err(|_| "serializing to json")?;

//     process
//         .stdin
//         .unwrap()
//         .write_all(s.as_bytes())
//         .map_err(|_| "writing to pandoc process")?;

//     let mut bytes = Vec::new();
//     process
//         .stdout
//         .ok_or("receiving pandoc process output")
//         .and_then(|mut c| {
//             c.read_to_end(&mut bytes)
//                 .or(Err("reading from pandoc process"))
//         })?;

//     std::str::from_utf8(&bytes)
//         .map_err(|_| "decoding markdown string".to_string())
//         .map(str::to_string)
// }

pub fn parse_file(path: &Path) -> Result {
    let source = path.to_str().ok_or("Failed to convert path to string")?;

    let output = Command::new(PANDOC)
        .args(ARGS)
        .arg(source)
        .output()
        .map_err(|_| "pandoc cli".to_string())?;

    if !output.status.success() {
        pandoc_from_bytes(&output.stdout)
    } else {
        Err(format!("call to pandoc failed with {:?}", output.status))
    }
}

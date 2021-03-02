//! A context is a gathering of related artifact repositories.
//! TODO Expand

use {
    // crate::locations,
    anyhow::Result,
    // std::path::PathBuf,
    thiserror::Error,
};

#[derive(Error, Debug)]
enum Error {}

pub struct Context {
    name: String,
}

impl Context {
    pub fn new(name: &str) -> Result<Context> {
        let ctxt = Context {
            name: name.to_string(),
        };
        Ok(ctxt)
    }
}

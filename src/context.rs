//! A context is a gathering of related artifact repositories.
//! TODO Expand

use std::fmt;

// #[derive(Error, Debug)]
// enum Error {}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Context {
    pub name: String,
}

impl Context {
    pub fn new(name: String) -> Context {
        Context { name }
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

//! A context is a gathering of related artifact repositories.
//! TODO Expand

// #[derive(Error, Debug)]
// enum Error {}

pub struct Context {
    pub name: String,
}

impl Context {
    pub fn new(name: &String) -> Context {
        Context {
            name: name.to_string(),
        }
    }
}

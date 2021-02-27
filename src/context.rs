//! A context is a gathering of related artifact repositories.
//! TODO Expand

use std::collections::HashSet;

use crate::repo::Repo;

pub struct Context {
    name: String,
    registry: HashSet<Repo>,
}

impl Context {
    // FIXME
    #[allow(clippy::unnecessary_wraps)]
    fn new(name: String) -> Result<Context, String> {
        let ctxt = Context {
            name,
            registry: HashSet::new(),
        };

        // TODO ctxt.save()?;
        Ok(ctxt)
    }

    // TODO Write saveable/loadable traits
    // fn save(&self) -> Result<(), String> {
    //     let path = locations::contexts_dir().map(|p| p.join(self.name))?;
    // }
}

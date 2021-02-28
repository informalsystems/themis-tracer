//! A context is a gathering of related artifact repositories.
//! TODO Expand

use {crate::locations, sled, std::path::PathBuf, thiserror::Error};

#[derive(Error, Debug)]
enum Error {
    #[error("Cannot find db directory: {0}")]
    FindDb(String),

    #[error("Cannot load db: {0}")]
    LoadDb(#[from] sled::Error),
}

pub struct Context {
    name: String,
    db: sled::Db,
}

impl Context {
    pub fn new(name: &String) -> Result<Context, String> {
        let ctxt = Context {
            name: name.clone(),
            db: Context::db(&name)?,
        };
        // Initilize the trees
        ctxt.repos()?;
        ctxt.units()?;
        Ok(ctxt)
    }

    pub fn repos(&self) -> Result<sled::Tree, String> {
        self.db
            .open_tree("repos")
            .map_err(|_| "repos tree".to_string())
    }

    pub fn units(&self) -> Result<sled::Tree, String> {
        self.db
            .open_tree("units")
            .map_err(|_| "repos tree".to_string())
    }

    fn db(name: &String) -> Result<sled::Db, String> {
        let dir = Context::db_dir(name).map_err(|_| "TODO".to_string())?;
        sled::open(dir).map_err(|_| "TODO".to_string())
    }

    fn db_dir(name: &String) -> Result<PathBuf, Error> {
        locations::contexts_dir()
            .map(|p| p.join(name))
            .map_err(|_| Error::FindDb(name.clone()))
    }

    fn exists(name: &String) -> Result<bool, Error> {
        Context::db_dir(name).map(|p| p.exists())
    }
}

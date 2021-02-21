// TODO
use crate::artifact::Artifact;
use std::collections::HashSet;
use std::path::Path;

pub struct Repo<'a> {
    artifacts: HashSet<Artifact<'a>>,
    remote: String,
    local: Option<&'a Path>,
}

impl<'a> Repo<'a> {
    pub fn new(
        artifacts: HashSet<Artifact<'a>>,
        remote: String,
        local: Option<&'a Path>,
    ) -> Repo<'a> {
        Repo {
            artifacts,
            remote,
            local,
        }
    }

    // pub fn from_local(path: &Path) -> Result<Repo<'a>, String> {}
}

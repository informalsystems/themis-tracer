//! Repos represent any self-contained repository of artifacts.
//!
//! A repo can be directory of files or a git repository, or, as a degenerate
//! case, a single flat file.

use crate::artifact::Artifact;
use std::collections::HashSet;
use std::path::PathBuf;

pub struct Repo {
    artifacts: HashSet<Artifact>,
    remote: String,
    local: Option<PathBuf>,
}

impl Repo {
    pub fn new(artifacts: HashSet<Artifact>, remote: String, local: Option<PathBuf>) -> Repo {
        Repo {
            artifacts,
            remote,
            local,
        }
    }

    // pub fn from_local(path: &Path) -> Result<Repo<'a>, String> {}
}

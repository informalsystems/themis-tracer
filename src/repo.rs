//! Repos represent any self-contained repository of artifacts.
//!
//! A repo can be directory of files or a git repository, or, as a degenerate
//! case, a single flat file.

use crate::artifact::Artifact;
use std::collections::HashSet;
use std::path::PathBuf;

pub struct Local {
    path: PathBuf,
    // Used to determine wehre to sync from
    default_upstream: Option<String>,
    // Used to determine wehre to sync from
    default_branch: Option<String>,
}

pub struct Remote {
    // TODO URL type?
    url: String,
    // Used to determine wehre to sync from
    default_branch: Option<String>,
}

pub enum Location {
    Local(Local),
    Remote(Remote),
}

pub struct Repo {
    artifacts: HashSet<Artifact>,
    location: Location,
}

impl Repo {
    pub fn new(artifacts: HashSet<Artifact>, location: Location) -> Repo {
        Repo {
            artifacts,
            location,
        }
    }

    // pub fn from_local(path: &Path) -> Result<Repo<'a>, String> {}
}

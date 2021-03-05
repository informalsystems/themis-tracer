//! Repos represent any self-contained repository of artifacts.
//!
//! A repo can be directory of files or a git repository, or, as a degenerate
//! case, a single flat file.

use {
    crate::logical_unit::LogicalUnit,
    serde::{Deserialize, Serialize},
    std::{cmp::Ordering, collections::HashSet, fmt, path::PathBuf},
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Local {
    pub path: PathBuf,
    // Used to determine wehre to sync from
    pub upstream: Option<String>,
    // Used to determine wehre to sync from
    pub branch: Option<String>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Remote {
    // TODO Use URL type: https://docs.rs/url/2.2.1/url/
    pub url: String,
    // Used to determine wehre to sync from
    pub branch: Option<String>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Location {
    Local(Local),
    Remote(Remote),
}

impl Location {
    pub fn new_local(path: PathBuf, upstream: Option<String>, branch: Option<String>) -> Location {
        Location::Local(Local {
            path,
            upstream,
            branch,
        })
    }
}

#[derive(Eq, Serialize, Deserialize)]
pub struct Repo {
    units: HashSet<LogicalUnit>,
    location: Location,
}

impl Repo {
    pub fn new(location: Location) -> Repo {
        let units: HashSet<LogicalUnit> = HashSet::new();
        Repo { units, location }
    }

    // TODO support for default branch and upstream
    pub fn new_local(path: PathBuf) -> Repo {
        let location = Location::new_local(path, None, None);
        Repo::new(location)
    }
    // pub fn from_local(path: &Path) -> Result<Repo<'a>, String> {}

    pub fn path_as_string(&self) -> String {
        self.location.to_string()
    }
}

impl PartialEq for Repo {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location
    }
}

impl PartialOrd for Repo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.location.cmp(&other.location))
    }
}

impl Ord for Repo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.location.cmp(&other.location)
    }
}

impl fmt::Display for Local {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.as_path().display())
    }
}

impl fmt::Display for Remote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url)
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Location::Local(l) => write!(f, "{}", l),
            Location::Remote(r) => write!(f, "{}", r),
        }
    }
}

impl fmt::Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO Show other fields too?
        write!(f, "{}", self.location)
    }
}

#[cfg(test)]
mod test {
    use {super::*, std::path::PathBuf};

    #[test]
    fn repo_path_to_string() {
        let path = "/foo/bar/baz";
        let repo = Repo::new_local(PathBuf::from(path));
        assert_eq!(path, repo.path_as_string());
    }
}

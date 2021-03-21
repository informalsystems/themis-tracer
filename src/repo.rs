//! Repos represent any self-contained repository of artifacts.
//!
//! A repo can be directory of files or a git repository, or, as a degenerate
//! case, a single flat file.

use {
    serde::{Deserialize, Serialize},
    std::{fmt, path::PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
struct Info {
    // The local path, either in the users working files or in the tools private cache
    pub path: PathBuf,
    // Used to determine where to sync from
    pub upstream: Option<String>,
    // Used to determine wehre to sync from
    pub branch: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
enum LocationInfo {
    Local(Info),
    Remote(Info),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Location {
    inner: LocationInfo,
}

impl Location {
    fn new_local(path: PathBuf, upstream: Option<String>, branch: Option<String>) -> Location {
        Location {
            inner: LocationInfo::Local(Info {
                path,
                upstream,
                branch,
            }),
        }
    }

    fn get_info(&self) -> Info {
        match &self.inner {
            LocationInfo::Local(info) | LocationInfo::Remote(info) => info.clone(),
        }
    }

    fn get_upstream_url(&self) -> Option<String> {
        self.get_info().upstream
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Repo {
    location: Location,
}

impl Repo {
    // TODO support for default branch and upstream
    pub fn new_local(path: PathBuf) -> Repo {
        let location = Location::new_local(path, None, None);
        Repo { location }
    }
    // pub fn from_local(path: &Path) -> Result<Repo<'a>, String> {}

    pub fn path_as_string(&self) -> String {
        self.location.to_string()
    }

    /// The local path of a repo
    pub fn path(&self) -> PathBuf {
        match &self.location.inner {
            LocationInfo::Local(info) | LocationInfo::Remote(info) => info.path.clone(),
        }
    }

    pub fn get_url(&self) -> String {
        self.location
            .get_upstream_url()
            .unwrap_or_else(|| self.path_as_string())
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            LocationInfo::Local(l) => write!(f, "{}", l.path.as_path().display()),
            // It should be impossible to construct a remote remote without an upstream
            LocationInfo::Remote(r) => write!(f, "{}", r.upstream.as_ref().unwrap()),
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

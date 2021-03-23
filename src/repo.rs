//! Repos represent any self-contained repository of artifacts.
//!
//! A repo can be directory of files or a git repository, or, as a degenerate
//! case, a single flat file.

use {
    git2,
    serde::{Deserialize, Serialize},
    std::{
        fmt,
        path::{Path, PathBuf},
    },
};

const GIT_SSH_PREFIX: &str = "git@github.com:";
const GITHUB_URL_PREFIX: &str = "https://github.com/";

// NOTE: the level of nesting in this data structure may be unnecssary complicated.
// The motivation was to abstrasct the underlying representation, in anticipation of
// supporing differences between local and remote repos.
// TODO: Consider refactoring to simplify the internal preresentation.

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

    fn set_upstream_url(&mut self, url: Option<&str>) {
        match self.inner {
            LocationInfo::Local(ref mut info) | LocationInfo::Remote(ref mut info) => {
                info.upstream = url.map(|s| s.to_string())
            }
        };
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Repo {
    location: Location,
}

impl Repo {
    // TODO support for default branch and upstream
    pub fn new_local(path: PathBuf) -> Repo {
        let upstream = get_repo_remote(&path);
        let location = Location::new_local(path, upstream, None);
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

    /// The upstream URL (if one exists), or else the the absolute local path
    /// where the repo is located
    pub fn get_url(&self) -> String {
        self.location
            .get_upstream_url()
            .map(|s| normalize_repo_url(&s))
            .unwrap_or_else(|| self.path_as_string())
    }

    pub fn sync(&mut self) {
        let url = get_repo_remote(&self.path());
        // TODO Update branch
        self.location.set_upstream_url(url.as_deref())
    }
}

fn get_repo_remote(path: &Path) -> Option<String> {
    let repo = git2::Repository::open(&path).ok()?;
    let remote = repo
        .find_remote("upstream")
        .or_else(|_| repo.find_remote("origin"))
        .ok()?;
    remote.url().map(|s| s.to_string())
}

// git@github.com:informalsystems/themis-tracer.git -> https://github.com/informalsystems/themis-tracer
fn normalize_repo_url(url: &str) -> String {
    if url.starts_with(GIT_SSH_PREFIX) {
        let url = url.replace(GIT_SSH_PREFIX, GITHUB_URL_PREFIX);
        url.strip_suffix(".git").unwrap_or(&url).to_string()
    } else {
        url.to_string()
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

    #[test]
    fn can_normalize_repo_url() {
        assert_eq!(
            normalize_repo_url("git@github.com:informalsystems/themis-tracer.git"),
            "https://github.com/informalsystems/themis-tracer".to_string()
        )
    }
}

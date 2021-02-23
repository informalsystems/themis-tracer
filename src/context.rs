//! A context is a gathering of related artifact repositories.
//! TODO Expand

use std::collections::HashSet;

use crate::repo::Repo;

pub struct Context {
    registry: HashSet<Repo>,
}

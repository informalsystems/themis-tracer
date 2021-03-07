//! This module implements the `parse` subcommand
//!
//! The `parse` subcommand parses a single file or string into a vector of
//! [`LogicalUnit`]s and then renders those in the specified
//! serialization [`Format`].
//!
//! [`LogicalUnit`]: crate::logical_unit::LogicalUnit
//! [`Format`]: Format

use {
    crate::{artifact::Artifact, cmd::format::Format},
    anyhow::Result,
    std::path::Path,
};

/// Run the the parser on the file `path` rendering the data in `format`
/// to `stdout`.
pub fn run(path: &Path, format: Format) -> Result<()> {
    // TODO Get repo from context?
    Artifact::from_file(None, path)
        .map(|a| a.logical_units.iter().cloned().collect())
        .and_then(|units| format.units(units))
}

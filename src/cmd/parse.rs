//! This module implements the `parse` subcommand
//!
//! The `parse` subcommand parses a single file or string into a vector of
//! [`LogicalUnit`]s and then renders those in the specified
//! serialization [`Format`].
//!
//! [`LogicalUnit`]: crate::logical_unit::LogicalUnit
//! [`Format`]: Format

use {
    crate::{artifact::Artifact, cmd::format::Format, logical_unit::LogicalUnit},
    anyhow::Result,
    std::{io, path::Path},
};

/// Run the the parser on the file `path` rendering the data in `format`
/// to `stdout`.
pub fn run(path: &Path, format: Format) -> Result<()> {
    // TODO Get repo from context?
    Artifact::from_file(None, path)
        .map(|a| a.logical_units.iter().cloned().collect())
        .and_then(|lus| render(format, lus))
}

/// Render the [`LogicalUnits`]s `lus` according to `format`.
/// Prints rendered results to stdout.
pub fn render(format: Format, mut lus: Vec<LogicalUnit>) -> Result<()> {
    lus.sort();

    match format {
        Format::Csv => {
            // See https://docs.rs/csv/1.1.3/csv/tutorial/index.html#writing-csv
            let mut wtr = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(io::stdout());
            // TODO include headers?
            // Write the headers
            // wtr.serialize([
            //     "tag",
            //     "kind",
            //     "repo",
            //     "file",
            //     "line",
            //     "content",
            //     "references",
            // ])?;
            lus.iter()
                .try_for_each(|x| wtr.serialize(x).map_err(|e| e.into()))
        }
        Format::Json => lus.iter().try_for_each(|x| {
            serde_json::to_string(x)
                .map_err(|e| e.into())
                .map(|x| println!("{}", x))
        }),
    }
}

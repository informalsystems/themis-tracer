//! This module implements the `parse` subcommand
//!
//! The `parse` subcommand parses a single file or string into a vector of
//! [`LogicalUnit`]s and then renders those in the specified
//! serialization [`Format`].
//!
//! [`LogicalUnit`]: crate::logical_unit::LogicalUnit
//! [`Format`]: Format

use crate::artifact::Artifact;
use crate::logical_unit::LogicalUnit;
use std::io;
use std::{fmt, path::Path};

/// Formats supported for rendering parsed requirement data
#[derive(Debug)]
pub enum Format {
    Csv,
    Json,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Format::Csv => "csv",
            Format::Json => "json",
        };
        write!(f, "{}", s)
    }
}

impl Default for Format {
    fn default() -> Self {
        Format::Json
    }
}

impl std::str::FromStr for Format {
    type Err = ParseFormatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "csv" => Ok(Format::Csv),
            "json" => Ok(Format::Json),
            _ => Err(ParseFormatError(s.to_string())),
        }
    }
}

/// Errors arising from parsing invalid formats arguments
#[derive(Debug, Clone)]
pub struct ParseFormatError(String);

impl fmt::Display for ParseFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unable to parse format {}", self.0)
    }
}

/// Run the the parser on the file `path` rendering the data in `format`
/// to `stdout`.
pub fn run(path: &Path, format: Format) -> Result<(), String> {
    // TODO Error handling
    Artifact::from_file(path)
        .map(|a| a.logical_units.iter().cloned().collect())
        .and_then(|lus| render(format, lus))
}

/// Render the [`LogicalUnits`]s `lus` according to `format`.
/// Prints rendered results to stdout.
fn render(format: Format, mut lus: Vec<LogicalUnit>) -> Result<(), String> {
    lus.sort();

    match format {
        Format::Csv => {
            // See https://docs.rs/csv/1.1.3/csv/tutorial/index.html#writing-csv
            let mut wtr = csv::Writer::from_writer(io::stdout());
            lus.iter()
                .try_for_each(|x| wtr.serialize(x).map_err(|e| format!("{}", e)))
        }
        Format::Json => lus.iter().try_for_each(|x| {
            serde_json::to_string(x)
                .map_err(|e| format!("{}", e))
                .map(|x| println!("{}", x))
        }),
    }
}
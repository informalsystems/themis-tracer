//! The `cmd` submodule contains all the logic for CLI.

/// ## Special submodules

/// Definition of the CLI
pub mod opt;

/// Utility for serialized output formats
mod format;

/// ## Subcommand executors
///
/// Each of the following modules contains an executors for the corresponding
/// subcommand defined in `opt`. Each module exports a `run` function for this
/// purpose.
mod context;
mod file;
mod graph;
mod init;
mod linkify;
mod parse;
mod repo;
mod site;
mod sync;
mod unit;

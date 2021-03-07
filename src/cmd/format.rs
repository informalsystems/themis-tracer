use {
    crate::logical_unit::LogicalUnit,
    anyhow::Result,
    std::{fmt, io},
};

impl fmt::Display for ParseFormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unable to parse format {}", self.0)
    }
}

/// Errors arising from parsing invalid formats arguments
#[derive(Debug, Clone)]
pub struct ParseFormatError(String);

/// Formats supported for rendering parsed requirement data
#[derive(Debug)]
pub enum Format {
    Csv,
    Json,
}

impl Format {
    pub fn units(&self, mut lus: Vec<LogicalUnit>) -> Result<()> {
        lus.sort();

        match self {
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

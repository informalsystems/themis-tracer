use {
    crate::{
        cmd::{format::Format, opt},
        db,
        logical_unit::LogicalUnit,
    },
    anyhow::Result,
    std::io::{stdout, Write},
    tabwriter::TabWriter,
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("No logical unit tagged {0} is registered in the current context")]
    LogicalUnitNotFound(String),
}

pub fn run(opt: opt::Unit) -> Result<()> {
    match opt {
        opt::Unit::List { format } => list(format),
        opt::Unit::Show { tag, format } => show(tag, format),
    }
}

fn list(format: Option<Format>) -> Result<()> {
    let conn = db::connection()?;
    let mut units: Vec<LogicalUnit> = db::unit::get_all_in_context(&conn)?;
    units.sort();

    match format {
        None => list_human(units),
        Some(fmt) => fmt.units(units),
    }
}

fn show(tag: String, format: Option<Format>) -> Result<()> {
    let conn = db::connection()?;
    let unit = db::unit::get(&conn, &tag)?.ok_or(Error::LogicalUnitNotFound(tag))?;
    match format {
        None => show_human(unit),
        Some(fmt) => fmt.units(vec![unit]),
    }
}

fn list_human(units: Vec<LogicalUnit>) -> Result<()> {
    let mut tw = TabWriter::new(stdout());

    for unit in units {
        let (tag, content, path) = unit.synopsis();
        writeln!(&mut tw, "{}\t{}\t{}", tag, content, path)?;
    }
    let () = tw.flush()?;
    Ok(())
}

fn show_human(unit: LogicalUnit) -> Result<()> {
    let mut tw = TabWriter::new(stdout());
    let repo = unit.repo.clone().map_or("".into(), |r| r.path_as_string());
    let file: String = unit
        .file
        .map_or("".into(), |f| f.as_path().display().to_string());
    let line = unit.line.map_or("".into(), |l| l.to_string());
    let refs = unit
        .references
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(" ");
    let info = format!(
        "tag:\t{tag}
kind:\t{kind}
repo:\t{repo}
file:\t{file}
line:\t{line}
refs:\t{refs}

{content}
",
        tag = unit.id,
        kind = unit.kind,
        repo = repo,
        file = file,
        line = line,
        refs = refs,
        content = unit.content
    );
    tw.write_all(info.as_bytes())?;

    let () = tw.flush()?;
    Ok(())
}

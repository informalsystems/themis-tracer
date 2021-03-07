use {
    crate::{
        cmd::{format::Format, opt, parse},
        db,
        logical_unit::LogicalUnit,
    },
    anyhow::Result,
    std::io::{stdout, Write},
    tabwriter::TabWriter,
};

fn list_human(units: Vec<LogicalUnit>) -> Result<()> {
    let mut tw = TabWriter::new(stdout());

    for unit in units {
        let (tag, content, path) = unit.synopsis();
        writeln!(&mut tw, "{}\t{}\t{}", tag, content, path)?;
    }
    let () = tw.flush()?;
    Ok(())
}

fn list(format: Option<Format>) -> Result<()> {
    let conn = db::connection()?;
    let mut units: Vec<LogicalUnit> = db::unit::get_all_in_context(&conn)?;
    units.sort();

    match format {
        None => list_human(units),
        Some(fmt) => parse::render(fmt, units),
    }
}

pub fn run(opt: opt::Unit) -> Result<()> {
    match opt.cmd {
        opt::UnitCmd::List { format } => list(format),
    }
}

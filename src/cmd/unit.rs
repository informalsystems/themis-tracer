use {
    crate::{cmd::opt, db, logical_unit::LogicalUnit},
    anyhow::Result,
    std::io::{stdout, Write},
    tabwriter::TabWriter,
};

fn list() -> Result<()> {
    let conn = db::connection()?;
    let mut units: Vec<LogicalUnit> = db::unit::get_all_in_context(&conn)?;
    units.sort();

    let mut tw = TabWriter::new(stdout());

    for unit in units {
        let (tag, content, path) = unit.synopsis();
        writeln!(&mut tw, "{}\t{}\t{}", tag, content, path)?;
    }
    let () = tw.flush()?;
    Ok(())
}

pub fn run(opt: opt::Unit) -> Result<()> {
    match opt.cmd {
        opt::UnitCmd::List {} => list(),
    }
}

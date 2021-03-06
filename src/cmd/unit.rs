use {
    crate::{cmd::opt, db, logical_unit::LogicalUnit},
    anyhow::Result,
};

fn list() -> Result<()> {
    let conn = db::connection()?;
    let mut units: Vec<LogicalUnit> = db::unit::get_all_in_context(&conn)?;
    units.sort();

    for unit in units {
        println!("  {}", unit)
    }

    Ok(())
}

pub fn run(opt: opt::Unit) -> Result<()> {
    match opt.cmd {
        opt::UnitCmd::List {} => list(),
    }
}

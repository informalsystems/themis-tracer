//! Generate HTML sites

use {
    crate::{db, graph, site::Html},
    anyhow::Result,
};

/// Output HTML summarizing a context to stdout
pub fn run() -> Result<()> {
    let conn = db::connection()?;
    let units = db::unit::get_all_in_context(&conn)?;
    let graph = graph::of_units(&units);
    print!("{}", Html::from(&graph));
    Ok(())
}

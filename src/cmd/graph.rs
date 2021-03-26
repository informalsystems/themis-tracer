use {
    crate::{cmd::format::dot::Format, db, dot, graph},
    anyhow::Result,
};

pub fn run(fmt: Format) -> Result<()> {
    let conn = db::connection()?;
    let units = db::unit::get_all_in_context(&conn)?;
    let graph = graph::of_units(&units);
    let dot: String = graph::as_dot("TODO", &graph);
    match fmt {
        Format::Svg => dot::to_svg(&dot)?,
        Format::Dot => println!("{}", dot),
    }
    Ok(())
}

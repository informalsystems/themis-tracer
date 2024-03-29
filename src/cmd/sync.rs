use {
    crate::{cmd, db},
    anyhow::Result,
};

pub fn run() -> Result<()> {
    let conn = db::connection()?;
    let repos = db::repo::get_all_in_context(&conn)?;
    for mut repo in repos {
        db::repo::update(&conn, &mut repo)?;
        db::unit::purge(&conn, &repo)?;
        cmd::repo::load_units_from_repo(&conn, &repo)?;
    }
    Ok(())
}

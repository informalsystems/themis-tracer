use {
    crate::{db, linkify},
    anyhow::Result,
    std::path::PathBuf,
};

pub fn run(paths: &[PathBuf]) -> Result<()> {
    let conn = db::connection()?;
    for path in paths.iter() {
        linkify::file_via_pandoc(&conn, path, true)?
    }
    Ok(())
}

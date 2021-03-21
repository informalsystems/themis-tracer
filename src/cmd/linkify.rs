use {
    crate::{db, linkify},
    anyhow::Result,
    std::path::Path,
};

pub fn run(path: &Path) -> Result<()> {
    let conn = db::connection()?;
    linkify::file_via_pandoc(&conn, path)
}

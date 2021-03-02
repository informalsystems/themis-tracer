use {crate::locations, anyhow::Result, rusqlite as sql, thiserror::Error};

#[derive(Error, Debug)]
enum Error {
    #[error("Cannot read database path")]
    DbPath,
}

pub fn connection() -> Result<sql::Connection> {
    let dir = locations::tracer_db()?;
    let db_loc = dir.to_str().ok_or(Error::DbPath)?;
    let conn = sql::Connection::open(db_loc)?;
    Ok(conn)
}

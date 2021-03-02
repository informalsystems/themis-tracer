use {crate::locations, anyhow::Result, rusqlite as sql, thiserror::Error};

#[derive(Error, Debug)]
enum Error {
    #[error("Cannot read database path")]
    DbPath,
}

/// Functions that create the tables used in the db
mod table {
    use anyhow::Context;

    use super::*;

    fn create(conn: &sql::Connection, name: &str, statement: &str) -> Result<()> {
        conn.prepare(statement)
            .with_context(|| format!("createing table {}", name))?
            .execute(sql::NO_PARAMS)
            .map(|_| ()) // We don't care about the number of rows changed
            .map_err(|e| e.into())
    }

    pub fn init(conn: &sql::Connection) -> Result<()> {
        // Names of tables paired with the statement to the create the
        // table. The names are used in error reporting.
        // TODO Dedup name?
        let statements = vec![
            // Records the state of the app
            (
                "appstate",
                r#"
            CREATE TABLE IF NOT EXISTS appstate (
                id      INTEGER PRIMARY KEY CHECK (id = 0),
                context INTEGER NOT NULL,
                FOREIGN KEY(context) REFERENCES context(id)
            );
            "#,
            ),
            // Records all contexts and their properties
            (
                "context",
                r#"
            CREATE TABLE IF NOT EXISTS context (
                id   INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );
            "#,
            ),
            // Records all repos and their properties
            (
                "repo",
                r#"
            CREATE TABLE IF NOT EXISTS repo (
                id       INTEGER PRIMARY KEY,
                location TEXT       -- A JSON serialization of the rust struct
            );
            "#,
            ),
            // Records all logical units and their properties
            (
                "units",
                r#"
            CREATE TABLE IF NOT EXISTS units (
                id      INTEGER PRIMARY KEY,
                tag     TEXT NOT NULL,
                kind    TEXT NOT NULL,
                content TEXT NOT NULL,
                refs    TEXT NOT NULL,      -- JSON encoded tag list
                source  TEXT NOT NULL       -- JSON encoded source data
            );
            "#,
            ),
            // Records which repos are in which contexts
            (
                "context_repo",
                r#"
            CREATE TABLE IF NOT EXISTS context_repo (
                id      INTEGER PRIMARY KEY,
                context INTEGER NOT NULL,
                repo    INTEGER NOT NULL,
                FOREIGN KEY(context) REFERENCES context(id),
                FOREIGN KEY(repo) REFERENCES repo(id)
            );
            "#,
            ),
            // Records which units are in which repos
            (
                "unit_repo",
                r#"
            CREATE TABLE IF NOT EXISTS unit_repo (
                id      INTEGER PRIMARY KEY,
                unit    INTEGER NOT NULL,
                repo    INTEGER NOT NULL,
                FOREIGN KEY(unit) REFERENCES unit(id),
                FOREIGN KEY(repo) REFERENCES repo(id)
            );
            "#,
            ),
        ];

        statements
            .iter()
            .map(|(name, statement)| create(conn, name, statement))
            .collect()
    }

    // pub(super) fn context(conn: &sql::Connection) -> Result<()> {
    //     create(
    //         conn,
    //         r#"
    //         CREATE TABLE IF NOT EXISTS context (
    //             id INTEGER PRIMARY KEY CHECK,
    //             name TEXT NOT NULL,
    //         );
    //         "#,
    //     )
    // }

    // pub(super) fn appstate(conn: &sql::Connection) -> Result<()> {
    //     create(
    //         conn,
    //         r#"
    //         CREATE TABLE IF NOT EXISTS appstate (
    //             id INTEGER PRIMARY KEY CHECK (id = 0),
    //             context INTEGER NOT NULL,
    //             FOREIGN KEY(context) REFERENCES context(id)
    //         );
    //         "#,
    //     )
    // }
}

/// Initialize the app db with all tables.
/// This should be idempotent.
pub fn init(conn: &sql::Connection) -> Result<()> {
    table::init(conn)
}

/// Open a connection to the app db
pub fn connection() -> Result<sql::Connection> {
    let dir = locations::tracer_db()?;
    let db_loc = dir.to_str().ok_or(Error::DbPath)?;
    let conn = sql::Connection::open(db_loc)?;
    // Enable foreign keys
    // https://sqlite.org/foreignkeys.html#fk_enable
    conn.execute("PRAGMA foreign_keys = ON", sql::NO_PARAMS)?;
    Ok(conn)
}

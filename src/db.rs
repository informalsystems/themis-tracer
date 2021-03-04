use {
    crate::{context::Context, locations},
    anyhow::{Context as AnyhowContext, Result},
    rusqlite as sql,
    thiserror::Error,
};

#[derive(Error, Debug)]
enum Error {
    #[error("Cannot read database path")]
    DbPath,
    #[error("Error while querying database: {0}")]
    Query(#[from] sql::Error),
}

fn create(conn: &sql::Connection, name: &str, statement: &str) -> Result<()> {
    conn.prepare(statement)
        .with_context(|| format!("init step: {}", name))?
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
            "create appstate table",
            r#"
            CREATE TABLE IF NOT EXISTS appstate (
                id      INTEGER PRIMARY KEY CHECK (id = 1), -- Ensure there is only a single row
                context INTEGER,
                FOREIGN KEY(context) REFERENCES context(id)
            );
            "#,
        ),
        // TODO Need to make context name unique
        // Records all contexts and their properties
        (
            "create context table",
            r#"
            CREATE TABLE IF NOT EXISTS context (
                id   INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );
            "#,
        ),
        // Records all repos and their properties
        (
            "create repo table",
            r#"
            CREATE TABLE IF NOT EXISTS repo (
                id       INTEGER PRIMARY KEY,
                location TEXT       -- A JSON serialization of the rust struct
            );
            "#,
        ),
        // Records all logical units and their properties
        (
            "create units table",
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
            "create context_repo table",
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
            "create unit_repo table",
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
        // Initialize a blank state of the app
        (
            "insert empty appstate row",
            r#"
           INSERT INTO appstate (context)
           VALUES (NULL)
           "#,
        ),
    ];

    statements
        .iter()
        .try_for_each(|(name, statement)| create(conn, name, statement))
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

pub mod context {
    use sql::OptionalExtension;

    use super::*;

    pub fn add(conn: &sql::Connection, ctx: Context) -> Result<()> {
        conn.prepare("INSERT INTO context (name) VALUES (:name)")?
            .execute_named(&[(":name", &ctx.name)])
            .map(|_| ())
            .map_err(|e| e.into())
    }

    fn of_row(row: &sql::Row) -> sql::Result<Context> {
        let name = row.get(1)?;
        Ok(Context::new(&name))
    }

    pub fn get_all(conn: &sql::Connection) -> Result<Vec<Context>> {
        let mut stmt = conn.prepare("SELECT * FROM context")?;
        let rows = stmt
            .query_map(sql::NO_PARAMS, of_row)
            .map_err(|e| Error::Query(e))
            .context("fetching all contexts")?;

        let mut ctxs = Vec::new();
        for r in rows {
            ctxs.push(r?);
        }

        Ok(ctxs)
    }

    /// `get(&conn, name)` is:
    ///
    /// - `Ok(Some(context))` if there is a `context` with the given `name` in
    ///    the db
    /// - `Ok(None)` if there is not a context with the the given `name`
    /// - `Err(err)` if the query fails for some reason
    pub fn get(conn: &sql::Connection, name: String) -> Result<Option<Context>> {
        let mut stmt = conn.prepare("SELECT * FROM context WHERE name = :name")?;
        stmt.query_row_named(&[(":name", &name)], of_row)
            .optional()
            .map_err(|e| Error::Query(e).into())
    }

    pub fn set(conn: &sql::Connection, name: String) -> Result<()> {
        let query = r#"
            UPDATE OR FAIL appstate
            SET context = context.id
            FROM context
            WHERE context.name = :name
            "#;
        let mut stmt = conn.prepare(query)?;
        stmt.execute_named(&[(":name", &name)])
            .map_err(|e| Error::Query(e).into())
            .map(|_| ())
    }

    pub fn current(conn: &sql::Connection) -> Result<Option<Context>> {
        let query = r#"
            SELECT c.*
            FROM context c
            INNER JOIN appstate a ON a.context = c.id
            "#;
        let mut stmt = conn.prepare(query)?;
        stmt.query_row(sql::NO_PARAMS, of_row)
            .optional()
            .map_err(|e| Error::Query(e).into())
    }
}

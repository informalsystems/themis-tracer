use {
    crate::{context::Context, locations},
    anyhow::{Context as AnyhowContext, Result},
    rusqlite as sql,
    std::path::Path,
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Cannot read database path")]
    DbPath,

    // TODO Record the query that failed?
    #[error("Querying database: {0}")]
    Query(#[from] sql::Error),

    #[error("Context {0} does not exists")]
    NonexistentContext(String),

    #[error("No context is set. Try: `context switch <context>`")]
    NoContext,

    #[error("No unit found corresponding to tag {0}")]
    UnitNotFound(String),

    #[error("No repo found related to unit with tag {0}")]
    RelatedRepoNotFound(String),

    #[error("Duplicate logical units found {0} {1}")]
    DuplicateUnits(String, String),
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
        // Records all contexts and their properties
        (
            "create context table",
            r#"
            CREATE TABLE IF NOT EXISTS context (
                id   INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            );
            "#,
        ),
        // Records all repos and their properties
        (
            "create repo table",
            r#"
            CREATE TABLE IF NOT EXISTS repo (
                id    INTEGER PRIMARY KEY,
                path  TEXT NOT NULL UNIQUE, -- Path to repo (remote or local)
                json  TEXT NOT NULL         -- A JSON serialization of the Repo struct
            );
            "#,
        ),
        // Index repos by path, for quick lookup
        (
            "index repo table by path",
            r#"
            CREATE UNIQUE INDEX idx_repo_path
            ON repo (path)
            "#,
        ),
        // Records all logical units and their properties
        (
            "create units table",
            r#"
            CREATE TABLE IF NOT EXISTS unit (
                id      INTEGER PRIMARY KEY,
                tag     TEXT NOT NULL UNIQUE,
                json    TEXT NOT NULL         -- A JSON serialization of the LogicalUnit struct
            );
            "#,
        ),
        // Index units by tag, for quick lookup
        (
            "index unit table by tag",
            r#"
            CREATE UNIQUE INDEX idx_unit_tag
            ON unit (tag)
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
                FOREIGN KEY(context) REFERENCES context(id) ON DELETE CASCADE,
                FOREIGN KEY(repo) REFERENCES repo(id) ON DELETE CASCADE,
                UNIQUE(context, repo)
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
                FOREIGN KEY(unit) REFERENCES unit(id) ON DELETE CASCADE,
                FOREIGN KEY(repo) REFERENCES repo(id) ON DELETE CASCADE,
                UNIQUE(unit, repo)
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
        Ok(Context::new(name))
    }

    pub fn get_all(conn: &sql::Connection) -> Result<Vec<Context>> {
        let mut stmt = conn.prepare("SELECT * FROM context")?;
        let rows = stmt
            .query_map(sql::NO_PARAMS, of_row)
            .map_err(Error::Query)
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
    pub fn get(conn: &sql::Connection, name: &str) -> Result<Option<Context>> {
        let mut stmt = conn.prepare("SELECT * FROM context WHERE name = :name")?;
        stmt.query_row_named(&[(":name", &name)], of_row)
            .optional()
            .map_err(|e| Error::Query(e).into())
    }

    pub fn set(conn: &sql::Connection, name: String) -> Result<()> {
        if get(conn, &name)?.is_none() {
            Err(Error::NonexistentContext(name).into())
        } else {
            // It would be cleaner to use UPDATE-FROM here, but that requires
            // sqlite version 3.33, which was only released in 2020-08.
            // See https://sqlite.org/lang_update.html#update_from
            // If we're still using sqlite on the backend after 2022,
            // this should be updated.
            let query = r#"
                UPDATE OR FAIL appstate
                SET context = (SELECT id FROM context WHERE name = :name)
            "#;
            let mut stmt = conn.prepare(query)?;
            stmt.execute_named(&[(":name", &name)])
                .map_err(|e| Error::Query(e).into())
                .map(|_| ())
        }
    }

    /// `current(conn)` is
    ///
    /// - `Ok(Some(context))`, where `context` is the current working context,
    ///   if a context is set and the query succeeds.
    /// - `Ok(None)` if no context is currently set
    /// - `Err(err)` if there is an error when looking up the context
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

pub mod repo {
    use {super::*, crate::repo::Repo, serde_json};

    pub(super) fn of_row(row: &sql::Row) -> sql::Result<Repo> {
        let json: String = row.get(2)?;
        serde_json::from_str(&*json)
            // TODO I'm not sure how to get the right error type here at the moment...
            .map_err(|_| sql::Error::InvalidParameterName("TODO returning wrong error".into()))
    }

    pub fn get(conn: &sql::Connection, path: &Path) -> Result<Repo> {
        let mut stmt = conn.prepare("SELECT * FROM repo WHERE repo.path = :path")?;
        let row =
            stmt.query_row_named(&[(":path", &path.to_string_lossy().to_string())], of_row)?;
        Ok(row)
    }

    // TODO Is this actually useful?
    /// `purge(&conn, &repo)` removes the `repo` from the db as well as all units
    /// registered to that repo and all relations to that repo
    pub fn purge(conn: &sql::Connection, repo: &Repo) -> Result<()> {
        unit::purge(&conn, &repo)?;
        let mut stmt = conn.prepare("DELETE FROM repo WHERE path = :path")?;
        let _ = stmt.execute_named(&[(":path", &repo.path_as_string())])?;
        Ok(())
    }

    pub fn update(conn: &sql::Connection, repo: &mut Repo) -> Result<()> {
        repo.update()?;
        let encoded = serde_json::to_string(repo)?;
        let path = repo.path_as_string();

        let mut stmt = conn.prepare("UPDATE repo SET json = :json WHERE path = :path")?;
        stmt.execute_named(&[(":path", &path), (":json", &encoded)])
            .map_err(|e| Error::Query(e).into())
            .map(|_| ())
    }

    fn insert(conn: &sql::Connection, repo: &Repo) -> Result<()> {
        let encoded = serde_json::to_string(repo)?;
        let path = repo.path_as_string();

        let mut stmt =
            conn.prepare("INSERT OR IGNORE INTO repo (path, json) VALUES (:path, :json)")?;
        stmt.execute_named(&[(":path", &path), (":json", &encoded)])
            .map_err(|e| Error::Query(e).into())
            .map(|_| ())
    }

    fn relate_to_context(conn: &sql::Connection, repo: &Repo, ctx: &Context) -> Result<()> {
        let query = r#"
            INSERT INTO context_repo (context, repo)
            VALUES ((SELECT id FROM context WHERE name = :context),
                    (SELECT id FROM repo WHERE path = :repo))
        "#;
        let mut stmt = conn.prepare(query)?;
        stmt.execute_named(&[(":context", &ctx.name), (":repo", &repo.path_as_string())])
            .map_err(|e| Error::Query(e).into())
            .map(|_| ())
    }

    /// `add(&conn, &repo)` adds the
    pub fn add(conn: &sql::Connection, repo: &Repo) -> Result<()> {
        // TODO Should be able to do in one query?
        let current_ctx = context::current(&conn)?;

        match current_ctx {
            None => Err(Error::NoContext.into()),
            Some(ctx) => {
                insert(conn, repo)?;
                relate_to_context(conn, repo, &ctx)
            }
        }
    }

    /// `get_all_in_context(conn)` is
    ///
    /// - `Ok(repos)` where `repos` are all the repos registered to the current context
    /// - `Err(err)` in the event of a query error
    pub fn get_all_in_context(conn: &sql::Connection) -> Result<Vec<Repo>> {
        let query = r#"
            SELECT *
            FROM repo
            INNER JOIN appstate ON appstate.id = 1
            INNER JOIN context_repo ON context_repo.context = appstate.context
            WHERE repo.id = context_repo.repo
            "#;
        let mut stmt = conn.prepare(query)?;
        let rows = stmt
            .query_map(sql::NO_PARAMS, of_row)
            .map_err(Error::Query)
            .context("fetching all contexts")?;

        let mut repos = Vec::new();
        for r in rows {
            repos.push(r?);
        }

        Ok(repos)
    }
}

// interfaces to logical units in db
pub mod unit {
    use {
        super::*,
        crate::{logical_unit::LogicalUnit, repo::Repo},
        sql::OptionalExtension,
    };

    fn of_row(row: &sql::Row) -> sql::Result<LogicalUnit> {
        let json: String = row.get(2)?;
        serde_json::from_str(&*json)
            // TODO I'm not sure how to get the right error type here at the moment...
            .map_err(|_| sql::Error::InvalidParameterName("TODO returning wrong error".into()))
    }

    /// `get(&conn, tag)` is:
    ///
    /// - `Ok(Some(unit))` if there is a `unit` with the given `tag` in
    ///    the db
    /// - `Ok(None)` if there is not a unit with the the given `tag`
    /// - `Err(err)` if the query fails for some reason
    pub fn get(conn: &sql::Connection, tag: &str) -> Result<Option<LogicalUnit>> {
        let mut stmt = conn.prepare("SELECT * FROM unit WHERE tag = :tag")?;
        stmt.query_row_named(&[(":tag", &tag)], of_row)
            .optional()
            .map_err(|e| Error::Query(e).into())
    }

    /// `get_uri(&conn, &tag)` is the uri to the unit indicated by `tag`.
    /// If a remote URL can be onbtained for the source repo, that is is used
    /// as the base, otherwise it falls back to the local path of the repo.
    pub fn get_path(conn: &sql::Connection, tag: &str) -> Result<String> {
        let unit = get(conn, &tag)?.ok_or_else(|| Error::UnitNotFound(tag.into()))?;

        let q = r#"
            SELECT repo.id, repo.path, repo.json
            FROM repo
            INNER JOIN unit ON unit.tag = :tag
            INNER JOIN unit_repo ON unit_repo.unit = unit.id
            WHERE repo.id = unit_repo.repo
            "#;
        let mut stmt = conn.prepare(q)?;
        let repo = stmt
            .query_row_named(&[(":tag", &tag)], repo::of_row)
            .optional()
            .map_err(|e: sql::Error| -> anyhow::Error { Error::Query(e).into() })?
            .ok_or_else(|| Error::RelatedRepoNotFound(tag.into()))?;

        // Construct the URL to a file on github from its upstream URL and default branch
        let branch = (&repo).get_branch().unwrap_or_else(|| "master".to_string());
        let mut url = (&repo).get_url();
        url.push_str("/blob/");
        url.push_str(&branch);
        url.push('/');
        url.push_str(&unit.file_path_as_str().unwrap_or_else(|| "".to_string()));
        url.push('#');
        url.push_str(tag);
        Ok(url)
    }

    // Insert a unit into the db. This should not be used outside of this module,
    // since it does not enforce any of the expected invariants.
    fn insert(conn: &sql::Connection, unit: &LogicalUnit) -> Result<()> {
        let encoded = serde_json::to_string(unit)?;
        let mut stmt = conn.prepare("INSERT INTO unit (tag, json) VALUES (:tag, :json)")?;
        stmt.execute_named(&[(":tag", &unit.id.to_string()), (":json", &encoded)])
            .map_err(|e| Error::Query(e).into())
            .map(|_| ())
    }

    fn relate_to_repo(conn: &sql::Connection, repo: &Repo, unit: &LogicalUnit) -> Result<()> {
        let query = r#"
            INSERT INTO unit_repo (unit, repo)
            VALUES ((SELECT id FROM unit WHERE tag = :tag),
                    (SELECT id FROM repo WHERE path = :path))
        "#;
        let mut stmt = conn.prepare(query)?;
        stmt.execute_named(&[
            (":tag", &unit.id.to_string()),
            (":path", &repo.path_as_string()),
        ])
        .map_err(|e| Error::Query(e).into())
        .map(|_| ())
    }

    /// Adds the `unit` to the current context, and associates it with `repo`.
    ///
    /// If a conflicting unit is already present in the context, a
    /// `Error::DuplicateUnits` is returned.
    pub fn add(conn: &sql::Connection, repo: &Repo, unit: &LogicalUnit) -> Result<()> {
        if let Some(other_unit) = get(&conn, &unit.id.to_string())? {
            Err(Error::DuplicateUnits(unit.to_string(), other_unit.to_string()).into())
        } else {
            insert(conn, unit)?;
            relate_to_repo(conn, repo, unit)
        }
    }

    pub fn get_all_in_context(conn: &sql::Connection) -> Result<Vec<LogicalUnit>> {
        let query = r#"
            SELECT *
            FROM unit
            INNER JOIN appstate ON appstate.id = 1
            INNER JOIN context_repo ON context_repo.context = appstate.context
            INNER JOIN unit_repo ON unit_repo.repo = context_repo.repo
            WHERE unit.id = unit_repo.unit
            "#;
        let mut stmt = conn.prepare(query)?;
        let rows = stmt
            .query_map(sql::NO_PARAMS, of_row)
            .map_err(Error::Query)
            .context("fetching all units in current context")?;

        let mut units = Vec::new();
        for u in rows {
            units.push(u?);
        }
        // TODO Replace with an ordered set
        units.sort();

        Ok(units)
    }

    /// `purge(&conn, &repo)` purges all units registered to the `repo`
    pub fn purge(conn: &sql::Connection, repo: &Repo) -> Result<()> {
        let mut stmt = conn.prepare(
            r#"
            DELETE FROM unit
            WHERE id IN (
                SELECT unit_repo.unit FROM unit_repo
                INNER JOIN repo ON repo.path = :path
                WHERE unit_repo.repo = repo.id
            )
            "#,
        )?;

        stmt.execute_named(&[(":path", &repo.path_as_string())])
            .map_err(|e| Error::Query(e).into())
            .map(|_| ())
    }
}

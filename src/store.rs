use std::env;
use std::path::PathBuf;

use serde::{Serialize};
use rusqlite::{Connection, Result as SqlResult};

const DB_FILE: &str = "smrs.db";
const DEFAULT_DIR: &str = "/smrs/data";

const SCHEMA_VERSION: u32 = 1;

fn initialize(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE links (
            token TEXT PRIMARY KEY NOT NULL,
            url TEXT NOT NULL,
            session TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            deleted_at INTEGER DEFAULT NULL
        )", (),
    ).map_err(|err| err.to_string())?;

    conn.execute(
        "CREATE TABLE schema (
            version UNSIGNED INTEGER NOT NULL,
            update_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )", (),
    ).map_err(|err| err.to_string())?;

    conn.execute(
        "INSERT INTO schema (version) VALUES (?)", (SCHEMA_VERSION,)
    ).map_err(|err| err.to_string())?;

    Ok(())
}

fn confirm_version(conn: &Connection) -> Result<(), String> {
    let version: u32 = match conn.query_row(
        "SELECT version FROM schema ORDER BY update_at DESC LIMIT 1",
        (),
        |row| row.get(0)
    ) {
            Ok(version) => version,
            Err(err) => return Err(err.to_string()),
        };

    if version != SCHEMA_VERSION {
        return Err(format!("Schema version mismatch: expected {}, got {}", SCHEMA_VERSION, version));
    }

    Ok(())
}

pub fn open() -> Result<Connection, String> {
    let db_dir = match env::var("DATA_DIR") {
        Ok(dir) => dir,
        Err(_) => {
            match env::var("DOCUMENT_ROOT") {
                Ok(dir) => dir,
                Err(_) => DEFAULT_DIR.to_string(),
            }
        }
    };

    let mut path = PathBuf::from(db_dir);
    path.push(DB_FILE);

    let is_new = !path.exists();

    let conn = match Connection::open(path) {
        Ok(conn) => conn,
        Err(err) => return Err(err.to_string()),
    };

    if is_new {
        initialize(&conn)?
    } else {
        confirm_version(&conn)?
    }

    Ok(conn)
}

pub fn create_link(conn: &Connection, token: &str, url: &str, session: &str) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO links (token, url, session) VALUES (?, ?, ?)",
        (token, url, session)
    )?;

    Ok(())
}

pub fn get_link(conn: &Connection, token: &str) -> Result<Option<String>, String> {
    let url: String = match conn.query_row(
        "SELECT url FROM links WHERE token = ? AND deleted_at IS NULL",
        (token,),
        |row| row.get(0)
    ) {
        Ok(url) => url,
        Err(err) => return Err(err.to_string()),
    };

    Ok(Some(url))
}

pub fn delete_link(conn: &Connection, token: &str, session: &str) -> SqlResult<usize> {
    let result = conn.execute(
        "UPDATE links SET deleted_at = strftime('%s', 'now') WHERE token = ? AND session = ?",
        (token, session)
    )?;

    Ok(result)
}

#[derive(Serialize)]
pub struct Link {
    pub token: String,
    pub url: String,
    pub created_at: i64,
}

pub fn list_links(conn: &Connection, session: &str) -> SqlResult<Vec<Link>> {
    let mut stmt = conn.prepare(
        "SELECT token, url, created_at FROM links WHERE session = ? AND deleted_at IS NULL ORDER BY created_at DESC"
    )?;
    let link_iter = stmt.query_map(
        (session,),
        |row| {
            let token: String = row.get(0)?;
            let url: String = row.get(1)?;
            let created_at: i64 = row.get(2)?;

            Ok(Link {
                token,
                url,
                created_at,
            })
        }
    )?;

    let links: SqlResult<Vec<Link>> = link_iter.collect();
    return links
}

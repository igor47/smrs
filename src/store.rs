use std::env;
use std::path::PathBuf;

use rusqlite::{Connection, Result as SqlResult};

const DB_FILE: &str = "smrs.db";
const DEFAULT_DIR: &str = "/smrs/data";

const SCHEMA_VERSION: u32 = 1;

fn initialize(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE links (
            id INTEGER PRIMARY KEY,
            token TEXT NOT NULL UNIQUE,
            url TEXT NOT NULL,
            session TEXT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            deleted_at DATETIME
        )", (),
    ).map_err(|err| err.to_string())?;

    conn.execute(
        "CREATE TABLE schema (
            version UNSIGNED INTEGER NOT NULL,
            update_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
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

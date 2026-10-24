use std::path::PathBuf;

use rusqlite::{Connection, params};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("db error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(String),
}

pub type Result<T> = std::result::Result<T, CacheError>;

pub struct Cache {
    conn: Connection,
    path: PathBuf,
}

impl Cache {
    pub fn open() -> Result<Self> {
        let dir = dirs::cache_dir().unwrap_or_else(|| PathBuf::from(".")).join("gtw");
        std::fs::create_dir_all(&dir).map_err(|e| CacheError::Io(e.to_string()))?;
        let path = dir.join("cache.db");
        let conn = Connection::open(&path)?;
        let c = Self { conn, path };
        c.migrate()?;
        Ok(c)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS articles(
                title TEXT PRIMARY KEY,
                html_zst BLOB,
                etag TEXT,
                last_modified TEXT,
                fetched_at INTEGER,
                revision INTEGER
            );
            CREATE TABLE IF NOT EXISTS history(
                id INTEGER PRIMARY KEY,
                title TEXT,
                visited_at INTEGER
            );
            CREATE TABLE IF NOT EXISTS bookmarks(
                title TEXT PRIMARY KEY,
                added_at INTEGER
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS fts USING fts5(title, body);",
        )?;
        Ok(())
    }
}

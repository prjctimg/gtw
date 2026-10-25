use std::path::PathBuf;

use rusqlite::Connection;
use thiserror::Error;

#[allow(clippy::type_complexity)]
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum CacheError {
    #[error("db error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(String),
}

#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, CacheError>;

#[allow(dead_code)]
pub struct Cache {
    conn: Connection,
    #[allow(dead_code)]
    path: PathBuf,
}

#[allow(clippy::type_complexity)]
impl Cache {
    #[allow(dead_code)]
    pub fn open() -> Result<Self> {
        let dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gtw");
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

    #[allow(dead_code)]
    pub fn put_article(
        &self,
        title: &str,
        html: &str,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<()> {
        let compressed =
            zstd::encode_all(html.as_bytes(), 3).map_err(|e| CacheError::Io(e.to_string()))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.conn.execute(
            "INSERT OR REPLACE INTO articles(title, html_zst, etag, last_modified, fetched_at) VALUES(?1,?2,?3,?4,?5)",
            rusqlite::params![title, compressed, etag, last_modified, now],
        )?;
        // Update FTS
        let _ = self
            .conn
            .execute("DELETE FROM fts WHERE title = ?1", rusqlite::params![title]);
        let _ = self.conn.execute(
            "INSERT INTO fts(title, body) VALUES(?1,?2)",
            rusqlite::params![title, html],
        );
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_article(
        &self,
        title: &str,
    ) -> Result<Option<(String, Option<String>, Option<String>)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT html_zst, etag, last_modified FROM articles WHERE title = ?1")?;
        let mut rows = stmt.query(rusqlite::params![title])?;
        if let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            let etag: Option<String> = row.get(1)?;
            let last_modified: Option<String> = row.get(2)?;
            let decompressed =
                zstd::decode_all(&blob[..]).map_err(|e| CacheError::Io(e.to_string()))?;
            let html =
                String::from_utf8(decompressed).map_err(|e| CacheError::Io(e.to_string()))?;
            Ok(Some((html, etag, last_modified)))
        } else {
            Ok(None)
        }
    }

    #[allow(dead_code)]
    pub fn add_bookmark(&self, title: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.conn.execute(
            "INSERT OR IGNORE INTO bookmarks(title, added_at) VALUES(?1,?2)",
            rusqlite::params![title, now],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn remove_bookmark(&self, title: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM bookmarks WHERE title = ?1",
            rusqlite::params![title],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn list_bookmarks(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT title FROM bookmarks ORDER BY added_at DESC")?;
        let mut rows = stmt.query([])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            if let Ok(t) = row.get::<_, String>(0) {
                out.push(t);
            }
        }
        Ok(out)
    }
}

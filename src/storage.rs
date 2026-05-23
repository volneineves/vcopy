use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipItem {
    pub id: i64,
    pub kind: ClipKind,
    pub content: String,
    pub image_path: Option<PathBuf>,
    pub image_width: Option<usize>,
    pub image_height: Option<usize>,
    pub copied_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClipKind {
    Text,
    Image,
}

impl ClipKind {
    pub fn label(self) -> &'static str {
        match self {
            ClipKind::Text => "text",
            ClipKind::Image => "image",
        }
    }
}

impl ClipItem {
    pub fn display_content(&self) -> String {
        match self.kind {
            ClipKind::Text => self.content.clone(),
            ClipKind::Image => format!(
                "[image {}x{}]",
                self.image_width.unwrap_or_default(),
                self.image_height.unwrap_or_default(),
            ),
        }
    }
}

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let path = db_path();
        std::fs::create_dir_all(path.parent().unwrap())?;
        let conn = Connection::open(&path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS history (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                content   TEXT NOT NULL,
                copied_at TEXT NOT NULL
             );
             CREATE UNIQUE INDEX IF NOT EXISTS idx_content ON history(content);",
        )?;
        migrate(&conn)?;
        Ok(Self { conn })
    }

    pub fn insert_text(&self, content: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        // Upsert: duplicate content bumps copied_at instead of creating a new row.
        self.conn.execute(
            "INSERT INTO history (kind, content, copied_at) VALUES ('text', ?1, ?2)
             ON CONFLICT(content) DO UPDATE SET copied_at = excluded.copied_at",
            params![content, now],
        )?;
        Ok(())
    }

    pub fn insert_image(&self, width: usize, height: usize, bytes: &[u8]) -> Result<String> {
        let hash = image_key(width, height, bytes);
        let path = image_path(&hash);
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, bytes)?;

        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO history (kind, content, image_path, image_width, image_height, copied_at)
             VALUES ('image', ?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(content) DO UPDATE SET copied_at = excluded.copied_at",
            params![
                hash,
                path.display().to_string(),
                width as i64,
                height as i64,
                now,
            ],
        )?;
        Ok(hash)
    }

    pub fn prune(&self, limit: usize) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT image_path FROM history
             WHERE kind = 'image'
               AND id NOT IN (
                 SELECT id FROM history ORDER BY copied_at DESC LIMIT ?1
               )",
        )?;
        let image_paths: Vec<PathBuf> = stmt
            .query_map(params![limit as i64], |row| {
                Ok(PathBuf::from(row.get::<_, String>(0)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        self.conn.execute(
            "DELETE FROM history
             WHERE id NOT IN (
                SELECT id FROM history ORDER BY copied_at DESC LIMIT ?1
             )",
            params![limit as i64],
        )?;
        remove_files(&image_paths);
        Ok(())
    }

    pub fn list(&self, limit: usize) -> Result<Vec<ClipItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, content, image_path, image_width, image_height, copied_at
             FROM history ORDER BY copied_at DESC LIMIT ?1",
        )?;
        let items = stmt
            .query_map(params![limit as i64], map_clip_item)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(items)
    }

    pub fn get(&self, id: i64) -> Result<Option<ClipItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, content, image_path, image_width, image_height, copied_at
             FROM history WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(map_clip_item(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<ClipItem>> {
        let pattern = format!("%{}%", query.to_lowercase());
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, content, image_path, image_width, image_height, copied_at
             FROM history
             WHERE lower(content) LIKE ?1
                OR (kind = 'image' AND 'image' LIKE ?1)
                OR (kind = 'image' AND CAST(image_width AS TEXT) LIKE ?1)
                OR (kind = 'image' AND CAST(image_height AS TEXT) LIKE ?1)
             ORDER BY copied_at DESC
             LIMIT ?2",
        )?;
        let items = stmt
            .query_map(params![pattern, limit as i64], map_clip_item)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(items)
    }

    pub fn delete(&self, id: i64) -> Result<bool> {
        let path = self.image_path_for(id)?;
        let changed = self.conn.execute("DELETE FROM history WHERE id = ?1", params![id])?;
        if let Some(path) = path {
            remove_file(&path);
        }
        Ok(changed > 0)
    }

    pub fn update_text(&self, id: i64, new_content: &str) -> Result<bool> {
        let changed = self.conn.execute(
            "UPDATE history SET content = ?1 WHERE id = ?2 AND kind = 'text'",
            params![new_content, id],
        )?;
        Ok(changed > 0)
    }

    pub fn clear(&self) -> Result<()> {
        let paths = self.image_paths()?;
        self.conn.execute("DELETE FROM history", [])?;
        remove_files(&paths);
        Ok(())
    }

    pub fn read_image(&self, item: &ClipItem) -> Result<Vec<u8>> {
        let path = item.image_path.as_ref().context("image path missing")?;
        Ok(std::fs::read(path)?)
    }

    fn image_path_for(&self, id: i64) -> Result<Option<PathBuf>> {
        let mut stmt = self
            .conn
            .prepare("SELECT image_path FROM history WHERE id = ?1 AND kind = 'image'")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(row.get::<_, Option<String>>(0)?.map(PathBuf::from))
        } else {
            Ok(None)
        }
    }

    fn image_paths(&self) -> Result<Vec<PathBuf>> {
        let mut stmt = self
            .conn
            .prepare("SELECT image_path FROM history WHERE kind = 'image'")?;
        let paths = stmt
            .query_map([], |row| Ok(PathBuf::from(row.get::<_, String>(0)?)))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(paths)
    }
}

fn map_clip_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<ClipItem> {
    let kind = match row.get::<_, String>(1)?.as_str() {
        "image" => ClipKind::Image,
        _ => ClipKind::Text,
    };
    Ok(ClipItem {
        id: row.get(0)?,
        kind,
        content: row.get(2)?,
        image_path: row.get::<_, Option<String>>(3)?.map(PathBuf::from),
        image_width: row.get::<_, Option<i64>>(4)?.map(|v| v as usize),
        image_height: row.get::<_, Option<i64>>(5)?.map(|v| v as usize),
        copied_at: row.get::<_, String>(6)?.parse::<DateTime<Utc>>().unwrap_or_default(),
    })
}

fn db_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vcopy")
        .join("history.db")
}

fn image_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vcopy")
        .join("images")
}

fn image_path(hash: &str) -> PathBuf {
    image_dir().join(format!("{hash}.rgba"))
}

pub fn image_key(width: usize, height: usize, bytes: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    width.hash(&mut hasher);
    height.hash(&mut hasher);
    bytes.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn migrate(conn: &Connection) -> Result<()> {
    add_column(conn, "kind", "TEXT NOT NULL DEFAULT 'text'")?;
    add_column(conn, "image_path", "TEXT")?;
    add_column(conn, "image_width", "INTEGER")?;
    add_column(conn, "image_height", "INTEGER")?;
    Ok(())
}

fn add_column(conn: &Connection, name: &str, definition: &str) -> Result<()> {
    let exists = conn
        .prepare("PRAGMA table_info(history)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .any(|column| column == name);

    if !exists {
        conn.execute(&format!("ALTER TABLE history ADD COLUMN {name} {definition}"), [])?;
    }
    Ok(())
}

fn remove_files(paths: &[PathBuf]) {
    for path in paths {
        remove_file(path);
    }
}

fn remove_file(path: &Path) {
    if path.starts_with(image_dir()) {
        let _ = std::fs::remove_file(path);
    }
}

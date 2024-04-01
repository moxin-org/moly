use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use rusqlite::Row;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct DownloadedFile {
    pub id: Arc<String>,
    pub model_id: String,
    pub name: String,
    pub size: String,
    pub quantization: String,
    pub prompt_template: String,
    pub reverse_prompt: String,
    pub downloaded: bool,
    pub downloaded_path: String,
    pub downloaded_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub featured: bool,
}

impl DownloadedFile {
    pub fn save_to_db(&self, conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT INTO download_files (id, model_id, name, size, quantization, prompt_template, reverse_prompt, downloaded, downloaded_path, downloaded_at, tags, featured) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                self.id,
                self.model_id,
                self.name,
                self.size,
                self.quantization,
                self.prompt_template,
                self.reverse_prompt,
                self.downloaded,
                self.downloaded_path,
                self.downloaded_at.to_rfc3339(),
                serde_json::to_string(&self.tags).unwrap(),
                self.featured,
            ],
        )?;
        Ok(())
    }

    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let downloaded_at = chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
            .map(|s| s.to_utc())
            .unwrap_or_default();

        let tags = serde_json::from_str(row.get::<_, String>(10)?.as_str()).unwrap_or_default();

        Ok(DownloadedFile {
            id: Arc::new(row.get(0)?),
            model_id: row.get(1)?,
            name: row.get(2)?,
            size: row.get(3)?,
            quantization: row.get(4)?,
            prompt_template: row.get(5)?,
            reverse_prompt: row.get(6)?,
            downloaded: row.get(7)?,
            downloaded_path: row.get(8)?,
            downloaded_at,
            tags,
            featured: row.get(11)?,
        })
    }

    pub fn get_all(conn: &rusqlite::Connection) -> rusqlite::Result<HashMap<Arc<String>, Self>> {
        let mut stmt = conn.prepare("SELECT * FROM download_files")?;
        let mut rows = stmt.query([])?;
        let mut files = HashMap::new();

        while let Some(row) = rows.next()? {
            let file = Self::from_row(row)?;
            files.insert(file.id.clone(), file);
        }

        Ok(files)
    }

    pub fn get_by_models<S: AsRef<str>>(
        conn: &rusqlite::Connection,
        ids: &[S],
    ) -> rusqlite::Result<HashMap<Arc<String>, Self>> {
        let ids = ids.iter().map(|s| s.as_ref()).collect::<Vec<&str>>();
        let ids_str = ids.join(",");

        let mut stmt = conn.prepare("SELECT * FROM download_files WHERE model_id IN (?1)")?;
        let mut rows = stmt.query([ids_str])?;
        let mut files = HashMap::new();

        while let Some(row) = rows.next()? {
            let file = Self::from_row(row)?;
            files.insert(file.id.clone(), file);
        }

        Ok(files)
    }
}

pub fn create_table_download_files(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "BEGIN;
        CREATE TABLE IF NOT EXISTS download_files (
            id TEXT PRIMARY KEY,
            model_id TEXT NOT NULL,
            name TEXT NOT NULL,
            size TEXT NOT NULL,
            quantization TEXT NOT NULL,
            prompt_template TEXT DEFAULT '',
            reverse_prompt TEXT DEFAULT '',
            downloaded INTEGER DEFAULT 0,
            downloaded_path TEXT NOT NULL,
            downloaded_at TEXT NOT NULL,
            tags TEXT NOT NULL,
            featured INTEGER DEFAULT 0
        );
        CREATE INDEX IF NOT EXISTS index_model_id ON download_files (model_id);
        COMMIT;",
    )?;

    Ok(())
}

#[test]
fn test_sql() {
    let _ = std::fs::remove_file("test_download_files.db");
    let conn: rusqlite::Connection = rusqlite::Connection::open("test_download_files.db").unwrap();
    create_table_download_files(&conn).unwrap();

    let downloaded_file = DownloadedFile {
        id: Arc::new("test".to_string()),
        model_id: "test".to_string(),
        name: "test".to_string(),
        size: "test".to_string(),
        quantization: "test".to_string(),
        prompt_template: "test".to_string(),
        reverse_prompt: "test".to_string(),
        downloaded: false,
        downloaded_path: "test".to_string(),
        downloaded_at: Utc::now(),
        tags: vec!["test".to_string()],
        featured: false,
    };

    downloaded_file.save_to_db(&conn).unwrap();
    let files = DownloadedFile::get_all(&conn).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[&downloaded_file.id], downloaded_file);
}

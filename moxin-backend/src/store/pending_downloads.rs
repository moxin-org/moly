use rusqlite::Row;
use std::{sync::Arc, vec};

#[derive(Debug, Default, PartialEq, Clone)]
pub enum PendingDownloadsStatus {
    #[default]
    Downloading,
    Paused,
    Error,
}

impl PendingDownloadsStatus {
    pub fn to_string(&self) -> &str {
        match self {
            Self::Downloading => "downloading",
            Self::Paused => "paused",
            Self::Error => "error",
        }
    }

    pub fn from_string(s: &str) -> Self {
        match s {
            "downloading" => Self::Downloading,
            "paused" => Self::Paused,
            "error" => Self::Error,
            _ => Self::Downloading,
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct PendingDownloads {
    pub file_id: Arc<String>,
    pub progress: f64,
    pub status: PendingDownloadsStatus,
}

// TODO I'm not 100% convinced that this is the best way to handle this
// I will attempt to merge PendingDownloads and DownloadedFile into a single table, or
// at least a single struct, to see if that makes more sense

impl PendingDownloads {
    pub fn insert_into_db(&self, conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT INTO pending_downloads (file_id) VALUES (?1)",
            rusqlite::params![self.file_id],
        )?;
        Ok(())
    }

    pub fn save_to_db(&self, conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        conn.execute(
            "UPDATE pending_downloads
            SET progress = ?2,
                status = ?3
            WHERE file_id = ?1",
            rusqlite::params![self.file_id, self.progress, self.status.to_string()],
        )?;
        Ok(())
    }

    fn exists_by_id(conn: &rusqlite::Connection, id: String) -> rusqlite::Result<bool> {
        conn.query_row(
            "SELECT EXISTS (SELECT file_id FROM pending_downloads WHERE file_id = ?1)",
            [id],
            |row| row.get::<_, bool>(0),
        )
    }

    pub fn insert_if_not_exists(
        file_id: Arc<String>,
        conn: &rusqlite::Connection,
    ) -> rusqlite::Result<()> {
        if !Self::exists_by_id(conn, file_id.to_string())? {
            let pending_download = PendingDownloads {
                file_id: file_id.into(),
                ..Default::default()
            };
            pending_download.insert_into_db(conn)?;
        }

        Ok(())
    }

    pub fn remove(file_id: Arc<String>, conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        conn.execute(
            "DELETE FROM pending_downloads WHERE file_id = ?1",
            rusqlite::params![file_id],
        )?;
        Ok(())
    }

    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let status = PendingDownloadsStatus::from_string(row.get::<_, String>(2)?.as_str());

        Ok(PendingDownloads {
            file_id: Arc::new(row.get(0)?),
            progress: row.get(1)?,
            status: status,
        })
    }

    pub fn get_all(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<Self>> {
        let mut stmt = conn.prepare("SELECT * FROM pending_downloads")?;
        let mut rows = stmt.query([])?;
        let mut downloads = vec![];

        while let Some(row) = rows.next()? {
            let item = Self::from_row(row)?;
            downloads.push(item);
        }

        Ok(downloads)
    }
}

pub fn create_table_pending_downloads(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "BEGIN;
        CREATE TABLE IF NOT EXISTS pending_downloads (
            file_id TEXT PRIMARY KEY,
            progress REAL DEFAULT 0,
            status TEXT DEFAULT 'downloading'
        );
        CREATE INDEX IF NOT EXISTS index_pending_downloads_file_id ON pending_downloads (file_id);
        COMMIT;",
    )?;

    Ok(())
}

pub fn mark_pending_downloads_as_paused(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE pending_downloads
        SET status = 'paused'
        WHERE status = 'downloading'",
        [],
    )?;
    Ok(())
}

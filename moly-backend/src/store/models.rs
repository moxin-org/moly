use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use rusqlite::params;

use super::model_cards::Author;

pub fn create_table_models(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS models (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            summary TEXT NOT NULL,
            size TEXT NOT NULL,
            requires TEXT NOT NULL,
            architecture TEXT NOT NULL,
            released_at TEXT NOT NULL,
            prompt_template TEXT DEFAULT '',
            reverse_prompt TEXT DEFAULT '',
            author_name TEXT NOT NULL,
            author_url TEXT NOT NULL,
            author_description TEXT NOT NULL,
            like_count INTEGER NOT NULL,
            download_count INTEGER NOT NULL
        )",
        (),
    )?;
    Ok(())
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Model {
    pub id: Arc<String>,
    pub name: String,
    pub summary: String,
    pub size: String,
    pub requires: String,
    pub architecture: String,
    pub released_at: DateTime<Utc>,
    pub prompt_template: String,
    pub reverse_prompt: String,
    pub author: Arc<Author>,
    pub like_count: u32,
    pub download_count: u32,
}

impl Model {
    pub fn save_to_db(&self, conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT OR REPLACE INTO models (
                id, name, summary, size, requires, architecture, released_at, 
                prompt_template, reverse_prompt, author_name, author_url, 
                author_description, like_count, download_count)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                self.id,
                self.name,
                self.summary,
                self.size,
                self.requires,
                self.architecture,
                self.released_at.to_rfc3339(),
                self.prompt_template,
                self.reverse_prompt,
                self.author.name,
                self.author.url,
                self.author.description,
                self.like_count,
                self.download_count
            ],
        )?;
        Ok(())
    }

    pub fn get_all(conn: &rusqlite::Connection) -> rusqlite::Result<HashMap<String, Model>> {
        let mut stmt = conn.prepare("SELECT * FROM models")?;
        let mut rows = stmt.query([])?;
        let mut models = HashMap::new();

        while let Some(row) = rows.next()? {
            let released_at = chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                .map(|s| s.to_utc())
                .unwrap_or_default();

            let author = Arc::new(Author {
                name: row.get(9)?,
                url: row.get(10)?,
                description: row.get(11)?,
            });

            let id = row.get::<_, String>(0)?;

            models.insert(
                id.clone(),
                Model {
                    id: Arc::new(id),
                    name: row.get(1)?,
                    summary: row.get(2)?,
                    size: row.get(3)?,
                    requires: row.get(4)?,
                    architecture: row.get(5)?,
                    released_at,
                    prompt_template: row.get(7)?,
                    reverse_prompt: row.get(8)?,
                    author,
                    like_count: row.get(12)?,
                    download_count: row.get(13)?,
                },
            );
        }

        Ok(models)
    }
}

#[test]

fn test_sql() {
    let _ = std::fs::remove_file("test_models.db");
    let conn: rusqlite::Connection = rusqlite::Connection::open("test_models.db").unwrap();
    create_table_models(&conn).unwrap();

    let author = Arc::new(Author {
        name: "author1".to_string(),
        url: "url1".to_string(),
        description: "description1".to_string(),
    });

    let model = Model {
        id: Arc::new("1".to_string()),
        name: "model1".to_string(),
        summary: "summary1".to_string(),
        size: "size1".to_string(),
        requires: "requires1".to_string(),
        architecture: "architecture1".to_string(),
        released_at: Utc::now(),
        prompt_template: "prompt_template1".to_string(),
        reverse_prompt: "reverse_prompt1".to_string(),
        author,
        like_count: 0,
        download_count: 0,
    };

    model.save_to_db(&conn).unwrap();
    let models = Model::get_all(&conn).unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[model.id.as_ref()], model);
}

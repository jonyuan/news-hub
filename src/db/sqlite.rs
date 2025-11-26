use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::Path;

use crate::api::NewsItem;
use rusqlite::{params, Connection};

pub struct NewsDB {
    conn: Connection,
}

impl NewsDB {
    pub fn new(path: &str) -> Result<Self> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)
                .context("Failed to create database directory")?;
        }

        let conn = Connection::open(path)
            .context(format!("Failed to open database at {}", path))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS news (
                id TEXT PRIMARY KEY,
                source TEXT,
                title TEXT,
                url TEXT,
                summary TEXT,
                published TEXT
            );",
        )
        .context("Failed to create news table")?;

        Ok(Self { conn })
    }

    pub fn insert(&self, item: &NewsItem) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO news
            (id, source, title, url, summary, published)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                item.id,
                item.source,
                item.title,
                item.url,
                item.summary,
                item.published.to_rfc3339(),
            ],
        )
        .context("Failed to insert news item")?;
        Ok(())
    }

    pub fn load_all(&self) -> Vec<NewsItem> {
        let mut stmt = match self.conn.prepare(
            "SELECT id, source, title, url, summary, published FROM news
             ORDER BY published DESC LIMIT 500",
        ) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to prepare query: {}", e);
                return Vec::new();
            }
        };

        let rows = match stmt.query_map([], |row| {
            let published_str: String = row.get(5)?;
            let published = published_str
                .parse()
                .unwrap_or_else(|_| Utc::now());

            Ok(NewsItem {
                id: row.get(0)?,
                source: row.get(1)?,
                title: row.get(2)?,
                url: row.get(3)?,
                summary: row.get(4)?,
                published,
            })
        }) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to query news: {}", e);
                return Vec::new();
            }
        };

        rows.filter_map(|r| r.ok()).collect()
    }
}

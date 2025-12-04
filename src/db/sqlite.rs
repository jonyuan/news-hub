use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::Path;

use crate::models::NewsItem;
use rusqlite::{params, Connection};

pub struct NewsDB {
    conn: Connection,
}

impl NewsDB {
    pub fn new(path: &str) -> Result<Self> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent).context("Failed to create database directory")?;
        }

        let conn =
            Connection::open(path).context(format!("Failed to open database at {}", path))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS news (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                summary TEXT NOT NULL,
                published TEXT,
                updated_at TEXT NOT NULL,
                UNIQUE(source, url)
            );

            CREATE INDEX IF NOT EXISTS idx_news_published
                ON news(published DESC);

            CREATE INDEX IF NOT EXISTS idx_news_source
                ON news(source);",
        )
        .context("Failed to create news table and indexes")?;

        Ok(Self { conn })
    }

    // currently implemented as an upsert
    pub fn insert(&self, item: &NewsItem) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO news
                (id, source, title, url, summary, published, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                summary = excluded.summary,
                url = excluded.url,
                updated_at = excluded.updated_at",
                params![
                    item.id,
                    item.source,
                    item.title,
                    item.url,
                    item.summary,
                    item.published.to_rfc3339(),
                    item.updated_at.to_rfc3339(),
                ],
            )
            .context("Failed to upsert news item")?;
        Ok(())
    }

    pub fn load_all(&self) -> Result<Vec<NewsItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source, title, url, summary, published, updated_at FROM news
             ORDER BY published DESC LIMIT 500",
        )?;

        let rows = stmt.query_map([], |row| {
            let published_str: String = row.get(5)?;
            let published = published_str.parse().unwrap_or_else(|_| Utc::now());

            let updated_at_str: String = row.get(6)?;
            let updated_at = updated_at_str.parse().unwrap_or_else(|_| published);

            Ok(NewsItem {
                id: row.get(0)?,
                source: row.get(1)?,
                title: row.get(2)?,
                url: row.get(3)?,
                summary: row.get(4)?,
                published,
                updated_at,
            })
        })?;

        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}

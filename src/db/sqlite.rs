use crate::models::NewsItem;
use rusqlite::{params, Connection};

pub struct NewsDB {
    conn: Connection,
}

impl NewsDB {
    pub fn new(path: &str) -> Self {
        let conn = Connection::open(path).unwrap();
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
        .unwrap();
        Self { conn }
    }

    pub fn insert(&self, item: &NewsItem) {
        let _ = self.conn.execute(
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
        );
    }

    pub fn load_all(&self) -> Vec<NewsItem> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, source, title, url, summary, published FROM news
             ORDER BY published DESC LIMIT 500",
            )
            .unwrap();

        let rows = stmt
            .query_map([], |row| {
                Ok(NewsItem {
                    id: row.get(0)?,
                    source: row.get(1)?,
                    title: row.get(2)?,
                    url: row.get(3)?,
                    summary: row.get(4)?,
                    published: row.get::<_, String>(5)?.parse().unwrap(),
                })
            })
            .unwrap();

        rows.filter_map(|ok| ok.ok()).collect()
    }
}

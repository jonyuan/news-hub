use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub id: String,
    pub source: String,
    pub title: String,
    pub url: String,
    pub summary: String,
    pub published: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

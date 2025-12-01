use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rss::Channel;
use sha2::{Sha256, Digest};

use crate::models::NewsItem;
use super::NewsAdaptor;

// Hardcoded default RSS feeds (no API keys required)
pub const DEFAULT_RSS_FEEDS: &[(&str, &str)] = &[
    ("https://www.marketwatch.com/rss/topstories", "MarketWatch"),
    ("https://feeds.bloomberg.com/markets/news.rss", "Bloomberg"),
    ("https://www.cnbc.com/id/100003114/device/rss/rss.html", "CNBC"),
];

/// Generate URL hash for ID fallback
fn url_to_hash(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    format!("{:x}", hasher.finalize())[..16].to_string()
}

/// Generate stable ID from source, GUID (if available), or URL hash
fn generate_stable_id(source_name: &str, guid: Option<&str>, url: &str) -> String {
    let source_slug = source_name.to_lowercase().replace(" ", "-");

    let identifier = if let Some(g) = guid {
        let sanitized = g
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .take(64)
            .collect::<String>();

        if sanitized.is_empty() {
            url_to_hash(url)
        } else {
            sanitized
        }
    } else {
        url_to_hash(url)
    };

    format!("{}-{}", source_slug, identifier)
}

pub struct RssAdaptor {
    url: String,
    source_name: String,
}

impl RssAdaptor {
    pub fn new(url: String, source_name: String) -> Self {
        Self { url, source_name }
    }
}

#[async_trait]
impl NewsAdaptor for RssAdaptor {
    fn name(&self) -> &str {
        &self.source_name
    }

    async fn fetch(&self) -> Result<Vec<NewsItem>> {
        let content = reqwest::get(&self.url)
            .await
            .context("Failed to fetch RSS feed")?
            .bytes()
            .await
            .context("Failed to read RSS response")?;

        let channel = Channel::read_from(&content[..])
            .context("Failed to parse RSS XML")?;

        let now = Utc::now();

        let items = channel
            .items()
            .iter()
            .filter_map(|item| {
                let title = item.title()?.to_string();
                let link = item.link()?.to_string();
                let pub_date = item.pub_date().unwrap_or_default();

                let published = DateTime::parse_from_rfc2822(pub_date)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(now);

                // Extract GUID if available
                let guid = item.guid().map(|g| g.value());

                Some(NewsItem {
                    id: generate_stable_id(&self.source_name, guid, &link),
                    source: self.source_name.clone(),
                    title,
                    url: link,
                    summary: item.description().unwrap_or("").to_string(),
                    published,
                    updated_at: now,
                })
            })
            .collect();

        Ok(items)
    }
}

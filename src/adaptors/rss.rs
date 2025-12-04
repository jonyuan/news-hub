use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rss::Channel;
use sha2::{Digest, Sha256};

use super::NewsAdaptor;
use crate::models::NewsItem;

use tracing::warn;

// Hardcoded default RSS feeds (no API keys required)
pub const DEFAULT_RSS_FEEDS: &[(&str, &str)] = &[
    ("https://www.marketwatch.com/rss/topstories", "MarketWatch"),
    ("https://feeds.bloomberg.com/markets/news.rss", "Bloomberg"),
    (
        "https://www.cnbc.com/id/100003114/device/rss/rss.html",
        "CNBC",
    ),
    ("https://www.barrons.com/rss/topstories", "Barrons"),
    ("https://www.ft.com/rss/home/world", "Financial Times"),
    // looks discontinued
    // ("https://www.reuters.com/rss/worldNews", "Reuters"),
    ("https://www.wsj.com/news/world", "Wall Street Journal"),
    ("https://www.nytimes.com/rss/world", "New York Times"),
    (
        "https://www.investing.com/rss/news_25.rss",
        "Investing.com Stocks",
    ),
    (
        "https://www.investing.com/rss/news_301.rss",
        "Investing.com Crypto",
    ),
    (
        "https://www.investing.com/rss/news_1.rss",
        "Investing.com Forex",
    ),
    (
        "https://www.investing.com/rss/news_1062.rss",
        "Investing.com Earnings",
    ),
    // investing.com catch-all endpoint
    (
        "https://www.investing.com/rss/news.rss",
        "Investing.com Latest",
    ),
];

/// Generate URL hash for ID fallback
fn url_to_hash(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    // CR jyuan: truncate the hash to 16 chars. Collision should be extremely
    // unlikely, but there may be a smarter way to generate and compress
    format!("{:x}", hasher.finalize())[..16].to_string()
}

/// Generate stable ID from source, GUID (if available), or URL hash
/// Uses hybrid approach: hash if isPermaLink=true OR length > 16
fn generate_stable_id(source_name: &str, guid_obj: Option<rss::Guid>, url: &str) -> String {
    let source_slug = source_name.to_lowercase().replace(" ", "-");

    let identifier = if let Some(guid) = guid_obj {
        let guid_str = guid.value();

        // Hash if it's a permalink OR if it's too long (>16 chars)
        if guid.is_permalink() || guid_str.len() > 16 {
            format!("hash-{}", url_to_hash(guid_str))
        } else {
            // Keep original GUID, sanitized and normalized to 16 chars
            let sanitized = guid_str
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .take(16) // Normalize to 16 chars max
                .collect::<String>();

            if sanitized.is_empty() {
                format!("hash-{}", url_to_hash(url))
            } else {
                format!("guid-{}", sanitized)
            }
        }
    } else {
        // No GUID, hash the URL
        format!("hash-{}", url_to_hash(url))
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

    async fn fetch(&self) -> Result<(Vec<NewsItem>, Vec<String>)> {
        let content = reqwest::get(&self.url)
            .await
            .context("Failed to fetch RSS feed")?
            .bytes()
            .await
            .context("Failed to read RSS response")?;

        let channel = Channel::read_from(&content[..]).context("Failed to parse RSS XML")?;

        let now = Utc::now();

        let items: Vec<NewsItem> = channel
            .items()
            .iter()
            .filter_map(|item| {
                let title = item.title()?.to_string();
                let link = item.link()?.to_string();
                let pub_date = item.pub_date()?;

                // if we cannot parse the date, skip this entry
                let published = DateTime::parse_from_rfc2822(pub_date)
                    .or_else(|_| DateTime::parse_from_rfc3339(pub_date))
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()?;

                // Extract GUID object (not just string value) for hybrid handling
                let guid_obj = item.guid().cloned();

                Some(NewsItem {
                    id: generate_stable_id(&self.source_name, guid_obj, &link),
                    source: "RSS_".to_string() + &self.source_name.clone(),
                    title,
                    url: link,
                    summary: item.description().unwrap_or("").to_string(),
                    published,
                    updated_at: now,
                })
            })
            .collect();

        // Count how many items we dropped and build warnings
        let mut warnings = Vec::new();
        let dropped_count = channel.items().len() - items.len();
        if dropped_count > 0 {
            warn!("Dropped {} unparsable RSS items.", dropped_count);
            warnings.push(format!("Dropped {} unparsable items", dropped_count));
        }

        Ok((items, warnings))
    }
}

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rss::Channel;
use uuid::Uuid;

use crate::api::NewsItem;
use super::NewsAdaptor;

// Hardcoded default RSS feeds (no API keys required)
pub const DEFAULT_RSS_FEEDS: &[(&str, &str)] = &[
    ("https://www.marketwatch.com/rss/topstories", "MarketWatch"),
    ("https://feeds.bloomberg.com/markets/news.rss", "Bloomberg"),
    ("https://www.cnbc.com/id/100003114/device/rss/rss.html", "CNBC"),
];

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

        let items = channel
            .items()
            .iter()
            .filter_map(|item| {
                let title = item.title()?.to_string();
                let link = item.link()?.to_string();
                let pub_date = item.pub_date().unwrap_or_default();

                let published = DateTime::parse_from_rfc2822(pub_date)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(Utc::now());

                Some(NewsItem {
                    id: Uuid::new_v4().to_string(),
                    source: self.source_name.clone(),
                    title,
                    url: link,
                    summary: item.description().unwrap_or("").to_string(),
                    published,
                })
            })
            .collect();

        Ok(items)
    }
}

use anyhow::Result;
use async_trait::async_trait;

use crate::models::NewsItem;

mod benzinga;
mod rss;

pub use benzinga::BenzingaAdaptor;
pub use rss::{RssAdaptor, DEFAULT_RSS_FEEDS};

/// Trait for news adaptors - requires Send + Sync for tokio::spawn (thread safety)
#[async_trait]
pub trait NewsAdaptor: Send + Sync {
    /// Unique identifier for this adaptor
    fn name(&self) -> &str;

    /// Fetch news items from this source
    async fn fetch(&self) -> Result<Vec<NewsItem>>;

    /// Check if adaptor is properly configured (default: true)
    fn is_enabled(&self) -> bool {
        true
    }
}

/// Fetch from all enabled adaptors
pub async fn fetch_all(adaptors: &[Box<dyn NewsAdaptor>]) -> Vec<NewsItem> {
    let mut all_items = Vec::new();

    for adaptor in adaptors {
        if !adaptor.is_enabled() {
            continue;
        }

        match adaptor.fetch().await {
            Ok(items) => {
                eprintln!("✓ Fetched {} items from {}", items.len(), adaptor.name());
                all_items.extend(items);
            }
            Err(e) => {
                eprintln!("✗ Failed to fetch from {}: {}", adaptor.name(), e);
            }
        }
    }

    all_items
}

/// Build adaptors dynamically based on available API keys
pub fn build_adaptors(
    benzinga_key: Option<String>,
    // Future: add more API keys here
) -> Vec<Box<dyn NewsAdaptor>> {
    let mut adaptors: Vec<Box<dyn NewsAdaptor>> = Vec::new();

    // Always add RSS feeds (no API key required)
    for (url, name) in DEFAULT_RSS_FEEDS {
        adaptors.push(Box::new(RssAdaptor::new(
            url.to_string(),
            name.to_string(),
        )));
    }

    // Conditionally add API-based adaptors
    if let Some(key) = benzinga_key {
        adaptors.push(Box::new(BenzingaAdaptor::new(key)));
    }

    // Future: Add MarketAux, Reddit, etc.

    adaptors
}

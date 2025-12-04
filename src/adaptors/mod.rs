use anyhow::Result;
use async_trait::async_trait;

use crate::models::NewsItem;

mod benzinga;
mod rss;

pub use benzinga::BenzingaAdaptor;
pub use rss::{RssAdaptor, DEFAULT_RSS_FEEDS};

/// Diagnostic information for a single fetch operation
#[derive(Debug, Clone)]
pub struct FetchDiagnostic {
    pub source: String,
    pub success: bool,
    pub message: String,
    pub warnings: Vec<String>,
}

/// Result of fetching from all adaptors, including diagnostics
#[derive(Debug)]
pub struct FetchResult {
    pub items: Vec<NewsItem>,
    pub diagnostics: Vec<FetchDiagnostic>,
}

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
pub async fn fetch_all(adaptors: &[Box<dyn NewsAdaptor>]) -> FetchResult {
    let mut all_items = Vec::new();
    let mut diagnostics = Vec::new();

    for adaptor in adaptors {
        if !adaptor.is_enabled() {
            continue;
        }

        match adaptor.fetch().await {
            Ok(items) => {
                diagnostics.push(FetchDiagnostic {
                    source: adaptor.name().to_string(),
                    success: true,
                    message: format!("Fetched {} items", items.len()),
                    warnings: Vec::new(),
                });
                all_items.extend(items);
            }
            Err(e) => {
                diagnostics.push(FetchDiagnostic {
                    source: adaptor.name().to_string(),
                    success: false,
                    message: format!("Failed: {}", e),
                    warnings: Vec::new(),
                });
            }
        }
    }

    FetchResult {
        items: all_items,
        diagnostics,
    }
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

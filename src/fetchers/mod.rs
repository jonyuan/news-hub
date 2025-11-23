mod benzinga;
mod rss;

pub use benzinga::fetch_benzinga;
pub use rss::fetch_rss;

use crate::models::NewsItem;

pub async fn fetch_all(api_key: &str) -> Vec<NewsItem> {
    let mut out = Vec::new();

    // RSS
    let mw = fetch_rss("https://www.marketwatch.com/rss/topstories", "MarketWatch").await;
    out.extend(mw);

    // Benzinga API
    let bz = fetch_benzinga(api_key).await;
    out.extend(bz);

    out
}

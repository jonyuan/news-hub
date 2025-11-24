use crate::api::NewsItem;
use chrono::{DateTime, Utc};
use rss::Channel;
use uuid::Uuid;

pub async fn fetch_rss(url: &str, source: &str) -> Vec<NewsItem> {
    let content = reqwest::get(url).await.unwrap().bytes().await.unwrap();
    let channel = Channel::read_from(&content[..]).unwrap();

    channel
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
                source: source.to_string(),
                title,
                url: link,
                summary: item.description().unwrap_or("").to_string(),
                published,
            })
        })
        .collect()
}

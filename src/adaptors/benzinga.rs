use crate::api::NewsItem;
use chrono::{TimeZone, Utc};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct BenzArticle {
    id: i64,
    title: String,
    url: String,
    description: Option<String>,
    updated: i64,
}

#[derive(Deserialize)]
struct BenzResp {
    articles: Vec<BenzArticle>,
}

pub async fn fetch_benzinga(api_key: &str) -> Vec<NewsItem> {
    let url = "https://api.benzinga.com/api/v2/news";
    let client = Client::new();

    let resp: BenzResp = client
        .get(url)
        .query(&[("token", api_key), ("pagesize", "50")])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    resp.articles
        .into_iter()
        .map(|n| NewsItem {
            id: n.id.to_string(),
            source: "benzinga".into(),
            title: n.title,
            url: n.url,
            summary: n.description.unwrap_or_default(),
            published: Utc.timestamp_opt(n.updated, 0).unwrap(),
        })
        .collect()
}

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use reqwest::Client;
use serde::Deserialize;

use crate::models::NewsItem;
use super::NewsAdaptor;

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

pub struct BenzingaAdaptor {
    api_key: String,
    client: Client,
}

impl BenzingaAdaptor {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl NewsAdaptor for BenzingaAdaptor {
    fn name(&self) -> &str {
        "Benzinga"
    }

    fn is_enabled(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn fetch(&self) -> Result<Vec<NewsItem>> {
        let url = "https://api.benzinga.com/api/v2/news";

        let resp: BenzResp = self.client
            .get(url)
            .query(&[("token", self.api_key.as_str()), ("pagesize", "50")])
            .send()
            .await
            .context("Failed to connect to Benzinga API")?
            .json()
            .await
            .context("Failed to parse Benzinga response")?;

        let items = resp.articles
            .into_iter()
            .map(|n| NewsItem {
                id: format!("benzinga-{}", n.id),
                source: "Benzinga".into(),
                title: n.title,
                url: n.url,
                summary: n.description.unwrap_or_default(),
                published: Utc.timestamp_opt(n.updated, 0)
                    .single()
                    .unwrap_or_else(|| Utc::now()),
            })
            .collect();

        Ok(items)
    }
}

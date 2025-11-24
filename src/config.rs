use anyhow::{Context, Result};
use dotenvy::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub app: AppConfig,
    // pub adapters: AdapterSettings,
    // pub scoring: ScoringConfig,
    pub credentials: ApiKeys,
}

#[derive(Debug, Deserialize)]
pub struct TomlConfig {
    pub app: AppConfig,
    // pub adapters: AdapterSettings,
    // pub scoring: ScoringConfig,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub deduplication_enabled: bool,
    pub similarity_threshold: f32,
}

#[derive(Debug)]
pub struct ApiKeys {
    pub marketaux_key: Option<String>,
    pub reddit_client_id: Option<String>,
    pub reddit_client_secret: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Load .env file (silently fail if not present)
        dotenv().ok();

        // Load TOML config
        let config_contents =
            std::fs::read_to_string("config.toml").context("Failed to read config.toml")?;
        let toml_config: TomlConfig =
            toml::from_str(&config_contents).context("Failed to parse config.toml")?;

        // Load credentials from environment
        let credentials = ApiKeys {
            marketaux_key: env::var("MARKETAUX_API_KEY").ok(),
            reddit_client_id: env::var("REDDIT_CLIENT_ID").ok(),
            reddit_client_secret: env::var("REDDIT_CLIENT_SECRET").ok(),
        };

        // Validate at least one adapter is configured
        if credentials.marketaux_key.is_none() && credentials.reddit_client_id.is_none() {
            anyhow::bail!(
                "No API credentials found! Please set environment variables.\n\
                 See .env.example for instructions."
            );
        }

        Ok(Self {
            app: toml_config.app,
            // adapters: toml_config.adapters,
            // scoring: toml_config.scoring,
            credentials,
        })
    }
}

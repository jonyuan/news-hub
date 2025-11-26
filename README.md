# news-hub

A Poor Man's Bloomberg Terminal: a financial news aggregator and TUI feed built in Rust with [ratatui](https://ratatui.rs/). During my garden leave, I found my news sources quite fragmented, and wanted a lightweight, customizable news feed for personal use.

## Quickstart

```bash
# 1. Clone repo
git clone https://github.com/jonyuan/news-hub
cd news-hub

# 2. Copy example files
cp .env.example .env
cp config.toml.example config.toml

# 3. Edit .env with your API keys
vim .env

# 4. Run
cargo run
```

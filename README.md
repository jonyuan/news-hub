# news-hub

A Poor Man's Bloomberg Terminal: a financial news aggregator and TUI feed built in Rust with [ratatui](https://ratatui.rs/). During my garden leave, I found my news sources quite fragmented, and wanted a centralized, lightweight, cheap, and customizable news feed for personal use.

## Quickstart

```bash
# 1. Clone repo
git clone https://github.com/jonyuan/news-hub
cd news-hub

# 2. Copy example files
cp .env.example .env
cp config.toml.example config.toml

# 3. Edit .env with your API keys. You can skip this step for a
# barebones, out of the box setup
vim .env

# 4. Run
cargo run
```

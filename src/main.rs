use crossterm::event::{Event, KeyCode};
use crossterm::{event, execute, terminal};
use dotenvy::dotenv;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tokio::time::{sleep, Duration};

use news_hub::db::sqlite::NewsDB;
use news_hub::fetchers::fetch_all;
use news_hub::ui::draw_ui;

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    let api_key = std::env::var("BENZINGA_KEY").expect("Missing API key");

    let db = NewsDB::new("data/news.db");

    // TUI setup
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(&mut stdout, terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut news_cache = db.load_all();

    loop {
        draw_ui(&mut terminal, &news_cache)?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(k) = event::read()? {
                if k.code == KeyCode::Char('q') {
                    break;
                }
                if k.code == KeyCode::Char('r') {
                    let latest = fetch_all(&api_key).await;
                    for item in &latest {
                        db.insert(item);
                    }
                    news_cache = db.load_all();
                }
            }
        }

        // periodic auto-refresh every 5 minutes
        // (non-blocking)
        sleep(Duration::from_secs(5)).await;
    }

    // cleanup
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

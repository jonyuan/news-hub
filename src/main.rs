use crossterm::{event, execute, terminal};
use dotenvy::dotenv;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::Duration;

use news_hub::adaptors::{build_adaptors, fetch_all};
use news_hub::app::{App, AppMessage, AppState};
use news_hub::db::sqlite::NewsDB;
use news_hub::ui::{draw_ui, Action, StatusMessage};

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    // Initialize file-based logging
    let log_dir = std::path::Path::new("logs");
    if !log_dir.exists() {
        std::fs::create_dir_all(log_dir)?;
    }

    let log_file = std::fs::File::create("logs/news-hub.log")?;
    tracing_subscriber::fmt()
        .with_writer(log_file)
        .with_ansi(false)
        .init();

    let benzinga_key = std::env::var("BENZINGA_KEY").ok();

    let db = NewsDB::new("data/news.db").expect("Failed to initialize database");

    // Build adaptors dynamically based on available API keys
    let adaptors = Arc::new(build_adaptors(benzinga_key.clone()));

    // TUI setup
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(&mut stdout, terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app with database-loaded news
    let initial_news = match db.load_all() {
        Ok(news) => news,
        Err(e) => {
            eprintln!("Failed to load news from database: {}", e);
            Vec::new()
        }
    };

    // Check if empty before moving
    let is_empty = initial_news.is_empty();
    let mut app = App::new(initial_news);

    // Show initial status if database had errors
    if is_empty {
        let msg = StatusMessage::warning("Database is empty. Press 'r' to fetch news.".to_string());
        app.status_bar.set_message(msg);
    }

    // Channel for background task communication
    let (tx, mut rx) = mpsc::unbounded_channel();

    loop {
        // Draw UI with current state
        draw_ui(
            &mut terminal,
            &app.search_bar,
            &app.news_list,
            &app.detail_pane,
            &app.status_bar,
            app.app_state,
        )?;

        // Check for background task messages (non-blocking)
        if let Ok(msg) = rx.try_recv() {
            app.handle_message(msg, &db);
        }

        // Update spinner and check auto-dismiss
        app.tick();

        // Poll for keyboard input
        if event::poll(Duration::from_millis(200))? {
            let event = event::read()?;

            // Handle events through component system
            let action = app.handle_event(&event);

            // Handle refresh action in background
            if matches!(action, Action::RefreshRequested) {
                if matches!(app.app_state, AppState::Idle) {
                    app.app_state = AppState::Loading;

                    // Show loading message
                    let loading_msg = StatusMessage::loading("Fetching news...".to_string());
                    app.status_bar.set_message(loading_msg);

                    let tx = tx.clone();
                    let adaptors = Arc::clone(&adaptors);

                    tokio::spawn(async move {
                        let result = fetch_all(&adaptors).await;
                        let msg = if result.items.is_empty() {
                            AppMessage::RefreshFailed("No items fetched".to_string())
                        } else {
                            AppMessage::RefreshComplete {
                                items: result.items,
                                diagnostics: result.diagnostics,
                            }
                        };
                        let _ = tx.send(msg);
                    });
                }
            }

            // Handle other actions (like quit, open URL)
            if !app.handle_action(&action) {
                break;
            }
        }
    }

    // Cleanup
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

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
use news_hub::ui::{draw_ui, Action};

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
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
    let initial_news = db.load_all();
    let mut app = App::new(initial_news);

    // Channel for background task communication
    let (tx, mut rx) = mpsc::unbounded_channel();

    loop {
        // Draw UI with current state
        draw_ui(&mut terminal, &app.news_list, &app.detail_pane, app.app_state)?;

        // Check for background task messages (non-blocking)
        if let Ok(msg) = rx.try_recv() {
            app.handle_message(msg, &db);
        }

        // Poll for keyboard input
        if event::poll(Duration::from_millis(200))? {
            let event = event::read()?;

            // Handle events through component system
            let action = app.handle_event(&event);

            // Handle refresh action in background
            if matches!(action, Action::RefreshRequested) {
                if matches!(app.app_state, AppState::Idle) {
                    app.app_state = AppState::Loading;
                    let tx = tx.clone();
                    let adaptors = Arc::clone(&adaptors);

                    tokio::spawn(async move {
                        let items = fetch_all(&adaptors).await;
                        let msg = if items.is_empty() {
                            AppMessage::RefreshFailed("No items fetched".to_string())
                        } else {
                            AppMessage::RefreshComplete(items)
                        };
                        let _ = tx.send(msg);
                    });
                }
            }

            // Handle other actions (like quit, open URL)
            if !app.handle_action(&action) {
                break; // Quit signal
            }
        }
    }

    // Cleanup
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

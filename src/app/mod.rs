use crossterm::event::KeyCode;

use crate::api::NewsItem;
use crate::db::sqlite::NewsDB;

/// Messages sent from background tasks to main event loop
#[derive(Debug)]
pub enum AppMessage {
    RefreshComplete(Vec<NewsItem>),
    RefreshFailed(String),
}

/// Application state machine
#[derive(Debug, Clone, Copy)]
pub enum AppState {
    Idle,
    Loading,
}

/// Main application state
pub struct App {
    pub news_cache: Vec<NewsItem>,
    pub app_state: AppState,
    pub selected_index: usize,
}

impl App {
    pub fn new(initial_news: Vec<NewsItem>) -> Self {
        Self {
            news_cache: initial_news,
            app_state: AppState::Idle,
            selected_index: 0,
        }
    }

    /// Handle messages from background tasks
    pub fn handle_message(&mut self, msg: AppMessage, db: &NewsDB) {
        match msg {
            AppMessage::RefreshComplete(items) => {
                for item in &items {
                    if let Err(e) = db.insert(item) {
                        eprintln!("Failed to insert item: {}", e);
                    }
                }
                self.news_cache = db.load_all();
                self.app_state = AppState::Idle;
                self.selected_index = 0; // Reset to top
            }
            AppMessage::RefreshFailed(err) => {
                eprintln!("Fetch failed: {}", err);
                self.app_state = AppState::Idle;
            }
        }
    }

    /// Handle keyboard input
    /// Returns false if app should quit, true otherwise
    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('q') => return false, // Signal quit
            KeyCode::Down => {
                if self.selected_index < self.news_cache.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
            }
            KeyCode::Up => {
                self.selected_index = self.selected_index.saturating_sub(1);
            }
            KeyCode::Enter | KeyCode::Char('o') => {
                if let Some(item) = self.news_cache.get(self.selected_index) {
                    if let Err(e) = open::that(&item.url) {
                        eprintln!("Failed to open browser: {}", e);
                    }
                }
            }
            _ => {}
        }
        true // Continue running
    }
}

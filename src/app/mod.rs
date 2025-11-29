use crossterm::event::Event;

use crate::models::NewsItem;
use crate::db::sqlite::NewsDB;
use crate::ui::{Action, Component, NewsListComponent};

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
    pub news_list: NewsListComponent,
    pub app_state: AppState,
}

impl App {
    pub fn new(initial_news: Vec<NewsItem>) -> Self {
        Self {
            news_list: NewsListComponent::new(initial_news),
            app_state: AppState::Idle,
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
                let news = db.load_all();
                self.news_list.set_news(news);
                self.app_state = AppState::Idle;
            }
            AppMessage::RefreshFailed(err) => {
                eprintln!("Fetch failed: {}", err);
                self.app_state = AppState::Idle;
            }
        }
    }

    /// Handle keyboard/mouse events
    /// Returns the Action emitted by components
    pub fn handle_event(&mut self, event: &Event) -> Action {
        let action = self.news_list.handle_event(event);

        // Update all components based on the action
        self.news_list.update(&action);

        action
    }

    /// Handle an Action and perform side effects (like opening URLs)
    /// Returns false if app should quit, true otherwise
    pub fn handle_action(&mut self, action: &Action) -> bool {
        match action {
            Action::Quit => return false,
            Action::ArticleOpened(url) => {
                if let Err(e) = open::that(url) {
                    eprintln!("Failed to open browser: {}", e);
                }
            }
            _ => {}
        }
        true
    }
}

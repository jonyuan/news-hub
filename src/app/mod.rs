use crossterm::event::{Event, KeyCode, KeyEvent};

use crate::models::NewsItem;
use crate::db::sqlite::NewsDB;
use crate::ui::{Action, Component, NewsListComponent, DetailPaneComponent, SearchBarComponent};

/// Identifies which component currently has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentId {
    SearchBar,
    NewsList,
    DetailPane,
}

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
    pub search_bar: SearchBarComponent,
    pub news_list: NewsListComponent,
    pub detail_pane: DetailPaneComponent,
    pub app_state: AppState,
    pub focused_component: ComponentId,
}

impl App {
    pub fn new(initial_news: Vec<NewsItem>) -> Self {
        let mut search_bar = SearchBarComponent::new();
        search_bar.set_focus(false);

        let mut news_list = NewsListComponent::new(initial_news);
        news_list.set_focus(true); // NewsList starts with focus

        let mut detail_pane = DetailPaneComponent::new();
        detail_pane.set_focus(false);

        // Initialize detail pane with first article if available
        if let Some(first_article) = news_list.selected_item() {
            detail_pane.set_article(first_article.clone());
        }

        Self {
            search_bar,
            news_list,
            detail_pane,
            app_state: AppState::Idle,
            focused_component: ComponentId::NewsList,
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

                // Update detail pane with first article after refresh
                if let Some(first_article) = self.news_list.selected_item() {
                    self.detail_pane.set_article(first_article.clone());
                }

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
        // Handle Tab key for focus switching
        if let Event::Key(KeyEvent { code: KeyCode::Tab, .. }) = event {
            self.cycle_focus();
            return Action::None;
        }

        // Route event to focused component
        let action = match self.focused_component {
            ComponentId::SearchBar => self.search_bar.handle_event(event),
            ComponentId::NewsList => self.news_list.handle_event(event),
            ComponentId::DetailPane => self.detail_pane.handle_event(event),
        };

        // Update all components based on the action
        self.search_bar.update(&action);
        self.news_list.update(&action);
        self.detail_pane.update(&action);

        // Handle selection changes to update detail pane
        if let Action::SelectionChanged(_) = action {
            if let Some(selected_article) = self.news_list.selected_item() {
                self.detail_pane.set_article(selected_article.clone());
            }
        }

        action
    }

    /// Cycle focus between components (Tab key)
    fn cycle_focus(&mut self) {
        self.focused_component = match self.focused_component {
            ComponentId::SearchBar => {
                self.search_bar.set_focus(false);
                self.news_list.set_focus(true);
                ComponentId::NewsList
            }
            ComponentId::NewsList => {
                self.news_list.set_focus(false);
                self.detail_pane.set_focus(true);
                ComponentId::DetailPane
            }
            ComponentId::DetailPane => {
                self.detail_pane.set_focus(false);
                self.search_bar.set_focus(true);
                ComponentId::SearchBar
            }
        };
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

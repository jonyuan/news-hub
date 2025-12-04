use crossterm::event::{Event, KeyCode, KeyEvent};

use crate::adaptors::FetchDiagnostic;
use crate::db::sqlite::NewsDB;
use crate::models::NewsItem;
use crate::ui::{
    Action, Component, DetailPaneComponent, NewsListComponent, SearchBarComponent,
    StatusBarComponent, StatusMessage,
};

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
    RefreshComplete {
        items: Vec<NewsItem>,
        diagnostics: Vec<FetchDiagnostic>,
    },
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
    pub status_bar: StatusBarComponent,
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
            status_bar: StatusBarComponent::new(),
            app_state: AppState::Idle,
            focused_component: ComponentId::NewsList,
        }
    }

    /// Handle messages from background tasks
    pub fn handle_message(&mut self, msg: AppMessage, db: &NewsDB) {
        match msg {
            AppMessage::RefreshComplete { items, diagnostics } => {
                // Collect database insertion errors
                let mut db_errors = Vec::new();
                for item in &items {
                    if let Err(e) = db.insert(item) {
                        db_errors.push(format!("{}", e));
                    }
                }

                // Build status message from diagnostics
                let success_count = diagnostics.iter().filter(|d| d.success).count();
                let fail_count = diagnostics.iter().filter(|d| !d.success).count();

                let status_msg = if fail_count == 0 && db_errors.is_empty() {
                    StatusMessage::success(format!(
                        "Fetched {} items from {} sources",
                        items.len(),
                        success_count
                    ))
                } else if success_count > 0 {
                    let mut msg_parts = vec![format!(
                        "Fetched {} items from {} sources",
                        items.len(),
                        success_count
                    )];
                    if fail_count > 0 {
                        msg_parts.push(format!("{} sources failed", fail_count));
                    }
                    if !db_errors.is_empty() {
                        msg_parts.push(format!("{} DB errors", db_errors.len()));
                    }
                    StatusMessage::warning(msg_parts.join("; "))
                } else {
                    StatusMessage::error("All sources failed to fetch".to_string())
                };

                self.status_bar.set_message(status_msg);

                // Reload from database
                let news = match db.load_all() {
                    Ok(news) => news,
                    Err(e) => {
                        let msg =
                            StatusMessage::error(format!("Failed to load from database: {}", e));
                        self.status_bar.set_message(msg);
                        Vec::new()
                    }
                };
                self.news_list.set_news(news);

                // Update detail pane with first article after refresh
                if let Some(first_article) = self.news_list.selected_item() {
                    self.detail_pane.set_article(first_article.clone());
                }

                self.app_state = AppState::Idle;
            }
            AppMessage::RefreshFailed(err) => {
                let msg = StatusMessage::error(format!("Fetch failed: {}", err));
                self.status_bar.set_message(msg);
                self.app_state = AppState::Idle;
            }
        }
    }

    /// Handle keyboard/mouse events
    /// Returns the Action emitted by components
    pub fn handle_event(&mut self, event: &Event) -> Action {
        // TODO: move the status bar logic into its handle_event() method
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('h'),
            modifiers,
            ..
        }) = event
        {
            if modifiers.is_empty() && !self.search_bar.is_focused() {
                self.status_bar.set_focus(!self.status_bar.is_focused());
                return Action::None;
            }
        }

        if let Event::Key(KeyEvent {
            code: KeyCode::Esc, ..
        }) = event
        {
            // Esc is a triple-overloaded event:
            // leave the status bar, dismiss messsage, or leave search
            // (handled by search_bar.handle_event())
            if self.status_bar.is_focused() {
                self.status_bar.set_focus(false);
                return Action::None;
            }
            if !self.search_bar.is_focused() {
                let action = Action::DismissStatus;
                self.status_bar.update(&action);

                return Action::None;
            }
        }

        // Handle '/' to enter search mode
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('/'),
            modifiers,
            ..
        }) = event
        {
            if modifiers.is_empty() && !self.search_bar.is_focused() {
                self.search_bar.set_focus(true);
                return Action::None;
            }
        }

        // check if we are in search mode
        if self.search_bar.is_focused() {
            let action = self.search_bar.handle_event(event);

            // If SearchBar handled it, broadcast and return
            if !matches!(action, Action::None) {
                self.news_list.update(&action);
                self.detail_pane.update(&action);
                self.status_bar.update(&action);
                return action;
            }
            // SearchBar returned Action::None, fall through to focused component
        }

        // Handle Tab key for focus switching
        if let Event::Key(KeyEvent {
            code: KeyCode::Tab, ..
        }) = event
        {
            self.cycle_focus();
            return Action::None;
        }

        // Route to focused component (NewsList or DetailPane)
        let action = match self.focused_component {
            ComponentId::NewsList => self.news_list.handle_event(event),
            ComponentId::DetailPane => self.detail_pane.handle_event(event),
            ComponentId::SearchBar => Action::None,
        };

        // Broadcast action to all components including status_bar
        self.news_list.update(&action);
        self.detail_pane.update(&action);
        self.status_bar.update(&action);

        // Handle selection changes to update detail pane
        if let Action::SelectionChanged(_) = action {
            if let Some(selected_article) = self.news_list.selected_item() {
                self.detail_pane.set_article(selected_article.clone());
            }
        }

        action
    }

    /// Cycle focus between NewsList and DetailPane only (Tab key)
    fn cycle_focus(&mut self) {
        self.focused_component = match self.focused_component {
            ComponentId::NewsList => {
                self.news_list.set_focus(false);
                self.detail_pane.set_focus(true);
                ComponentId::DetailPane
            }
            ComponentId::DetailPane => {
                self.detail_pane.set_focus(false);
                self.news_list.set_focus(true);
                ComponentId::NewsList
            }
            ComponentId::SearchBar => {
                // Should not reach here, but default to NewsList
                self.search_bar.set_focus(false);
                self.news_list.set_focus(true);
                ComponentId::NewsList
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
                    let msg = StatusMessage::error(format!("Failed to open browser: {}", e));
                    self.status_bar.set_message(msg);
                }
            }
            _ => {}
        }
        true
    }

    /// Periodic update for spinner animation and auto-dismiss checks
    pub fn tick(&mut self) {
        self.status_bar.tick_spinner();
        self.status_bar.check_auto_dismiss();
    }
}

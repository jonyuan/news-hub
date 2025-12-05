use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::adaptors::FetchDiagnostic;
use crate::db::sqlite::NewsDB;
use crate::models::NewsItem;
use crate::ui::{
    Action, Component, DetailPaneComponent, NewsListComponent, SearchBarComponent,
    StatusBarComponent, StatusMessage,
};

/// Identifies which component currently has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabComponent {
    NewsList,
    DetailPane,
    StatusBar,
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
    pub focused_component: TabComponent,
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
            focused_component: TabComponent::NewsList,
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

                // Collect per-source warnings
                let warnings: Vec<String> = diagnostics
                    .iter()
                    .filter(|d| d.success && !d.warnings.is_empty())
                    .flat_map(|d| d.warnings.iter().map(|w| format!("{}: {}", d.source, w)))
                    .collect();

                let has_warnings = !warnings.is_empty();

                let status_msg = if fail_count == 0 && db_errors.is_empty() && !has_warnings {
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
                    if has_warnings {
                        msg_parts.extend(warnings);
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

    /// handle keyboard/mouse events. Returns the Action emitted by components
    pub fn handle_event(&mut self, event: &Event) -> Action {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }) = event
        {
            let was_showing = self.status_bar.is_showing_history();
            self.status_bar.toggle_history();
            let is_showing = self.status_bar.is_showing_history();

            // If we just closed history and StatusBar was focused, move focus to NewsList
            if was_showing && !is_showing && self.focused_component == TabComponent::StatusBar {
                self.status_bar.set_focus(false);
                self.news_list.set_focus(true);
                self.focused_component = TabComponent::NewsList;
            }

            return Action::None;
        }

        // Esc is an overloaded event. This one checks only for dismissing status
        if let Event::Key(KeyEvent {
            code: KeyCode::Esc, ..
        }) = event
        {
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

        if self.search_bar.is_focused() {
            let action = self.search_bar.handle_event(event);

            // If SearchBar handled it, broadcast and return
            if !matches!(action, Action::None) {
                self.update_all(&action);
                return action;
            }
            // SearchBar returned Action::None, fall through to focused component
        }

        // tab is unique
        if let Event::Key(KeyEvent {
            code: KeyCode::Tab, ..
        }) = event
        {
            self.cycle_focus();
            return Action::None;
        }

        // 'r' and 'q' should be handled by the app globally at this point
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('r'),
            ..
        }) = event
        {
            return Action::RefreshRequested;
        }

        if let Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            ..
        }) = event
        {
            return Action::Quit;
        }

        // Route to focused component (NewsList, DetailPane, or StatusBar)
        let action = match self.focused_component {
            TabComponent::NewsList => self.news_list.handle_event(event),
            TabComponent::DetailPane => self.detail_pane.handle_event(event),
            TabComponent::StatusBar => self.status_bar.handle_event(event),
        };
        self.update_all(&action);
        action
    }

    fn update_all(&mut self, action: &Action) {
        // Broadcast action to all components including status_bar
        self.news_list.update(&action);
        self.detail_pane.update(&action);
        self.search_bar.update(&action);
        self.status_bar.update(&action);

        // Handle selection changes that need to update detail pane
        match action {
            Action::SelectionChanged(_) | Action::SearchQueryChanged(_) => {
                if let Some(selected_article) = self.news_list.selected_item() {
                    self.detail_pane.set_article(selected_article.clone());
                }
            }
            _ => {}
        }
    }

    /// Tab to cycle focus (dynamic 2/3-way cycle based on history visibility)
    fn cycle_focus(&mut self) {
        self.focused_component = match self.focused_component {
            TabComponent::NewsList => {
                self.news_list.set_focus(false);
                self.detail_pane.set_focus(true);
                TabComponent::DetailPane
            }
            TabComponent::DetailPane => {
                self.detail_pane.set_focus(false);
                // Only include StatusBar in cycle if history is showing
                if self.status_bar.is_showing_history() {
                    self.status_bar.set_focus(true);
                    TabComponent::StatusBar
                } else {
                    // Skip StatusBar, go back to NewsList
                    self.news_list.set_focus(true);
                    TabComponent::NewsList
                }
            }
            TabComponent::StatusBar => {
                self.status_bar.set_focus(false);
                self.news_list.set_focus(true);
                TabComponent::NewsList
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

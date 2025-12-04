use crate::models::NewsItem;
use crate::ui::component::{Action, Component};
use chrono::Local;
use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub struct NewsListComponent {
    all_news: Vec<NewsItem>,      // Unfiltered news
    filtered_news: Vec<NewsItem>, // Filtered based on search query
    search_query: String,
    selected_index: usize,
    focused: bool,
}

impl NewsListComponent {
    pub fn new(news: Vec<NewsItem>) -> Self {
        let filtered_news = news.clone();
        Self {
            all_news: news,
            filtered_news,
            search_query: String::new(),
            selected_index: 0,
            focused: true,
        }
    }

    pub fn set_news(&mut self, news: Vec<NewsItem>) {
        self.all_news = news;
        self.apply_filter();
        self.selected_index = 0;
    }

    pub fn selected_item(&self) -> Option<&NewsItem> {
        self.filtered_news.get(self.selected_index)
    }

    fn apply_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_news = self.all_news.clone();
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.filtered_news = self
                .all_news
                .iter()
                .filter(|item| {
                    item.title.to_lowercase().contains(&query_lower)
                        || item.summary.to_lowercase().contains(&query_lower)
                        || item.source.to_lowercase().contains(&query_lower)
                })
                .cloned()
                .collect();
        }

        // Reset selection if out of bounds
        if self.selected_index >= self.filtered_news.len() {
            self.selected_index = 0;
        }
    }
}

impl Component for NewsListComponent {
    fn handle_event(&mut self, event: &Event) -> Action {
        if !self.focused {
            return Action::None;
        }

        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Down => {
                    if self.selected_index < self.filtered_news.len().saturating_sub(1) {
                        self.selected_index += 1;
                        return Action::SelectionChanged(self.selected_index);
                    }
                }
                KeyCode::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                        return Action::SelectionChanged(self.selected_index);
                    }
                }
                KeyCode::Enter | KeyCode::Char('o') => {
                    if let Some(item) = self.selected_item() {
                        return Action::ArticleOpened(item.url.clone());
                    }
                }
                KeyCode::Char('r') => {
                    return Action::RefreshRequested;
                }
                KeyCode::Char('q') => {
                    return Action::Quit;
                }
                _ => {}
            }
        }

        Action::None
    }

    fn update(&mut self, action: &Action) {
        match action {
            Action::SelectionChanged(index) => {
                if *index < self.filtered_news.len() {
                    self.selected_index = *index;
                }
            }
            Action::SearchQueryChanged(query) => {
                self.search_query = query.clone();
                self.apply_filter();
            }
            _ => {}
        }
    }

    fn render(&self, f: &mut Frame, area: Rect) {
        let title = if self.search_query.is_empty() {
            format!("News Feed ({} articles)", self.filtered_news.len())
        } else {
            format!(
                "News Feed ({}/{} filtered)",
                self.filtered_news.len(),
                self.all_news.len()
            )
        };

        let items: Vec<ListItem> = self
            .filtered_news
            .iter()
            .enumerate()
            .map(|(i, n)| {
                // CR jyuan: updated_at is not a great fallback for published date
                let time_diff = Local::now().signed_duration_since(n.published);
                let time_str = if time_diff.num_hours() < 1 {
                    format!("{}m ago", time_diff.num_minutes())
                } else if time_diff.num_hours() < 24 {
                    format!("{}h ago", time_diff.num_hours())
                } else {
                    format!("{}d ago", time_diff.num_days())
                };

                let content = format!("{:<8} {}  —  {}", time_str, n.title, n.source);

                if i == self.selected_index {
                    ListItem::new(content).style(
                        Style::default()
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    ListItem::new(content)
                }
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(if self.focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                }),
        );

        f.render_widget(list, area);
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

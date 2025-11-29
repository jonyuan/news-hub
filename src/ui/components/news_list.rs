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
    news: Vec<NewsItem>,
    selected_index: usize,
    focused: bool,
}

impl NewsListComponent {
    pub fn new(news: Vec<NewsItem>) -> Self {
        Self {
            news,
            selected_index: 0,
            focused: true,
        }
    }

    pub fn set_news(&mut self, news: Vec<NewsItem>) {
        self.news = news;
        self.selected_index = 0;
    }

    pub fn selected_item(&self) -> Option<&NewsItem> {
        self.news.get(self.selected_index)
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
                    if self.selected_index < self.news.len().saturating_sub(1) {
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
                if *index < self.news.len() {
                    self.selected_index = *index;
                }
            }
            _ => {}
        }
    }

    fn render(&self, f: &mut Frame, area: Rect) {
        let title = "News Feed [Press 'r' to refresh, 'q' to quit, Enter/o to open]";

        let items: Vec<ListItem> = self
            .news
            .iter()
            .enumerate()
            .map(|(i, n)| {
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

        let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));

        f.render_widget(list, area);
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

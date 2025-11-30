use crate::models::NewsItem;
use crate::ui::component::{Action, Component};
use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct DetailPaneComponent {
    article: Option<NewsItem>,
    scroll_offset: u16,
    focused: bool,
}

impl DetailPaneComponent {
    pub fn new() -> Self {
        Self {
            article: None,
            scroll_offset: 0,
            focused: false,
        }
    }

    pub fn set_article(&mut self, article: NewsItem) {
        self.article = Some(article);
        self.scroll_offset = 0; // Reset scroll when new article is selected
    }
}

impl Component for DetailPaneComponent {
    fn handle_event(&mut self, event: &Event) -> Action {
        if !self.focused {
            return Action::None;
        }

        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.scroll_offset = self.scroll_offset.saturating_add(1);
                    Action::None
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                    Action::None
                }
                KeyCode::PageDown => {
                    self.scroll_offset = self.scroll_offset.saturating_add(10);
                    Action::None
                }
                KeyCode::PageUp => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(10);
                    Action::None
                }
                KeyCode::Char('o') | KeyCode::Enter => {
                    if let Some(article) = &self.article {
                        Action::ArticleOpened(article.url.clone())
                    } else {
                        Action::None
                    }
                }
                _ => Action::None,
            }
        } else {
            Action::None
        }
    }

    fn update(&mut self, action: &Action) {
        // Listen for selection changes from NewsListComponent
        // Note: We'll need to get the actual article from NewsListComponent
        // For now, this is a placeholder - we'll handle this in App
        match action {
            _ => {}
        }
    }

    fn render(&self, f: &mut Frame, area: Rect) {
        let title = "Article Detail";

        let content = if let Some(article) = &self.article {
            format!(
                "Title: {}\n\nSource: {}\nPublished: {}\n\nURL: {}\n\n{}\n\n---\n\nSummary:\n{}",
                article.title,
                article.source,
                article.published.format("%Y-%m-%d %H:%M UTC"),
                article.url,
                "─".repeat(50),
                article.summary
            )
        } else {
            "No article selected\n\nSelect an article from the list to view details.".to_string()
        };

        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(if self.focused {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    }),
            )
            .wrap(Wrap { trim: true })
            .scroll((self.scroll_offset, 0));

        f.render_widget(paragraph, area);
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

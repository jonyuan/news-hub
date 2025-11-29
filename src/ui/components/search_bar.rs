use crate::ui::component::{Action, Component};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct SearchBarComponent {
    query: String,
    cursor_pos: usize,
    focused: bool,
}

impl SearchBarComponent {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            cursor_pos: 0,
            focused: false,
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }
}

impl Component for SearchBarComponent {
    fn handle_event(&mut self, event: &Event) -> Action {
        if !self.focused {
            return Action::None;
        }

        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                KeyCode::Char(c) if modifiers.is_empty() => {
                    // Insert character at cursor position
                    self.query.insert(self.cursor_pos, *c);
                    self.cursor_pos += 1;
                    return Action::SearchQueryChanged(self.query.clone());
                }
                KeyCode::Backspace => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                        self.query.remove(self.cursor_pos);
                        return Action::SearchQueryChanged(self.query.clone());
                    }
                }
                KeyCode::Delete => {
                    if self.cursor_pos < self.query.len() {
                        self.query.remove(self.cursor_pos);
                        return Action::SearchQueryChanged(self.query.clone());
                    }
                }
                KeyCode::Left => {
                    self.cursor_pos = self.cursor_pos.saturating_sub(1);
                }
                KeyCode::Right => {
                    if self.cursor_pos < self.query.len() {
                        self.cursor_pos += 1;
                    }
                }
                KeyCode::Home => {
                    self.cursor_pos = 0;
                }
                KeyCode::End => {
                    self.cursor_pos = self.query.len();
                }
                KeyCode::Esc => {
                    // Clear search
                    self.query.clear();
                    self.cursor_pos = 0;
                    return Action::SearchQueryChanged(String::new());
                }
                // Ctrl+W: Delete word backwards (common in CLI)
                KeyCode::Char('w') if modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.cursor_pos > 0 {
                        let before_cursor = &self.query[..self.cursor_pos];
                        let trimmed = before_cursor.trim_end();
                        let last_space = trimmed.rfind(' ').map(|i| i + 1).unwrap_or(0);
                        self.query.drain(last_space..self.cursor_pos);
                        self.cursor_pos = last_space;
                        return Action::SearchQueryChanged(self.query.clone());
                    }
                }
                _ => {}
            }
        }

        Action::None
    }

    fn update(&mut self, _action: &Action) {
        // SearchBar doesn't react to other component actions
    }

    fn render(&self, f: &mut Frame, area: Rect) {
        let title = if self.focused {
            "Search [Focused - Type to filter, Esc to clear, Ctrl+W to delete word]"
        } else {
            "Search"
        };

        let display_text = if self.query.is_empty() {
            if self.focused {
                "Type to search articles..."
            } else {
                "Press Tab to focus and search"
            }
        } else {
            &self.query
        };

        let style = if self.focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let paragraph = Paragraph::new(display_text).style(style).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(if self.focused {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default()
                }),
        );

        f.render_widget(paragraph, area);

        // Set cursor position when focused
        if self.focused {
            // Position cursor inside the block (accounting for border and padding)
            let cursor_x = area.x + 1 + self.cursor_pos as u16;
            let cursor_y = area.y + 1;

            // Only set cursor if it's within the visible area
            if cursor_x < area.x + area.width.saturating_sub(1) {
                f.set_cursor(cursor_x, cursor_y);
            }
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

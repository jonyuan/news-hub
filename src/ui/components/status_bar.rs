use crate::ui::component::{Action, Component};
use crate::ui::status_message::{MessageLevel, StatusMessage};
use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::VecDeque;

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub struct StatusBarComponent {
    current_message: Option<StatusMessage>,
    message_history: VecDeque<StatusMessage>,
    show_history: bool,
    history_scroll_offset: usize,
    spinner_frame: usize,
}

impl StatusBarComponent {
    pub fn new() -> Self {
        Self {
            current_message: None,
            message_history: VecDeque::new(),
            show_history: false,
            history_scroll_offset: 0,
            spinner_frame: 0,
        }
    }

    /// Set a new status message
    pub fn set_message(&mut self, message: StatusMessage) {
        // Add previous message to history
        if let Some(old_msg) = self.current_message.take() {
            self.message_history.push_back(old_msg);
            // Keep last 50 messages
            if self.message_history.len() > 50 {
                self.message_history.pop_front();
            }
        }
        self.current_message = Some(message);
    }

    pub fn clear_message(&mut self) {
        if let Some(msg) = self.current_message.take() {
            self.message_history.push_back(msg);
        }
    }

    /// Scroll history up (newer messages)
    pub fn scroll_history_up(&mut self) {
        if self.history_scroll_offset > 0 {
            self.history_scroll_offset -= 1;
        }
    }

    /// Scroll history down (older messages)
    pub fn scroll_history_down(&mut self) {
        let max_scroll = self.message_history.len().saturating_sub(15);
        if self.history_scroll_offset < max_scroll {
            self.history_scroll_offset += 1;
        }
    }

    /// Reset scroll when closing history
    fn reset_scroll(&mut self) {
        self.history_scroll_offset = 0;
    }

    /// Get the expanded height when history is shown
    pub fn get_height(&self) -> u16 {
        if self.show_history {
            15 // Expanded height
        } else {
            3 // Normal height
        }
    }

    /// deprecated: use is_focused() instead
    pub fn is_showing_history(&self) -> bool {
        self.is_focused()
    }

    pub fn tick_spinner(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
    }

    /// Check and auto-dismiss expired messages
    pub fn check_auto_dismiss(&mut self) {
        if let Some(msg) = &self.current_message {
            if msg.should_dismiss() {
                self.clear_message();
            }
        }
    }

    fn get_current_display_text(&self) -> Option<(String, MessageLevel)> {
        self.current_message.as_ref().map(|msg| {
            let text = if msg.level == MessageLevel::Loading {
                format!("{} {}", SPINNER_FRAMES[self.spinner_frame], msg.text)
            } else {
                msg.text.clone()
            };
            (text, msg.level)
        })
    }

    fn render_status_line(&self, f: &mut Frame, area: Rect) {
        let (content, style) = if let Some((text, level)) = self.get_current_display_text() {
            let color = match level {
                MessageLevel::Info => Color::Gray,
                MessageLevel::Success => Color::Green,
                MessageLevel::Warning => Color::Yellow,
                MessageLevel::Error => Color::Red,
                MessageLevel::Loading => Color::Cyan,
            };

            let prefix = match level {
                MessageLevel::Success => "✓ ",
                MessageLevel::Error => "✗ ",
                MessageLevel::Warning => "⚠ ",
                MessageLevel::Info => "ℹ ",
                MessageLevel::Loading => "", // Spinner already shown
            };

            let display_text = format!("{}{}", prefix, text);
            (display_text, Style::default().fg(color))
        } else {
            // Show help text when no status message
            let help_text = "/: Search | Tab: Switch | ↑/↓: Nav | Enter/o: Open | r: Refresh | h: Status History | q: Quit";
            (help_text.to_string(), Style::default().fg(Color::Gray))
        };

        let paragraph = Paragraph::new(content)
            .style(style)
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(paragraph, area);
    }

    fn render_history(&self, f: &mut Frame, area: Rect) {
        // Calculate how many lines we can fit (minus 2 for borders)
        let available_lines = area.height.saturating_sub(2) as usize;
        let total_messages = self.message_history.len();

        // Apply scroll offset
        let history_text: Vec<Line> = self
            .message_history
            .iter()
            .rev()
            .skip(self.history_scroll_offset)
            .take(available_lines)
            .map(|msg| {
                let time_str = msg.timestamp.format("%H:%M:%S");
                let color = match msg.level {
                    MessageLevel::Info => Color::Gray,
                    MessageLevel::Success => Color::Green,
                    MessageLevel::Warning => Color::Yellow,
                    MessageLevel::Error => Color::Red,
                    MessageLevel::Loading => Color::Cyan,
                };
                Line::from(vec![
                    Span::styled(
                        format!("[{}] ", time_str),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(msg.text.clone(), Style::default().fg(color)),
                ])
            })
            .collect();

        // Build title with scroll indicator
        let can_scroll_up = self.history_scroll_offset > 0;
        let can_scroll_down = self.history_scroll_offset + available_lines < total_messages;

        let title = if can_scroll_up || can_scroll_down {
            let up_arrow = if can_scroll_up { "↑" } else { " " };
            let down_arrow = if can_scroll_down { "↓" } else { " " };
            format!(
                "Message History {} {}/{} {} (↑/↓: Scroll, h/Esc: Close)",
                up_arrow,
                self.history_scroll_offset + 1,
                total_messages,
                down_arrow
            )
        } else {
            "Message History (h or Esc to close)".to_string()
        };

        let paragraph = Paragraph::new(history_text).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

        f.render_widget(paragraph, area);
    }
}

impl Component for StatusBarComponent {
    fn handle_event(&mut self, event: &Event) -> Action {
        // Only handle events when history is showing
        if !self.show_history {
            return Action::None;
        }

        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Up => {
                    self.scroll_history_up();
                    Action::None
                }
                KeyCode::Down => {
                    self.scroll_history_down();
                    Action::None
                }
                _ => Action::None,
            }
        } else {
            Action::None
        }
    }

    fn update(&mut self, action: &Action) {
        match action {
            Action::StatusMessage(msg) => {
                self.set_message(msg.clone());
            }
            Action::DismissStatus => {
                self.clear_message();
            }
            _ => {}
        }
    }

    fn render(&self, f: &mut Frame, area: Rect) {
        if self.show_history {
            self.render_history(f, area);
        } else {
            self.render_status_line(f, area);
        }
    }

    fn is_focused(&self) -> bool {
        self.show_history
    }

    fn set_focus(&mut self, focused: bool) {
        // StatusBar "focus" means it is showing history
        self.show_history = focused;
        if !focused {
            self.reset_scroll();
        }
    }
}

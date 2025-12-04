use crate::ui::component::{Action, Component};
use crate::ui::status_message::{MessageLevel, StatusMessage};
use crossterm::event::Event;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub struct StatusBarComponent {
    current_message: Option<StatusMessage>,
    message_history: Vec<StatusMessage>,
    show_history: bool,
    spinner_frame: usize,
}

impl StatusBarComponent {
    pub fn new() -> Self {
        Self {
            current_message: None,
            message_history: Vec::new(),
            show_history: false,
            spinner_frame: 0,
        }
    }

    /// Set a new status message
    pub fn set_message(&mut self, message: StatusMessage) {
        // Add previous message to history
        if let Some(old_msg) = self.current_message.take() {
            self.message_history.push(old_msg);
            // Keep last 50 messages
            if self.message_history.len() > 50 {
                self.message_history.remove(0);
            }
        }
        self.current_message = Some(message);
    }

    pub fn clear_message(&mut self) {
        if let Some(msg) = self.current_message.take() {
            self.message_history.push(msg);
        }
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
            let help_text = "/: Search | Tab: Switch | ↑/↓: Nav | Enter/o: Open | r: Refresh | Esc: Dismiss | h: History | q: Quit";
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

        let history_text: Vec<Line> = self
            .message_history
            .iter()
            .rev()
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

        let paragraph = Paragraph::new(history_text).block(
            Block::default()
                .title("Message History (h to close)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

        f.render_widget(paragraph, area);
    }
}

impl Component for StatusBarComponent {
    fn handle_event(&mut self, _event: &Event) -> Action {
        // StatusBar doesn't handle events directly
        // 'h' for history and 'Esc' for dismiss are handled globally in App
        Action::None
    }

    fn update(&mut self, action: &Action) {
        match action {
            Action::StatusMessage(msg) => {
                self.set_message(msg.clone());
            }
            Action::DismissStatus => {
                self.clear_message();
            }
            Action::ShowStatusHistory => {
                self.show_history = !self.show_history;
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
        false
    }

    fn set_focus(&mut self, _focused: bool) {
        // StatusBar never takes focus
    }
}

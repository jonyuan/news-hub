pub mod component;
pub mod components;

pub use component::{Action, Component};
pub use components::NewsListComponent;

use crate::app::AppState;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;

pub fn draw_ui(
    term: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    news_list: &NewsListComponent,
    _app_state: AppState,
) -> io::Result<()> {
    term.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
            .split(f.size());

        // Render the news list component
        news_list.render(f, chunks[0]);

        // Footer with summary
        let summary = if let Some(item) = news_list.selected_item() {
            format!("Summary: {}", item.summary)
        } else {
            String::from("No items available")
        };

        let footer = Paragraph::new(summary)
            .block(Block::default().title("Article Summary").borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray));

        f.render_widget(footer, chunks[1]);
    })?;
    Ok(())
}

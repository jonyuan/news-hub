pub mod component;
pub mod components;

pub use component::{Action, Component};
pub use components::{NewsListComponent, DetailPaneComponent};

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
    detail_pane: &DetailPaneComponent,
    _app_state: AppState,
) -> io::Result<()> {
    term.draw(|f| {
        // Main vertical split: content area + footer
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(3)].as_ref())
            .split(f.size());

        // Content area horizontal split: news list (60%) + detail pane (40%)
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
            .split(main_chunks[0]);

        // Render components
        news_list.render(f, content_chunks[0]);
        detail_pane.render(f, content_chunks[1]);

        // Footer with help text
        let footer_text = "Tab: Switch Focus | ↑/↓: Navigate/Scroll | Enter/o: Open Article | r: Refresh | q: Quit";
        let footer = Paragraph::new(footer_text)
            .block(Block::default().title("Controls").borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray));

        f.render_widget(footer, main_chunks[1]);
    })?;
    Ok(())
}

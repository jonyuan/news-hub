pub mod component;
pub mod components;
pub mod status_message;

pub use component::{Action, Component};
pub use components::{DetailPaneComponent, NewsListComponent, SearchBarComponent, StatusBarComponent};
pub use status_message::{MessageLevel, StatusMessage};

use crate::app::AppState;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::io;

pub fn draw_ui(
    term: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    search_bar: &SearchBarComponent,
    news_list: &NewsListComponent,
    detail_pane: &DetailPaneComponent,
    status_bar: &StatusBarComponent,
    _app_state: AppState,
) -> io::Result<()> {
    term.draw(|f| {
        // Main vertical split: search bar + content area + status bar
        let status_bar_height = status_bar.get_height();
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),                    // Search bar
                    Constraint::Min(5),                       // Content area
                    Constraint::Length(status_bar_height),    // Status bar (dynamic)
                ]
                .as_ref(),
            )
            .split(f.size());

        // Render search bar at top
        search_bar.render(f, main_chunks[0]);

        // Content area horizontal split: news list (60%) + detail pane (40%)
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
            .split(main_chunks[1]);

        // Render components
        news_list.render(f, content_chunks[0]);
        detail_pane.render(f, content_chunks[1]);

        // Render status bar at bottom
        status_bar.render(f, main_chunks[2]);
    })?;
    Ok(())
}

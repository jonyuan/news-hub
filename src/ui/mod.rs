use crate::api::NewsItem;
use crate::app::AppState;
use chrono::Local;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;

pub fn draw_ui(
    term: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    news: &[NewsItem],
    app_state: AppState,
    selected_index: usize,
) -> io::Result<()> {
    term.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
            .split(f.size());

        // Title with loading state
        let title = match app_state {
            AppState::Loading => "News Feed [LOADING...]",
            AppState::Idle => "News Feed [Press 'r' to refresh, 'q' to quit, Enter/o to open]",
        };

        // Build list items with timestamps and selection highlighting
        let items: Vec<ListItem> = news
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

                // Highlight selected item
                if i == selected_index {
                    ListItem::new(content).style(
                        Style::default()
                            .bg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD)
                    )
                } else {
                    ListItem::new(content)
                }
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title(title).borders(Borders::ALL));

        f.render_widget(list, chunks[0]);

        // Footer with summary
        let summary = if let Some(item) = news.get(selected_index) {
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

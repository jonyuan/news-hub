use crate::api::NewsItem;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};
use std::io;

pub fn draw_ui(
    term: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    news: &[NewsItem],
) -> io::Result<()> {
    term.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(f.size());

        let items: Vec<ListItem> = news
            .iter()
            .map(|n| ListItem::new(format!("{}  —  {}", n.title, n.source)))
            .collect();

        let list =
            List::new(items).block(Block::default().title("News Feed").borders(Borders::ALL));

        f.render_widget(list, chunks[0]);
    })?;
    Ok(())
}

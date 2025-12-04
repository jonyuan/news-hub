use crate::models::FilterState;
use crate::ui::status_message::StatusMessage;
use crossterm::event::Event;
use ratatui::{layout::Rect, Frame};

/// Actions that components can emit to coordinate with each other
#[derive(Clone, Debug)]
pub enum Action {
    None,
    SelectionChanged(usize),
    ArticleOpened(String), // URL
    SearchQueryChanged(String),
    FilterApplied(FilterState),
    RefreshRequested,
    Quit,

    // Status bar actions
    StatusMessage(StatusMessage),
    DismissStatus,
    ShowStatusHistory,
}

/// Core trait that all UI components must implement
pub trait Component {
    /// Handle input events and return an Action
    fn handle_event(&mut self, event: &Event) -> Action;

    /// Update component state based on an Action from another component
    fn update(&mut self, action: &Action);

    /// Render the component to the given area
    fn render(&self, f: &mut Frame, area: Rect);

    /// Check if this component currently has focus
    fn is_focused(&self) -> bool;

    /// Set the focus state of this component
    fn set_focus(&mut self, focused: bool);
}

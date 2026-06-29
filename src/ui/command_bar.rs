//! Command bar widget rendering.

use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::App;
use crate::ui::theme::Theme;

/// Renders the colon-prefixed command input bar at the bottom.
pub fn render(f: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    if !app.command_active {
        return;
    }

    let command_line = Line::from(vec![
        Span::styled(":", Style::default().fg(theme.accent)),
        Span::raw(&app.command_input),
    ]);

    let paragraph = Paragraph::new(command_line)
        .style(Style::default().bg(theme.status_bg).fg(theme.fg));
        
    f.render_widget(paragraph, area);
}

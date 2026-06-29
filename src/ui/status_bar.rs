//! Status bar widget rendering.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::{App, InputMode, AppMode};
use crate::ui::theme::Theme;

/// Renders the status bar at the bottom of the screen.
pub fn render(f: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let hints_width = (area.width as usize).saturating_sub(25).min(85).max(50) as u16;
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(10), // Mode indicator
            Constraint::Min(10),    // Status message / center content
            Constraint::Length(hints_width), // Key hints (right-aligned)
        ])
        .split(area);

    // 1. Render Mode Indicator
    let (mode_text, mode_style) = match app.input_mode {
        InputMode::Normal => (" NORMAL ", Style::default().bg(theme.accent).fg(theme.bg).add_modifier(Modifier::BOLD)),
        InputMode::Visual => (" VISUAL ", Style::default().bg(theme.warning).fg(theme.bg).add_modifier(Modifier::BOLD)),
        InputMode::Command => (" COMMAND", Style::default().bg(theme.muted).fg(theme.bg).add_modifier(Modifier::BOLD)),
        InputMode::Search => (" SEARCH ", Style::default().bg(theme.highlight_blue).fg(theme.bg).add_modifier(Modifier::BOLD)),
    };
    f.render_widget(Paragraph::new(mode_text).style(mode_style), chunks[0]);

    // 2. Render Status Message
    let status_text = if let Some((msg, timestamp)) = &app.status_message {
        if timestamp.elapsed() < std::time::Duration::from_secs(3) {
            msg.as_str()
        } else {
            ""
        }
    } else {
        ""
    };
    let status_paragraph = Paragraph::new(Span::raw(status_text))
        .style(Style::default().bg(theme.status_bg).fg(theme.fg));
    f.render_widget(status_paragraph, chunks[1]);

    // 3. Render Key Hints based on App Mode and Input Mode
    let hints = match app.input_mode {
        InputMode::Command => " Esc: Cancel | Enter: Run command ",
        InputMode::Search => " Esc: Cancel | Enter: Execute search ",
        InputMode::Visual => " Esc: Normal | h: Highlight | c: Clear selection ",
        InputMode::Normal => match app.mode {
            AppMode::Library => " Tab/S-Tab:Tabs | a:Add | r:Rename | d:Del | x:Archive | f:Fav | /:Search | q:Quit ",
            AppMode::Reader => " j/k:Scroll | G/gg:Top/End | h:Highlight | r:Rename | s:Summary | q:Lib | 1-4:Views ",
            AppMode::Review => " Space:Flip | 1-5:Rate | s:Skip | q:End | 1-4:Views ",
            AppMode::Highlights => " j/k:Nav | Enter:Open | d:Delete | q:Lib | 1-4:Views ",
        },
    };

    let hints_paragraph = Paragraph::new(Line::from(vec![
        Span::styled(hints, Style::default().fg(theme.muted))
    ]))
    .right_aligned()
    .style(Style::default().bg(theme.status_bg));
    f.render_widget(hints_paragraph, chunks[2]);
}

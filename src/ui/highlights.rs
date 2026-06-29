//! Highlights list view widget rendering.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::theme::Theme;

/// Renders the highlights workspace.
pub fn render(f: &mut Frame, app: &App, theme: &Theme) {
    let size = f.area();
    
    // Reserve the last line for status/command bar
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1), // status/command bar
        ])
        .split(size);

    let content_area = main_layout[0];
    let status_area = main_layout[1];

    // Split workspace: Header and List
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // List area
        ])
        .split(content_area);

    // 1. Render Header
    let title_span = Span::styled(" 🖋️ HIGHLIGHTS & NOTES ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));
    let count_text = format!("Total: {} highlights", app.all_highlights.len());
    
    let header_line = Line::from(vec![title_span]);
    let right_line = Line::from(vec![Span::raw(count_text)]);

    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.border));

    f.render_widget(Paragraph::new(header_line).block(header_block.clone()), chunks[0]);
    f.render_widget(Paragraph::new(right_line).alignment(Alignment::Right).block(header_block), chunks[0]);

    // 2. Render List
    let list_area = chunks[1];

    if app.all_highlights.is_empty() {
        let empty_block = Block::default();
        let empty_p = Paragraph::new("No highlights saved yet. Highlight passages while reading in Reader mode.")
            .alignment(Alignment::Center)
            .block(empty_block)
            .style(Style::default().fg(theme.muted));
            
        f.render_widget(empty_p, list_area);
    } else {
        let items: Vec<ListItem> = app.all_highlights.iter().enumerate().map(|(idx, (highlight, article_title))| {
            let is_selected = idx == app.highlight_index;
            
            // Map highlight color name to theme color
            let text_color = match highlight.color.to_lowercase().as_str() {
                "green" => theme.highlight_green,
                "blue" => theme.highlight_blue,
                "red" => theme.highlight_red,
                "purple" => theme.highlight_purple,
                _ => theme.highlight_yellow, // default/yellow
            };

            let h_style = Style::default().fg(text_color).add_modifier(Modifier::BOLD);
            
            // Format lines
            let mut item_lines = vec![
                Line::from(vec![
                    Span::styled(format!("“{}”", highlight.text), h_style)
                ]),
                Line::from(vec![
                    Span::styled(format!("  Article: {}", article_title), Style::default().fg(theme.muted)),
                    Span::styled(format!(" • Saved: {}", highlight.created_at), Style::default().fg(theme.muted)),
                ])
            ];

            if let Some(note) = &highlight.note {
                item_lines.push(Line::from(vec![
                    Span::styled("  Note: ", Style::default().fg(theme.accent).add_modifier(Modifier::ITALIC)),
                    Span::styled(note, Style::default().fg(theme.fg).add_modifier(Modifier::ITALIC)),
                ]));
            }

            // Spacing line
            item_lines.push(Line::raw(""));

            let item_block = if is_selected {
                Style::default().bg(theme.selection_bg)
            } else {
                Style::default()
            };

            ListItem::new(item_lines).style(item_block)
        }).collect();

        let list_widget = List::new(items)
            .block(Block::default());
            
        f.render_widget(list_widget, list_area);
    }

    // Render Status/Command Bar
    if app.command_active {
        super::command_bar::render(f, app, theme, status_area);
    } else {
        super::status_bar::render(f, app, theme, status_area);
    }
}

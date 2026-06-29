//! Review (spaced repetition) view widget rendering.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::theme::Theme;

/// Renders the spaced repetition flashcard review interface.
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

    // Check if the review queue is empty
    if app.review_queue.is_empty() || app.review_index >= app.review_queue.len() {
        let empty_block = Block::default();
        let success_text = vec![
            Line::raw(""),
            Line::raw(""),
            Line::styled("🎉 All done!", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Line::raw(""),
            Line::styled("No flashcards are currently due for review.", Style::default().fg(theme.fg)),
            Line::styled("Come back later or add new articles to generate more cards.", Style::default().fg(theme.muted)),
        ];
        
        let empty_p = Paragraph::new(success_text)
            .alignment(Alignment::Center)
            .block(empty_block);
            
        f.render_widget(empty_p, content_area);
        
        if app.command_active {
            super::command_bar::render(f, app, theme, status_area);
        } else {
            super::status_bar::render(f, app, theme, status_area);
        }
        return;
    }

    // Split review space: top metrics header and central card
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Metrics Header
            Constraint::Min(0),    // Centered Flashcard
        ])
        .split(content_area);

    let card = &app.review_queue[app.review_index];
    let remaining = app.review_queue.len() - app.review_index;
    let reviewed = app.review_session.cards_reviewed;
    let correct = app.review_session.cards_correct;

    let accuracy = if reviewed > 0 {
        (correct as f64 / reviewed as f64 * 100.0).round() as i64
    } else {
        100
    };

    // 1. Render Metrics Header
    let metrics_text = format!(
        " Cards Reviewed: {} | Remaining in Session: {} | Session Accuracy: {}%",
        reviewed, remaining, accuracy
    );
    let metrics_p = Paragraph::new(Span::styled(metrics_text, Style::default().fg(theme.muted)))
        .alignment(Alignment::Center);
    f.render_widget(metrics_p, chunks[0]);

    // 2. Render Centered Card
    let card_area = centered_rect(70, 50, chunks[1]);
    
    let card_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent))
        .title(format!(" Flashcard {}/{} ", app.review_index + 1, app.review_queue.len()))
        .title_alignment(Alignment::Center);

    let mut card_lines = Vec::new();
    card_lines.push(Line::raw(""));
    
    if !app.card_flipped {
        // Front side
        card_lines.push(Line::styled("Question:", Style::default().fg(theme.muted).add_modifier(Modifier::UNDERLINED)));
        card_lines.push(Line::raw(""));
        card_lines.push(Line::styled(&card.front, Style::default().add_modifier(Modifier::BOLD)));
        card_lines.push(Line::raw(""));
        card_lines.push(Line::raw(""));
        card_lines.push(Line::styled("[ Press Space to flip card ]", Style::default().fg(theme.accent)));
    } else {
        // Back side
        card_lines.push(Line::styled("Question:", Style::default().fg(theme.muted)));
        card_lines.push(Line::styled(&card.front, Style::default().fg(theme.muted)));
        card_lines.push(Line::raw(""));
        card_lines.push(Line::styled("Answer:", Style::default().fg(theme.muted).add_modifier(Modifier::UNDERLINED)));
        card_lines.push(Line::raw(""));
        card_lines.push(Line::styled(&card.back, Style::default().add_modifier(Modifier::BOLD)));
        card_lines.push(Line::raw(""));
        card_lines.push(Line::raw(""));
        
        let hint_line = Line::from(vec![
            Span::raw("Rate Recall: "),
            Span::styled(" 1 ", Style::default().bg(theme.error).fg(theme.bg)),
            Span::raw(" Forgot  "),
            Span::styled(" 2 ", Style::default().bg(theme.warning).fg(theme.bg)),
            Span::raw(" Hard  "),
            Span::styled(" 3 ", Style::default().bg(theme.muted).fg(theme.bg)),
            Span::raw(" Good  "),
            Span::styled(" 4 ", Style::default().bg(theme.accent_dim).fg(theme.bg)),
            Span::raw(" Easy  "),
            Span::styled(" 5 ", Style::default().bg(theme.success).fg(theme.bg)),
            Span::raw(" Perfect"),
        ]);
        card_lines.push(hint_line);
    }

    let card_p = Paragraph::new(card_lines)
        .alignment(Alignment::Center)
        .block(card_block);
        
    f.render_widget(card_p, card_area);

    // Render Status/Command Bar
    if app.command_active {
        super::command_bar::render(f, app, theme, status_area);
    } else {
        super::status_bar::render(f, app, theme, status_area);
    }
}

/// Helper to generate a centered Rect constraint.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

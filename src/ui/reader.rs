//! Reader view widget rendering.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::theme::Theme;

/// Renders the article reading interface.
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

    let article = match &app.current_article {
        Some(a) => a,
        None => {
            let empty_p = Paragraph::new("No article currently loaded.")
                .alignment(Alignment::Center)
                .style(Style::default().fg(theme.muted));
            f.render_widget(empty_p, content_area);
            super::status_bar::render(f, app, theme, status_area);
            return;
        }
    };

    // Split reader area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header info
            Constraint::Min(0),    // Scrollable content viewport
            Constraint::Length(2), // Progress bar gauge (with top border spacing)
        ])
        .split(content_area);

    // 1. Render Header
    let author_text = article.author.as_deref().unwrap_or("Unknown Author");
    let site_text = article.site_name.as_deref().unwrap_or("Unknown Source");
    
    let title_line = Line::from(vec![
        Span::styled(format!(" {} ", article.source_icon()), Style::default().fg(theme.accent)),
        Span::styled(&article.title, Style::default().add_modifier(Modifier::BOLD)),
    ]);

    let meta_line = Line::from(vec![
        Span::styled(format!(" By {} • {} ", author_text, site_text), Style::default().fg(theme.muted)),
        Span::styled(format!("• {} words • {} min read", article.word_count, article.reading_time_min), Style::default().fg(theme.muted)),
    ]);

    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.border));

    f.render_widget(Paragraph::new(vec![title_line, meta_line]).block(header_block), chunks[0]);

    // 2. Render Scrollable Viewport
    let content_chunk = chunks[1];
    let height = content_chunk.height as usize;
    let total_lines = app.reader_lines.len();

    let start = app.reader_scroll;
    let end = (start + height).min(total_lines);

    let visible_lines = if total_lines > 0 && start < total_lines {
        &app.reader_lines[start..end]
    } else {
        &[]
    };

    // Combine visible lines with newlines
    let text_content = visible_lines.join("\n");
    let content_p = Paragraph::new(text_content)
        .style(Style::default().fg(theme.fg));
        
    f.render_widget(content_p, content_chunk);

    // 3. Render Progress Gauge
    let progress_pct = if total_lines == 0 {
        0
    } else {
        let read_count = end;
        let pct = (read_count as f64 / total_lines as f64 * 100.0).round() as u16;
        pct.min(100)
    };

    let scroll_pos_text = format!(" Line {}/{} ", start.saturating_add(1).min(total_lines), total_lines);
    let gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme.border))
            .title(scroll_pos_text)
            .title_alignment(Alignment::Right)
            .title_style(Style::default().fg(theme.muted))
        )
        .gauge_style(Style::default().fg(theme.accent).bg(theme.border))
        .percent(progress_pct);

    f.render_widget(gauge, chunks[2]);

    // Render Status/Command Bar
    if app.command_active {
        super::command_bar::render(f, app, theme, status_area);
    } else {
        super::status_bar::render(f, app, theme, status_area);
    }
}

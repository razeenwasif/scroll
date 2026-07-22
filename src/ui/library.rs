//! Library view widget rendering.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};
use crate::app::App;
use crate::ui::theme::Theme;
use crate::models::FilterTab;

/// Renders the library workspace.
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

    // Determine layout components
    let mut constraints = vec![
        Constraint::Length(3), // Header
        Constraint::Length(2), // Tabs (including spacing)
    ];

    if app.search_active {
        constraints.push(Constraint::Length(3)); // Search box
    }
    constraints.push(Constraint::Min(0)); // Articles list

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(content_area);

    let mut current_chunk = 0;

    // 1. Render Header
    let header_area = chunks[current_chunk];
    current_chunk += 1;

    let title_span = Span::styled(" 📜 SCROLL ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));
    let count_text = format!("Library: {} articles", app.articles.len());
    let mut right_spans = vec![Span::raw(count_text)];

    if app.due_count > 0 {
        right_spans.push(Span::raw("  "));
        right_spans.push(Span::styled(
            format!(" [{}] DUE ", app.due_count),
            Style::default().bg(theme.warning).fg(theme.bg).add_modifier(Modifier::BOLD),
        ));
    }

    let header_line = Line::from(vec![title_span]);
    let right_line = Line::from(right_spans);

    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.border));

    f.render_widget(Paragraph::new(header_line).block(header_block.clone()), header_area);
    f.render_widget(Paragraph::new(right_line).alignment(Alignment::Right).block(header_block), header_area);

    // 2. Render Filter Tabs
    let tabs_area = chunks[current_chunk];
    current_chunk += 1;

    let tab_titles: Vec<&str> = FilterTab::all().iter().map(|t| t.label()).collect();
    let tabs = Tabs::new(tab_titles.iter().map(|&s| s))
        .select(app.filter_tab as usize)
        .style(Style::default().fg(theme.muted))
        .highlight_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .divider(" | ");
        
    f.render_widget(tabs, tabs_area);

    // 3. Render Search box if active
    if app.search_active {
        let search_area = chunks[current_chunk];
        current_chunk += 1;
        
        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.accent))
            .title(" Search Query ");

        let search_text = Paragraph::new(format!(" {}", app.search_input))
            .block(search_block);
            
        f.render_widget(search_text, search_area);
    }

    // 4. Split list area horizontally: 25% Sidebar, 75% Article List
    let list_area = chunks[current_chunk];
    
    let list_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(18), // Left sidebar
            Constraint::Percentage(82), // Right list
        ])
        .split(list_area);
        
    let sidebar_area = list_layout[0];
    let right_list_area = list_layout[1];

    // Render Sidebar Categories
    let sidebar_border_style = if app.sidebar_focused {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.border)
    };
    
    let sidebar_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(sidebar_border_style)
        .title(" 📁 Folders ");

    let sidebar_items: Vec<ListItem> = app.categories.iter().enumerate().map(|(idx, cat)| {
        let is_selected = idx == app.selected_category_index;
        let prefix = if is_selected { "▸ " } else { "  " };
        let style = if is_selected {
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg)
        };
        ListItem::new(Line::from(vec![
            Span::styled(prefix, Style::default().fg(theme.accent)),
            Span::styled(cat, style),
        ]))
    }).collect();

    let sidebar_list = List::new(sidebar_items).block(sidebar_block);
    let mut sidebar_state = app.sidebar_list_state.borrow_mut();
    sidebar_state.select(if app.categories.is_empty() {
        None
    } else {
        Some(app.selected_category_index.min(app.categories.len() - 1))
    });
    f.render_stateful_widget(sidebar_list, sidebar_area, &mut sidebar_state);

    // Render Articles List in the right pane
    let list_border_style = if !app.sidebar_focused {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.border)
    };
    
    let right_list_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(list_border_style)
        .title(" 📜 Articles ");

    if app.articles.is_empty() {
        let empty_p = Paragraph::new(if app.search_active && !app.search_input.is_empty() {
            "\n\n  No search results match your query."
        } else {
            "\n\n  No articles in this folder.\n  Press 'a' to clip/add a URL.\n  Press 'c' on an article to categorize it."
        })
        .alignment(Alignment::Center)
        .block(right_list_block)
        .style(Style::default().fg(theme.muted));
            
        f.render_widget(empty_p, right_list_area);
    } else {
        let items: Vec<ListItem> = app.articles.iter().enumerate().map(|(idx, article)| {
            let status_icon = article.status_icon();
            let fav_star = if article.is_favorite { "★" } else { "☆" };
            let site = article.site_name.as_deref().unwrap_or("Unknown");
            
            let is_selected = idx == app.selected_index;
            let title_style = if is_selected {
                Style::default().fg(theme.selection_fg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)
            };
            
            let mut line1_spans = vec![
                Span::styled(format!(" {} {} ", status_icon, fav_star), Style::default().fg(theme.accent)),
                Span::styled(&article.title, title_style),
                Span::styled(format!(" • {}", site), Style::default().fg(theme.muted)),
            ];
            
            if let Some(ref cat) = article.category {
                line1_spans.push(Span::styled("  ", Style::default()));
                line1_spans.push(Span::styled(
                    format!(" [{}] ", cat),
                    Style::default().fg(theme.accent).add_modifier(Modifier::DIM),
                ));
            }
            
            let line1 = Line::from(line1_spans);
            
            let prog_bar = make_progress_bar(article.reading_progress, 10);
            let time_info = format!("{} words • {} min read", article.word_count, article.reading_time_min);
            let line2 = Line::from(vec![
                Span::styled(format!("   {}  ", prog_bar), Style::default().fg(theme.accent)),
                Span::styled(format!("{} • {}", time_info, article.created_at), Style::default().fg(theme.muted)),
            ]);

            let item_block = if is_selected && !app.sidebar_focused {
                Style::default().bg(theme.selection_bg)
            } else if is_selected {
                Style::default().bg(theme.border)
            } else {
                Style::default()
            };

            ListItem::new(vec![line1, line2, Line::raw("")]).style(item_block)
        }).collect();

        let article_list = List::new(items)
            .block(right_list_block);

        let mut list_state = app.article_list_state.borrow_mut();
        list_state.select(Some(app.selected_index.min(app.articles.len() - 1)));
        f.render_stateful_widget(article_list, right_list_area, &mut list_state);
    }

    // Render Status/Command Bar
    if app.command_active {
        super::command_bar::render(f, app, theme, status_area);
    } else {
        super::status_bar::render(f, app, theme, status_area);
    }
}

/// Utility function to create a text progress bar.
fn make_progress_bar(progress: f64, width: usize) -> String {
    let progress_clamped = progress.clamp(0.0, 1.0);
    let filled = (progress_clamped * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}] {:.0}%", "█".repeat(filled), "░".repeat(empty), progress_clamped * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ScrollConfig;
    use crate::models::{Article, SourceType};
    use crate::storage::Database;
    use ratatui::{backend::TestBackend, Terminal};
    use std::path::Path;

    fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<Vec<_>>()
            .join("")
    }

    /// The article list must scroll so the selected row stays visible even
    /// when the selection is far below the fold.
    #[test]
    fn selected_article_stays_visible_when_scrolled_past_viewport() {
        let db = Database::new(Path::new(":memory:")).unwrap();
        for i in 0..40 {
            let mut art = Article::new(
                format!("Article number {:02}", i),
                "content".to_string(),
                SourceType::Web,
            );
            art.id = format!("art-{:02}", i);
            db.insert_article(&art).unwrap();
        }

        let mut app = App::new(db, ScrollConfig::default()).unwrap();
        assert_eq!(app.articles.len(), 40);
        app.sidebar_focused = false;
        app.selected_index = 35;

        let theme = app.theme.clone();
        let mut terminal = Terminal::new(TestBackend::new(120, 30)).unwrap();
        terminal.draw(|f| render(f, &app, &theme)).unwrap();

        let text = buffer_text(&terminal);
        let selected_title = &app.articles[35].title;
        assert!(
            text.contains(selected_title.as_str()),
            "selected article {:?} was not rendered", selected_title
        );
    }
}

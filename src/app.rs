//! Core application state and event loop runner.

use std::cell::RefCell;
use std::io;
use std::time::{Duration, Instant};
use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, MouseEvent,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Frame, Terminal};

use crate::models::{Article, ArticleFilter, ArticleStatus, FilterTab, Flashcard, Highlight, ReviewSession, SortField};
use crate::storage::Database;
use crate::config::ScrollConfig;
use crate::ui::theme::Theme;
use crate::ai::OllamaClient;
use crate::engine::sm2_review;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Library,
    Reader,
    Review,
    Highlights,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    #[allow(dead_code)]
    Visual,
    Command,
    Search,
}

/// The main application state container.
pub struct App {
    pub db: Database,
    pub config: ScrollConfig,
    pub mode: AppMode,
    pub input_mode: InputMode,
    pub should_quit: bool,
    pub theme: Theme,
    
    // Library
    pub articles: Vec<Article>,
    pub selected_index: usize,
    pub filter_tab: FilterTab,
    pub article_filter: ArticleFilter,
    pub categories: Vec<String>,
    pub selected_category_index: usize,
    pub sidebar_focused: bool,
    /// Scroll offsets for the library panes, kept across frames so the
    /// selected row stays inside the viewport.
    pub article_list_state: RefCell<ListState>,
    pub sidebar_list_state: RefCell<ListState>,

    // Reader
    pub current_article: Option<Article>,
    pub reader_scroll: usize,
    pub reader_lines: Vec<String>,
    pub current_highlights: Vec<Highlight>,
    
    // Review
    pub review_queue: Vec<Flashcard>,
    pub review_index: usize,
    pub card_flipped: bool,
    pub review_session: ReviewSession,
    
    // Highlights
    pub all_highlights: Vec<(Highlight, String)>,
    pub highlight_index: usize,
    pub highlight_list_state: RefCell<ListState>,

    // Search
    pub search_input: String,
    pub search_active: bool,
    
    // Command bar
    pub command_input: String,
    pub command_active: bool,
    
    // Status
    pub status_message: Option<(String, Instant)>,
    pub due_count: i64,
}

impl App {
    /// Creates a new App state instance.
    pub fn new(db: Database, config: ScrollConfig) -> Result<Self> {
        let theme = Theme::get(&config.theme);
        let article_filter = ArticleFilter::default();
        let due_count = db.count_due_today().unwrap_or(0);

        let mut app = Self {
            db,
            config,
            mode: AppMode::Library,
            input_mode: InputMode::Normal,
            should_quit: false,
            theme,
            
            articles: Vec::new(),
            selected_index: 0,
            filter_tab: FilterTab::All,
            article_filter,
            categories: Vec::new(),
            selected_category_index: 0,
            sidebar_focused: true,
            article_list_state: RefCell::new(ListState::default()),
            sidebar_list_state: RefCell::new(ListState::default()),

            current_article: None,
            reader_scroll: 0,
            reader_lines: Vec::new(),
            current_highlights: Vec::new(),
            
            review_queue: Vec::new(),
            review_index: 0,
            card_flipped: false,
            review_session: ReviewSession::new(),
            
            all_highlights: Vec::new(),
            highlight_index: 0,
            highlight_list_state: RefCell::new(ListState::default()),

            search_input: String::new(),
            search_active: false,
            
            command_input: String::new(),
            command_active: false,
            
            status_message: None,
            due_count,
        };

        app.reload_categories()?;
        app.load_articles()?;
        app.load_review_queue()?;
        app.load_all_highlights()?;
        
        Ok(app)
    }

    /// Sets a temporary status message shown at the bottom.
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), Instant::now()));
    }

    /// Loads articles from the database based on the active filter.
    pub fn load_articles(&mut self) -> Result<()> {
        // Apply the category filter based on selected_category_index
        if self.selected_category_index == 0 {
            self.article_filter.category = None;
        } else if self.selected_category_index == 1 {
            self.article_filter.category = Some("".to_string());
        } else if self.selected_category_index < self.categories.len() {
            self.article_filter.category = Some(self.categories[self.selected_category_index].clone());
        }

        self.articles = self.db.list_articles(&self.article_filter)?;
        if self.selected_index >= self.articles.len() {
            self.selected_index = self.articles.len().saturating_sub(1);
        }
        Ok(())
    }

    /// Reloads the unique categories list from the database.
    pub fn reload_categories(&mut self) -> Result<()> {
        let mut list = vec!["All".to_string(), "Unassigned".to_string()];
        let mut db_cats = self.db.list_categories()?;
        list.append(&mut db_cats);
        self.categories = list;
        if self.selected_category_index >= self.categories.len() {
            self.selected_category_index = self.categories.len().saturating_sub(1);
        }
        Ok(())
    }

    /// Reloads the review queue and updates due cards count.
    pub fn load_review_queue(&mut self) -> Result<()> {
        let max_cards = self.config.review.cards_per_day;
        self.review_queue = self.db.get_due_flashcards(max_cards)?;
        self.due_count = self.db.count_due_today().unwrap_or(0);
        Ok(())
    }

    /// Loads all highlights across the system.
    pub fn load_all_highlights(&mut self) -> Result<()> {
        let highlights = self.db.list_all_highlights()?;
        let mut loaded = Vec::new();
        for h in highlights {
            let article_title = if let Some(art) = self.db.get_article(&h.article_id)? {
                art.title
            } else {
                "Unknown".to_string()
            };
            loaded.push((h, article_title));
        }
        self.all_highlights = loaded;
        Ok(())
    }

    /// Opens an article in the reader viewport.
    pub fn open_article(&mut self, index: usize) -> Result<()> {
        if index >= self.articles.len() {
            return Ok(());
        }
        let article = self.articles[index].clone();
        
        // Wrap content based on terminal width
        let width = if let Ok((w, _)) = crossterm::terminal::size() {
            (w as usize).saturating_sub(6).max(40)
        } else {
            80
        };
        
        self.reader_lines = crate::engine::wrap_text(&article.content_markdown, width);
        self.reader_scroll = article.scroll_position as usize;
        self.current_article = Some(article.clone());
        self.current_highlights = self.db.get_highlights_for_article(&article.id)?;
        
        // Update status to reading if it was unread
        if article.status == ArticleStatus::Unread {
            self.db.update_article_status(&article.id, ArticleStatus::Reading)?;
            self.load_articles()?;
        }
        
        self.mode = AppMode::Reader;
        Ok(())
    }

    /// Runs the main rendering and event reading loop.
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        self.set_status("Welcome to Scroll! Press ? for help.");

        while !self.should_quit {
            terminal.draw(|f| self.render(f))?;
            
            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => {
                        self.handle_key_event(key).await?;
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse_event(mouse)?;
                    }
                    Event::Resize(w, _h) => {
                        // Re-wrap text if in Reader mode
                        if self.mode == AppMode::Reader {
                            if let Some(ref art) = self.current_article {
                                let width = (w as usize).saturating_sub(6).max(40);
                                self.reader_lines = crate::engine::wrap_text(&art.content_markdown, width);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        
        Ok(())
    }

    /// Renders the layout corresponding to the current app mode.
    fn render(&self, f: &mut Frame) {
        match self.mode {
            AppMode::Library => crate::ui::library::render(f, self, &self.theme),
            AppMode::Reader => crate::ui::reader::render(f, self, &self.theme),
            AppMode::Review => crate::ui::review::render(f, self, &self.theme),
            AppMode::Highlights => crate::ui::highlights::render(f, self, &self.theme),
        }
    }

    /// Scroll wheel support for the list views and the reader.
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<()> {
        if self.input_mode != InputMode::Normal {
            return Ok(());
        }

        let down = match mouse.kind {
            MouseEventKind::ScrollDown => true,
            MouseEventKind::ScrollUp => false,
            _ => return Ok(()),
        };

        match self.mode {
            AppMode::Library => {
                if self.sidebar_focused {
                    if !self.categories.is_empty() {
                        self.selected_category_index = if down {
                            (self.selected_category_index + 1).min(self.categories.len() - 1)
                        } else {
                            self.selected_category_index.saturating_sub(1)
                        };
                        self.selected_index = 0;
                        self.load_articles()?;
                    }
                } else if !self.articles.is_empty() {
                    self.selected_index = if down {
                        (self.selected_index + 1).min(self.articles.len() - 1)
                    } else {
                        self.selected_index.saturating_sub(1)
                    };
                }
            }
            AppMode::Highlights => {
                if !self.all_highlights.is_empty() {
                    self.highlight_index = if down {
                        (self.highlight_index + 1).min(self.all_highlights.len() - 1)
                    } else {
                        self.highlight_index.saturating_sub(1)
                    };
                }
            }
            AppMode::Reader => {
                let speed = self.config.reader.scroll_speed;
                let max_scroll = self.reader_lines.len().saturating_sub(1);
                self.reader_scroll = if down {
                    (self.reader_scroll + speed).min(max_scroll)
                } else {
                    self.reader_scroll.saturating_sub(speed)
                };
            }
            AppMode::Review => {}
        }

        Ok(())
    }

    /// Primary event router for key combinations.
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        // Check global shortcuts first
        if self.input_mode == InputMode::Normal {
            match key.code {
                KeyCode::Char('1') => {
                    self.mode = AppMode::Library;
                    self.load_articles()?;
                    return Ok(());
                }
                KeyCode::Char('2') => {
                    self.mode = AppMode::Reader;
                    return Ok(());
                }
                KeyCode::Char('3') => {
                    self.mode = AppMode::Review;
                    self.load_review_queue()?;
                    return Ok(());
                }
                KeyCode::Char('4') => {
                    self.mode = AppMode::Highlights;
                    self.load_all_highlights()?;
                    return Ok(());
                }
                KeyCode::Char(':') => {
                    self.input_mode = InputMode::Command;
                    self.command_active = true;
                    self.command_input.clear();
                    return Ok(());
                }
                _ => {}
            }
        }

        // Router by input mode
        match self.input_mode {
            InputMode::Command => self.handle_command_key(key).await,
            InputMode::Search => self.handle_search_key(key),
            InputMode::Visual => self.handle_visual_key(key),
            InputMode::Normal => match self.mode {
                AppMode::Library => self.handle_library_key(key),
                AppMode::Reader => self.handle_reader_key(key).await,
                AppMode::Review => self.handle_review_key(key),
                AppMode::Highlights => self.handle_highlights_key(key),
            },
        }
    }

    /// Handles keys when Command Line is active.
    async fn handle_command_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                let cmd = self.command_input.clone();
                self.command_active = false;
                self.input_mode = InputMode::Normal;
                self.handle_command(&cmd).await?;
            }
            KeyCode::Esc => {
                self.command_active = false;
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                self.command_input.push(c);
            }
            KeyCode::Backspace => {
                self.command_input.pop();
            }
            _ => {}
        }
        Ok(())
    }

    /// Parses and executes commands input in the command bar.
    async fn handle_command(&mut self, cmd: &str) -> Result<()> {
        let trimmed = cmd.trim();
        if trimmed == "q" || trimmed == "quit" {
            self.should_quit = true;
        } else if trimmed == "help" {
            self.set_status("Modes: 1: Library, 2: Reader, 3: Review, 4: Highlights. Press :q to exit.");
        } else if trimmed.starts_with("add ") {
            let url = trimmed[4..].trim().to_string();
            self.set_status(format!("Clipping URL in background: {}", url));
            let db = self.db.clone();
            tokio::spawn(async move {
                match crate::engine::scrape_url(&url).await {
                    Ok(article) => {
                        let _ = db.insert_article(&article);
                    }
                    Err(e) => {
                        eprintln!("Background clipping failed: {}", e);
                    }
                }
            });
        } else if trimmed.starts_with("category") {
            let cat_name = trimmed["category".len()..].trim().to_string();
            if !self.articles.is_empty() {
                let mut art = self.articles[self.selected_index].clone();
                if cat_name.is_empty() || cat_name.to_lowercase() == "none" || cat_name.to_lowercase() == "clear" {
                    art.category = None;
                    self.db.update_article(&art)?;
                    self.set_status("Category removed from article.");
                } else {
                    art.category = Some(cat_name.clone());
                    self.db.update_article(&art)?;
                    self.set_status(format!("Article assigned to category: {}", cat_name));
                }
                self.reload_categories()?;
                self.load_articles()?;
            }
        } else if trimmed.starts_with("rename ") {
            let new_title = trimmed["rename".len()..].trim().to_string();
            if new_title.is_empty() {
                self.set_status("Error: Title cannot be empty.");
            } else {
                let mut article_to_update = None;
                if self.mode == AppMode::Reader {
                    if let Some(ref art) = self.current_article {
                        let mut updated_art = art.clone();
                        updated_art.title = new_title.clone();
                        article_to_update = Some(updated_art);
                    }
                } else if !self.articles.is_empty() {
                    let mut updated_art = self.articles[self.selected_index].clone();
                    updated_art.title = new_title.clone();
                    article_to_update = Some(updated_art);
                }

                if let Some(art) = article_to_update {
                    self.db.update_article(&art)?;
                    
                    // If we are in Reader mode, update current_article as well so UI updates
                    if self.mode == AppMode::Reader {
                        self.current_article = Some(art);
                    }
                    
                    self.set_status(format!("Article renamed to: {}", new_title));
                    self.load_articles()?;
                    self.load_all_highlights()?;
                } else {
                    self.set_status("Error: No article selected.");
                }
            }
        } else if trimmed == "export" {
            // Export library to a markdown vault in workspace
            let export_dir = std::path::Path::new("/home/amaterasu/Scroll/exports");
            std::fs::create_dir_all(export_dir)?;
            for article in &self.articles {
                let file_name = format!("{}.md", article.id);
                let content = format!(
                    "# {}\nSource: {}\nDate: {}\n\n{}",
                    article.title,
                    article.url.as_deref().unwrap_or("Manual"),
                    article.created_at,
                    article.content_markdown
                );
                std::fs::write(export_dir.join(file_name), content)?;
            }
            self.set_status("Exported library articles to Scroll/exports/");
        } else {
            self.set_status(format!("Unknown command: {}", trimmed));
        }
        Ok(())
    }

    /// Handles keys when search input is active.
    fn handle_search_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                self.search_active = false;
                self.input_mode = InputMode::Normal;
                // Query FTS5
                if !self.search_input.trim().is_empty() {
                    self.articles = self.db.search_articles(&self.search_input)?;
                    self.selected_index = 0;
                } else {
                    self.load_articles()?;
                }
            }
            KeyCode::Esc => {
                self.search_active = false;
                self.input_mode = InputMode::Normal;
                self.load_articles()?;
            }
            KeyCode::Char(c) => {
                self.search_input.push(c);
            }
            KeyCode::Backspace => {
                self.search_input.pop();
            }
            _ => {}
        }
        Ok(())
    }

    /// Visual selection mode keys (for highlights/copying in future expansions).
    fn handle_visual_key(&mut self, _key: KeyEvent) -> Result<()> {
        // Simple visual selection state placeholder
        self.input_mode = InputMode::Normal;
        Ok(())
    }

    /// Normal mode keys in Library view.
    fn handle_library_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.sidebar_focused {
                    if !self.categories.is_empty() {
                        self.selected_category_index = (self.selected_category_index + 1).min(self.categories.len() - 1);
                        self.selected_index = 0;
                        self.load_articles()?;
                    }
                } else {
                    if !self.articles.is_empty() {
                        self.selected_index = (self.selected_index + 1).min(self.articles.len() - 1);
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.sidebar_focused {
                    self.selected_category_index = self.selected_category_index.saturating_sub(1);
                    self.selected_index = 0;
                    self.load_articles()?;
                } else {
                    if !self.articles.is_empty() {
                        self.selected_index = self.selected_index.saturating_sub(1);
                    }
                }
            }
            KeyCode::Enter => {
                if self.sidebar_focused {
                    self.sidebar_focused = false;
                    self.set_status("Focus: Article list");
                } else {
                    if !self.articles.is_empty() {
                        self.open_article(self.selected_index)?;
                    }
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if !self.sidebar_focused {
                    self.sidebar_focused = true;
                    self.set_status("Focus: Sidebar categories");
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if self.sidebar_focused {
                    self.sidebar_focused = false;
                    self.set_status("Focus: Article list");
                }
            }
            KeyCode::Tab => {
                // Cycle tabs forward
                let next_tab = match self.filter_tab {
                    FilterTab::All => FilterTab::Unread,
                    FilterTab::Unread => FilterTab::Reading,
                    FilterTab::Reading => FilterTab::Favorites,
                    FilterTab::Favorites => FilterTab::Archived,
                    FilterTab::Archived => FilterTab::All,
                };
                self.filter_tab = next_tab;
                self.article_filter = next_tab.to_filter();
                self.load_articles()?;
                self.selected_index = 0;
            }
            KeyCode::BackTab => {
                // Cycle tabs in reverse (Shift+Tab)
                let prev_tab = match self.filter_tab {
                    FilterTab::All => FilterTab::Archived,
                    FilterTab::Unread => FilterTab::All,
                    FilterTab::Reading => FilterTab::Unread,
                    FilterTab::Favorites => FilterTab::Reading,
                    FilterTab::Archived => FilterTab::Favorites,
                };
                self.filter_tab = prev_tab;
                self.article_filter = prev_tab.to_filter();
                self.load_articles()?;
                self.selected_index = 0;
            }
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_active = true;
                self.search_input.clear();
            }
            KeyCode::Char('a') => {
                self.input_mode = InputMode::Command;
                self.command_active = true;
                self.command_input = "add ".to_string();
            }
            KeyCode::Char('c') => {
                if !self.articles.is_empty() {
                    self.input_mode = InputMode::Command;
                    self.command_active = true;
                    self.command_input = "category ".to_string();
                }
            }
            KeyCode::Char('r') => {
                if !self.articles.is_empty() {
                    let current_title = &self.articles[self.selected_index].title;
                    self.input_mode = InputMode::Command;
                    self.command_active = true;
                    self.command_input = format!("rename {}", current_title);
                }
            }
            KeyCode::Char('d') => {
                // Permanently delete selected article
                if !self.articles.is_empty() {
                    let art = &self.articles[self.selected_index];
                    self.db.delete_article(&art.id)?;
                    self.set_status("Article permanently deleted.");
                    self.load_articles()?;
                    self.load_all_highlights()?;
                }
            }
            KeyCode::Char('x') => {
                // Archive selected article
                if !self.articles.is_empty() {
                    let art = &self.articles[self.selected_index];
                    self.db.update_article_status(&art.id, ArticleStatus::Archived)?;
                    self.set_status("Article archived.");
                    self.load_articles()?;
                }
            }
            KeyCode::Char('f') => {
                // Toggle favorite
                if !self.articles.is_empty() {
                    let art = &self.articles[self.selected_index];
                    let val = self.db.toggle_favorite(&art.id)?;
                    self.set_status(if val { "Added to favorites." } else { "Removed from favorites." });
                    self.load_articles()?;
                }
            }
            KeyCode::Char('s') => {
                // Cycle sorting
                let next_sort = match self.article_filter.sort_by {
                    SortField::CreatedAt => SortField::Title,
                    SortField::Title => SortField::ReadingProgress,
                    SortField::ReadingProgress => SortField::UpdatedAt,
                    SortField::UpdatedAt => SortField::CreatedAt,
                };
                self.article_filter.sort_by = next_sort;
                self.set_status(format!("Sorted by: {:?}", next_sort));
                self.load_articles()?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Normal mode keys in Reader view.
    async fn handle_reader_key(&mut self, key: KeyEvent) -> Result<()> {
        let scroll_speed = self.config.reader.scroll_speed;
        
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                // Update reading progress before leaving
                if let Some(ref art) = self.current_article {
                    let total = self.reader_lines.len();
                    if total > 0 {
                        // Estimate visible height
                        let height = 24;
                        let end = (self.reader_scroll + height).min(total);
                        let progress = end as f64 / total as f64;
                        let _ = self.db.update_reading_progress(&art.id, progress.min(1.0), self.reader_scroll as i64);
                    }
                }
                self.mode = AppMode::Library;
                self.load_articles()?;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let max_scroll = self.reader_lines.len().saturating_sub(1);
                self.reader_scroll = (self.reader_scroll + scroll_speed).min(max_scroll);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.reader_scroll = self.reader_scroll.saturating_sub(scroll_speed);
            }
            KeyCode::Char('G') => {
                self.reader_scroll = self.reader_lines.len().saturating_sub(1);
            }
            KeyCode::Char('g') => {
                // For 'gg' shortcut, we check modifiers or double key presses.
                // Simple implementation: 'g' takes you to top.
                self.reader_scroll = 0;
            }
            KeyCode::Char('h') => {
                // Highlight current visible top line
                if let Some(art) = self.current_article.clone() {
                    if !self.reader_lines.is_empty() && self.reader_scroll < self.reader_lines.len() {
                        let text = self.reader_lines[self.reader_scroll].trim().to_string();
                        if !text.is_empty() {
                            let highlight = Highlight::new(art.id.clone(), text, "yellow".to_string());
                            self.db.insert_highlight(&highlight)?;
                            
                            // Auto-generate flashcards if AI enabled
                            if self.config.ai.enabled {
                                self.set_status("Generating flashcards in background...");
                                let client = OllamaClient::new(&self.config.ai);
                                let db = self.db.clone();
                                let art_id = art.id.clone();
                                let hl_id = highlight.id.clone();
                                let hl_text = highlight.text.clone();
                                let context = self.reader_lines[self.reader_scroll..]
                                    .iter()
                                    .take(5)
                                    .cloned()
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                    
                                tokio::spawn(async move {
                                    match crate::ai::generate_flashcards(&client, &hl_text, &context).await {
                                        Ok(pairs) => {
                                            for (q, a) in pairs {
                                                let mut fc = Flashcard::new(art_id.clone(), q, a);
                                                fc.highlight_id = Some(hl_id.clone());
                                                let _ = db.insert_flashcard(&fc);
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to auto-generate flashcards: {}", e);
                                        }
                                    }
                                });
                            } else {
                                self.set_status("Highlight created!");
                            }
                        }
                    }
                }
            }
            KeyCode::Char('r') => {
                if let Some(ref art) = self.current_article {
                    self.input_mode = InputMode::Command;
                    self.command_active = true;
                    self.command_input = format!("rename {}", art.title);
                }
            }
            KeyCode::Char('s') => {
                // Trigger summary
                if let Some(art) = self.current_article.clone() {
                    if self.config.ai.enabled {
                        self.set_status("Generating article summary via LLM...");
                        let client = OllamaClient::new(&self.config.ai);
                        let db = self.db.clone();
                        let mut art_cloned = art;
                        tokio::spawn(async move {
                            match crate::ai::summarize_article(&client, &art_cloned).await {
                                Ok(summary) => {
                                    art_cloned.ai_summary = Some(summary);
                                    let _ = db.update_article(&art_cloned);
                                }
                                Err(e) => {
                                    eprintln!("Failed to summarize: {}", e);
                                }
                            }
                        });
                    } else {
                        self.set_status("AI is disabled in config.");
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Normal mode keys in Review view.
    fn handle_review_key(&mut self, key: KeyEvent) -> Result<()> {
        if self.review_queue.is_empty() || self.review_index >= self.review_queue.len() {
            return Ok(());
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                // Save review session log
                if self.review_session.cards_reviewed > 0 {
                    self.db.insert_review_session(&self.review_session)?;
                }
                self.mode = AppMode::Library;
                self.load_articles()?;
            }
            KeyCode::Char(' ') => {
                self.card_flipped = !self.card_flipped;
            }
            KeyCode::Char('s') => {
                // Skip card
                self.review_index += 1;
                self.card_flipped = false;
            }
            KeyCode::Char(c) if ('1'..='5').contains(&c) => {
                if self.card_flipped {
                    let rating = match key.code {
                        KeyCode::Char('1') => 1,
                        KeyCode::Char('2') => 2,
                        KeyCode::Char('3') => 3,
                        KeyCode::Char('4') => 4,
                        _ => 5,
                    };

                    let mut card = self.review_queue[self.review_index].clone();
                    sm2_review(&mut card, rating);
                    self.db.update_flashcard_schedule(&card)?;

                    // Log session stats
                    self.review_session.cards_reviewed += 1;
                    if rating >= 3 {
                        self.review_session.cards_correct += 1;
                    }

                    // Next card
                    self.review_index += 1;
                    self.card_flipped = false;

                    // If ended, save session
                    if self.review_index >= self.review_queue.len() {
                        self.db.insert_review_session(&self.review_session)?;
                        self.set_status("Spaced repetition session completed!");
                        self.due_count = self.db.count_due_today().unwrap_or(0);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Normal mode keys in Highlights view.
    fn handle_highlights_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.mode = AppMode::Library;
                self.load_articles()?;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if !self.all_highlights.is_empty() {
                    self.highlight_index = (self.highlight_index + 1).min(self.all_highlights.len() - 1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if !self.all_highlights.is_empty() {
                    self.highlight_index = self.highlight_index.saturating_sub(1);
                }
            }
            KeyCode::Enter => {
                // Open parent article
                if !self.all_highlights.is_empty() {
                    let art_id = self.all_highlights[self.highlight_index].0.article_id.clone();
                    // Find article index
                    let mut found_idx = None;
                    for (i, art) in self.articles.iter().enumerate() {
                        if art.id == art_id {
                            found_idx = Some(i);
                            break;
                        }
                    }
                    if let Some(idx) = found_idx {
                        self.open_article(idx)?;
                    } else {
                        // Not found in current filtered articles, open directly from database
                        if let Some(_art) = self.db.get_article(&art_id)? {
                            // Clear filters to show it
                            self.article_filter = ArticleFilter::default();
                            self.load_articles()?;
                            
                            // Re-find index
                            for (i, a) in self.articles.iter().enumerate() {
                                if a.id == art_id {
                                    self.open_article(i)?;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            KeyCode::Char('d') => {
                // Delete highlight
                if !self.all_highlights.is_empty() {
                    let (hl, _) = &self.all_highlights[self.highlight_index];
                    self.db.delete_highlight(&hl.id)?;
                    self.set_status("Highlight deleted.");
                    self.load_all_highlights()?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

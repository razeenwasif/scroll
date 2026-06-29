//! Data models shared across all Scroll modules.

use serde::{Deserialize, Serialize};

// ── Article ────────────────────────────────────────────────────────────────

/// A saved article, PDF, or manual note in the library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: String,
    pub url: Option<String>,
    pub title: String,
    pub author: Option<String>,
    pub site_name: Option<String>,
    pub content_markdown: String,
    pub content_html: Option<String>,
    pub excerpt: Option<String>,
    pub cover_image_url: Option<String>,
    pub word_count: i64,
    pub reading_time_min: i64,
    pub reading_progress: f64,
    pub scroll_position: i64,
    pub status: ArticleStatus,
    pub is_favorite: bool,
    pub source_type: SourceType,
    pub ai_summary: Option<String>,
    pub category: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Article {
    /// Create a new article with sensible defaults.
    pub fn new(title: String, content_markdown: String, source_type: SourceType) -> Self {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let word_count = content_markdown.split_whitespace().count() as i64;
        let reading_time_min = (word_count / 250).max(1);
        Self {
            id: nanoid::nanoid!(),
            url: None,
            title,
            author: None,
            site_name: None,
            content_markdown,
            content_html: None,
            excerpt: None,
            cover_image_url: None,
            word_count,
            reading_time_min,
            reading_progress: 0.0,
            scroll_position: 0,
            status: ArticleStatus::Unread,
            is_favorite: false,
            source_type,
            ai_summary: None,
            category: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// A short display string for the status.
    pub fn status_icon(&self) -> &'static str {
        match self.status {
            ArticleStatus::Unread => "○",
            ArticleStatus::Reading => "◐",
            ArticleStatus::Read => "●",
            ArticleStatus::Archived => "◌",
        }
    }

    /// A short display string for the source type.
    pub fn source_icon(&self) -> &'static str {
        match self.source_type {
            SourceType::Web => "🌐",
            SourceType::Pdf => "📄",
            SourceType::Manual => "✏",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArticleStatus {
    Unread,
    Reading,
    Read,
    Archived,
}

impl ArticleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unread => "unread",
            Self::Reading => "reading",
            Self::Read => "read",
            Self::Archived => "archived",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "reading" => Self::Reading,
            "read" => Self::Read,
            "archived" => Self::Archived,
            _ => Self::Unread,
        }
    }

    /// All variants for cycling through filters.
    #[allow(dead_code)]
    pub fn all() -> &'static [ArticleStatus] {
        &[Self::Unread, Self::Reading, Self::Read, Self::Archived]
    }
}

impl std::fmt::Display for ArticleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Web,
    Pdf,
    Manual,
}

impl SourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::Pdf => "pdf",
            Self::Manual => "manual",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "pdf" => Self::Pdf,
            "manual" => Self::Manual,
            _ => Self::Web,
        }
    }
}

// ── Tag ────────────────────────────────────────────────────────────────────

/// A user-defined tag for organizing articles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
}

impl Tag {
    pub fn new(name: String, color: String) -> Self {
        Self {
            id: nanoid::nanoid!(),
            name,
            color,
        }
    }
}

// ── Highlight ──────────────────────────────────────────────────────────────

/// A text highlight within an article, with optional annotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    pub id: String,
    pub article_id: String,
    pub text: String,
    pub note: Option<String>,
    pub color: String,
    pub start_offset: Option<i64>,
    pub end_offset: Option<i64>,
    pub created_at: String,
}

impl Highlight {
    pub fn new(article_id: String, text: String, color: String) -> Self {
        Self {
            id: nanoid::nanoid!(),
            article_id,
            text,
            note: None,
            color,
            start_offset: None,
            end_offset: None,
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

/// Available highlight colors.
#[allow(dead_code)]
pub const HIGHLIGHT_COLORS: &[&str] = &["yellow", "green", "blue", "red", "purple"];

// ── Flashcard ──────────────────────────────────────────────────────────────

/// A spaced-repetition flashcard, optionally linked to a highlight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flashcard {
    pub id: String,
    pub article_id: String,
    pub highlight_id: Option<String>,
    pub front: String,
    pub back: String,
    pub card_type: CardType,
    // SM-2 scheduling fields
    pub easiness_factor: f64,
    pub interval_days: i64,
    pub repetitions: i64,
    pub next_review_at: String,
    pub last_reviewed_at: Option<String>,
    pub created_at: String,
}

impl Flashcard {
    pub fn new(article_id: String, front: String, back: String) -> Self {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            id: nanoid::nanoid!(),
            article_id,
            highlight_id: None,
            front,
            back,
            card_type: CardType::Basic,
            easiness_factor: 2.5,
            interval_days: 0,
            repetitions: 0,
            next_review_at: now.clone(),
            last_reviewed_at: None,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CardType {
    Basic,
    Cloze,
    Highlight,
}

impl CardType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Basic => "basic",
            Self::Cloze => "cloze",
            Self::Highlight => "highlight",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "cloze" => Self::Cloze,
            "highlight" => Self::Highlight,
            _ => Self::Basic,
        }
    }
}

// ── Review Session ─────────────────────────────────────────────────────────

/// Tracks statistics for a single review session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSession {
    pub id: String,
    pub date: String,
    pub cards_reviewed: i64,
    pub cards_correct: i64,
    pub duration_seconds: i64,
    pub created_at: String,
}

impl ReviewSession {
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: nanoid::nanoid!(),
            date: now.format("%Y-%m-%d").to_string(),
            cards_reviewed: 0,
            cards_correct: 0,
            duration_seconds: 0,
            created_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

impl Default for ReviewSession {
    fn default() -> Self {
        Self::new()
    }
}

// ── Filters & Sorting ─────────────────────────────────────────────────────

/// Filter criteria for listing articles.
#[derive(Debug, Clone, Default)]
pub struct ArticleFilter {
    pub status: Option<ArticleStatus>,
    pub is_favorite: Option<bool>,
    pub tag_id: Option<String>,
    pub category: Option<String>,
    pub search_query: Option<String>,
    pub sort_by: SortField,
    pub sort_desc: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortField {
    #[default]
    CreatedAt,
    Title,
    ReadingProgress,
    UpdatedAt,
}

impl SortField {
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::CreatedAt => "created_at",
            Self::Title => "title",
            Self::ReadingProgress => "reading_progress",
            Self::UpdatedAt => "updated_at",
        }
    }
}

/// Which filter tab is currently active in the library view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterTab {
    #[default]
    All,
    Unread,
    Reading,
    Favorites,
    Archived,
}

impl FilterTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Unread => "Unread",
            Self::Reading => "Reading",
            Self::Favorites => "★ Favorites",
            Self::Archived => "Archived",
        }
    }

    pub fn all() -> &'static [FilterTab] {
        &[Self::All, Self::Unread, Self::Reading, Self::Favorites, Self::Archived]
    }

    pub fn to_filter(&self) -> ArticleFilter {
        match self {
            Self::All => ArticleFilter::default(),
            Self::Unread => ArticleFilter {
                status: Some(ArticleStatus::Unread),
                sort_desc: true,
                ..Default::default()
            },
            Self::Reading => ArticleFilter {
                status: Some(ArticleStatus::Reading),
                sort_desc: true,
                ..Default::default()
            },
            Self::Favorites => ArticleFilter {
                is_favorite: Some(true),
                sort_desc: true,
                ..Default::default()
            },
            Self::Archived => ArticleFilter {
                status: Some(ArticleStatus::Archived),
                sort_desc: true,
                ..Default::default()
            },
        }
    }
}

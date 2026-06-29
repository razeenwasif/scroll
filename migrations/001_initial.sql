-- Scroll database schema v1
-- Migration: 001_initial

-- ── Articles ───────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS articles (
    id              TEXT PRIMARY KEY,
    url             TEXT,
    title           TEXT NOT NULL,
    author          TEXT,
    site_name       TEXT,
    content_markdown TEXT NOT NULL,
    content_html    TEXT,
    excerpt         TEXT,
    cover_image_url TEXT,
    word_count      INTEGER DEFAULT 0,
    reading_time_min INTEGER DEFAULT 0,
    reading_progress REAL DEFAULT 0.0,
    scroll_position INTEGER DEFAULT 0,
    status          TEXT DEFAULT 'unread',
    is_favorite     INTEGER DEFAULT 0,
    source_type     TEXT DEFAULT 'web',
    ai_summary      TEXT,
    category        TEXT,
    created_at      TEXT DEFAULT (datetime('now')),
    updated_at      TEXT DEFAULT (datetime('now'))
);

-- ── Tags ───────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS tags (
    id    TEXT PRIMARY KEY,
    name  TEXT UNIQUE NOT NULL,
    color TEXT DEFAULT 'blue'
);

CREATE TABLE IF NOT EXISTS article_tags (
    article_id TEXT REFERENCES articles(id) ON DELETE CASCADE,
    tag_id     TEXT REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (article_id, tag_id)
);

-- ── Highlights ─────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS highlights (
    id           TEXT PRIMARY KEY,
    article_id   TEXT NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
    text         TEXT NOT NULL,
    note         TEXT,
    color        TEXT DEFAULT 'yellow',
    start_offset INTEGER,
    end_offset   INTEGER,
    created_at   TEXT DEFAULT (datetime('now'))
);

-- ── Flashcards ─────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS flashcards (
    id               TEXT PRIMARY KEY,
    article_id       TEXT NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
    highlight_id     TEXT REFERENCES highlights(id) ON DELETE SET NULL,
    front            TEXT NOT NULL,
    back             TEXT NOT NULL,
    card_type        TEXT DEFAULT 'basic',
    easiness_factor  REAL DEFAULT 2.5,
    interval_days    INTEGER DEFAULT 0,
    repetitions      INTEGER DEFAULT 0,
    next_review_at   TEXT DEFAULT (datetime('now')),
    last_reviewed_at TEXT,
    created_at       TEXT DEFAULT (datetime('now'))
);

-- ── Review Sessions ────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS review_sessions (
    id               TEXT PRIMARY KEY,
    date             TEXT NOT NULL,
    cards_reviewed   INTEGER DEFAULT 0,
    cards_correct    INTEGER DEFAULT 0,
    duration_seconds INTEGER DEFAULT 0,
    created_at       TEXT DEFAULT (datetime('now'))
);

-- ── Full-Text Search ───────────────────────────────────────────────────────

CREATE VIRTUAL TABLE IF NOT EXISTS articles_fts USING fts5(
    title,
    content_markdown,
    ai_summary,
    excerpt,
    content='articles',
    content_rowid='rowid'
);

-- Triggers to keep the FTS index synchronized with the articles table.

CREATE TRIGGER IF NOT EXISTS articles_fts_insert AFTER INSERT ON articles BEGIN
    INSERT INTO articles_fts(rowid, title, content_markdown, ai_summary, excerpt)
    VALUES (new.rowid, new.title, new.content_markdown, new.ai_summary, new.excerpt);
END;

CREATE TRIGGER IF NOT EXISTS articles_fts_delete AFTER DELETE ON articles BEGIN
    INSERT INTO articles_fts(articles_fts, rowid, title, content_markdown, ai_summary, excerpt)
    VALUES ('delete', old.rowid, old.title, old.content_markdown, old.ai_summary, old.excerpt);
END;

CREATE TRIGGER IF NOT EXISTS articles_fts_update AFTER UPDATE ON articles BEGIN
    INSERT INTO articles_fts(articles_fts, rowid, title, content_markdown, ai_summary, excerpt)
    VALUES ('delete', old.rowid, old.title, old.content_markdown, old.ai_summary, old.excerpt);
    INSERT INTO articles_fts(rowid, title, content_markdown, ai_summary, excerpt)
    VALUES (new.rowid, new.title, new.content_markdown, new.ai_summary, new.excerpt);
END;

-- ── Indexes ────────────────────────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS idx_articles_status       ON articles(status);
CREATE INDEX IF NOT EXISTS idx_articles_created_at   ON articles(created_at);
CREATE INDEX IF NOT EXISTS idx_articles_is_favorite  ON articles(is_favorite);
CREATE INDEX IF NOT EXISTS idx_articles_source_type  ON articles(source_type);
CREATE INDEX IF NOT EXISTS idx_highlights_article_id ON highlights(article_id);
CREATE INDEX IF NOT EXISTS idx_flashcards_article_id ON flashcards(article_id);
CREATE INDEX IF NOT EXISTS idx_flashcards_next_review ON flashcards(next_review_at);
CREATE INDEX IF NOT EXISTS idx_review_sessions_date  ON review_sessions(date);

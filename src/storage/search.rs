//! Full-text search (FTS) operations on articles.

use anyhow::{Context, Result};
use rusqlite::params;
use crate::models::Article;
use super::db::Database;

impl Database {
    /// Searches articles using SQLite FTS5 MATCH on title, markdown content, summary, and excerpt.
    pub fn search_articles(&self, query: &str) -> Result<Vec<Article>> {
        let conn = self.conn();
        
        // Return all articles if query is empty
        if query.trim().is_empty() {
            return self.list_articles(&crate::models::ArticleFilter::default());
        }

        // Query joining articles with their FTS virtual table matching the user's terms
        let mut stmt = conn.prepare(
            "SELECT a.* FROM articles a 
             INNER JOIN articles_fts fts ON a.rowid = fts.rowid 
             WHERE articles_fts MATCH ?1 
             ORDER BY rank"
        )?;

        // Map database row using row_to_article from articles submodule.
        // Wait, row_to_article is in the same impl Database block but defined in articles.rs.
        // Since Rust impl blocks are split across files but belong to the same struct,
        // we can call row_to_article directly if it is visible or define a local row_to_article helper,
        // or since row_to_article is a helper in articles.rs, wait!
        // Is row_to_article defined as pub or at module/crate level?
        // In articles.rs: `fn row_to_article(...)` is a private function in crate::storage::articles module.
        // Oh! In Rust, a private function defined in `articles.rs` (which is `crate::storage::articles` module)
        // is NOT visible in `crate::storage::search` even though they both impl `Database`.
        // To make it visible, we can declare `pub(crate) fn row_to_article` in `articles.rs` and refer to it as `super::articles::row_to_article`,
        // or we can define a shared helper or make it `pub(super) fn row_to_article`.
        // Let's modify `articles.rs` to make it `pub(super) fn row_to_article` or make a shared helper.
        // Actually, we can just make it `pub(super) fn row_to_article` in `articles.rs`.
        // Wait! Let's check `search.rs`'s code. If we refer to it, we can do `super::articles::row_to_article`.
        // Let's write `search.rs` referring to `super::articles::row_to_article`.
        // Let's edit `articles.rs` later if needed, but wait! We already wrote `articles.rs` with `fn row_to_article`.
        // Let's check: yes, `fn row_to_article` was defined as private. We will need to change it to `pub(super) fn row_to_article` in `articles.rs`.
        // Let's do that right now by writing `search.rs` and then adjusting `articles.rs` with a single replace/edit, or writing `search.rs` first.
        
        let article_iter = stmt.query_map(params![query], super::articles::row_to_article)?;

        let mut articles = Vec::new();
        for item in article_iter {
            articles.push(item?);
        }
        Ok(articles)
    }

    /// Rebuilds the FTS5 search index from the current articles table.
    #[allow(dead_code)]
    pub fn rebuild_search_index(&self) -> Result<()> {
        let conn = self.conn();
        conn.execute("INSERT INTO articles_fts(articles_fts) VALUES('rebuild')", [])
            .context("Failed to rebuild search index")?;
        Ok(())
    }
}

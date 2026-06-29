//! Article database operations.

use std::collections::HashMap;
use anyhow::{Context, Result};
use rusqlite::params;
use crate::models::{Article, ArticleFilter, ArticleStatus, SourceType};
use super::db::Database;

impl Database {
    /// Inserts a new article into the database.
    pub fn insert_article(&self, article: &Article) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO articles (
                id, url, title, author, site_name, content_markdown, content_html, 
                excerpt, cover_image_url, word_count, reading_time_min, 
                reading_progress, scroll_position, status, is_favorite, 
                source_type, ai_summary, category, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
            params![
                article.id,
                article.url,
                article.title,
                article.author,
                article.site_name,
                article.content_markdown,
                article.content_html,
                article.excerpt,
                article.cover_image_url,
                article.word_count,
                article.reading_time_min,
                article.reading_progress,
                article.scroll_position,
                article.status.as_str(),
                if article.is_favorite { 1 } else { 0 },
                article.source_type.as_str(),
                article.ai_summary,
                article.category,
                article.created_at,
                article.updated_at,
            ],
        )
        .context("Failed to insert article")?;
        Ok(())
    }

    /// Retrieves a single article by ID.
    pub fn get_article(&self, id: &str) -> Result<Option<Article>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT * FROM articles WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_article(row)?))
        } else {
            Ok(None)
        }
    }

    /// Lists articles matching the given filter.
    pub fn list_articles(&self, filter: &ArticleFilter) -> Result<Vec<Article>> {
        let conn = self.conn();
        let mut query = String::from("SELECT * FROM articles WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(status) = &filter.status {
            query.push_str(" AND status = ?");
            params_vec.push(Box::new(status.as_str().to_string()));
        }

        if let Some(is_favorite) = filter.is_favorite {
            query.push_str(" AND is_favorite = ?");
            params_vec.push(Box::new(if is_favorite { 1 } else { 0 }));
        }

        if let Some(tag_id) = &filter.tag_id {
            query.push_str(" AND id IN (SELECT article_id FROM article_tags WHERE tag_id = ?)");
            params_vec.push(Box::new(tag_id.clone()));
        }

        if let Some(category) = &filter.category {
            if category.is_empty() {
                query.push_str(" AND (category IS NULL OR category = '')");
            } else {
                query.push_str(" AND category = ?");
                params_vec.push(Box::new(category.clone()));
            }
        }

        if let Some(search_query) = &filter.search_query {
            query.push_str(" AND (title LIKE ? OR content_markdown LIKE ?)");
            let term = format!("%{}%", search_query);
            params_vec.push(Box::new(term.clone()));
            params_vec.push(Box::new(term));
        }

        // Apply sort
        let sort_col = filter.sort_by.as_sql();
        let sort_order = if filter.sort_desc { "DESC" } else { "ASC" };
        query.push_str(&format!(" ORDER BY {} {}", sort_col, sort_order));

        let mut stmt = conn.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let article_iter = stmt.query_map(&param_refs[..], row_to_article)?;

        let mut articles = Vec::new();
        for item in article_iter {
            articles.push(item?);
        }
        Ok(articles)
    }

    /// Updates an existing article's metadata and content.
    pub fn update_article(&self, article: &Article) -> Result<()> {
        let conn = self.conn();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "UPDATE articles SET 
                url = ?2, title = ?3, author = ?4, site_name = ?5, 
                content_markdown = ?6, content_html = ?7, excerpt = ?8, 
                cover_image_url = ?9, word_count = ?10, reading_time_min = ?11, 
                reading_progress = ?12, scroll_position = ?13, status = ?14, 
                is_favorite = ?15, source_type = ?16, ai_summary = ?17, 
                category = ?18, updated_at = ?19 
             WHERE id = ?1",
            params![
                article.id,
                article.url,
                article.title,
                article.author,
                article.site_name,
                article.content_markdown,
                article.content_html,
                article.excerpt,
                article.cover_image_url,
                article.word_count,
                article.reading_time_min,
                article.reading_progress,
                article.scroll_position,
                article.status.as_str(),
                if article.is_favorite { 1 } else { 0 },
                article.source_type.as_str(),
                article.ai_summary,
                article.category,
                now,
            ],
        )
        .context("Failed to update article")?;
        Ok(())
    }

    /// Updates the reading progress and scroll position of an article.
    pub fn update_reading_progress(&self, id: &str, progress: f64, scroll_pos: i64) -> Result<()> {
        let conn = self.conn();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        // If progress is near 100%, update status to read, otherwise if progress > 0, set to reading
        let status = if progress >= 0.99 {
            ArticleStatus::Read
        } else if progress > 0.0 {
            ArticleStatus::Reading
        } else {
            ArticleStatus::Unread
        };

        conn.execute(
            "UPDATE articles SET reading_progress = ?2, scroll_position = ?3, status = ?4, updated_at = ?5 WHERE id = ?1",
            params![id, progress, scroll_pos, status.as_str(), now],
        )
        .context("Failed to update reading progress")?;
        Ok(())
    }

    /// Updates the status of an article (e.g. read, unread, archived).
    pub fn update_article_status(&self, id: &str, status: ArticleStatus) -> Result<()> {
        let conn = self.conn();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "UPDATE articles SET status = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, status.as_str(), now],
        )
        .context("Failed to update article status")?;
        Ok(())
    }

    /// Toggles the favorite status of an article and returns the new value.
    pub fn toggle_favorite(&self, id: &str) -> Result<bool> {
        let conn = self.conn();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut stmt = conn.prepare("SELECT is_favorite FROM articles WHERE id = ?1")?;
        let is_favorite: i32 = stmt.query_row(params![id], |r| r.get(0))?;
        
        let new_value = is_favorite == 0;
        conn.execute(
            "UPDATE articles SET is_favorite = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, if new_value { 1 } else { 0 }, now],
        )
        .context("Failed to toggle favorite")?;
        
        Ok(new_value)
    }

    /// Deletes an article from the database (cascade deletes tags, highlights, flashcards due to FK constraints).
    pub fn delete_article(&self, id: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute("DELETE FROM articles WHERE id = ?1", params![id])
            .context("Failed to delete article")?;
        Ok(())
    }

    /// Counts the articles grouped by status.
    #[allow(dead_code)]
    pub fn count_articles_by_status(&self) -> Result<HashMap<String, i64>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT status, COUNT(*) FROM articles GROUP BY status")?;
        let mut rows = stmt.query([])?;
        
        let mut counts = HashMap::new();
        while let Some(row) = rows.next()? {
            let status: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            counts.insert(status, count);
        }
        Ok(counts)
    }

    /// Lists all unique categories in the database.
    pub fn list_categories(&self) -> Result<Vec<String>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT DISTINCT category FROM articles WHERE category IS NOT NULL AND category != '' ORDER BY category ASC")?;
        let rows = stmt.query_map([], |r| r.get::<_, Option<String>>(0))?;
        let mut categories = Vec::new();
        for r in rows {
            if let Ok(Some(cat)) = r {
                if !cat.trim().is_empty() {
                    categories.push(cat);
                }
            }
        }
        Ok(categories)
    }
}

/// Maps a database row to the Article structure.
pub(super) fn row_to_article(row: &rusqlite::Row) -> rusqlite::Result<Article> {
    let is_favorite_int: i32 = row.get("is_favorite")?;
    let status_str: String = row.get("status")?;
    let source_str: String = row.get("source_type")?;
    
    Ok(Article {
        id: row.get("id")?,
        url: row.get("url")?,
        title: row.get("title")?,
        author: row.get("author")?,
        site_name: row.get("site_name")?,
        content_markdown: row.get("content_markdown")?,
        content_html: row.get("content_html")?,
        excerpt: row.get("excerpt")?,
        cover_image_url: row.get("cover_image_url")?,
        word_count: row.get("word_count")?,
        reading_time_min: row.get("reading_time_min")?,
        reading_progress: row.get("reading_progress")?,
        scroll_position: row.get("scroll_position")?,
        status: ArticleStatus::from_str(&status_str),
        is_favorite: is_favorite_int != 0,
        source_type: SourceType::from_str(&source_str),
        ai_summary: row.get("ai_summary")?,
        category: row.get("category")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SourceType;
    use std::path::Path;

    #[test]
    fn test_article_rename() {
        let db = Database::new(Path::new(":memory:")).unwrap();
        let mut article = Article::new(
            "Original Title".to_string(),
            "Some markdown content".to_string(),
            SourceType::Web,
        );
        article.id = "test-article-1".to_string();

        db.insert_article(&article).unwrap();

        // Retrieve and assert
        let retrieved = db.get_article("test-article-1").unwrap().unwrap();
        assert_eq!(retrieved.title, "Original Title");

        // Rename/update title
        let mut updated = retrieved;
        updated.title = "Renamed Title".to_string();
        db.update_article(&updated).unwrap();

        // Retrieve and assert rename
        let renamed = db.get_article("test-article-1").unwrap().unwrap();
        assert_eq!(renamed.title, "Renamed Title");
    }
}

//! Tag database operations.

use anyhow::{Context, Result};
use rusqlite::params;
use crate::models::Tag;
use super::db::Database;

impl Database {
    /// Inserts a new tag, or returns success if a tag with the same name already exists.
    pub fn insert_tag(&self, tag: &Tag) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT OR IGNORE INTO tags (id, name, color) VALUES (?1, ?2, ?3)",
            params![tag.id, tag.name, tag.color],
        )
        .context("Failed to insert tag")?;
        Ok(())
    }

    /// Lists all tags in the database.
    #[allow(dead_code)]
    pub fn list_tags(&self) -> Result<Vec<Tag>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT * FROM tags ORDER BY name ASC")?;
        let tag_iter = stmt.query_map([], row_to_tag)?;

        let mut tags = Vec::new();
        for t in tag_iter {
            tags.push(t?);
        }
        Ok(tags)
    }

    /// Associates a tag with an article.
    pub fn tag_article(&self, article_id: &str, tag_id: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT OR IGNORE INTO article_tags (article_id, tag_id) VALUES (?1, ?2)",
            params![article_id, tag_id],
        )
        .context("Failed to associate tag with article")?;
        Ok(())
    }

    /// Removes a tag association from an article.
    #[allow(dead_code)]
    pub fn untag_article(&self, article_id: &str, tag_id: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "DELETE FROM article_tags WHERE article_id = ?1 AND tag_id = ?2",
            params![article_id, tag_id],
        )
        .context("Failed to untag article")?;
        Ok(())
    }

    /// Gets all tags associated with a specific article.
    #[allow(dead_code)]
    pub fn get_tags_for_article(&self, article_id: &str) -> Result<Vec<Tag>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT t.* FROM tags t 
             INNER JOIN article_tags at ON t.id = at.tag_id 
             WHERE at.article_id = ?1 
             ORDER BY t.name ASC"
        )?;
        
        let tag_iter = stmt.query_map(params![article_id], row_to_tag)?;

        let mut tags = Vec::new();
        for t in tag_iter {
            tags.push(t?);
        }
        Ok(tags)
    }

    /// Deletes a tag entirely (removes tag and all its associations due to cascade delete).
    #[allow(dead_code)]
    pub fn delete_tag(&self, id: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute("DELETE FROM tags WHERE id = ?1", params![id])
            .context("Failed to delete tag")?;
        Ok(())
    }
}

/// Helper to map database rows to a Tag struct.
#[allow(dead_code)]
fn row_to_tag(row: &rusqlite::Row) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get("id")?,
        name: row.get("name")?,
        color: row.get("color")?,
    })
}

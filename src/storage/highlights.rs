//! Highlight database operations.

use anyhow::{Context, Result};
use rusqlite::params;
use crate::models::Highlight;
use super::db::Database;

impl Database {
    /// Inserts a new highlight.
    pub fn insert_highlight(&self, highlight: &Highlight) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO highlights (
                id, article_id, text, note, color, start_offset, end_offset, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                highlight.id,
                highlight.article_id,
                highlight.text,
                highlight.note,
                highlight.color,
                highlight.start_offset,
                highlight.end_offset,
                highlight.created_at,
            ],
        )
        .context("Failed to insert highlight")?;
        Ok(())
    }

    /// Gets all highlights for a specific article.
    pub fn get_highlights_for_article(&self, article_id: &str) -> Result<Vec<Highlight>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT * FROM highlights WHERE article_id = ?1 ORDER BY created_at ASC")?;
        let highlight_iter = stmt.query_map(params![article_id], row_to_highlight)?;

        let mut highlights = Vec::new();
        for h in highlight_iter {
            highlights.push(h?);
        }
        Ok(highlights)
    }

    /// Lists all highlights across all articles.
    pub fn list_all_highlights(&self) -> Result<Vec<Highlight>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT * FROM highlights ORDER BY created_at DESC")?;
        let highlight_iter = stmt.query_map([], row_to_highlight)?;

        let mut highlights = Vec::new();
        for h in highlight_iter {
            highlights.push(h?);
        }
        Ok(highlights)
    }

    /// Updates the note/comment attached to a highlight.
    #[allow(dead_code)]
    pub fn update_highlight_note(&self, id: &str, note: &str) -> Result<()> {
        let conn = self.conn();
        let note_value = if note.trim().is_empty() { None } else { Some(note.to_string()) };
        conn.execute(
            "UPDATE highlights SET note = ?2 WHERE id = ?1",
            params![id, note_value],
        )
        .context("Failed to update highlight note")?;
        Ok(())
    }

    /// Deletes a highlight.
    pub fn delete_highlight(&self, id: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute("DELETE FROM highlights WHERE id = ?1", params![id])
            .context("Failed to delete highlight")?;
        Ok(())
    }
}

/// Helper to map database rows to a Highlight struct.
fn row_to_highlight(row: &rusqlite::Row) -> rusqlite::Result<Highlight> {
    Ok(Highlight {
        id: row.get("id")?,
        article_id: row.get("article_id")?,
        text: row.get("text")?,
        note: row.get("note")?,
        color: row.get("color")?,
        start_offset: row.get("start_offset")?,
        end_offset: row.get("end_offset")?,
        created_at: row.get("created_at")?,
    })
}

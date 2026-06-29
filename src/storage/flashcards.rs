//! Flashcard and review session database operations.

use anyhow::{Context, Result};
use rusqlite::params;
use crate::models::{Flashcard, CardType, ReviewSession};
use super::db::Database;

impl Database {
    /// Inserts a new flashcard.
    pub fn insert_flashcard(&self, card: &Flashcard) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO flashcards (
                id, article_id, highlight_id, front, back, card_type, 
                easiness_factor, interval_days, repetitions, next_review_at, 
                last_reviewed_at, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                card.id,
                card.article_id,
                card.highlight_id,
                card.front,
                card.back,
                card.card_type.as_str(),
                card.easiness_factor,
                card.interval_days,
                card.repetitions,
                card.next_review_at,
                card.last_reviewed_at,
                card.created_at,
            ],
        )
        .context("Failed to insert flashcard")?;
        Ok(())
    }

    /// Gets flashcards that are due for review.
    pub fn get_due_flashcards(&self, limit: usize) -> Result<Vec<Flashcard>> {
        let conn = self.conn();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        let mut stmt = conn.prepare(
            "SELECT * FROM flashcards 
             WHERE next_review_at <= ?1 
             ORDER BY next_review_at ASC 
             LIMIT ?2"
        )?;
        
        let card_iter = stmt.query_map(params![now, limit], row_to_flashcard)?;
        
        let mut cards = Vec::new();
        for card in card_iter {
            cards.push(card?);
        }
        Ok(cards)
    }

    /// Updates the scheduling/SM-2 parameters for a flashcard.
    pub fn update_flashcard_schedule(&self, card: &Flashcard) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "UPDATE flashcards SET 
                easiness_factor = ?2, 
                interval_days = ?3, 
                repetitions = ?4, 
                next_review_at = ?5, 
                last_reviewed_at = ?6 
             WHERE id = ?1",
            params![
                card.id,
                card.easiness_factor,
                card.interval_days,
                card.repetitions,
                card.next_review_at,
                card.last_reviewed_at,
            ],
        )
        .context("Failed to update flashcard schedule")?;
        Ok(())
    }

    /// Deletes a flashcard by ID.
    #[allow(dead_code)]
    pub fn delete_flashcard(&self, id: &str) -> Result<()> {
        let conn = self.conn();
        conn.execute("DELETE FROM flashcards WHERE id = ?1", params![id])
            .context("Failed to delete flashcard")?;
        Ok(())
    }

    /// Counts how many flashcards are currently due.
    pub fn count_due_today(&self) -> Result<i64> {
        let conn = self.conn();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM flashcards WHERE next_review_at <= ?1",
            params![now],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Inserts a log of a completed review session.
    pub fn insert_review_session(&self, session: &ReviewSession) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO review_sessions (
                id, date, cards_reviewed, cards_correct, duration_seconds, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                session.id,
                session.date,
                session.cards_reviewed,
                session.cards_correct,
                session.duration_seconds,
                session.created_at,
            ],
        )
        .context("Failed to insert review session log")?;
        Ok(())
    }
}

/// Helper to map database rows to a Flashcard struct.
fn row_to_flashcard(row: &rusqlite::Row) -> rusqlite::Result<Flashcard> {
    let card_type_str: String = row.get("card_type")?;
    Ok(Flashcard {
        id: row.get("id")?,
        article_id: row.get("article_id")?,
        highlight_id: row.get("highlight_id")?,
        front: row.get("front")?,
        back: row.get("back")?,
        card_type: CardType::from_str(&card_type_str),
        easiness_factor: row.get("easiness_factor")?,
        interval_days: row.get("interval_days")?,
        repetitions: row.get("repetitions")?,
        next_review_at: row.get("next_review_at")?,
        last_reviewed_at: row.get("last_reviewed_at")?,
        created_at: row.get("created_at")?,
    })
}

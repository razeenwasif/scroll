//! SM-2 spaced repetition algorithm implementation.

use crate::models::Flashcard;

/// Updates a flashcard's scheduling parameters using the SM-2 algorithm.
///
/// `quality` is the user's score from 0 (forgot completely) to 5 (perfect recall).
pub fn sm2_review(card: &mut Flashcard, quality: u8) {
    let now = chrono::Utc::now();
    let q = quality.min(5) as f64;

    // 1. Calculate next interval and repetitions
    if quality >= 3 {
        // Correct response
        match card.repetitions {
            0 => card.interval_days = 1,
            1 => card.interval_days = 6,
            _ => {
                let prev_interval = card.interval_days as f64;
                card.interval_days = (prev_interval * card.easiness_factor).round() as i64;
            }
        }
        card.repetitions += 1;
    } else {
        // Incorrect response - reset repetitions and interval
        card.repetitions = 0;
        card.interval_days = 1;
    }

    // 2. Calculate new easiness factor (EF)
    // Formula: EF' = EF + (0.1 - (5 - q) * (0.08 + (5 - q) * 0.02))
    let new_ef = card.easiness_factor + (0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02));
    card.easiness_factor = new_ef.max(1.3);

    // 3. Set review dates
    let next_review = now + chrono::Duration::days(card.interval_days);
    card.next_review_at = next_review.format("%Y-%m-%d %H:%M:%S").to_string();
    card.last_reviewed_at = Some(now.format("%Y-%m-%d %H:%M:%S").to_string());
}

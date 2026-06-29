//! Core parsing, scraping, and calculation logic.

mod scraper;
mod sm2;
mod markdown;

pub use scraper::scrape_url;
pub use sm2::sm2_review;
pub use markdown::wrap_text;

//! Storage module for Scroll database persistence.

mod db;
mod articles;
mod highlights;
mod flashcards;
mod tags;
mod search;

pub use db::Database;

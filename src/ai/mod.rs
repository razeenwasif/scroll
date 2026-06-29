//! AI features powered by local Ollama instances.

mod client;
mod summarize;
mod flashcards;
mod chat;

pub use client::OllamaClient;
pub use summarize::summarize_article;
pub use flashcards::generate_flashcards;

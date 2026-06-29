//! Article summarization using local LLM.

use anyhow::Result;
use crate::models::Article;
use super::client::OllamaClient;

/// Generates a concise 3-sentence summary of the article's content.
pub async fn summarize_article(client: &OllamaClient, article: &Article) -> Result<String> {
    // Truncate content to avoid exceeding context window (about 4000 characters)
    let content_chars: Vec<char> = article.content_markdown.chars().collect();
    let truncated_content: String = if content_chars.len() > 4000 {
        content_chars[..4000].iter().collect()
    } else {
        article.content_markdown.clone()
    };

    let system_prompt = "You are a concise summarizer. Provide exactly 3 sentences summarizing the key points of the text.";
    let user_prompt = format!(
        "Title: {}\n\nContent:\n{}",
        article.title,
        truncated_content
    );

    let summary = client.generate(&user_prompt, Some(system_prompt)).await?;
    Ok(summary.trim().to_string())
}

//! Interactive Q&A chat with an article using local LLM.

use anyhow::Result;
use crate::models::Article;
use super::client::OllamaClient;

/// Ask a question about the article content.
#[allow(dead_code)]
pub async fn ask_article(
    client: &OllamaClient,
    article: &Article,
    question: &str,
) -> Result<String> {
    // Truncate content context if it is extremely long
    let content_chars: Vec<char> = article.content_markdown.chars().collect();
    let truncated_content: String = if content_chars.len() > 6000 {
        content_chars[..6000].iter().collect()
    } else {
        article.content_markdown.clone()
    };

    let system_prompt = format!(
        "You are an assistant answering questions about the following article.\n\n\
        Title: {}\n\
        Author: {}\n\
        Site: {}\n\n\
        Content:\n{}\n\n\
        Answer the user's question concisely based ONLY on the provided article context. If the answer cannot be found in the article, politely state that you do not know.",
        article.title,
        article.author.as_deref().unwrap_or("Unknown"),
        article.site_name.as_deref().unwrap_or("Unknown"),
        truncated_content
    );

    let answer = client.generate(question, Some(&system_prompt)).await?;
    Ok(answer.trim().to_string())
}

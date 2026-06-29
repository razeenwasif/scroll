//! Spaced repetition flashcard generation using local LLM.

use anyhow::Result;
use super::client::OllamaClient;

/// Generates 2 to 3 flashcard Q&A pairs from highlighted text and its surrounding context.
pub async fn generate_flashcards(
    client: &OllamaClient,
    text: &str,
    context: &str,
) -> Result<Vec<(String, String)>> {
    let system_prompt = "You are a learning assistant. Generate 2 to 3 high-quality flashcard Q&A pairs based on the text highlight and its surrounding context. \
    Output the cards using the exact format shown below, with no other text, introduction, or formatting:\n\
    Q: [Question here]\n\
    A: [Answer here]\n\
    Q: [Another question here]\n\
    A: [Another answer here]";

    let user_prompt = format!(
        "Highlight:\n\"{}\"\n\nContext:\n\"{}\"",
        text,
        context
    );

    let response = client.generate(&user_prompt, Some(system_prompt)).await?;
    let qa_pairs = parse_qa_pairs(&response);
    
    Ok(qa_pairs)
}

/// Parses Q&A pairs from the LLM's text output.
fn parse_qa_pairs(response: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let mut current_question = String::new();

    for line in response.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("Q:") || trimmed.starts_with("q:") {
            current_question = trimmed[2..].trim().to_string();
        } else if (trimmed.starts_with("A:") || trimmed.starts_with("a:")) && !current_question.is_empty() {
            let answer = trimmed[2..].trim().to_string();
            pairs.push((current_question.clone(), answer));
            current_question.clear();
        }
    }

    pairs
}

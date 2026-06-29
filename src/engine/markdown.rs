//! Markdown and text utilities for rendering and processing.

/// Wraps text to a given width while preserving original paragraph breaks and empty lines.
pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut wrapped_lines = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            wrapped_lines.push(String::new());
        } else {
            let lines = textwrap::wrap(line, width);
            for l in lines {
                wrapped_lines.push(l.into_owned());
            }
        }
    }
    wrapped_lines
}

/// Truncates a string to a maximum number of characters, appending an ellipsis if truncated.
/// This function is unicode-safe.
#[allow(dead_code)]
pub fn truncate_text(text: &str, max_len: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_len {
        text.to_string()
    } else {
        let end_idx = max_len.saturating_sub(1);
        let truncated: String = chars[..end_idx].iter().collect();
        format!("{}…", truncated)
    }
}

/// Extracts header lines from a markdown text to generate a table of contents.
/// Returns a list of tuples containing (header_level, header_text).
#[allow(dead_code)]
pub fn extract_headings(markdown: &str) -> Vec<(usize, String)> {
    let mut headings = Vec::new();
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let hashes = parts[0];
                if hashes.chars().all(|c| c == '#') {
                    let level = hashes.len();
                    let text = parts[1].trim().to_string();
                    if !text.is_empty() && level <= 6 {
                        headings.push((level, text));
                    }
                }
            }
        }
    }
    headings
}

/// Estimates reading time in minutes based on average 250 WPM.
#[allow(dead_code)]
pub fn estimate_reading_time(word_count: i64) -> i64 {
    (word_count / 250).max(1)
}

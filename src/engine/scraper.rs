//! Web scraping pipeline to extract article content.

use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use scraper::{Html, Selector};
use url::Url;
use crate::models::{Article, SourceType};

/// Scrapes article metadata and content from the given URL.
pub async fn scrape_url(url_str: &str) -> Result<Article> {
    // 1. Validate URL
    let url = Url::parse(url_str).map_err(|e| anyhow!("Invalid URL: {}", e))?;

    // 2. Fetch page with reqwest
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let html_content = client.get(url.as_str())
        .send()
        .await?
        .text()
        .await?;

    // 3. Parse HTML
    let document = Html::parse_document(&html_content);

    // 4. Extract metadata
    let og_title_sel = Selector::parse("meta[property=\"og:title\"]").unwrap();
    let title_sel = Selector::parse("title").unwrap();
    let h1_sel = Selector::parse("h1").unwrap();

    let mut title = String::new();
    if let Some(el) = document.select(&og_title_sel).next() {
        title = el.value().attr("content").unwrap_or("").to_string();
    }
    if title.is_empty() {
        if let Some(el) = document.select(&title_sel).next() {
            title = el.text().collect::<Vec<_>>().join(" ");
        }
    }
    if title.is_empty() {
        if let Some(el) = document.select(&h1_sel).next() {
            title = el.text().collect::<Vec<_>>().join(" ");
        }
    }
    title = title.trim().to_string();
    if title.is_empty() {
        title = "Untitled Article".to_string();
    }

    let author_sel = Selector::parse("meta[name=\"author\"]").unwrap();
    let art_author_sel = Selector::parse("meta[property=\"article:author\"]").unwrap();
    let mut author = None;
    if let Some(el) = document.select(&author_sel).next() {
        let auth = el.value().attr("content").unwrap_or("").trim().to_string();
        if !auth.is_empty() {
            author = Some(auth);
        }
    }
    if author.is_none() {
        if let Some(el) = document.select(&art_author_sel).next() {
            let auth = el.value().attr("content").unwrap_or("").trim().to_string();
            if !auth.is_empty() {
                author = Some(auth);
            }
        }
    }

    let site_sel = Selector::parse("meta[property=\"og:site_name\"]").unwrap();
    let mut site_name = None;
    if let Some(el) = document.select(&site_sel).next() {
        let site = el.value().attr("content").unwrap_or("").trim().to_string();
        if !site.is_empty() {
            site_name = Some(site);
        }
    }
    if site_name.is_none() {
        site_name = url.host_str().map(|h| h.to_string());
    }

    let desc_sel = Selector::parse("meta[name=\"description\"]").unwrap();
    let og_desc_sel = Selector::parse("meta[property=\"og:description\"]").unwrap();
    let mut excerpt = None;
    if let Some(el) = document.select(&desc_sel).next() {
        let desc = el.value().attr("content").unwrap_or("").trim().to_string();
        if !desc.is_empty() {
            excerpt = Some(desc);
        }
    }
    if excerpt.is_none() {
        if let Some(el) = document.select(&og_desc_sel).next() {
            let desc = el.value().attr("content").unwrap_or("").trim().to_string();
            if !desc.is_empty() {
                excerpt = Some(desc);
            }
        }
    }

    let img_sel = Selector::parse("meta[property=\"og:image\"]").unwrap();
    let mut cover_image_url = None;
    if let Some(el) = document.select(&img_sel).next() {
        let img = el.value().attr("content").unwrap_or("").trim().to_string();
        if !img.is_empty() {
            cover_image_url = Some(img);
        }
    }

    // 5. Find main content element
    let content_selectors = &[
        "article", "main", "[role=\"main\"]", ".post-content", 
        ".article-content", ".entry-content", "body"
    ];
    let mut content_elem = None;
    for sel_str in content_selectors {
        if let Ok(selector) = Selector::parse(sel_str) {
            for el in document.select(&selector) {
                let text_len = el.text().collect::<Vec<_>>().join(" ").len();
                if text_len > 100 {
                    content_elem = Some(el);
                    break;
                }
            }
        }
        if content_elem.is_some() {
            break;
        }
    }

    let content_node = content_elem
        .ok_or_else(|| anyhow!("Could not find main article body content"))?;

    // 6 & 7. Convert HTML element and its children recursively to markdown, skipping boilerplates
    let mut raw_markdown = String::new();
    node_to_markdown(*content_node, &mut raw_markdown);

    let final_markdown = post_process_markdown(&raw_markdown);

    // 8. Create Article struct
    let mut article = Article::new(title, final_markdown, SourceType::Web);
    article.url = Some(url_str.to_string());
    article.author = author;
    article.site_name = site_name;
    article.excerpt = excerpt;
    article.cover_image_url = cover_image_url;

    // Use full HTML string as option if needed
    article.content_html = Some(content_node.html());

    Ok(article)
}

/// Recursively converts an HTML node tree to Markdown formatting.
fn node_to_markdown(node: ego_tree::NodeRef<'_, scraper::node::Node>, markdown: &mut String) {
    use scraper::node::Node;
    match node.value() {
        Node::Text(text) => {
            markdown.push_str(&text);
        }
        Node::Element(elem) => {
            let name = elem.name();
            
            // Check for boilerplates in ID or classes to exclude early
            let class = elem.attr("class").unwrap_or("").to_lowercase();
            let id = elem.attr("id").unwrap_or("").to_lowercase();
            let is_ignored_class_or_id = [
                "nav", "footer", "sidebar", "menu", "comment", "ad", 
                "social", "share", "related", "banner", "cookie", "widget"
            ]
            .iter()
            .any(|&term| class.contains(term) || id.contains(term));

            if is_ignored_class_or_id {
                return;
            }

            match name {
                "script" | "style" | "nav" | "footer" | "header" | "aside" | "form" | "iframe" | "noscript" => {
                    // Exclude styling/nav/functional elements
                }
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    let level = name.chars().nth(1).and_then(|c| c.to_digit(10)).unwrap_or(1) as usize;
                    markdown.push_str("\n\n");
                    markdown.push_str(&"#".repeat(level));
                    markdown.push_str(" ");
                    for child in node.children() {
                        node_to_markdown(child, markdown);
                    }
                    markdown.push_str("\n\n");
                }
                "p" => {
                    markdown.push_str("\n\n");
                    for child in node.children() {
                        node_to_markdown(child, markdown);
                    }
                    markdown.push_str("\n\n");
                }
                "strong" | "b" => {
                    markdown.push_str("**");
                    for child in node.children() {
                        node_to_markdown(child, markdown);
                    }
                    markdown.push_str("**");
                }
                "em" | "i" => {
                    markdown.push_str("*");
                    for child in node.children() {
                        node_to_markdown(child, markdown);
                    }
                    markdown.push_str("*");
                }
                "code" => {
                    markdown.push_str(" `");
                    for child in node.children() {
                        node_to_markdown(child, markdown);
                    }
                    markdown.push_str("` ");
                }
                "pre" => {
                    markdown.push_str("\n\n```\n");
                    for child in node.children() {
                        node_to_markdown(child, markdown);
                    }
                    markdown.push_str("\n```\n\n");
                }
                "a" => {
                    let href = elem.attr("href").unwrap_or("");
                    // Only render if there's actual link text
                    let mut link_text = String::new();
                    for child in node.children() {
                        node_to_markdown(child, &mut link_text);
                    }
                    let link_text = link_text.trim();
                    if !link_text.is_empty() {
                        if href.starts_with("http") || href.starts_with('/') {
                            markdown.push_str(&format!(" [{}]({}) ", link_text, href));
                        } else {
                            markdown.push_str(link_text);
                        }
                    }
                }
                "blockquote" => {
                    markdown.push_str("\n\n> ");
                    let mut inner = String::new();
                    for child in node.children() {
                        node_to_markdown(child, &mut inner);
                    }
                    markdown.push_str(&inner.trim().replace('\n', "\n> "));
                    markdown.push_str("\n\n");
                }
                "ul" | "ol" => {
                    markdown.push_str("\n");
                    for child in node.children() {
                        node_to_markdown(child, markdown);
                    }
                    markdown.push_str("\n");
                }
                "li" => {
                    markdown.push_str("\n- ");
                    for child in node.children() {
                        node_to_markdown(child, markdown);
                    }
                }
                "img" => {
                    let src = elem.attr("src").unwrap_or("");
                    let alt = elem.attr("alt").unwrap_or("Image");
                    if !src.is_empty() {
                        markdown.push_str(&format!("\n\n![{}]({})\n\n", alt, src));
                    }
                }
                "br" => {
                    markdown.push_str("\n");
                }
                _ => {
                    for child in node.children() {
                        node_to_markdown(child, markdown);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Helper to cleanup excessive newlines and whitespace from generated markdown.
fn post_process_markdown(md: &str) -> String {
    let mut result = String::new();
    let mut consecutive_newlines = 0;
    
    for c in md.chars() {
        if c == '\n' {
            consecutive_newlines += 1;
            if consecutive_newlines <= 2 {
                result.push(c);
            }
        } else {
            consecutive_newlines = 0;
            result.push(c);
        }
    }
    
    // Final clean up of line spacings
    let mut lines = Vec::new();
    for line in result.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            lines.push("");
        } else {
            lines.push(line); // Preserve leading indentation for things like lists if desired
        }
    }
    
    lines.join("\n").trim().to_string()
}

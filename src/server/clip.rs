//! Web clipper page and API routes.

use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
    Form, Json,
};
use serde::Deserialize;
use crate::storage::Database;
use crate::models::{Article, Tag};

/// Structure for clipping form submission.
#[derive(Deserialize)]
pub struct ClipForm {
    url: String,
    tags: Option<String>,
}

/// Serve the clipper form HTML.
pub async fn clip_form() -> Html<String> {
    Html(get_clipper_page(None))
}

/// Handles URL submission for clipping.
pub async fn clip_submit(
    State(db): State<Database>,
    Form(data): Form<ClipForm>,
) -> Html<String> {
    let url = data.url.trim();
    if url.is_empty() {
        return Html(get_clipper_page(Some(Err("URL cannot be empty".to_string()))));
    }

    match crate::engine::scrape_url(url).await {
        Ok(article) => {
            let article_id = article.id.clone();
            let title = article.title.clone();
            
            // Insert article
            if let Err(e) = db.insert_article(&article) {
                return Html(get_clipper_page(Some(Err(format!("Database insert failed: {}", e)))));
            }

            // Associate tags
            if let Some(tags_str) = &data.tags {
                for t_name in tags_str.split(',') {
                    let name = t_name.trim();
                    if !name.is_empty() {
                        match get_or_create_tag(&db, name) {
                            Ok(tag_id) => {
                                let _ = db.tag_article(&article_id, &tag_id);
                            }
                            Err(e) => {
                                eprintln!("Failed to process tag '{}': {}", name, e);
                            }
                        }
                    }
                }
            }

            Html(get_clipper_page(Some(Ok(title))))
        }
        Err(e) => Html(get_clipper_page(Some(Err(format!("Scraping failed: {}", e))))),
    }
}

/// API to list articles.
pub async fn list_articles_api(State(db): State<Database>) -> Result<Json<Vec<Article>>, StatusCodeError> {
    match db.list_articles(&crate::models::ArticleFilter::default()) {
        Ok(articles) => Ok(Json(articles)),
        Err(e) => Err(StatusCodeError(axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// API to get a single article by ID.
pub async fn get_article_api(
    State(db): State<Database>,
    Path(id): Path<String>,
) -> Result<Json<Article>, StatusCodeError> {
    match db.get_article(&id) {
        Ok(Some(article)) => Ok(Json(article)),
        Ok(None) => Err(StatusCodeError(axum::http::StatusCode::NOT_FOUND, "Article not found".to_string())),
        Err(e) => Err(StatusCodeError(axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// Helper structure to map server errors to status codes.
pub struct StatusCodeError(axum::http::StatusCode, String);

impl IntoResponse for StatusCodeError {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}

/// Helper to get or insert a tag.
fn get_or_create_tag(db: &Database, name: &str) -> anyhow::Result<String> {
    let name_trimmed = name.trim();
    if name_trimmed.is_empty() {
        return Err(anyhow::anyhow!("Tag name is empty"));
    }

    // Check if tag exists
    {
        let conn = db.conn();
        let mut stmt = conn.prepare("SELECT id FROM tags WHERE name = ?1")?;
        let mut rows = stmt.query(rusqlite::params![name_trimmed])?;
        if let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            return Ok(id);
        }
    } // Release conn lock before calling insert_tag to prevent deadlock

    let tag = Tag::new(name_trimmed.to_string(), "blue".to_string());
    db.insert_tag(&tag)?;
    Ok(tag.id)
}

/// Generates the dark-themed web clipper HTML template.
fn get_clipper_page(status: Option<Result<String, String>>) -> String {
    let status_html = match status {
        Some(Ok(title)) => format!(
            "<div class=\"status-message success\"><strong>Success!</strong> Clipped article: <br>\"{}\"</div>",
            title
        ),
        Some(Err(err)) => format!(
            "<div class=\"status-message error\"><strong>Error:</strong> {}</div>",
            err
        ),
        None => String::new(),
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Scroll • Web Clipper</title>
    <style>
        body {{
            background-color: #0f0f19;
            color: #e2e8f0;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            padding: 20px;
            box-sizing: border-box;
        }}
        .card {{
            background: rgba(20, 20, 35, 0.85);
            border: 1px solid rgba(0, 212, 255, 0.2);
            border-radius: 12px;
            padding: 36px;
            width: 100%;
            max-width: 480px;
            box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.5);
            backdrop-filter: blur(8px);
            -webkit-backdrop-filter: blur(8px);
        }}
        h1 {{
            color: #00d4ff;
            font-size: 26px;
            margin-top: 0;
            margin-bottom: 8px;
            text-align: center;
            font-weight: 700;
            letter-spacing: 0.5px;
        }}
        .subtitle {{
            text-align: center;
            color: #64748b;
            font-size: 14px;
            margin-bottom: 32px;
        }}
        .form-group {{
            margin-bottom: 24px;
        }}
        label {{
            display: block;
            margin-bottom: 8px;
            font-size: 13px;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            color: #94a3b8;
        }}
        input[type="text"] {{
            width: 100%;
            padding: 12px 16px;
            background: #141424;
            border: 1px solid #2d2d44;
            border-radius: 6px;
            color: #fff;
            font-size: 15px;
            box-sizing: border-box;
            transition: border-color 0.2s, box-shadow 0.2s;
        }}
        input[type="text"]:focus {{
            outline: none;
            border-color: #00d4ff;
            box-shadow: 0 0 0 2px rgba(0, 212, 255, 0.15);
        }}
        button {{
            width: 100%;
            padding: 14px;
            background: linear-gradient(135deg, #0052d4 0%, #4364f7 50%, #6fb1fc 100%);
            border: none;
            border-radius: 6px;
            color: #fff;
            font-size: 16px;
            font-weight: 600;
            cursor: pointer;
            transition: opacity 0.2s, transform 0.1s;
        }}
        button:hover {{
            opacity: 0.95;
        }}
        button:active {{
            transform: scale(0.98);
        }}
        .status-message {{
            margin-top: 24px;
            padding: 16px;
            border-radius: 6px;
            font-size: 14px;
            line-height: 1.5;
        }}
        .success {{
            background: rgba(16, 185, 129, 0.15);
            border: 1px solid #10b981;
            color: #34d399;
        }}
        .error {{
            background: rgba(239, 68, 68, 0.15);
            border: 1px solid #ef4444;
            color: #f87171;
        }}
    </style>
</head>
<body>
    <div class="card">
        <h1>Scroll</h1>
        <div class="subtitle">Web Clipper & Spaced Repetition Reading Hub</div>
        <form action="/clip" method="POST">
            <div class="form-group">
                <label for="url">URL to Clip</label>
                <input type="text" id="url" name="url" placeholder="https://example.com/article" required autocomplete="off">
            </div>
            <div class="form-group">
                <label for="tags">Tags (comma-separated, optional)</label>
                <input type="text" id="tags" name="tags" placeholder="reading, developer, tech" autocomplete="off">
            </div>
            <button type="submit">Clip Article</button>
        </form>
        {}
    </div>
</body>
</html>"#,
        status_html
    )
}

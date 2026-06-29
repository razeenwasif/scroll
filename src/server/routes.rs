//! Axum router configuration and server startup.

use axum::{
    routing::get,
    Router,
    response::Redirect,
};
use crate::storage::Database;

/// Starts the embedded Axum web server on the specified port.
pub async fn start_server(db: Database, port: u16) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(|| async { Redirect::to("/clip") }))
        .route("/clip", get(super::clip::clip_form).post(super::clip::clip_submit))
        .route("/api/articles", get(super::clip::list_articles_api))
        .route("/api/articles/:id", get(super::clip::get_article_api))
        .with_state(db);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    // Server starts in background or as command.
    // Logging will help debug.
    axum::serve(listener, app).await?;
    Ok(())
}

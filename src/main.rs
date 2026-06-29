//! Main entry point for the Scroll application.

mod app;
mod config;
mod models;
mod storage;
mod engine;
mod ai;
mod server;
mod ui;

use clap::{Parser, Subcommand};
use anyhow::Result;

use config::ScrollConfig;
use storage::Database;

#[derive(Parser)]
#[command(name = "scroll")]
#[command(version = "0.1.0")]
#[command(about = "A self-hosted reading hub, web clipper, and spaced-repetition engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Clip a URL and exit
    Clip {
        /// The URL of the web page to scrape
        url: String,
        
        /// Optional comma-separated list of tags to apply
        #[arg(long)]
        tags: Option<String>,
    },
    
    /// Launch directly into spaced-repetition review session
    Review,
    
    /// Run the web clipper background server only (no TUI)
    Serve,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Parse CLI args
    let cli = Cli::parse();

    // 2. Load config
    let config = ScrollConfig::load()?;

    // 3. Open database connection and run schema migrations
    let db_path = ScrollConfig::db_path()?;
    let db = Database::new(&db_path)?;

    // 4. Match subcommands
    match cli.command {
        Some(Commands::Clip { url, tags }) => {
            println!("Clipping: {}", url);
            match engine::scrape_url(&url).await {
                Ok(article) => {
                    let article_id = article.id.clone();
                    db.insert_article(&article)?;
                    println!("Successfully clipped: \"{}\"", article.title);
                    
                    if let Some(tags_str) = tags {
                        for t in tags_str.split(',') {
                            let tag_name = t.trim();
                            if !tag_name.is_empty() {
                                let tag_id = get_or_create_tag_cli(&db, tag_name)?;
                                db.tag_article(&article_id, &tag_id)?;
                                println!("Added tag: {}", tag_name);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Scraping failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Some(Commands::Serve) => {
            let port = config.server.port;
            println!("Running background server only on port {}...", port);
            server::start_server(db, port).await?;
        }
        
        Some(Commands::Review) => {
            let mut app = app::App::new(db, config)?;
            app.mode = app::AppMode::Review;
            app.load_review_queue()?;
            app.run().await?;
        }
        
        None => {
            // Default run: Spawn background server if enabled, and launch TUI
            if config.server.enabled {
                let db_cloned = db.clone();
                let port = config.server.port;
                tokio::spawn(async move {
                    if let Err(e) = server::start_server(db_cloned, port).await {
                        eprintln!("Background clipper server failed to start: {}", e);
                    }
                });
            }

            let mut app = app::App::new(db, config)?;
            app.run().await?;
        }
    }

    Ok(())
}

/// CLI Helper to fetch or create a tag.
fn get_or_create_tag_cli(db: &Database, name: &str) -> anyhow::Result<String> {
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
    } // Release conn lock

    let tag = models::Tag::new(name_trimmed.to_string(), "blue".to_string());
    db.insert_tag(&tag)?;
    Ok(tag.id)
}

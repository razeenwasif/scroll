//! Database connection management and migrations.

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use anyhow::{Context, Result};
use rusqlite::Connection;

/// A thread-safe, cloneable wrapper around a SQLite database connection.
#[derive(Debug, Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Opens a connection to the SQLite database at the given path and runs migrations.
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open database at {:?}", path))?;
        
        // Enable WAL mode for concurrency and performance
        conn.pragma_update(None, "journal_mode", &"WAL")
            .context("Failed to enable WAL mode")?;
        
        // Enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON;", [])
            .context("Failed to enable foreign key constraints")?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.run_migrations().context("Database migration failed")?;

        Ok(db)
    }

    /// Runs the initial database schema migration.
    pub fn run_migrations(&self) -> Result<()> {
        let schema_sql = include_str!("../../migrations/001_initial.sql");
        let conn = self.conn();
        
        // Execute the entire schema script
        conn.execute_batch(schema_sql)
            .context("Failed to execute database schema migrations")?;
            
        // Check if category column exists in articles table, if not add it dynamically
        let has_category: Result<i64, _> = conn.query_row(
            "SELECT count(*) FROM pragma_table_info('articles') WHERE name='category'",
            [],
            |r| r.get(0)
        );
        if let Ok(0) = has_category {
            let _ = conn.execute("ALTER TABLE articles ADD COLUMN category TEXT", []);
        }
            
        Ok(())
    }

    /// Obtains a guard to the inner SQLite connection.
    pub fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().expect("Database lock poisoned")
    }
}

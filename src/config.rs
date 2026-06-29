//! Configuration loading and management.
//!
//! Config file location: `~/.config/scroll/config.toml`
//! Database location: `~/.local/share/scroll/scroll.db`

use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Top-level configuration for Scroll.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScrollConfig {
    pub general: GeneralConfig,
    pub reader: ReaderConfig,
    pub theme: ThemeConfig,
    pub ai: AiConfig,
    pub review: ReviewConfig,
    pub server: ServerConfig,
    pub export: ExportConfig,
}

impl Default for ScrollConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            reader: ReaderConfig::default(),
            theme: ThemeConfig::default(),
            ai: AiConfig::default(),
            review: ReviewConfig::default(),
            server: ServerConfig::default(),
            export: ExportConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Default view on launch: "library" or "review".
    pub default_view: String,
    /// Number of articles per page in the library.
    pub items_per_page: usize,
    /// Date display format (strftime).
    pub date_format: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_view: "library".into(),
            items_per_page: 25,
            date_format: "%Y-%m-%d".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ReaderConfig {
    /// Maximum line width (characters) in the reader view.
    pub max_width: usize,
    /// Whether to show line numbers in the reader.
    pub show_line_numbers: bool,
    /// Lines to scroll per j/k keypress.
    pub scroll_speed: usize,
}

impl Default for ReaderConfig {
    fn default() -> Self {
        Self {
            max_width: 80,
            show_line_numbers: false,
            scroll_speed: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    /// Accent color name: cyan, green, purple, gold, rose.
    pub accent: String,
    /// Theme style: "dark" or "light".
    pub style: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            accent: "cyan".into(),
            style: "dark".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AiConfig {
    /// Whether AI features are enabled.
    pub enabled: bool,
    /// Ollama server URL.
    pub ollama_url: String,
    /// Model name to use for AI features.
    pub model: String,
    /// Auto-generate summary when an article is clipped.
    pub auto_summarize: bool,
    /// Auto-suggest tags when an article is clipped.
    pub auto_tag: bool,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ollama_url: "http://localhost:11434".into(),
            model: "gemma2".into(),
            auto_summarize: true,
            auto_tag: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ReviewConfig {
    /// Maximum flashcards to review per session.
    pub cards_per_day: usize,
    /// Maximum new cards to introduce per session.
    pub new_cards_per_day: usize,
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            cards_per_day: 20,
            new_cards_per_day: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Whether to start the embedded web server alongside the TUI.
    pub enabled: bool,
    /// Port for the embedded web server.
    pub port: u16,
    /// Whether to auto-open the browser for PDF/clip pages.
    pub open_browser: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 3131,
            open_browser: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportConfig {
    /// Path to an Onyx vault for highlight export (empty = disabled).
    pub onyx_vault_path: String,
    /// Export format: "markdown" or "json".
    pub format: String,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            onyx_vault_path: String::new(),
            format: "markdown".into(),
        }
    }
}

impl ScrollConfig {
    /// Load config from `~/.config/scroll/config.toml`.
    /// Creates a default config file if one doesn't exist.
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Self = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save the current config to disk.
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    /// Returns the path to the config file.
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        Ok(config_dir.join("scroll").join("config.toml"))
    }

    /// Returns the path to the SQLite database file.
    /// Creates the parent directory if needed.
    pub fn db_path() -> Result<PathBuf> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?;
        let scroll_dir = data_dir.join("scroll");
        std::fs::create_dir_all(&scroll_dir)?;
        Ok(scroll_dir.join("scroll.db"))
    }
}

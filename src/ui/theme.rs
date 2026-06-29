//! Theme and color settings for the ratatui interface.

use ratatui::style::Color;
use crate::config::ThemeConfig;

/// A collection of styles and colors representing a UI theme.
#[derive(Debug, Clone)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub accent_dim: Color,
    pub border: Color,
    pub muted: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub status_bg: Color,
    #[allow(dead_code)]
    pub status_fg: Color,
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    // Highlight colors
    pub highlight_yellow: Color,
    pub highlight_green: Color,
    pub highlight_blue: Color,
    pub highlight_red: Color,
    pub highlight_purple: Color,
}

impl Theme {
    /// Generates a theme based on the configuration.
    pub fn get(config: &ThemeConfig) -> Self {
        Self::dark(&config.accent)
    }

    /// Creates a dark theme with the specified accent color.
    pub fn dark(accent_name: &str) -> Self {
        // Rich dark background (RGB: 13, 13, 23)
        let bg = Color::Rgb(13, 13, 23);
        let fg = Color::Rgb(226, 232, 240); // slate-200

        // Resolve accent colors
        let (accent, accent_dim) = match accent_name.to_lowercase().as_str() {
            "green" => (Color::Rgb(0, 255, 136), Color::Rgb(0, 150, 80)),
            "purple" => (Color::Rgb(179, 136, 255), Color::Rgb(110, 80, 180)),
            "gold" => (Color::Rgb(255, 215, 0), Color::Rgb(180, 150, 0)),
            "rose" => (Color::Rgb(255, 107, 157), Color::Rgb(180, 70, 100)),
            _ => (Color::Rgb(0, 212, 255), Color::Rgb(0, 128, 160)), // cyan default
        };

        Self {
            bg,
            fg,
            accent,
            accent_dim,
            border: Color::Rgb(45, 45, 68),
            muted: Color::Rgb(100, 116, 139), // slate-500
            selection_bg: accent_dim,
            selection_fg: Color::Rgb(255, 255, 255),
            status_bg: Color::Rgb(30, 30, 46),
            status_fg: fg,
            success: Color::Rgb(16, 185, 129),
            error: Color::Rgb(239, 68, 68),
            warning: Color::Rgb(245, 158, 11),
            // Highlight styling colors
            highlight_yellow: Color::Rgb(250, 204, 21),
            highlight_green: Color::Rgb(74, 222, 128),
            highlight_blue: Color::Rgb(96, 165, 250),
            highlight_red: Color::Rgb(248, 113, 113),
            highlight_purple: Color::Rgb(192, 132, 252),
        }
    }
}

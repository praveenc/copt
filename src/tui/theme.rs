//! Theme definitions for the TUI
//!
//! Single theme designed to work well on both dark and light terminals.

use ratatui::style::{Color, Modifier, Style};

/// Application theme with consistent styling
#[derive(Debug, Clone)]
pub struct Theme {
    /// Primary accent color (cyan) - headers, highlights
    pub primary: Style,
    /// Secondary color (blue) - secondary text
    pub secondary: Style,
    /// Success color (green) - positive results, checkmarks
    pub success: Style,
    /// Warning color (yellow) - warnings, caution items
    pub warning: Style,
    /// Error color (red) - errors, critical issues
    pub error: Style,
    /// Muted color (gray) - less important info
    pub muted: Style,
    /// Default text color
    pub text: Style,
    /// Selected/highlighted item
    pub selected: Style,
    /// Border styling
    pub border: Style,
    /// Title styling
    pub title: Style,
    /// Key hint styling (for status bar)
    pub key_hint: Style,
    /// Key styling (for status bar)
    pub key: Style,
    /// Progress bar filled portion
    pub progress_filled: Style,
    /// Progress bar empty portion
    pub progress_empty: Style,
    /// Diff added lines
    pub diff_added: Style,
    /// Diff removed lines
    pub diff_removed: Style,
    /// Diff unchanged lines
    pub diff_unchanged: Style,
}

impl Theme {
    /// Create the default theme
    pub fn default() -> Self {
        Self {
            primary: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            secondary: Style::default().fg(Color::Blue),
            success: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            warning: Style::default().fg(Color::Yellow),
            error: Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
            muted: Style::default().fg(Color::DarkGray),
            text: Style::default().fg(Color::White),
            selected: Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            border: Style::default().fg(Color::DarkGray),
            title: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            key_hint: Style::default().fg(Color::DarkGray),
            key: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            progress_filled: Style::default().fg(Color::Green),
            progress_empty: Style::default().fg(Color::DarkGray),
            diff_added: Style::default().fg(Color::Green),
            diff_removed: Style::default().fg(Color::Red),
            diff_unchanged: Style::default().fg(Color::DarkGray),
        }
    }
}

impl std::default::Default for Theme {
    fn default() -> Self {
        Self::default()
    }
}

/// Global theme instance
pub fn theme() -> &'static Theme {
    use std::sync::OnceLock;
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(Theme::default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let theme = Theme::default();
        assert_eq!(theme.primary.fg, Some(Color::Cyan));
        assert_eq!(theme.error.fg, Some(Color::Red));
    }

    #[test]
    fn test_global_theme() {
        let t1 = theme();
        let t2 = theme();
        // Should return same reference
        assert!(std::ptr::eq(t1, t2));
    }
}

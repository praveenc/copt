//! Terminal User Interface module
//!
//! Provides a beautiful, interactive terminal UI using ratatui.
//! Implements the Elm (MVU) architecture for state management.
//!
//! # Architecture
//!
//! - `model.rs` - State definitions (Model)
//! - `update.rs` - Event handling (Update)
//! - `view.rs` - Rendering dispatch (View)
//! - `app.rs` - Main event loop
//! - `widgets/` - Individual UI components
//!
//! # Modes
//!
//! - **Interactive**: Full-screen ratatui UI with keyboard navigation
//! - **Linear**: Enhanced output that scrolls (default for TTY)
//! - **Plain**: Basic output for non-TTY (piped)
//! - **Json/Quiet**: Handled by main.rs, not this module

// New ratatui-based modules
pub mod app;
pub mod icons;
pub mod linear;
pub mod model;
pub mod terminal;
pub mod theme;
pub mod update;
pub mod view;
pub mod widgets;

// Snapshot tests
#[cfg(test)]
mod snapshot_tests;

// Legacy icon constants for backward compatibility with old modules
// The old modules (renderer.rs, stats.rs, diff.rs) use these
#[allow(dead_code)]
pub mod legacy_icons {
    pub const CHECK: &str = "✓";
    pub const CROSS: &str = "✗";
    pub const WARNING: &str = "⚠";
    pub const INFO: &str = "ℹ";
    pub const LIGHTNING: &str = "⚡";
    pub const INBOX: &str = "📥";
    pub const CHART: &str = "📊";
    pub const GEAR: &str = "⚙";
    pub const SPARKLES: &str = "✨";
    pub const FILE: &str = "📄";
}

// Legacy modules still used by main.rs:
// - renderer: start_optimizing_spinner, stop_optimizing_spinner, print_optimized_prompt
// - diff: print_diff
// - stats: print_save_success
pub mod diff;
pub mod renderer;
pub mod stats;

/// Box-drawing characters for terminal UI
#[allow(dead_code)]
pub mod chars {
    pub const TOP_LEFT: &str = "╭";
    pub const TOP_RIGHT: &str = "╮";
    pub const BOTTOM_LEFT: &str = "╰";
    pub const BOTTOM_RIGHT: &str = "╯";
    pub const HORIZONTAL: &str = "─";
    pub const VERTICAL: &str = "│";
    pub const T_DOWN: &str = "┬";
    pub const T_UP: &str = "┴";
    pub const T_RIGHT: &str = "├";
    pub const T_LEFT: &str = "┤";
    pub const CROSS: &str = "┼";
}

/// Terminal width utilities
pub fn terminal_width() -> usize {
    console::Term::stdout().size().1 as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_width() {
        assert!(terminal_width() > 0);
    }
}

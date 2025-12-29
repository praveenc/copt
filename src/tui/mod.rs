//! Terminal User Interface module
//!
//! Provides beautiful, colorful terminal output for the prompt optimizer.
//! Uses the `colored` and `console` crates for styling.

#![allow(dead_code)]

pub mod components;
pub mod diff;
pub mod renderer;
pub mod spinner;
pub mod stats;

// Re-export commonly used functions
pub use diff::print_diff;
pub use renderer::{print_analysis, print_header, print_input_info};
pub use stats::print_stats;

/// Box-drawing characters for terminal UI
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

/// Icon constants
pub mod icons {
    pub const CHECK: &str = "✓";
    pub const CROSS: &str = "✗";
    pub const WARNING: &str = "⚠";
    pub const INFO: &str = "ℹ";
    pub const LIGHTNING: &str = "⚡";
    pub const INBOX: &str = "📥";
    pub const CHART: &str = "📊";
    pub const GEAR: &str = "⚙";
    pub const ROCKET: &str = "🚀";
    pub const SPARKLES: &str = "✨";
    pub const PENCIL: &str = "📝";
    pub const FOLDER: &str = "📁";
    pub const FILE: &str = "📄";
    pub const CLOCK: &str = "⏱";
    pub const PLUG: &str = "🔌";
    pub const ROBOT: &str = "🤖";
}

/// Terminal width utilities
pub fn terminal_width() -> usize {
    console::Term::stdout().size().1 as usize
}

/// Truncate a string to fit within a width, adding ellipsis if needed
pub fn truncate(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else if max_width > 3 {
        format!("{}...", &s[..max_width - 3])
    } else {
        s[..max_width].to_string()
    }
}

/// Pad a string to a fixed width
pub fn pad_right(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s.len()))
    }
}

/// Center a string within a width
pub fn center(s: &str, width: usize) -> String {
    if s.len() >= width {
        s.to_string()
    } else {
        let padding = (width - s.len()) / 2;
        format!(
            "{}{}{}",
            " ".repeat(padding),
            s,
            " ".repeat(width - s.len() - padding)
        )
    }
}

/// Draw a horizontal line
pub fn draw_line(width: usize) -> String {
    chars::HORIZONTAL.repeat(width)
}

/// Draw a box top border
pub fn draw_box_top(width: usize, title: Option<&str>) -> String {
    match title {
        Some(t) => {
            let title_part = format!(" {} ", t);
            let remaining = width.saturating_sub(title_part.len() + 2);
            let left = remaining / 2;
            let right = remaining - left;
            format!(
                "{}{}{}{}{}",
                chars::TOP_LEFT,
                chars::HORIZONTAL.repeat(left),
                title_part,
                chars::HORIZONTAL.repeat(right),
                chars::TOP_RIGHT
            )
        }
        None => {
            format!(
                "{}{}{}",
                chars::TOP_LEFT,
                chars::HORIZONTAL.repeat(width),
                chars::TOP_RIGHT
            )
        }
    }
}

/// Draw a box bottom border
pub fn draw_box_bottom(width: usize) -> String {
    format!(
        "{}{}{}",
        chars::BOTTOM_LEFT,
        chars::HORIZONTAL.repeat(width),
        chars::BOTTOM_RIGHT
    )
}

/// Draw a box line (content between borders)
pub fn draw_box_line(content: &str, width: usize) -> String {
    let content_width = width.saturating_sub(2);
    let truncated = truncate(content, content_width);
    format!(
        "{} {}{} {}",
        chars::VERTICAL,
        truncated,
        " ".repeat(content_width.saturating_sub(truncated.len())),
        chars::VERTICAL
    )
}

/// Color utilities for consistent theming
pub mod colors {
    use colored::ColoredString;
    use colored::Colorize;

    pub fn primary(s: &str) -> ColoredString {
        s.cyan().bold()
    }

    pub fn secondary(s: &str) -> ColoredString {
        s.bright_blue()
    }

    pub fn success(s: &str) -> ColoredString {
        s.green().bold()
    }

    pub fn warning(s: &str) -> ColoredString {
        s.yellow()
    }

    pub fn error(s: &str) -> ColoredString {
        s.red().bold()
    }

    pub fn info(s: &str) -> ColoredString {
        s.blue()
    }

    pub fn muted(s: &str) -> ColoredString {
        s.bright_black()
    }

    pub fn highlight(s: &str) -> ColoredString {
        s.magenta().bold()
    }

    pub fn value(s: &str) -> ColoredString {
        s.white().bold()
    }

    pub fn label(s: &str) -> ColoredString {
        s.bright_white()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("hi", 2), "hi");
    }

    #[test]
    fn test_pad_right() {
        assert_eq!(pad_right("hello", 10), "hello     ");
        assert_eq!(pad_right("hello", 3), "hello");
    }

    #[test]
    fn test_center() {
        assert_eq!(center("hi", 6), "  hi  ");
        assert_eq!(center("hello", 3), "hello");
    }

    #[test]
    fn test_draw_box_top() {
        let top = draw_box_top(20, Some("Test"));
        assert!(top.starts_with(chars::TOP_LEFT));
        assert!(top.ends_with(chars::TOP_RIGHT));
        assert!(top.contains("Test"));
    }
}
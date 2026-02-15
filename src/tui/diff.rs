//! Diff visualization for prompt comparison
//!
//! Displays side-by-side diff view of original vs optimized prompts.

use colored::Colorize;
use similar::{ChangeTag, TextDiff};

use super::{chars, legacy_icons as icons, terminal_width};

/// Print a side-by-side diff of original and optimized prompts
pub fn print_diff(original: &str, optimized: &str) {
    let width = terminal_width().min(120);
    let half_width = (width - 3) / 2;

    println!();
    println!(
        "{}",
        format!(
            "{} {} Changes {}",
            chars::TOP_LEFT,
            icons::SPARKLES,
            chars::HORIZONTAL.repeat(width - 14)
        )
        .cyan()
    );

    // Headers
    let orig_header = format!("{} Original", icons::FILE);
    let opt_header = format!("{} Optimized", icons::SPARKLES);

    println!(
        "{} {}{} {} {}{}",
        chars::VERTICAL.cyan(),
        orig_header.bright_black(),
        " ".repeat(half_width.saturating_sub(orig_header.len() + 1)),
        chars::VERTICAL.bright_black(),
        opt_header.green(),
        " ".repeat(half_width.saturating_sub(opt_header.len()))
    );

    println!(
        "{}{}{}{}{}",
        chars::T_RIGHT.cyan(),
        chars::HORIZONTAL.repeat(half_width).bright_black(),
        chars::CROSS.bright_black(),
        chars::HORIZONTAL.repeat(half_width).bright_black(),
        chars::T_LEFT.cyan()
    );

    // Generate diff
    let diff = TextDiff::from_lines(original, optimized);

    for change in diff.iter_all_changes() {
        let content = change.value().trim_end();

        match change.tag() {
            ChangeTag::Delete => {
                let left = truncate_with_style(content, half_width - 2);
                println!(
                    "{} {}{} {} {}",
                    chars::VERTICAL.cyan(),
                    format!("- {}", left).red(),
                    " ".repeat(half_width.saturating_sub(left.len() + 3)),
                    chars::VERTICAL.bright_black(),
                    " ".repeat(half_width)
                );
            }
            ChangeTag::Insert => {
                let right = truncate_with_style(content, half_width - 2);
                println!(
                    "{} {}{} {} {}{}",
                    chars::VERTICAL.cyan(),
                    " ".repeat(half_width - 1),
                    chars::VERTICAL.bright_black(),
                    "+".green(),
                    right.green(),
                    " ".repeat(half_width.saturating_sub(right.len() + 3))
                );
            }
            ChangeTag::Equal => {
                let text = truncate_with_style(content, half_width - 2);
                println!(
                    "{} {}{} {} {}",
                    chars::VERTICAL.cyan(),
                    text.bright_black(),
                    " ".repeat(half_width.saturating_sub(text.len() + 1)),
                    chars::VERTICAL.bright_black(),
                    text.bright_black(),
                );
            }
        }
    }

    println!(
        "{}{}{}",
        chars::BOTTOM_LEFT.cyan(),
        chars::HORIZONTAL.repeat(width - 2).cyan(),
        chars::BOTTOM_RIGHT.cyan()
    );
    println!();
}

/// Truncate a string for display, respecting terminal width
fn truncate_with_style(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else if max_width > 3 {
        format!("{}...", &s[..max_width - 3])
    } else {
        s[..max_width].to_string()
    }
}

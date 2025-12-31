//! Diff visualization for prompt comparison
//!
//! Displays side-by-side or unified diff view of original vs optimized prompts.

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

/// Print a unified diff format
pub fn print_unified_diff(original: &str, optimized: &str) {
    let diff = TextDiff::from_lines(original, optimized);

    println!();
    println!("{}", "─".repeat(60).bright_black());
    println!("{}", "Unified Diff".cyan().bold());
    println!("{}", "─".repeat(60).bright_black());

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            println!("{}", "...".bright_black());
        }

        for op in group {
            for change in diff.iter_changes(op) {
                let (sign, line) = match change.tag() {
                    ChangeTag::Delete => ("-".red(), change.value().red()),
                    ChangeTag::Insert => ("+".green(), change.value().green()),
                    ChangeTag::Equal => (" ".normal(), change.value().normal()),
                };

                print!("{}{}", sign, line);
                if !change.value().ends_with('\n') {
                    println!();
                }
            }
        }
    }

    println!("{}", "─".repeat(60).bright_black());
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

/// Calculate diff statistics
pub fn diff_stats(original: &str, optimized: &str) -> DiffStats {
    let diff = TextDiff::from_lines(original, optimized);

    let mut added = 0;
    let mut removed = 0;
    let mut unchanged = 0;

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Insert => added += 1,
            ChangeTag::Delete => removed += 1,
            ChangeTag::Equal => unchanged += 1,
        }
    }

    DiffStats {
        added,
        removed,
        unchanged,
        similarity: diff.ratio(),
    }
}

/// Statistics about a diff
#[derive(Debug, Clone)]
pub struct DiffStats {
    pub added: usize,
    pub removed: usize,
    pub unchanged: usize,
    pub similarity: f32,
}

impl DiffStats {
    pub fn total_changes(&self) -> usize {
        self.added + self.removed
    }

    pub fn change_ratio(&self) -> f32 {
        let total = self.added + self.removed + self.unchanged;
        if total == 0 {
            0.0
        } else {
            self.total_changes() as f32 / total as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_stats() {
        let original = "line 1\nline 2\nline 3";
        let optimized = "line 1\nline 2 modified\nline 3\nline 4";

        let stats = diff_stats(original, optimized);
        assert!(stats.added > 0 || stats.removed > 0);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate_with_style("hello", 10), "hello");
        assert_eq!(truncate_with_style("hello world!", 8), "hello...");
    }

    #[test]
    fn test_identical_diff() {
        let text = "same text";
        let stats = diff_stats(text, text);
        assert_eq!(stats.added, 0);
        assert_eq!(stats.removed, 0);
        assert_eq!(stats.similarity, 1.0);
    }
}

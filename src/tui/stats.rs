//! Statistics display component for the TUI
//!
//! Legacy module — only print_save_success is still used by main.rs.

use colored::Colorize;

use super::legacy_icons as icons;

/// Print a success banner for saved output
pub fn print_save_success(path: &str, _is_dir: bool) {
    println!();
    println!("  {}", "─".repeat(70).bright_black());
    println!(
        "  {}  {} {}",
        icons::CHECK.green(),
        "Saved to:".green(),
        path.white().bold()
    );
    println!("  {}", "─".repeat(70).bright_black());
    println!();
}

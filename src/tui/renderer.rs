//! TUI Renderer module
//!
//! Provides terminal output functions still used by main.rs.

use colored::Colorize;

use super::legacy_icons as icons;

/// Create and return an optimization spinner with elapsed time
/// Returns a handle that can be used to stop the spinner
pub fn start_optimizing_spinner(model: &str) -> indicatif::ProgressBar {
    use indicatif::{ProgressBar, ProgressStyle};
    use std::time::Duration;

    let model_short = if model.len() > 40 {
        format!("{}...", &model[..37])
    } else {
        model.to_string()
    };

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("  {spinner:.cyan} {msg} [{elapsed_precise}]")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✓"]),
    );
    spinner.set_message(format!("Optimizing with {}...", model_short));
    spinner.enable_steady_tick(Duration::from_millis(80));

    spinner
}

/// Stop the optimization spinner with success
pub fn stop_optimizing_spinner(spinner: indicatif::ProgressBar) {
    spinner.finish_with_message("Optimization complete".to_string());
}

/// Print the optimized prompt
pub fn print_optimized_prompt(prompt: &str) {
    println!("  {}", "─".repeat(70).bright_black());
    println!(
        "  {}  {}",
        icons::SPARKLES.cyan(),
        "Optimized Prompt".white().bold()
    );
    println!("  {}", "─".repeat(70).bright_black());
    println!();

    // Print prompt content with indentation
    for line in prompt.lines() {
        if line.is_empty() {
            println!();
        } else {
            // Word wrap long lines
            let max_width = 72;
            let words: Vec<&str> = line.split_whitespace().collect();
            let mut current_line = String::new();

            for word in words {
                if current_line.is_empty() {
                    current_line = word.to_string();
                } else if current_line.len() + 1 + word.len() <= max_width {
                    current_line.push(' ');
                    current_line.push_str(word);
                } else {
                    println!("  {}", current_line);
                    current_line = word.to_string();
                }
            }

            if !current_line.is_empty() {
                println!("  {}", current_line);
            }
        }
    }

    println!();
}

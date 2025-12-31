//! TUI Renderer module
//!
//! Provides beautiful, colorful terminal output for the prompt optimizer.
//! Uses a clean, box-free design for reliable alignment across all terminals.

#![allow(dead_code)]

use colored::Colorize;
use std::path::PathBuf;

use super::legacy_icons as icons;
use crate::analyzer::{Issue, Severity};

/// Print the application header
pub fn print_header() {
    let version = env!("CARGO_PKG_VERSION");

    println!();
    println!(
        "  {}  {}",
        icons::LIGHTNING.cyan(),
        "CLAUDE PROMPT OPTIMIZER".cyan().bold()
    );
    println!(
        "     {}",
        format!("v{} • Optimize prompts for Claude 4.5", version).bright_black()
    );
    println!();
}

/// Print offline mode banner
pub fn print_offline_banner() {
    println!(
        "  {}  {}",
        "⚠".yellow(),
        "OFFLINE MODE - Using static analysis only (no LLM calls)"
            .yellow()
            .bold()
    );
    println!();
}

/// Print information about the input prompt
pub fn print_input_info(prompt: &str, file: &Option<PathBuf>) {
    let char_count = prompt.len();
    let token_count = crate::utils::count_tokens(prompt);

    let source = match file {
        Some(path) => format!("{}", path.display()),
        None => "stdin".to_string(),
    };

    println!(
        "  {}  {} {} ({} chars, {} tokens)",
        icons::INBOX.cyan(),
        "Input:".white().bold(),
        source.white(),
        char_count.to_string().cyan(),
        token_count.to_string().cyan()
    );
    println!();
}

/// Print analysis results showing detected issues
pub fn print_analysis(issues: &[Issue]) {
    // Section header
    println!("  {}", "─".repeat(70).bright_black());
    println!(
        "  {}  {}",
        icons::CHART.cyan(),
        "Analysis Results".white().bold()
    );
    println!("  {}", "─".repeat(70).bright_black());
    println!();

    if issues.is_empty() {
        println!(
            "  {}  {}",
            icons::CHECK.green(),
            "No issues detected - your prompt looks good!".green()
        );
        println!();
        return;
    }

    // Group issues by category
    let mut categories: std::collections::HashMap<String, Vec<&Issue>> =
        std::collections::HashMap::new();
    for issue in issues {
        categories
            .entry(issue.category.clone())
            .or_default()
            .push(issue);
    }

    // Count by severity
    let warnings = issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Warning))
        .count();
    let infos = issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Info))
        .count();
    let errors = issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Error))
        .count();

    // Summary line
    let mut summary_parts = vec![];
    if errors > 0 {
        summary_parts.push(
            format!("{} error{}", errors, if errors == 1 { "" } else { "s" })
                .red()
                .to_string(),
        );
    }
    if warnings > 0 {
        summary_parts.push(
            format!(
                "{} warning{}",
                warnings,
                if warnings == 1 { "" } else { "s" }
            )
            .yellow()
            .to_string(),
        );
    }
    if infos > 0 {
        summary_parts.push(format!("{} info", infos).blue().to_string());
    }

    println!(
        "  Found {} across {} {}",
        summary_parts.join(", "),
        categories.len(),
        if categories.len() == 1 {
            "category"
        } else {
            "categories"
        }
    );
    println!();

    // Print each category with aggregated issues
    for (category, cat_issues) in categories.iter() {
        let category_display = format_category_name(category);

        // Group issues by rule ID for aggregation
        let mut rule_groups: std::collections::HashMap<String, Vec<&Issue>> =
            std::collections::HashMap::new();
        for issue in cat_issues {
            rule_groups.entry(issue.id.clone()).or_default().push(issue);
        }

        // Count unique rules
        let unique_rules = rule_groups.len();
        println!(
            "  {}  {} ({} issue{})",
            "●".cyan(),
            category_display.white().bold(),
            unique_rules,
            if unique_rules == 1 { "" } else { "s" }
        );

        // Print aggregated issues
        for (rule_id, rule_issues) in rule_groups.iter() {
            let first_issue = rule_issues[0];
            let severity_icon = match first_issue.severity {
                Severity::Error => icons::CROSS.red().to_string(),
                Severity::Warning => icons::WARNING.yellow().to_string(),
                Severity::Info => icons::INFO.blue().to_string(),
            };

            // Get base message (without line-specific info)
            let base_msg = first_issue
                .message
                .split(':')
                .next()
                .unwrap_or(&first_issue.message);

            // Truncate message if too long
            let max_msg_len = 50;
            let msg = if base_msg.len() > max_msg_len {
                format!("{}...", &base_msg[..max_msg_len - 3])
            } else {
                base_msg.to_string()
            };

            // Show line count if multiple occurrences
            let count_info = if rule_issues.len() > 1 {
                format!(" ({} lines)", rule_issues.len())
                    .bright_black()
                    .to_string()
            } else if let Some(line) = first_issue.line {
                format!(" (L{})", line).bright_black().to_string()
            } else {
                String::new()
            };

            println!(
                "     {} {} {}{}",
                severity_icon,
                rule_id.bright_black(),
                msg,
                count_info
            );
        }
        println!();
    }
}

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

/// Print optimizing progress indicator (simple, non-animated version)
pub fn print_optimizing(model: &str) {
    let model_short = if model.len() > 50 {
        format!("{}...", &model[..47])
    } else {
        model.to_string()
    };

    println!(
        "  {}  {} {}",
        icons::GEAR.cyan(),
        "Optimizing with".bright_black(),
        model_short.white()
    );
    println!();
}

/// Print success message
pub fn print_success(message: &str) {
    println!("  {}  {}", icons::CHECK.green(), message.green());
}

/// Print error message
pub fn print_error(message: &str) {
    eprintln!("  {}  {}", icons::CROSS.red(), message.red());
}

/// Print a warning message
pub fn print_warning(message: &str) {
    println!("  {}  {}", icons::WARNING.yellow(), message.yellow());
}

/// Format category name for display
fn format_category_name(category: &str) -> String {
    match category.to_lowercase().as_str() {
        "explicitness" => "Explicitness",
        "style" => "Style",
        "tools" => "Tool Usage",
        "formatting" => "Formatting",
        "verbosity" => "Verbosity",
        "agentic" => "Agentic Coding",
        "long_horizon" => "Long-Horizon",
        "frontend" => "Frontend Design",
        other => other,
    }
    .to_string()
}

/// Print a separator line
pub fn print_separator() {
    println!("  {}", "─".repeat(70).bright_black());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_category_name() {
        assert_eq!(format_category_name("explicitness"), "Explicitness");
        assert_eq!(format_category_name("long_horizon"), "Long-Horizon");
        assert_eq!(format_category_name("unknown"), "unknown");
    }
}

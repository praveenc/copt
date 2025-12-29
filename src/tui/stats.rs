//! Statistics display component for the TUI
//!
//! Displays optimization statistics in a clean, box-free format.

use colored::Colorize;

use super::icons;
use crate::OptimizationStats;

/// Print optimization statistics
pub fn print_stats(stats: &OptimizationStats) {
    println!();
    println!("  {}", "─".repeat(70).bright_black());
    println!(
        "  {}  {}",
        icons::CHART.cyan(),
        "Optimization Results".white().bold()
    );
    println!("  {}", "─".repeat(70).bright_black());
    println!();

    // Token Analysis
    println!("  {}", "TOKEN ANALYSIS".cyan().bold());
    println!();

    // Create visual bars
    let max_tokens = stats.original_tokens.max(stats.optimized_tokens).max(1);
    let bar_width: usize = 30;

    let orig_bar_len =
        (stats.original_tokens as f64 / max_tokens as f64 * bar_width as f64) as usize;
    let opt_bar_len =
        (stats.optimized_tokens as f64 / max_tokens as f64 * bar_width as f64) as usize;

    let orig_bar = format!(
        "{}{}",
        "█".repeat(orig_bar_len),
        "░".repeat(bar_width - orig_bar_len)
    );
    let opt_bar = format!(
        "{}{}",
        "█".repeat(opt_bar_len),
        "░".repeat(bar_width - opt_bar_len)
    );

    println!(
        "  {:<18} {} {}",
        "Original:".bright_black(),
        orig_bar.bright_black(),
        stats.original_tokens.to_string().white()
    );
    println!(
        "  {:<18} {} {}",
        "Optimized:".bright_black(),
        opt_bar.green(),
        stats.optimized_tokens.to_string().white().bold()
    );

    // Token change percentage
    let token_change = if stats.original_tokens > 0 {
        let change = ((stats.optimized_tokens as f64 - stats.original_tokens as f64)
            / stats.original_tokens as f64
            * 100.0) as i32;
        if change >= 0 {
            format!("+{}%", change).green().bold().to_string()
        } else {
            format!("{}%", change).yellow().bold().to_string()
        }
    } else {
        "N/A".to_string()
    };
    println!("  {:<18} {}", "Change:".bright_black(), token_change);

    println!();

    // Performance
    println!("  {}", "PERFORMANCE".cyan().bold());
    println!();

    let time_display = if stats.processing_time_ms < 1000 {
        format!("{}ms", stats.processing_time_ms)
    } else {
        format!("{:.2}s", stats.processing_time_ms as f64 / 1000.0)
    };

    println!(
        "  {:<18} {}",
        "Processing time:".bright_black(),
        time_display.green()
    );
    println!(
        "  {:<18} {}",
        "Rules applied:".bright_black(),
        stats.rules_applied.to_string().white()
    );
    println!(
        "  {:<18} {}",
        "Categories fixed:".bright_black(),
        stats.categories_improved.to_string().white()
    );

    println!();

    // Provider
    println!("  {}", "PROVIDER".cyan().bold());
    println!();

    println!(
        "  {:<18} {}",
        "Service:".bright_black(),
        capitalize_first(&stats.provider).white().bold()
    );

    // Truncate model name if too long
    let model_display = if stats.model.len() > 50 {
        format!("{}...", &stats.model[..47])
    } else {
        stats.model.clone()
    };
    println!(
        "  {:<18} {}",
        "Model:".bright_black(),
        model_display.bright_black()
    );

    println!();
}

/// Print a compact one-line summary
pub fn print_stats_compact(stats: &OptimizationStats) {
    let token_change = if stats.original_tokens > 0 {
        let change = ((stats.optimized_tokens as f64 - stats.original_tokens as f64)
            / stats.original_tokens as f64
            * 100.0) as i32;
        if change >= 0 {
            format!("+{}%", change).green().bold()
        } else {
            format!("{}%", change).yellow().bold()
        }
    } else {
        "N/A".normal()
    };

    println!(
        "  {}  {} {} {} ({}) | {} rules | {:.1}s",
        icons::CHECK.green(),
        stats.original_tokens.to_string().bright_black(),
        "→".cyan(),
        stats.optimized_tokens.to_string().white().bold(),
        token_change,
        stats.rules_applied.to_string().cyan(),
        stats.processing_time_ms as f64 / 1000.0,
    );
}

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

/// Capitalize the first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().to_string() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("anthropic"), "Anthropic");
        assert_eq!(capitalize_first(""), "");
        assert_eq!(capitalize_first("AWS"), "AWS");
    }
}
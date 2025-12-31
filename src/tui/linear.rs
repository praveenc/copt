//! Linear (non-interactive) output mode
//!
//! Renders enhanced output that scrolls like traditional CLI output.
//! Uses ratatui to render to a string buffer, then prints to stdout.

#![allow(dead_code)]

use std::io::{self, Write};

use colored::Colorize;

use super::icons::icons;
use super::model::{AppPhase, Model};
use crate::analyzer::Severity;

/// Render the model in linear mode (prints to stdout)
pub fn render(model: &Model) -> io::Result<()> {
    let mut stdout = io::stdout();

    // Header
    render_header(&mut stdout, model)?;

    // Input info
    render_input_info(&mut stdout, model)?;

    // Analysis results
    render_analysis(&mut stdout, model)?;

    // Stats (if available)
    if model.stats.is_some() && model.phase == AppPhase::Done {
        render_stats(&mut stdout, model)?;
    }

    Ok(())
}

/// Render the header
fn render_header(w: &mut impl Write, model: &Model) -> io::Result<()> {
    let icons = icons();
    let version = env!("CARGO_PKG_VERSION");

    writeln!(w)?;
    write!(
        w,
        "  {}  {}",
        icons.lightning.cyan(),
        "CLAUDE PROMPT OPTIMIZER".cyan().bold()
    )?;

    if model.offline_mode {
        writeln!(w, " {}", "[OFFLINE MODE]".yellow())?;
    } else {
        writeln!(w)?;
    }

    writeln!(
        w,
        "     {}",
        format!("v{} • Optimize prompts for Claude 4.5", version).bright_black()
    )?;
    writeln!(w)?;

    Ok(())
}

/// Render input information
fn render_input_info(w: &mut impl Write, model: &Model) -> io::Result<()> {
    let icons = icons();

    let source = model.input_file.as_deref().unwrap_or("stdin");

    let char_count = model.original_prompt.len();
    let token_count = crate::utils::count_tokens(&model.original_prompt);

    writeln!(
        w,
        "  {}  {} {} ({} chars, {} tokens)",
        icons.inbox.cyan(),
        "Input:".white().bold(),
        source.white(),
        char_count.to_string().cyan(),
        token_count.to_string().cyan()
    )?;
    writeln!(w)?;

    Ok(())
}

/// Render analysis results
fn render_analysis(w: &mut impl Write, model: &Model) -> io::Result<()> {
    let icons = icons();

    // Section header
    writeln!(w, "  {}", "─".repeat(70).bright_black())?;
    writeln!(
        w,
        "  {}  {}",
        icons.chart.cyan(),
        "Analysis Results".white().bold()
    )?;
    writeln!(w, "  {}", "─".repeat(70).bright_black())?;
    writeln!(w)?;

    if model.issue_tree.categories.is_empty() {
        writeln!(
            w,
            "  {}  {}",
            icons.check.green(),
            "No issues detected - your prompt looks good!".green()
        )?;
        writeln!(w)?;
        return Ok(());
    }

    // Count by severity
    let total_issues: usize = model
        .issue_tree
        .categories
        .iter()
        .map(|c| c.issues.len())
        .sum();

    let errors: usize = model
        .issue_tree
        .categories
        .iter()
        .flat_map(|c| &c.issues)
        .filter(|i| matches!(i.severity, Severity::Error))
        .count();

    let warnings: usize = model
        .issue_tree
        .categories
        .iter()
        .flat_map(|c| &c.issues)
        .filter(|i| matches!(i.severity, Severity::Warning))
        .count();

    let infos = total_issues - errors - warnings;

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

    writeln!(
        w,
        "  Found {} across {} {}",
        summary_parts.join(", "),
        model.issue_tree.categories.len(),
        if model.issue_tree.categories.len() == 1 {
            "category"
        } else {
            "categories"
        }
    )?;
    writeln!(w)?;

    // Print each category
    for cat in &model.issue_tree.categories {
        writeln!(
            w,
            "  {}  {} ({} issue{})",
            icons.bullet.cyan(),
            cat.display_name.white().bold(),
            cat.issues.len(),
            if cat.issues.len() == 1 { "" } else { "s" }
        )?;

        for issue in &cat.issues {
            let severity_icon = match issue.severity {
                Severity::Error => icons.cross.red().to_string(),
                Severity::Warning => icons.warning.yellow().to_string(),
                Severity::Info => icons.info.blue().to_string(),
            };

            let line_info = issue.line.map(|l| format!(" (L{})", l)).unwrap_or_default();

            // Truncate message
            let max_msg_len = 50;
            let msg = if issue.message.len() > max_msg_len {
                format!("{}...", &issue.message[..max_msg_len - 3])
            } else {
                issue.message.clone()
            };

            writeln!(
                w,
                "     {} {} {}{}",
                severity_icon,
                issue.id.bright_black(),
                msg,
                line_info.bright_black()
            )?;
        }
        writeln!(w)?;
    }

    Ok(())
}

/// Render optimization statistics
fn render_stats(w: &mut impl Write, model: &Model) -> io::Result<()> {
    let icons = icons();

    let Some(ref stats) = model.stats else {
        return Ok(());
    };

    writeln!(w)?;
    writeln!(w, "  {}", "─".repeat(70).bright_black())?;
    writeln!(
        w,
        "  {}  {}",
        icons.chart.cyan(),
        "Optimization Results".white().bold()
    )?;
    writeln!(w, "  {}", "─".repeat(70).bright_black())?;
    writeln!(w)?;

    // Token Analysis
    writeln!(w, "  {}", "TOKEN ANALYSIS".cyan().bold())?;
    writeln!(w)?;

    let max_tokens = stats.original_tokens.max(stats.optimized_tokens).max(1);
    let bar_width: usize = 30;

    let orig_bar_len = (stats.original_tokens * bar_width) / max_tokens;
    let opt_bar_len = (stats.optimized_tokens * bar_width) / max_tokens;

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

    writeln!(
        w,
        "  {:<18} {} {}",
        "Original:".bright_black(),
        orig_bar.bright_black(),
        stats.original_tokens.to_string().white()
    )?;
    writeln!(
        w,
        "  {:<18} {} {}",
        "Optimized:".bright_black(),
        opt_bar.green(),
        stats.optimized_tokens.to_string().white().bold()
    )?;

    // Token change
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
    writeln!(w, "  {:<18} {}", "Change:".bright_black(), token_change)?;
    writeln!(w)?;

    // Performance
    writeln!(w, "  {}", "PERFORMANCE".cyan().bold())?;
    writeln!(w)?;

    let time_display = if stats.processing_time_ms < 1000 {
        format!("{}ms", stats.processing_time_ms)
    } else {
        format!("{:.2}s", stats.processing_time_ms as f64 / 1000.0)
    };

    writeln!(
        w,
        "  {:<18} {}",
        "Processing time:".bright_black(),
        time_display.green()
    )?;
    writeln!(
        w,
        "  {:<18} {}",
        "Rules applied:".bright_black(),
        stats.rules_applied.to_string().white()
    )?;
    writeln!(w)?;

    // Provider
    writeln!(w, "  {}", "PROVIDER".cyan().bold())?;
    writeln!(w)?;

    let provider = {
        let mut chars = stats.provider.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().to_string() + chars.as_str(),
        }
    };

    writeln!(
        w,
        "  {:<18} {}",
        "Service:".bright_black(),
        provider.white().bold()
    )?;

    let model_display = if stats.model.len() > 50 {
        format!("{}...", &stats.model[..47])
    } else {
        stats.model.clone()
    };
    writeln!(
        w,
        "  {:<18} {}",
        "Model:".bright_black(),
        model_display.bright_black()
    )?;
    writeln!(w)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_header() {
        let mut buf = Vec::new();
        let model = Model::default();
        render_header(&mut buf, &model).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("CLAUDE PROMPT OPTIMIZER"));
    }

    #[test]
    fn test_render_empty_analysis() {
        let mut buf = Vec::new();
        let model = Model::default();
        render_analysis(&mut buf, &model).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("No issues"));
    }
}

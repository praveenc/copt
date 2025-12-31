//! Statistics dashboard widget

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::tui::icons::icons;
use crate::tui::model::Model;
use crate::tui::theme::theme;

/// Render the statistics dashboard
pub fn render_dashboard(frame: &mut Frame, area: Rect, model: &Model) {
    let theme = theme();
    let icons = icons();

    let block = Block::default()
        .title(format!(" {} Optimization Results ", icons.chart))
        .title_style(theme.title)
        .borders(Borders::ALL)
        .border_style(theme.border);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let Some(ref stats) = model.stats else {
        return;
    };

    // Split into three sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Token analysis
            Constraint::Length(4), // Performance
            Constraint::Length(3), // Provider
        ])
        .split(inner_area);

    // Token Analysis Section
    render_token_section(frame, chunks[0], stats);

    // Performance Section
    render_performance_section(frame, chunks[1], stats);

    // Provider Section
    render_provider_section(frame, chunks[2], stats);
}

/// Render token analysis with horizontal bars
fn render_token_section(frame: &mut Frame, area: Rect, stats: &crate::OptimizationStats) {
    let theme = theme();

    let max_tokens = stats.original_tokens.max(stats.optimized_tokens).max(1);
    let bar_width = (area.width as usize).saturating_sub(30);

    let orig_bar_len = (stats.original_tokens * bar_width) / max_tokens;
    let opt_bar_len = (stats.optimized_tokens * bar_width) / max_tokens;

    let orig_bar = format!(
        "{}{}",
        "█".repeat(orig_bar_len),
        "░".repeat(bar_width.saturating_sub(orig_bar_len))
    );
    let opt_bar = format!(
        "{}{}",
        "█".repeat(opt_bar_len),
        "░".repeat(bar_width.saturating_sub(opt_bar_len))
    );

    // Calculate change percentage
    let change = if stats.original_tokens > 0 {
        let pct = ((stats.optimized_tokens as f64 - stats.original_tokens as f64)
            / stats.original_tokens as f64
            * 100.0) as i32;
        if pct >= 0 {
            format!("+{}%", pct)
        } else {
            format!("{}%", pct)
        }
    } else {
        "N/A".to_string()
    };

    let change_style = if stats.optimized_tokens >= stats.original_tokens {
        theme.success
    } else {
        theme.warning
    };

    let text = vec![
        Line::from(Span::styled("TOKEN ANALYSIS", theme.primary.bold())),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{:<12}", "Original:"), theme.muted),
            Span::styled(&orig_bar, theme.muted),
            Span::styled(format!(" {}", stats.original_tokens), theme.text),
        ]),
        Line::from(vec![
            Span::styled(format!("{:<12}", "Optimized:"), theme.muted),
            Span::styled(&opt_bar, theme.success),
            Span::styled(
                format!(" {} ({})", stats.optimized_tokens, change),
                change_style,
            ),
        ]),
    ];

    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, area);
}

/// Render performance metrics
fn render_performance_section(frame: &mut Frame, area: Rect, stats: &crate::OptimizationStats) {
    let theme = theme();

    let time_display = if stats.processing_time_ms < 1000 {
        format!("{}ms", stats.processing_time_ms)
    } else {
        format!("{:.2}s", stats.processing_time_ms as f64 / 1000.0)
    };

    let text = vec![
        Line::from(Span::styled("PERFORMANCE", theme.primary.bold())),
        Line::from(vec![
            Span::styled(format!("{:<18}", "Processing time:"), theme.muted),
            Span::styled(time_display, theme.success),
        ]),
        Line::from(vec![
            Span::styled(format!("{:<18}", "Rules applied:"), theme.muted),
            Span::styled(stats.rules_applied.to_string(), theme.text),
        ]),
        Line::from(vec![
            Span::styled(format!("{:<18}", "Categories fixed:"), theme.muted),
            Span::styled(stats.categories_improved.to_string(), theme.text),
        ]),
    ];

    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, area);
}

/// Render provider information
fn render_provider_section(frame: &mut Frame, area: Rect, stats: &crate::OptimizationStats) {
    let theme = theme();

    // Capitalize provider name
    let provider = {
        let mut chars = stats.provider.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().to_string() + chars.as_str(),
        }
    };

    // Truncate model name if too long
    let model_display = if stats.model.len() > 50 {
        format!("{}...", &stats.model[..47])
    } else {
        stats.model.clone()
    };

    let text = vec![
        Line::from(Span::styled("PROVIDER", theme.primary.bold())),
        Line::from(vec![
            Span::styled(format!("{:<18}", "Service:"), theme.muted),
            Span::styled(provider, theme.text.bold()),
        ]),
        Line::from(vec![
            Span::styled(format!("{:<18}", "Model:"), theme.muted),
            Span::styled(model_display, theme.muted),
        ]),
    ];

    let paragraph = Paragraph::new(text);
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OptimizationStats;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_render_dashboard() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut model = Model::default();
        model.stats = Some(OptimizationStats {
            original_tokens: 100,
            optimized_tokens: 150,
            processing_time_ms: 2500,
            rules_applied: 5,
            categories_improved: 3,
            provider: "bedrock".to_string(),
            model: "claude-sonnet".to_string(),
            ..Default::default()
        });

        terminal
            .draw(|frame| {
                render_dashboard(frame, frame.area(), &model);
            })
            .unwrap();

        // Should render without panic
    }
}

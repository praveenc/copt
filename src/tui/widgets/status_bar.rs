//! Status bar widget with keyboard hints

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::model::Model;
use crate::tui::theme::theme;

/// Render the main status bar with keyboard hints
pub fn render_status_bar(frame: &mut Frame, area: Rect, model: &Model) {
    let theme = theme();

    let mut hints = vec![
        key_hint("↑↓", "nav"),
        Span::raw("  "),
        key_hint("Enter", "expand"),
    ];

    // Add action hints if results available
    if model.has_results() {
        hints.push(Span::raw("  "));
        hints.push(key_hint("d", "diff"));
        hints.push(Span::raw("  "));
        hints.push(key_hint("c", "copy"));
        hints.push(Span::raw("  "));
        hints.push(key_hint("s", "save"));
    }

    hints.push(Span::raw("  "));
    hints.push(key_hint("?", "help"));
    hints.push(Span::raw("  "));
    hints.push(key_hint("q", "quit"));

    let status = Paragraph::new(Line::from(hints)).style(theme.muted);

    frame.render_widget(status, area);
}

/// Render status bar for diff view
pub fn render_status_bar_diff(frame: &mut Frame, area: Rect, _model: &Model) {
    let hints = vec![
        key_hint("Esc", "return"),
        Span::raw("  "),
        key_hint("↑↓", "scroll"),
        Span::raw("  "),
        key_hint("c", "copy"),
        Span::raw("  "),
        key_hint("s", "save"),
        Span::raw("  "),
        key_hint("q", "quit"),
    ];

    let status = Paragraph::new(Line::from(hints)).style(theme().muted);

    frame.render_widget(status, area);
}

/// Render status bar for help view
pub fn render_status_bar_help(frame: &mut Frame, area: Rect, _model: &Model) {
    let hints = vec![
        key_hint("Esc", "return"),
        Span::raw("  "),
        key_hint("q", "quit"),
    ];

    let status = Paragraph::new(Line::from(hints)).style(theme().muted);

    frame.render_widget(status, area);
}

/// Create a key hint span pair
fn key_hint<'a>(key: &'a str, action: &'a str) -> Span<'a> {
    let theme = theme();
    // Create a combined span - ratatui doesn't allow mixed styles in a single Span
    // So we return just the formatted string with the key highlighted
    Span::styled(format!("{}:{}", key, action), theme.key_hint)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_render_status_bar() {
        let backend = TestBackend::new(80, 1);
        let mut terminal = Terminal::new(backend).unwrap();

        let model = Model::default();

        terminal
            .draw(|frame| {
                render_status_bar(frame, frame.area(), &model);
            })
            .unwrap();

        // Should render without panic
    }
}

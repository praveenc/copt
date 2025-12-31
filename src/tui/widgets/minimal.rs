//! Minimal view widget for small terminals

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::icons::icons;
use crate::tui::model::{AppPhase, Model};
use crate::tui::theme::theme;

/// Render a minimal summary for small terminals
pub fn render_minimal_summary(frame: &mut Frame, area: Rect, model: &Model) {
    let theme = theme();
    let icons = icons();

    let status_line = match model.phase {
        AppPhase::Ready => Line::from(vec![
            Span::styled(format!("{} ", icons.lightning), theme.primary),
            Span::styled("copt - Ready", theme.text),
        ]),
        AppPhase::Analyzing => Line::from(vec![
            Span::styled(format!("{} ", icons.gear), theme.primary),
            Span::styled("Analyzing...", theme.text),
        ]),
        AppPhase::AnalysisDone => {
            let issue_count = model.total_issues();
            Line::from(vec![
                Span::styled(format!("{} ", icons.check), theme.success),
                Span::styled(format!("{} issues found", issue_count), theme.text),
            ])
        }
        AppPhase::Optimizing => Line::from(vec![
            Span::styled(format!("{} ", icons.gear), theme.primary),
            Span::styled("Optimizing...", theme.text),
        ]),
        AppPhase::Done => {
            let tokens = model
                .stats
                .as_ref()
                .map(|s| format!("{} → {} tokens", s.original_tokens, s.optimized_tokens))
                .unwrap_or_default();
            Line::from(vec![
                Span::styled(format!("{} ", icons.check), theme.success),
                Span::styled("Done ", theme.success),
                Span::styled(tokens, theme.muted),
            ])
        }
        AppPhase::Error => Line::from(vec![
            Span::styled(format!("{} ", icons.cross), theme.error),
            Span::styled("Error occurred", theme.error),
        ]),
    };

    let paragraph = Paragraph::new(status_line);
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_render_minimal_summary() {
        let backend = TestBackend::new(40, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        let model = Model::default();

        terminal
            .draw(|frame| {
                render_minimal_summary(frame, frame.area(), &model);
            })
            .unwrap();

        // Should render without panic
    }
}

//! Progress widget for optimization status

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

use crate::tui::icons::icons;
use crate::tui::model::{AppPhase, Model};
use crate::tui::theme::theme;

/// Render progress indicator or placeholder
pub fn render_progress(frame: &mut Frame, area: Rect, model: &Model) {
    let theme = theme();
    let icons = icons();

    let block = Block::default()
        .title(format!(" {} Status ", icons.gear))
        .title_style(theme.title)
        .borders(Borders::ALL)
        .border_style(theme.border);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    match model.phase {
        AppPhase::Ready => {
            let text = Paragraph::new(Line::from(vec![
                Span::styled(format!("{} ", icons.info), theme.secondary),
                Span::styled("Ready to analyze", theme.muted),
            ]));
            frame.render_widget(text, inner_area);
        }
        AppPhase::Analyzing => {
            render_spinner(frame, inner_area, "Analyzing prompt...");
        }
        AppPhase::AnalysisDone => {
            if model.offline_mode {
                let text = Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled(format!("{} ", icons.check), theme.success),
                        Span::styled("Analysis complete", theme.success),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Run without --offline to optimize with LLM",
                        theme.muted,
                    )),
                ]);
                frame.render_widget(text, inner_area);
            } else {
                let text = Paragraph::new(Line::from(vec![
                    Span::styled(format!("{} ", icons.check), theme.success),
                    Span::styled("Analysis complete - ready to optimize", theme.text),
                ]));
                frame.render_widget(text, inner_area);
            }
        }
        AppPhase::Optimizing => {
            render_optimization_progress(frame, inner_area, model);
        }
        AppPhase::Done => {
            let text = Paragraph::new(Line::from(vec![
                Span::styled(format!("{} ", icons.check), theme.success),
                Span::styled("Optimization complete!", theme.success),
            ]));
            frame.render_widget(text, inner_area);
        }
        AppPhase::Error => {
            let text = Paragraph::new(Line::from(vec![
                Span::styled(format!("{} ", icons.cross), theme.error),
                Span::styled("An error occurred", theme.error),
            ]));
            frame.render_widget(text, inner_area);
        }
    }
}

/// Render a simple spinner animation
fn render_spinner(frame: &mut Frame, area: Rect, message: &str) {
    let theme = theme();

    // Simple spinner using elapsed time
    let tick = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        / 100) as usize;
    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner = frames[tick % frames.len()];

    let text = Paragraph::new(Line::from(vec![
        Span::styled(format!("{} ", spinner), theme.primary),
        Span::styled(message, theme.text),
    ]));

    frame.render_widget(text, area);
}

/// Render optimization progress with gauge
fn render_optimization_progress(frame: &mut Frame, area: Rect, _model: &Model) {
    let theme = theme();
    let icons = icons();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(2)])
        .split(area);

    // Status text
    let text = Paragraph::new(Line::from(vec![
        Span::styled(format!("{} ", icons.gear), theme.primary),
        Span::styled("Optimizing with LLM...", theme.text),
    ]));
    frame.render_widget(text, chunks[0]);

    // Progress gauge (indeterminate for now)
    let gauge = Gauge::default()
        .gauge_style(theme.progress_filled)
        .ratio(0.0) // Would update based on actual progress
        .label("Processing...");

    frame.render_widget(gauge, chunks[1]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_render_progress_ready() {
        let backend = TestBackend::new(60, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        let model = Model::default();

        terminal
            .draw(|frame| {
                render_progress(frame, frame.area(), &model);
            })
            .unwrap();

        // Should render without panic
    }
}

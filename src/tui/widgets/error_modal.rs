//! Error modal widget

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::model::Model;
use crate::tui::theme::theme;
use crate::tui::view::centered_rect;

/// Render the error modal
pub fn render_error_modal(frame: &mut Frame, model: &Model) {
    let theme = theme();

    let Some(ref error) = model.error else {
        return;
    };

    // Create centered area for modal
    let area = centered_rect(60, 30, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Error ")
        .title_style(theme.error)
        .borders(Borders::ALL)
        .border_style(theme.error);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = vec![
        Line::from(Span::styled(&error.message, theme.error)),
        Line::from(""),
    ];

    if let Some(ref details) = error.details {
        lines.push(Line::from(Span::styled(details.as_str(), theme.muted)));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        "Press Enter to continue",
        theme.muted,
    )));

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, inner_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::model::ErrorState;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_render_error_modal() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut model = Model::default();
        model.error = Some(ErrorState::new("Test error message"));

        terminal
            .draw(|frame| {
                render_error_modal(frame, &model);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer.content().iter().map(|c| c.symbol()).collect::<String>();
        assert!(content.contains("Error"));
    }

    #[test]
    fn test_render_error_modal_no_error() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let model = Model::default();

        terminal
            .draw(|frame| {
                render_error_modal(frame, &model);
            })
            .unwrap();

        // Should render without panic (no-op when no error)
    }
}

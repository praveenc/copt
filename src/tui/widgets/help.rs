//! Help screen widget

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::tui::model::Model;
use crate::tui::theme::theme;

/// Render the help screen
pub fn render_help(frame: &mut Frame, area: Rect, _model: &Model) {
    let theme = theme();

    let block = Block::default()
        .title(" Keyboard Shortcuts ")
        .title_style(theme.title)
        .borders(Borders::ALL)
        .border_style(theme.border);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let help_text = vec![
        Line::from(Span::styled("NAVIGATION", theme.primary)),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ↑/↓        ", theme.key),
            Span::styled("Move selection up/down", theme.text),
        ]),
        Line::from(vec![
            Span::styled("  Enter      ", theme.key),
            Span::styled("Expand/collapse category", theme.text),
        ]),
        Line::from(vec![
            Span::styled("  PgUp/PgDn  ", theme.key),
            Span::styled("Scroll content", theme.text),
        ]),
        Line::from(vec![
            Span::styled("  Home       ", theme.key),
            Span::styled("Go to top", theme.text),
        ]),
        Line::from(""),
        Line::from(Span::styled("VIEWS", theme.primary)),
        Line::from(""),
        Line::from(vec![
            Span::styled("  d          ", theme.key),
            Span::styled("Toggle diff view", theme.text),
        ]),
        Line::from(vec![
            Span::styled("  ?          ", theme.key),
            Span::styled("Toggle help (this screen)", theme.text),
        ]),
        Line::from(vec![
            Span::styled("  Esc        ", theme.key),
            Span::styled("Return to main view", theme.text),
        ]),
        Line::from(""),
        Line::from(Span::styled("ACTIONS", theme.primary)),
        Line::from(""),
        Line::from(vec![
            Span::styled("  c          ", theme.key),
            Span::styled("Copy optimized prompt to clipboard", theme.text),
        ]),
        Line::from(vec![
            Span::styled("  s          ", theme.key),
            Span::styled("Save optimized prompt to file", theme.text),
        ]),
        Line::from(vec![
            Span::styled("  r          ", theme.key),
            Span::styled("Re-run optimization", theme.text),
        ]),
        Line::from(""),
        Line::from(Span::styled("GENERAL", theme.primary)),
        Line::from(""),
        Line::from(vec![
            Span::styled("  q          ", theme.key),
            Span::styled("Quit application", theme.text),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+C     ", theme.key),
            Span::styled("Quit application", theme.text),
        ]),
    ];

    let paragraph = Paragraph::new(help_text);
    frame.render_widget(paragraph, inner_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_render_help() {
        let backend = TestBackend::new(60, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        let model = Model::default();

        terminal
            .draw(|frame| {
                render_help(frame, frame.area(), &model);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer.content().iter().map(|c| c.symbol()).collect::<String>();
        assert!(content.contains("NAVIGATION"));
    }
}

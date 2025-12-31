//! Header widget

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::tui::icons::icons;
use crate::tui::model::Model;
use crate::tui::theme::theme;

/// Render the main application header
pub fn render_header(frame: &mut Frame, area: Rect, model: &Model) {
    let theme = theme();
    let icons = icons();

    let version = env!("CARGO_PKG_VERSION");

    let mut title_spans = vec![
        Span::styled(format!("{} ", icons.lightning), theme.primary),
        Span::styled("CLAUDE PROMPT OPTIMIZER", theme.primary),
        Span::styled(format!(" v{}", version), theme.muted),
    ];

    // Add offline mode indicator
    if model.offline_mode {
        title_spans.push(Span::raw(" "));
        title_spans.push(Span::styled("[OFFLINE MODE]", theme.warning));
    }

    let subtitle = if model.offline_mode {
        "Static analysis only (no LLM calls)"
    } else {
        "Optimize prompts for Claude 4.5"
    };

    // Input info line
    let input_info = if let Some(ref file) = model.input_file {
        format!(
            "{} Input: {} ({} chars, {} tokens)",
            icons.inbox,
            file,
            model.original_prompt.len(),
            crate::utils::count_tokens(&model.original_prompt)
        )
    } else {
        format!(
            "{} Input: stdin ({} chars, {} tokens)",
            icons.inbox,
            model.original_prompt.len(),
            crate::utils::count_tokens(&model.original_prompt)
        )
    };

    let text = vec![
        Line::from(title_spans),
        Line::from(Span::styled(subtitle, theme.muted)),
        Line::from(Span::styled(input_info, theme.text)),
    ];

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(theme.border);

    let paragraph = Paragraph::new(text).block(block);

    frame.render_widget(paragraph, area);
}

/// Render a compact header (for diff/help views)
pub fn render_header_compact(frame: &mut Frame, area: Rect, model: &Model) {
    let theme = theme();
    let icons = icons();

    let version = env!("CARGO_PKG_VERSION");

    let mut spans = vec![
        Span::styled(format!("{} ", icons.lightning), theme.primary),
        Span::styled("CLAUDE PROMPT OPTIMIZER", theme.primary),
        Span::styled(format!(" v{}", version), theme.muted),
    ];

    if model.offline_mode {
        spans.push(Span::raw(" "));
        spans.push(Span::styled("[OFFLINE]", theme.warning));
    }

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(theme.border);

    let paragraph = Paragraph::new(Line::from(spans)).block(block);

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_render_header() {
        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        let model = Model::default();

        terminal
            .draw(|frame| {
                render_header(frame, frame.area(), &model);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();
        assert!(content.contains("CLAUDE PROMPT OPTIMIZER"));
    }
}

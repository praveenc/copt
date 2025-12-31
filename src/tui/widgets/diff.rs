//! Diff view widget

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use similar::{ChangeTag, TextDiff};

use crate::tui::icons::icons;
use crate::tui::model::Model;
use crate::tui::theme::theme;

/// Render the diff view (side-by-side comparison)
pub fn render_diff(frame: &mut Frame, area: Rect, model: &Model) {
    let theme = theme();
    let icons = icons();

    let block = Block::default()
        .title(format!(" {} Changes ", icons.sparkles))
        .title_style(theme.title)
        .borders(Borders::ALL)
        .border_style(theme.border);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let Some(ref optimized) = model.optimized_prompt else {
        let text = Paragraph::new("No optimization results yet");
        frame.render_widget(text, inner_area);
        return;
    };

    // Split into two columns with a divider
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner_area);

    // Left side: Original
    render_diff_panel(
        frame,
        chunks[0],
        &model.original_prompt,
        optimized,
        true, // is_original
        model.scroll_offset,
    );

    // Right side: Optimized
    render_diff_panel(
        frame,
        chunks[1],
        &model.original_prompt,
        optimized,
        false, // is_original
        model.scroll_offset,
    );
}

/// Render one side of the diff
fn render_diff_panel(
    frame: &mut Frame,
    area: Rect,
    original: &str,
    optimized: &str,
    is_original: bool,
    scroll_offset: u16,
) {
    let theme = theme();
    let icons = icons();

    let (title, _content) = if is_original {
        (format!("{} Original", icons.file), original)
    } else {
        (format!("{} Optimized", icons.sparkles), optimized)
    };

    let title_style = if is_original {
        theme.muted
    } else {
        theme.success
    };

    let block = Block::default()
        .title(Span::styled(title, title_style))
        .borders(Borders::ALL)
        .border_style(theme.border);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Generate diff and highlight changes
    let diff = TextDiff::from_lines(original, optimized);
    let mut lines: Vec<Line> = Vec::new();

    for change in diff.iter_all_changes() {
        let line_content = change.value().trim_end();

        let (style, prefix) = match (change.tag(), is_original) {
            (ChangeTag::Delete, true) => (theme.diff_removed, "- "),
            (ChangeTag::Delete, false) => continue, // Skip deletions on optimized side
            (ChangeTag::Insert, true) => continue,  // Skip insertions on original side
            (ChangeTag::Insert, false) => (theme.diff_added, "+ "),
            (ChangeTag::Equal, _) => (theme.diff_unchanged, "  "),
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(line_content.to_string(), style),
        ]));
    }

    // Apply scroll offset
    let visible_lines: Vec<Line> = lines.into_iter().skip(scroll_offset as usize).collect();

    let paragraph = Paragraph::new(visible_lines).wrap(Wrap { trim: false });

    frame.render_widget(paragraph, inner_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_render_diff() {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        let mut model = Model::default();
        model.original_prompt = "Hello world\nThis is a test".to_string();
        model.optimized_prompt =
            Some("Hello world\nThis is an improved test\nWith more detail".to_string());

        terminal
            .draw(|frame| {
                render_diff(frame, frame.area(), &model);
            })
            .unwrap();

        // Should render without panic
    }

    #[test]
    fn test_render_diff_no_results() {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        let model = Model::default();

        terminal
            .draw(|frame| {
                render_diff(frame, frame.area(), &model);
            })
            .unwrap();

        // Should render without panic
    }
}

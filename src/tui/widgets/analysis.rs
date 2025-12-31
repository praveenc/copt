//! Analysis tree widget (collapsible issue tree)

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::analyzer::Severity;
use crate::tui::icons::icons;
use crate::tui::model::Model;
use crate::tui::theme::theme;

/// Render the analysis results as a collapsible tree
pub fn render_analysis(frame: &mut Frame, area: Rect, model: &Model) {
    let theme = theme();
    let icons = icons();

    let block = Block::default()
        .title(format!(" {} Analysis Results ", icons.chart))
        .title_style(theme.title)
        .borders(Borders::ALL)
        .border_style(theme.border);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if model.issue_tree.categories.is_empty() {
        // No issues found
        let text = Line::from(vec![
            Span::styled(format!("{} ", icons.check), theme.success),
            Span::styled(
                "No issues detected - your prompt looks good!",
                theme.success,
            ),
        ]);
        let paragraph = ratatui::widgets::Paragraph::new(text);
        frame.render_widget(paragraph, inner_area);
        return;
    }

    // Build list items from the issue tree
    let mut items: Vec<ListItem> = Vec::new();
    let mut current_idx = 0;

    for cat in &model.issue_tree.categories {
        // Category header
        let expand_icon = if cat.expanded {
            icons.folder_open
        } else {
            icons.folder_closed
        };

        let cat_style = if current_idx == model.issue_tree.flat_index {
            theme.selected
        } else {
            theme.text
        };

        let cat_line = Line::from(vec![
            Span::styled(format!("{} ", expand_icon), theme.primary),
            Span::styled(&cat.display_name, cat_style.add_modifier(Modifier::BOLD)),
            Span::styled(format!(" ({} issues)", cat.issue_count()), theme.muted),
        ]);
        items.push(ListItem::new(cat_line));
        current_idx += 1;

        // Issues (if expanded)
        if cat.expanded {
            for issue in &cat.issues {
                let severity_style = match issue.severity {
                    Severity::Error => theme.error,
                    Severity::Warning => theme.warning,
                    Severity::Info => theme.secondary,
                };

                let severity_icon = match issue.severity {
                    Severity::Error => icons.cross,
                    Severity::Warning => icons.warning,
                    Severity::Info => icons.info,
                };

                let issue_style = if current_idx == model.issue_tree.flat_index {
                    theme.selected
                } else {
                    Style::default()
                };

                // Truncate message if too long
                let max_msg_len = (inner_area.width as usize).saturating_sub(20);
                let msg = if issue.message.len() > max_msg_len {
                    format!("{}...", &issue.message[..max_msg_len.saturating_sub(3)])
                } else {
                    issue.message.clone()
                };

                let line_info = issue.line.map(|l| format!(" (L{})", l)).unwrap_or_default();

                let issue_line = Line::from(vec![
                    Span::raw("   "), // Indent
                    Span::styled(format!("{} ", severity_icon), severity_style),
                    Span::styled(&issue.id, theme.muted),
                    Span::raw(" "),
                    Span::styled(msg, issue_style),
                    Span::styled(line_info, theme.muted),
                ]);
                items.push(ListItem::new(issue_line));
                current_idx += 1;
            }
        }
    }

    let list = List::new(items).highlight_style(theme.selected);

    // Create list state for selection
    let mut state = ListState::default();
    state.select(Some(model.issue_tree.flat_index));

    frame.render_stateful_widget(list, inner_area, &mut state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::Issue;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn create_test_model() -> Model {
        let mut model = Model::default();
        model.set_issues(&[
            Issue {
                id: "EXP001".to_string(),
                category: "explicitness".to_string(),
                severity: Severity::Warning,
                message: "Test warning".to_string(),
                line: Some(1),
                suggestion: Some("Fix it".to_string()),
            },
            Issue {
                id: "STY001".to_string(),
                category: "style".to_string(),
                severity: Severity::Error,
                message: "Test error".to_string(),
                line: None,
                suggestion: Some("Fix style".to_string()),
            },
        ]);
        model
    }

    #[test]
    fn test_render_analysis() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        let model = create_test_model();

        terminal
            .draw(|frame| {
                render_analysis(frame, frame.area(), &model);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();
        // Should contain analysis title
        assert!(content.contains("Analysis"));
    }

    #[test]
    fn test_render_empty_analysis() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        let model = Model::default();

        terminal
            .draw(|frame| {
                render_analysis(frame, frame.area(), &model);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();
        assert!(content.contains("No issues"));
    }
}

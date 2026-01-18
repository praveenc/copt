//! Suggestion modal widget for vague prompt improvements
//!
//! Displays a modal dialog when EXP005/EXP006 issues are detected,
//! allowing users to interactively select improvements to add.

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::cli::suggest::{Suggestion, OPENENDED_SUGGESTIONS, ROLE_SUGGESTIONS};
use crate::tui::theme::theme;
use crate::tui::view::centered_rect;

/// State for the suggest modal
#[derive(Debug, Clone, Default)]
pub struct SuggestModalState {
    /// Available suggestions
    pub suggestions: Vec<&'static Suggestion>,
    /// Which suggestions are selected (checkbox state)
    pub selections: Vec<bool>,
    /// Current cursor position
    pub cursor: usize,
    /// Whether the modal is visible
    pub visible: bool,
    /// Detected issue IDs that triggered the modal
    pub trigger_issues: Vec<String>,
}

impl SuggestModalState {
    /// Create a new modal state from detected issues
    pub fn from_issues(issues: &[crate::Issue]) -> Self {
        let has_exp005 = issues.iter().any(|i| i.id == "EXP005");
        let has_exp006 = issues.iter().any(|i| i.id == "EXP006");

        let mut suggestions: Vec<&'static Suggestion> = Vec::new();
        let mut trigger_issues = Vec::new();

        if has_exp005 {
            suggestions.extend(ROLE_SUGGESTIONS.iter());
            trigger_issues.push("EXP005".to_string());
        }

        if has_exp006 {
            suggestions.extend(OPENENDED_SUGGESTIONS.iter());
            trigger_issues.push("EXP006".to_string());
        }

        // Deduplicate by id
        suggestions.sort_by_key(|s| s.id);
        suggestions.dedup_by_key(|s| s.id);

        let selections = vec![false; suggestions.len()];

        Self {
            suggestions,
            selections,
            cursor: 0,
            visible: !trigger_issues.is_empty(),
            trigger_issues,
        }
    }

    /// Check if any issues should trigger the modal
    pub fn should_show(issues: &[crate::Issue]) -> bool {
        issues.iter().any(|i| i.id == "EXP005" || i.id == "EXP006")
    }

    /// Move cursor up
    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor down
    pub fn cursor_down(&mut self) {
        if self.cursor < self.suggestions.len().saturating_sub(1) {
            self.cursor += 1;
        }
    }

    /// Toggle selection at cursor
    pub fn toggle_current(&mut self) {
        if self.cursor < self.selections.len() {
            self.selections[self.cursor] = !self.selections[self.cursor];
        }
    }

    /// Select all suggestions
    pub fn select_all(&mut self) {
        for sel in &mut self.selections {
            *sel = true;
        }
    }

    /// Deselect all suggestions
    pub fn deselect_all(&mut self) {
        for sel in &mut self.selections {
            *sel = false;
        }
    }

    /// Get selected suggestions
    pub fn get_selected(&self) -> Vec<&'static Suggestion> {
        self.suggestions
            .iter()
            .zip(self.selections.iter())
            .filter(|(_, &selected)| selected)
            .map(|(&suggestion, _)| suggestion)
            .collect()
    }

    /// Check if any suggestions are selected
    pub fn has_selections(&self) -> bool {
        self.selections.iter().any(|&s| s)
    }

    /// Apply selected suggestions to a prompt
    pub fn apply_to_prompt(&self, original: &str) -> String {
        let selected = self.get_selected();
        if selected.is_empty() {
            return original.to_string();
        }

        let mut enhanced = original.trim().to_string();
        enhanced.push('\n');

        for suggestion in selected {
            enhanced.push_str(suggestion.template);
            enhanced.push('\n');
        }

        enhanced
    }

    /// Dismiss the modal
    pub fn dismiss(&mut self) {
        self.visible = false;
    }
}

/// Render the suggest modal
pub fn render_suggest_modal(frame: &mut Frame, state: &SuggestModalState) {
    if !state.visible || state.suggestions.is_empty() {
        return;
    }

    let theme = theme();

    // Use percentages for centered_rect (it expects percent values 0-100)
    // Width: 80% of screen, Height: 80% of screen to ensure all content fits
    let modal_width_percent = 80;
    let modal_height_percent = 80;

    // Create centered area for modal
    let area = centered_rect(modal_width_percent, modal_height_percent, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    // Build title with trigger issues
    let title = format!(
        " ⚠ Vague Prompt Detected ({}) ",
        state.trigger_issues.join(", ")
    );

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(theme.warning.fg.unwrap_or_default()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.warning.fg.unwrap_or_default()));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Build content
    let mut lines = vec![
        Line::from(Span::styled(
            "This prompt lacks specific guidance. Claude 4.5 works best",
            theme.text,
        )),
        Line::from(Span::styled(
            "with explicit instructions. Select improvements to add:",
            theme.text,
        )),
        Line::from(""),
    ];

    // Add suggestions with checkboxes
    for (idx, suggestion) in state.suggestions.iter().enumerate() {
        let is_selected = state.selections.get(idx).copied().unwrap_or(false);
        let is_cursor = idx == state.cursor;

        let checkbox = if is_selected { "[✓]" } else { "[ ]" };

        let line_style = if is_cursor {
            Style::default()
                .fg(theme.primary.fg.unwrap_or_default())
                .add_modifier(Modifier::BOLD)
        } else {
            theme.text
        };

        let cursor_indicator = if is_cursor { "▸ " } else { "  " };

        lines.push(Line::from(vec![
            Span::styled(cursor_indicator, line_style),
            Span::styled(checkbox, line_style),
            Span::styled(" ", Style::default()),
            Span::styled(suggestion.label, line_style),
        ]));

        // Description on next line (indented)
        lines.push(Line::from(Span::styled(
            format!("      {}", suggestion.description),
            theme.muted,
        )));
    }

    // Add footer with keybindings
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ↑/↓ ", theme.key),
        Span::styled("Navigate  ", theme.muted),
        Span::styled("Space ", theme.key),
        Span::styled("Toggle  ", theme.muted),
        Span::styled("Enter ", theme.key),
        Span::styled("Apply  ", theme.muted),
        Span::styled("Esc ", theme.key),
        Span::styled("Skip", theme.muted),
    ]));

    // Show selection count
    let selected_count = state.selections.iter().filter(|&&s| s).count();
    if selected_count > 0 {
        lines.push(Line::from(Span::styled(
            format!("  {} improvement(s) selected", selected_count),
            Style::default().fg(theme.success.fg.unwrap_or_default()),
        )));
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });

    frame.render_widget(paragraph, inner_area);
}

/// Handle key events for the suggest modal
/// Returns: (handled, should_apply, should_dismiss)
pub fn handle_suggest_modal_key(
    state: &mut SuggestModalState,
    key: crossterm::event::KeyEvent,
) -> (bool, bool, bool) {
    use crossterm::event::KeyCode;

    if !state.visible {
        return (false, false, false);
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.cursor_up();
            (true, false, false)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.cursor_down();
            (true, false, false)
        }
        KeyCode::Char(' ') => {
            state.toggle_current();
            (true, false, false)
        }
        KeyCode::Enter => {
            // Apply selections and close
            (true, true, true)
        }
        KeyCode::Esc => {
            // Skip/dismiss without applying
            state.dismiss();
            (true, false, true)
        }
        KeyCode::Char('a') => {
            // Select all
            state.select_all();
            (true, false, false)
        }
        KeyCode::Char('n') => {
            // Deselect all (none)
            state.deselect_all();
            (true, false, false)
        }
        _ => (false, false, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::Severity;
    use crate::Issue;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn make_issue(id: &str) -> Issue {
        Issue {
            id: id.to_string(),
            category: "explicitness".to_string(),
            severity: Severity::Warning,
            message: "Test issue".to_string(),
            line: None,
            suggestion: None,
        }
    }

    #[test]
    fn test_suggest_modal_state_from_exp005() {
        let issues = vec![make_issue("EXP005")];
        let state = SuggestModalState::from_issues(&issues);

        assert!(state.visible);
        assert!(!state.suggestions.is_empty());
        assert!(state.trigger_issues.contains(&"EXP005".to_string()));
    }

    #[test]
    fn test_suggest_modal_state_from_exp006() {
        let issues = vec![make_issue("EXP006")];
        let state = SuggestModalState::from_issues(&issues);

        assert!(state.visible);
        assert!(!state.suggestions.is_empty());
        assert!(state.trigger_issues.contains(&"EXP006".to_string()));
    }

    #[test]
    fn test_suggest_modal_not_visible_for_other_issues() {
        let issues = vec![make_issue("EXP001"), make_issue("STY001")];
        let state = SuggestModalState::from_issues(&issues);

        assert!(!state.visible);
        assert!(state.suggestions.is_empty());
    }

    #[test]
    fn test_cursor_navigation() {
        let issues = vec![make_issue("EXP005")];
        let mut state = SuggestModalState::from_issues(&issues);

        assert_eq!(state.cursor, 0);

        state.cursor_down();
        assert_eq!(state.cursor, 1);

        state.cursor_up();
        assert_eq!(state.cursor, 0);

        // Should not go negative
        state.cursor_up();
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn test_toggle_selection() {
        let issues = vec![make_issue("EXP005")];
        let mut state = SuggestModalState::from_issues(&issues);

        assert!(!state.selections[0]);

        state.toggle_current();
        assert!(state.selections[0]);

        state.toggle_current();
        assert!(!state.selections[0]);
    }

    #[test]
    fn test_select_all_deselect_all() {
        let issues = vec![make_issue("EXP005"), make_issue("EXP006")];
        let mut state = SuggestModalState::from_issues(&issues);

        assert!(!state.has_selections());

        state.select_all();
        assert!(state.selections.iter().all(|&s| s));
        assert!(state.has_selections());

        state.deselect_all();
        assert!(state.selections.iter().all(|&s| !s));
        assert!(!state.has_selections());
    }

    #[test]
    fn test_apply_to_prompt() {
        let issues = vec![make_issue("EXP005")];
        let mut state = SuggestModalState::from_issues(&issues);

        let original = "You are an assistant.";

        // No selections - should return original
        let result = state.apply_to_prompt(original);
        assert_eq!(result.trim(), original);

        // With selection - should append template
        state.toggle_current();
        let result = state.apply_to_prompt(original);
        assert!(result.starts_with(original));
        assert!(result.len() > original.len());
    }

    #[test]
    fn test_render_suggest_modal() {
        let backend = TestBackend::new(100, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        let issues = vec![make_issue("EXP005")];
        let state = SuggestModalState::from_issues(&issues);

        terminal
            .draw(|frame| {
                render_suggest_modal(frame, &state);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<String>();

        // Modal should show the trigger issue
        assert!(content.contains("EXP005"));
        // Modal should show at least some content (title or suggestions visible)
        assert!(
            content.contains("Vague") || content.contains("Response"),
            "Modal content should be visible"
        );
    }

    #[test]
    fn test_render_modal_not_visible() {
        let backend = TestBackend::new(80, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        let state = SuggestModalState::default();

        terminal
            .draw(|frame| {
                render_suggest_modal(frame, &state);
            })
            .unwrap();

        // Should render without panic when not visible
    }
}

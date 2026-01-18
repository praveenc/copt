//! Model (state) definitions for the TUI
//!
//! Implements the Model part of the Elm (MVU) architecture.

#![allow(dead_code)]

use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::analyzer::Issue;
use crate::tui::widgets::SuggestModalState;
use crate::OptimizationStats;

/// Current view being displayed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Main,
    Diff,
    Help,
}

/// Render mode based on CLI flags and environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderMode {
    /// Full-screen interactive mode (--interactive flag)
    Interactive,
    /// Enhanced linear output (default TTY)
    #[default]
    Linear,
    /// Plain text output (non-TTY, piped)
    Plain,
    /// JSON output (--format json)
    Json,
    /// Quiet mode (--quiet)
    Quiet,
}

/// Application state for the analysis phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppPhase {
    /// Initial state, ready to analyze
    #[default]
    Ready,
    /// Analyzing prompt
    Analyzing,
    /// Analysis complete, ready to optimize
    AnalysisDone,
    /// Optimizing with LLM
    Optimizing,
    /// Optimization complete
    Done,
    /// Error occurred
    Error,
}

/// A category node in the collapsible issue tree
#[derive(Debug, Clone)]
pub struct CategoryNode {
    pub category: String,
    pub display_name: String,
    pub issues: Vec<Issue>,
    pub expanded: bool,
}

impl CategoryNode {
    pub fn new(category: String, issues: Vec<Issue>) -> Self {
        let display_name = format_category_name(&category);
        Self {
            category,
            display_name,
            issues,
            expanded: true, // Start expanded by default
        }
    }

    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }
}

/// The collapsible issue tree
#[derive(Debug, Clone, Default)]
pub struct IssueTree {
    pub categories: Vec<CategoryNode>,
    pub selected_index: usize,
    /// Tracks which index in the flattened view is selected
    pub flat_index: usize,
}

impl IssueTree {
    /// Create a new issue tree from a list of issues
    pub fn from_issues(issues: &[Issue]) -> Self {
        use std::collections::HashMap;

        // Group issues by category
        let mut grouped: HashMap<String, Vec<Issue>> = HashMap::new();
        for issue in issues {
            grouped
                .entry(issue.category.clone())
                .or_default()
                .push(issue.clone());
        }

        // Convert to CategoryNodes, sorted by category name
        let mut categories: Vec<CategoryNode> = grouped
            .into_iter()
            .map(|(cat, issues)| CategoryNode::new(cat, issues))
            .collect();
        categories.sort_by(|a, b| a.category.cmp(&b.category));

        Self {
            categories,
            selected_index: 0,
            flat_index: 0,
        }
    }

    /// Get total number of items in the flattened view
    pub fn flat_len(&self) -> usize {
        self.categories
            .iter()
            .map(|c| 1 + if c.expanded { c.issues.len() } else { 0 })
            .sum()
    }

    /// Check if a category at given index is expanded
    pub fn is_category_at(&self, flat_idx: usize) -> bool {
        let mut idx = 0;
        for cat in &self.categories {
            if idx == flat_idx {
                return true;
            }
            idx += 1;
            if cat.expanded {
                idx += cat.issues.len();
            }
        }
        false
    }

    /// Toggle expansion of the category containing the current selection
    pub fn toggle_current(&mut self) {
        let mut idx = 0;
        for cat in &mut self.categories {
            if idx == self.flat_index {
                cat.expanded = !cat.expanded;
                return;
            }
            idx += 1;
            if cat.expanded {
                idx += cat.issues.len();
            }
        }
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        if self.flat_index > 0 {
            self.flat_index -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.flat_index < self.flat_len().saturating_sub(1) {
            self.flat_index += 1;
        }
    }

    /// Collapse all categories
    pub fn collapse_all(&mut self) {
        for cat in &mut self.categories {
            cat.expanded = false;
        }
        self.flat_index = 0;
    }

    /// Expand all categories
    pub fn expand_all(&mut self) {
        for cat in &mut self.categories {
            cat.expanded = true;
        }
    }

    /// Get expanded categories (for display)
    pub fn expanded_categories(&self) -> HashSet<&str> {
        self.categories
            .iter()
            .filter(|c| c.expanded)
            .map(|c| c.category.as_str())
            .collect()
    }
}

/// Error state for display
#[derive(Debug, Clone, Default)]
pub struct ErrorState {
    pub message: String,
    pub details: Option<String>,
}

impl ErrorState {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// Main application model (state)
#[derive(Debug, Clone)]
pub struct Model {
    /// Current render mode
    pub render_mode: RenderMode,
    /// Current view
    pub current_view: View,
    /// Current phase
    pub phase: AppPhase,
    /// Whether offline mode is enabled
    pub offline_mode: bool,
    /// Original prompt text
    pub original_prompt: String,
    /// Optimized prompt text (if available)
    pub optimized_prompt: Option<String>,
    /// Issue tree for analysis results
    pub issue_tree: IssueTree,
    /// Optimization statistics
    pub stats: Option<OptimizationStats>,
    /// Error state (if any)
    pub error: Option<ErrorState>,
    /// Input file path (if provided)
    pub input_file: Option<String>,
    /// Scroll offset for content
    pub scroll_offset: u16,
    /// Whether to show the diff view
    pub show_diff: bool,
    /// Should the app quit?
    pub should_quit: bool,
    /// Terminal width (updated on resize)
    pub terminal_width: u16,
    /// Terminal height (updated on resize)
    pub terminal_height: u16,
    /// Suggest modal state for vague prompt improvements
    pub suggest_modal: SuggestModalState,
    /// Temporary status message (e.g., "Copied to clipboard")
    pub status_message: Option<String>,
    /// When to auto-clear the status message
    pub status_clear_at: Option<Instant>,
}

impl Model {
    /// Create a new model with default values
    pub fn new() -> Self {
        Self {
            render_mode: RenderMode::default(),
            current_view: View::default(),
            phase: AppPhase::default(),
            offline_mode: false,
            original_prompt: String::new(),
            optimized_prompt: None,
            issue_tree: IssueTree::default(),
            stats: None,
            error: None,
            input_file: None,
            scroll_offset: 0,
            show_diff: false,
            should_quit: false,
            terminal_width: 80,
            terminal_height: 24,
            suggest_modal: SuggestModalState::default(),
            status_message: None,
            status_clear_at: None,
        }
    }

    /// Set the issues from analysis
    pub fn set_issues(&mut self, issues: &[Issue]) {
        self.issue_tree = IssueTree::from_issues(issues);
        self.phase = AppPhase::AnalysisDone;

        // Initialize suggest modal if vague prompt detected
        if SuggestModalState::should_show(issues) {
            self.suggest_modal = SuggestModalState::from_issues(issues);
        }
    }

    /// Set the optimization result
    pub fn set_optimization_result(&mut self, optimized: String, stats: OptimizationStats) {
        self.optimized_prompt = Some(optimized);
        self.stats = Some(stats);
        self.phase = AppPhase::Done;
        // Default to Diff view when optimization completes (better UX - user sees changes immediately)
        self.current_view = View::Diff;
    }

    /// Set error state
    pub fn set_error(&mut self, error: ErrorState) {
        self.error = Some(error);
        self.phase = AppPhase::Error;
    }

    /// Clear error state
    pub fn clear_error(&mut self) {
        self.error = None;
        // Restore previous phase based on state
        if self.optimized_prompt.is_some() {
            self.phase = AppPhase::Done;
        } else if !self.issue_tree.categories.is_empty() {
            self.phase = AppPhase::AnalysisDone;
        } else {
            self.phase = AppPhase::Ready;
        }
    }

    /// Check if we have optimization results
    pub fn has_results(&self) -> bool {
        self.optimized_prompt.is_some()
    }

    /// Get total issue count
    pub fn total_issues(&self) -> usize {
        self.issue_tree
            .categories
            .iter()
            .map(|c| c.issues.len())
            .sum()
    }

    /// Set a temporary status message that auto-clears after duration
    pub fn set_status_message(&mut self, message: impl Into<String>, duration: Duration) {
        self.status_message = Some(message.into());
        self.status_clear_at = Some(Instant::now() + duration);
    }

    /// Clear the status message
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
        self.status_clear_at = None;
    }

    /// Check if status message should be cleared (expired)
    pub fn check_status_expiry(&mut self) -> bool {
        if let Some(clear_at) = self.status_clear_at {
            if Instant::now() >= clear_at {
                self.clear_status_message();
                return true;
            }
        }
        false
    }

    /// Check if the currently selected item in issue tree is a category header
    pub fn is_current_selection_category(&self) -> bool {
        self.issue_tree.is_category_at(self.issue_tree.flat_index)
    }

    /// Check if the currently selected category is expanded
    pub fn is_current_category_expanded(&self) -> Option<bool> {
        let mut idx = 0;
        for cat in &self.issue_tree.categories {
            if idx == self.issue_tree.flat_index {
                return Some(cat.expanded);
            }
            idx += 1;
            if cat.expanded {
                idx += cat.issues.len();
            }
        }
        None
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}

/// Format category name for display
fn format_category_name(category: &str) -> String {
    match category.to_lowercase().as_str() {
        "explicitness" => "Explicitness".to_string(),
        "style" => "Style".to_string(),
        "tools" => "Tool Usage".to_string(),
        "formatting" => "Formatting".to_string(),
        "verbosity" => "Verbosity".to_string(),
        "agentic" => "Agentic Coding".to_string(),
        "long_horizon" => "Long-Horizon".to_string(),
        "frontend" => "Frontend Design".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::Severity;

    fn create_test_issues() -> Vec<Issue> {
        vec![
            Issue {
                id: "EXP001".to_string(),
                category: "explicitness".to_string(),
                severity: Severity::Warning,
                message: "Test issue 1".to_string(),
                line: Some(1),
                suggestion: Some("Fix it".to_string()),
            },
            Issue {
                id: "EXP002".to_string(),
                category: "explicitness".to_string(),
                severity: Severity::Info,
                message: "Test issue 2".to_string(),
                line: Some(2),
                suggestion: Some("Fix it too".to_string()),
            },
            Issue {
                id: "STY001".to_string(),
                category: "style".to_string(),
                severity: Severity::Error,
                message: "Style issue".to_string(),
                line: None,
                suggestion: Some("Restyle".to_string()),
            },
        ]
    }

    #[test]
    fn test_issue_tree_from_issues() {
        let issues = create_test_issues();
        let tree = IssueTree::from_issues(&issues);

        assert_eq!(tree.categories.len(), 2);
    }

    #[test]
    fn test_issue_tree_navigation() {
        let issues = create_test_issues();
        let mut tree = IssueTree::from_issues(&issues);

        assert_eq!(tree.flat_index, 0);
        tree.select_next();
        assert_eq!(tree.flat_index, 1);
        tree.select_prev();
        assert_eq!(tree.flat_index, 0);
    }

    #[test]
    fn test_model_creation() {
        let model = Model::new();
        assert_eq!(model.phase, AppPhase::Ready);
        assert!(!model.should_quit);
    }

    #[test]
    fn test_format_category_name() {
        assert_eq!(format_category_name("explicitness"), "Explicitness");
        assert_eq!(format_category_name("long_horizon"), "Long-Horizon");
        assert_eq!(format_category_name("unknown"), "unknown");
    }
}

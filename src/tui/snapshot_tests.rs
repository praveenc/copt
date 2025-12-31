//! Snapshot tests for TUI widgets
//!
//! Uses insta for snapshot testing and ratatui's TestBackend for rendering.

#![cfg(test)]

use insta::assert_snapshot;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

use super::model::{AppPhase, Model, View};
use super::view::render;
use crate::analyzer::{Issue, Severity};
use crate::OptimizationStats;

/// Render the TUI to a string buffer for snapshot testing
fn render_to_string(model: &Model, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|frame| {
            render(frame, model);
        })
        .unwrap();

    // Convert buffer to string
    let buffer = terminal.backend().buffer();
    let mut output = String::new();

    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let cell = &buffer[(x, y)];
            output.push_str(cell.symbol());
        }
        output.push('\n');
    }

    output
}

/// Create a model with test data for snapshot testing
fn create_test_model() -> Model {
    let mut model = Model::new();
    model.original_prompt = "You should try to think carefully about this problem.".to_string();
    model.input_file = Some("test_prompt.txt".to_string());
    model.terminal_width = 80;
    model.terminal_height = 24;

    // Add test issues
    let issues = vec![
        Issue {
            id: "STY003".to_string(),
            category: "style".to_string(),
            severity: Severity::Warning,
            message: "Word 'think' detected - sensitive in Claude Opus".to_string(),
            line: Some(1),
            suggestion: Some("Consider rephrasing".to_string()),
        },
        Issue {
            id: "EXP001".to_string(),
            category: "explicitness".to_string(),
            severity: Severity::Info,
            message: "Vague instruction detected".to_string(),
            line: Some(1),
            suggestion: Some("Be more specific".to_string()),
        },
    ];
    model.set_issues(&issues);

    model
}

/// Create a model with optimization results
fn create_optimized_model() -> Model {
    let mut model = create_test_model();
    model.optimized_prompt = Some(
        "Please analyze this problem systematically and provide a detailed solution.".to_string(),
    );
    model.stats = Some(OptimizationStats {
        original_chars: 54,
        optimized_chars: 76,
        original_tokens: 12,
        optimized_tokens: 15,
        rules_applied: 2,
        categories_improved: 2,
        processing_time_ms: 1234,
        provider: "bedrock".to_string(),
        model: "claude-sonnet-4".to_string(),
    });
    model.phase = AppPhase::Done;
    model
}

#[test]
fn test_main_view_analysis_done() {
    // Note: We test AnalysisDone instead of Analyzing because Analyzing uses
    // a time-based spinner that makes snapshots non-deterministic
    let mut model = create_test_model();
    model.phase = AppPhase::AnalysisDone;
    let output = render_to_string(&model, 80, 24);
    assert_snapshot!("main_view_analysis_done", output);
}

#[test]
fn test_main_view_done() {
    let model = create_optimized_model();
    let output = render_to_string(&model, 80, 24);
    assert_snapshot!("main_view_done", output);
}

#[test]
fn test_diff_view() {
    let mut model = create_optimized_model();
    model.current_view = View::Diff;
    let output = render_to_string(&model, 80, 24);
    assert_snapshot!("diff_view", output);
}

#[test]
fn test_help_view() {
    let mut model = create_test_model();
    model.current_view = View::Help;
    let output = render_to_string(&model, 80, 24);
    assert_snapshot!("help_view", output);
}

#[test]
fn test_small_terminal() {
    let model = create_test_model();
    let output = render_to_string(&model, 50, 12);
    assert_snapshot!("small_terminal", output);
}

#[test]
fn test_offline_mode() {
    let mut model = create_test_model();
    model.offline_mode = true;
    model.phase = AppPhase::AnalysisDone;
    let output = render_to_string(&model, 80, 24);
    assert_snapshot!("offline_mode", output);
}

#[test]
fn test_no_issues() {
    let mut model = Model::new();
    model.original_prompt = "Clear, well-structured prompt.".to_string();
    model.phase = AppPhase::AnalysisDone;
    let output = render_to_string(&model, 80, 24);
    assert_snapshot!("no_issues", output);
}

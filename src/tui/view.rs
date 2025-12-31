//! View (rendering) for the TUI
//!
//! Implements the View part of the Elm (MVU) architecture.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

use super::model::{Model, View};
use super::widgets;

/// Minimum terminal dimensions for full layout
const MIN_WIDTH: u16 = 60;
const MIN_HEIGHT: u16 = 15;

/// Render the entire UI
pub fn render(frame: &mut Frame, model: &Model) {
    let size = frame.area();

    // Check if terminal is too small
    if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
        render_minimal(frame, model);
        return;
    }

    // Render the appropriate view
    match model.current_view {
        View::Main => render_main(frame, model),
        View::Diff => render_diff(frame, model),
        View::Help => render_help(frame, model),
    }

    // Render error modal on top if there's an error
    if model.error.is_some() {
        widgets::render_error_modal(frame, model);
    }
}

/// Render the main view with header, analysis, stats, and status bar
fn render_main(frame: &mut Frame, model: &Model) {
    let size = frame.area();

    // Create layout: header, content, status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // Header
            Constraint::Min(10),    // Content (analysis + stats)
            Constraint::Length(1),  // Status bar
        ])
        .split(size);

    // Render header
    widgets::render_header(frame, chunks[0], model);

    // Split content area between analysis and stats
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Analysis
            Constraint::Percentage(40), // Stats
        ])
        .split(chunks[1]);

    // Render analysis tree
    widgets::render_analysis(frame, content_chunks[0], model);

    // Render stats dashboard (if we have stats)
    if model.stats.is_some() {
        widgets::render_dashboard(frame, content_chunks[1], model);
    } else {
        // Show progress or placeholder
        widgets::render_progress(frame, content_chunks[1], model);
    }

    // Render status bar
    widgets::render_status_bar(frame, chunks[2], model);
}

/// Render the diff view
fn render_diff(frame: &mut Frame, model: &Model) {
    let size = frame.area();

    // Create layout: header, diff content, status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Diff content
            Constraint::Length(1),  // Status bar
        ])
        .split(size);

    // Render compact header
    widgets::render_header_compact(frame, chunks[0], model);

    // Render diff
    widgets::render_diff(frame, chunks[1], model);

    // Render status bar (diff mode)
    widgets::render_status_bar_diff(frame, chunks[2], model);
}

/// Render the help view
fn render_help(frame: &mut Frame, model: &Model) {
    let size = frame.area();

    // Create layout: header, help content, status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Help content
            Constraint::Length(1),  // Status bar
        ])
        .split(size);

    // Render compact header
    widgets::render_header_compact(frame, chunks[0], model);

    // Render help content
    widgets::render_help(frame, chunks[1], model);

    // Render status bar (help mode)
    widgets::render_status_bar_help(frame, chunks[2], model);
}

/// Render minimal layout for small terminals
fn render_minimal(frame: &mut Frame, model: &Model) {
    let size = frame.area();

    // Just show a summary and status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(2),     // Minimal content
            Constraint::Length(1),  // Status bar
        ])
        .split(size);

    // Render minimal summary
    widgets::render_minimal_summary(frame, chunks[0], model);

    // Render minimal status bar
    widgets::render_status_bar(frame, chunks[1], model);
}

/// Helper to create a centered rect for modals
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centered_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let centered = centered_rect(50, 50, area);

        // Should be roughly centered
        assert!(centered.x > 0);
        assert!(centered.y > 0);
        assert!(centered.width < area.width);
        assert!(centered.height < area.height);
    }
}

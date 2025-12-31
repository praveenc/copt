//! Application main loop
//!
//! Implements the main event loop using the Elm (MVU) architecture.

#![allow(dead_code)]

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};

use super::model::{Model, RenderMode};
use super::terminal;
use super::update::{update, Msg};
use super::view::render;

/// Run the interactive TUI application
pub fn run_interactive(model: &mut Model) -> io::Result<()> {
    // Initialize safety measures (panic hooks, signal handlers)
    terminal::init_safety()?;

    // Create terminal guard for cleanup on drop
    let _guard = terminal::TerminalGuard::new();

    // Initialize terminal
    let mut terminal = terminal::init()?;

    // Update terminal size in model
    let size = terminal.size()?;
    model.terminal_width = size.width;
    model.terminal_height = size.height;

    // Main event loop
    loop {
        // Render
        terminal.draw(|frame| render(frame, model))?;

        // Handle events with timeout
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    update(model, Msg::Key(key));
                }
                Event::Resize(width, height) => {
                    update(model, Msg::Resize(width, height));
                }
                _ => {}
            }
        }

        // Check if we should quit
        if model.should_quit {
            break;
        }
    }

    // Terminal will be restored by TerminalGuard drop

    Ok(())
}

/// Run non-interactive linear mode (enhanced output)
pub fn run_linear(model: &Model) -> io::Result<()> {
    use super::linear;
    linear::render(model)
}

/// Determine the render mode from CLI flags and environment
pub fn detect_render_mode(
    interactive: bool,
    quiet: bool,
    format_json: bool,
    is_tty: bool,
) -> RenderMode {
    if format_json {
        RenderMode::Json
    } else if quiet {
        RenderMode::Quiet
    } else if !is_tty {
        RenderMode::Plain
    } else if interactive {
        RenderMode::Interactive
    } else {
        RenderMode::Linear
    }
}

/// Main entry point for the TUI
///
/// Chooses between interactive and linear mode based on render mode.
pub fn run(model: &mut Model) -> io::Result<()> {
    match model.render_mode {
        RenderMode::Interactive => run_interactive(model),
        RenderMode::Linear => run_linear(model),
        RenderMode::Plain | RenderMode::Json | RenderMode::Quiet => {
            // These modes don't use the TUI - handled by main.rs
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_render_mode_json() {
        assert_eq!(
            detect_render_mode(false, false, true, true),
            RenderMode::Json
        );
    }

    #[test]
    fn test_detect_render_mode_quiet() {
        assert_eq!(
            detect_render_mode(false, true, false, true),
            RenderMode::Quiet
        );
    }

    #[test]
    fn test_detect_render_mode_non_tty() {
        assert_eq!(
            detect_render_mode(false, false, false, false),
            RenderMode::Plain
        );
    }

    #[test]
    fn test_detect_render_mode_interactive() {
        assert_eq!(
            detect_render_mode(true, false, false, true),
            RenderMode::Interactive
        );
    }

    #[test]
    fn test_detect_render_mode_linear() {
        assert_eq!(
            detect_render_mode(false, false, false, true),
            RenderMode::Linear
        );
    }
}

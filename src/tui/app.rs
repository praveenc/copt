//! Application main loop
//!
//! Implements the main event loop using the Elm (MVU) architecture.

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};

use super::model::Model;
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
        } else {
            // No event - send Tick for periodic updates (e.g., status message expiry)
            update(model, Msg::Tick);
        }

        // Check if we should quit
        if model.should_quit {
            break;
        }
    }

    // Terminal will be restored by TerminalGuard drop

    Ok(())
}

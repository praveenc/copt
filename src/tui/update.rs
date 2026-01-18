//! Update (message handling) for the TUI
//!
//! Implements the Update part of the Elm (MVU) architecture.

#![allow(dead_code)]

use std::time::Duration;

use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::model::{Model, View};
use super::widgets::handle_suggest_modal_key;

/// Messages that can be sent to update the model
#[derive(Debug, Clone)]
pub enum Msg {
    /// Key press event
    Key(KeyEvent),
    /// Terminal resized
    Resize(u16, u16),
    /// Tick for animations
    Tick,
    /// Quit the application
    Quit,
}

/// Update the model based on a message
///
/// Returns true if a redraw is needed
pub fn update(model: &mut Model, msg: Msg) -> bool {
    match msg {
        Msg::Key(key) => handle_key(model, key),
        Msg::Resize(width, height) => {
            model.terminal_width = width;
            model.terminal_height = height;
            true // Always redraw on resize
        }
        Msg::Tick => {
            // Check if status message should be cleared
            model.check_status_expiry()
        }
        Msg::Quit => {
            model.should_quit = true;
            false
        }
    }
}

/// Handle key press events
fn handle_key(model: &mut Model, key: KeyEvent) -> bool {
    // Handle error modal first
    if model.error.is_some() {
        return handle_error_keys(model, key);
    }

    // Handle suggest modal if visible
    if model.suggest_modal.visible {
        let (handled, should_apply, _dismissed) =
            handle_suggest_modal_key(&mut model.suggest_modal, key);
        if handled {
            // If user applied suggestions, update the prompt
            if should_apply && model.suggest_modal.has_selections() {
                let enhanced = model.suggest_modal.apply_to_prompt(&model.original_prompt);
                model.original_prompt = enhanced;
            }
            return true;
        }
    }

    // Global keys (work in any view)
    match key.code {
        KeyCode::Char('q') => {
            model.should_quit = true;
            return false;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            model.should_quit = true;
            return false;
        }
        _ => {}
    }

    // View-specific key handling
    match model.current_view {
        View::Main => handle_main_keys(model, key),
        View::Diff => handle_diff_keys(model, key),
        View::Help => handle_help_keys(model, key),
    }
}

/// Handle keys in the error modal
fn handle_error_keys(model: &mut Model, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Enter | KeyCode::Esc => {
            model.clear_error();
            true
        }
        _ => false,
    }
}

/// Handle keys in the main view
fn handle_main_keys(model: &mut Model, key: KeyEvent) -> bool {
    match key.code {
        // Navigation
        KeyCode::Up | KeyCode::Char('k') => {
            model.issue_tree.select_prev();
            true
        }
        KeyCode::Down | KeyCode::Char('j') => {
            model.issue_tree.select_next();
            true
        }
        KeyCode::Enter => {
            model.issue_tree.toggle_current();
            true
        }

        // View switching
        KeyCode::Char('d') if model.has_results() => {
            model.current_view = View::Diff;
            true
        }
        KeyCode::Char('?') => {
            model.current_view = View::Help;
            true
        }

        // Actions (only when results available)
        KeyCode::Char('c') if model.has_results() => handle_copy(model),
        KeyCode::Char('s') if model.has_results() => handle_save(model),
        KeyCode::Char('r') if model.has_results() => {
            // Re-run - would need async handling
            false
        }

        // Scroll
        KeyCode::PageUp => {
            model.scroll_offset = model.scroll_offset.saturating_sub(10);
            true
        }
        KeyCode::PageDown => {
            model.scroll_offset = model.scroll_offset.saturating_add(10);
            true
        }
        KeyCode::Home => {
            model.scroll_offset = 0;
            true
        }

        _ => false,
    }
}

/// Handle keys in the diff view
fn handle_diff_keys(model: &mut Model, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('d') => {
            model.current_view = View::Main;
            true
        }
        KeyCode::Char('c') => handle_copy(model),
        KeyCode::Char('s') => handle_save(model),
        KeyCode::Up => {
            model.scroll_offset = model.scroll_offset.saturating_sub(1);
            true
        }
        KeyCode::Down => {
            model.scroll_offset = model.scroll_offset.saturating_add(1);
            true
        }
        KeyCode::PageUp => {
            model.scroll_offset = model.scroll_offset.saturating_sub(10);
            true
        }
        KeyCode::PageDown => {
            model.scroll_offset = model.scroll_offset.saturating_add(10);
            true
        }
        _ => false,
    }
}

/// Handle keys in the help view
fn handle_help_keys(model: &mut Model, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Enter => {
            model.current_view = View::Main;
            true
        }
        _ => false,
    }
}

/// Handle copy to clipboard action
fn handle_copy(model: &mut Model) -> bool {
    if let Some(ref optimized) = model.optimized_prompt {
        match copy_to_clipboard(optimized) {
            Ok(()) => {
                model.set_status_message("✓ Copied to clipboard", Duration::from_secs(3));
            }
            Err(e) => {
                model.set_status_message(format!("✗ Copy failed: {}", e), Duration::from_secs(5));
            }
        }
        return true; // Trigger redraw to show feedback
    }
    false
}

/// Handle save action
fn handle_save(model: &mut Model) -> bool {
    if let Some(ref optimized) = model.optimized_prompt {
        // Generate output path
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("optimized_{}.txt", timestamp);
        let output_dir = std::path::PathBuf::from("copt-output");
        let output_path = output_dir.join(&filename);

        // Create output directory if needed
        if let Err(e) = std::fs::create_dir_all(&output_dir) {
            model.set_status_message(
                format!("✗ Failed to create directory: {}", e),
                Duration::from_secs(5),
            );
            return true;
        }

        // Write the optimized prompt
        match std::fs::write(&output_path, optimized) {
            Ok(()) => {
                model.set_status_message(
                    format!("✓ Saved to {}", output_path.display()),
                    Duration::from_secs(5),
                );
            }
            Err(e) => {
                model.set_status_message(format!("✗ Save failed: {}", e), Duration::from_secs(5));
            }
        }
        return true; // Trigger redraw to show feedback
    }
    false
}

/// Copy text to system clipboard
fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Try using pbcopy on macOS, xclip on Linux, etc.
    #[cfg(target_os = "macos")]
    {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut child = Command::new("pbcopy").stdin(Stdio::piped()).spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes())?;
        }
        child.wait()?;
    }

    #[cfg(target_os = "linux")]
    {
        use std::io::Write;
        use std::process::{Command, Stdio};

        // Try xclip first, fall back to xsel
        let result = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn();

        let mut child = match result {
            Ok(child) => child,
            Err(_) => Command::new("xsel")
                .args(["--clipboard", "--input"])
                .stdin(Stdio::piped())
                .spawn()?,
        };

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes())?;
        }
        child.wait()?;
    }

    #[cfg(windows)]
    {
        // Windows clipboard handling would go here
        // For now, return an error
        return Err("Clipboard not supported on Windows yet".into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quit_message() {
        let mut model = Model::default();
        update(&mut model, Msg::Quit);
        assert!(model.should_quit);
    }

    #[test]
    fn test_resize_message() {
        let mut model = Model::default();
        let needs_redraw = update(&mut model, Msg::Resize(100, 50));
        assert!(needs_redraw);
        assert_eq!(model.terminal_width, 100);
        assert_eq!(model.terminal_height, 50);
    }

    #[test]
    fn test_quit_key() {
        let mut model = Model::default();
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        handle_key(&mut model, key);
        assert!(model.should_quit);
    }

    #[test]
    fn test_help_toggle() {
        let mut model = Model::default();
        let key = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE);

        handle_key(&mut model, key);
        assert_eq!(model.current_view, View::Help);

        handle_key(&mut model, key);
        assert_eq!(model.current_view, View::Main);
    }
}

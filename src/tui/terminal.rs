//! Terminal management for ratatui
//!
//! Handles terminal initialization, restoration, and panic recovery.
//! Implements "belt + suspenders" approach for robust error recovery.

use std::io::{self, Stdout};
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

/// Flag to track if terminal is in raw mode (for signal handlers)
static TERMINAL_RAW: AtomicBool = AtomicBool::new(false);

/// Type alias for our terminal
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal for interactive mode
///
/// This enters the alternate screen, enables raw mode, and optionally enables mouse capture.
pub fn init() -> io::Result<Tui> {
    enable_raw_mode()?;
    TERMINAL_RAW.store(true, Ordering::SeqCst);

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

/// Restore the terminal to its original state
///
/// This is safe to call multiple times and won't panic.
pub fn restore() {
    // Only restore if we're in raw mode
    if TERMINAL_RAW.swap(false, Ordering::SeqCst) {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
    }
}

/// Install panic hook that restores terminal before printing panic message
///
/// This ensures the terminal is usable even after a panic.
pub fn install_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        restore();
        original_hook(panic_info);
    }));
}

/// Install signal handlers for clean shutdown
///
/// Handles SIGINT (Ctrl+C) and SIGTERM for graceful termination.
pub fn install_signal_handlers() -> io::Result<()> {
    // Use ctrlc crate for cross-platform signal handling
    ctrlc::set_handler(move || {
        restore();
        std::process::exit(130); // 128 + SIGINT (2)
    })
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok(())
}

/// Initialize all safety measures at once
///
/// Call this before entering the TUI to ensure clean recovery on any exit path.
pub fn init_safety() -> io::Result<()> {
    install_panic_hook();
    install_signal_handlers()?;
    Ok(())
}

/// RAII guard that restores terminal on drop
///
/// Use this to ensure terminal restoration even with early returns.
pub struct TerminalGuard {
    _private: (),
}

impl TerminalGuard {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore();
    }
}

impl Default for TerminalGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restore_is_idempotent() {
        // Calling restore multiple times should be safe
        restore();
        restore();
        restore();
    }

    #[test]
    fn test_terminal_guard_creation() {
        let _guard = TerminalGuard::new();
        // Guard should drop cleanly
    }
}

//! Spinner and progress indicator components
//!
//! Provides animated spinners and progress bars for long-running operations.

use std::io::{self, Write};

/// Spinner animation frames
pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Dots animation frames
pub const DOTS_FRAMES: &[&str] = &["   ", ".  ", ".. ", "..."];

/// A simple spinner for terminal output
pub struct Spinner {
    message: String,
    frames: &'static [&'static str],
    current_frame: usize,
    active: bool,
}

impl Spinner {
    /// Create a new spinner with a message
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            frames: SPINNER_FRAMES,
            current_frame: 0,
            active: false,
        }
    }

    /// Create a spinner with custom frames
    pub fn with_frames(message: &str, frames: &'static [&'static str]) -> Self {
        Self {
            message: message.to_string(),
            frames,
            current_frame: 0,
            active: false,
        }
    }

    /// Start the spinner (in a real impl, this would spawn a thread)
    pub fn start(&mut self) {
        self.active = true;
        self.render();
    }

    /// Stop the spinner
    pub fn stop(&mut self) {
        self.active = false;
        self.clear_line();
    }

    /// Stop with a success message
    pub fn success(&mut self, message: &str) {
        self.stop();
        println!("\x1b[32m✓\x1b[0m {}", message);
    }

    /// Stop with a failure message
    pub fn fail(&mut self, message: &str) {
        self.stop();
        println!("\x1b[31m✗\x1b[0m {}", message);
    }

    /// Update the spinner message
    pub fn set_message(&mut self, message: &str) {
        self.message = message.to_string();
        if self.active {
            self.render();
        }
    }

    /// Advance to the next frame
    pub fn tick(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.frames.len();
        if self.active {
            self.render();
        }
    }

    /// Render the current frame
    fn render(&self) {
        let frame = self.frames[self.current_frame];
        print!("\r\x1b[36m{}\x1b[0m {}", frame, self.message);
        let _ = io::stdout().flush();
    }

    /// Clear the current line
    fn clear_line(&self) {
        print!("\r\x1b[K");
        let _ = io::stdout().flush();
    }
}

/// Progress bar for tracking completion
pub struct ProgressBar {
    total: usize,
    current: usize,
    width: usize,
    message: String,
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new(total: usize) -> Self {
        Self {
            total,
            current: 0,
            width: 40,
            message: String::new(),
        }
    }

    /// Create with a message
    pub fn with_message(total: usize, message: &str) -> Self {
        Self {
            total,
            current: 0,
            width: 40,
            message: message.to_string(),
        }
    }

    /// Set the progress bar width
    pub fn set_width(&mut self, width: usize) {
        self.width = width;
    }

    /// Increment progress by one
    pub fn inc(&mut self) {
        self.current = (self.current + 1).min(self.total);
        self.render();
    }

    /// Set progress to a specific value
    pub fn set(&mut self, value: usize) {
        self.current = value.min(self.total);
        self.render();
    }

    /// Set the message
    pub fn set_message(&mut self, message: &str) {
        self.message = message.to_string();
        self.render();
    }

    /// Finish the progress bar
    pub fn finish(&self) {
        println!();
    }

    /// Finish with a message
    pub fn finish_with_message(&self, message: &str) {
        print!("\r\x1b[K");
        println!("\x1b[32m✓\x1b[0m {}", message);
    }

    /// Get the completion percentage
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            100.0
        } else {
            (self.current as f64 / self.total as f64) * 100.0
        }
    }

    /// Render the progress bar
    fn render(&self) {
        let filled = if self.total > 0 {
            (self.current * self.width) / self.total
        } else {
            self.width
        };
        let empty = self.width - filled;

        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

        print!(
            "\r\x1b[36m{}\x1b[0m {:>3.0}% {}",
            bar,
            self.percentage(),
            self.message
        );
        let _ = io::stdout().flush();
    }
}

/// Print a simple step indicator
pub fn print_step(step: usize, total: usize, message: &str) {
    println!("\x1b[36m[{}/{}]\x1b[0m {}", step, total, message);
}

/// Print a completion checkmark
pub fn print_done(message: &str) {
    println!("\x1b[32m✓\x1b[0m {}", message);
}

/// Print a pending indicator
pub fn print_pending(message: &str) {
    println!("\x1b[33m○\x1b[0m {}", message);
}

/// Print a skipped indicator
pub fn print_skipped(message: &str) {
    println!("\x1b[90m⊘\x1b[0m {}", message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_creation() {
        let spinner = Spinner::new("Loading");
        assert_eq!(spinner.message, "Loading");
        assert!(!spinner.active);
    }

    #[test]
    fn test_progress_bar_percentage() {
        let mut bar = ProgressBar::new(100);
        assert_eq!(bar.percentage(), 0.0);

        bar.set(50);
        assert_eq!(bar.percentage(), 50.0);

        bar.set(100);
        assert_eq!(bar.percentage(), 100.0);
    }

    #[test]
    fn test_progress_bar_empty_total() {
        let bar = ProgressBar::new(0);
        assert_eq!(bar.percentage(), 100.0);
    }
}
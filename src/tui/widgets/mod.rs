//! Widget modules for the TUI
//!
//! Each widget is a separate module for maintainability.

mod analysis;
mod dashboard;
mod diff;
mod header;
mod help;
mod progress;
mod status_bar;

// Re-export all rendering functions
pub use analysis::render_analysis;
pub use dashboard::render_dashboard;
pub use diff::render_diff;
pub use header::{render_header, render_header_compact};
pub use help::render_help;
pub use progress::render_progress;
pub use status_bar::{render_status_bar, render_status_bar_diff, render_status_bar_help};

// Additional utilities
mod error_modal;
mod minimal;

pub use error_modal::render_error_modal;
pub use minimal::render_minimal_summary;

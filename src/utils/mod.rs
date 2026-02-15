//! Utility modules for the prompt optimizer
//!
//! This module provides common utilities used across the application:
//! - Text processing (token counting, text manipulation)
//! - Editor command building

pub mod editor;
pub mod text;

// Re-export commonly used items
pub use text::count_tokens;
pub use text::generate_prompt_slug;

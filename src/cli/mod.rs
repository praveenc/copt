//! CLI module for copt (Claude Optimizer)
//!
//! Handles command-line argument processing and configuration.

#![allow(dead_code)]

pub mod config;

/// Default model to use for optimization (Bedrock inference profile ID)
pub const DEFAULT_MODEL: &str = "us.anthropic.claude-sonnet-4-5-20250929-v1:0";

/// Default max tokens for optimization requests
pub const DEFAULT_MAX_TOKENS: u32 = 4096;

/// Available Claude 4.5 models (Bedrock inference profile IDs)
pub const AVAILABLE_MODELS: &[&str] = &[
    "us.anthropic.claude-sonnet-4-5-20250929-v1:0",
    "global.anthropic.claude-opus-4-5-20251101-v1:0",
    "us.anthropic.claude-haiku-4-5-20251001-v1:0",
];

/// Short aliases for models
pub const MODEL_ALIASES: &[(&str, &str)] = &[
    ("sonnet", "us.anthropic.claude-sonnet-4-5-20250929-v1:0"),
    ("sonnet-4.5", "us.anthropic.claude-sonnet-4-5-20250929-v1:0"),
    ("opus", "global.anthropic.claude-opus-4-5-20251101-v1:0"),
    ("opus-4.5", "global.anthropic.claude-opus-4-5-20251101-v1:0"),
    ("haiku", "us.anthropic.claude-haiku-4-5-20251001-v1:0"),
    ("haiku-4.5", "us.anthropic.claude-haiku-4-5-20251001-v1:0"),
];

/// Resolve a model name or alias to a full model ID
pub fn resolve_model_id(model: &str) -> String {
    // Check if it's an alias
    for (alias, full_id) in MODEL_ALIASES {
        if model.eq_ignore_ascii_case(alias) {
            return full_id.to_string();
        }
    }

    // If it already looks like a full ID, use it directly
    if model.contains("anthropic.claude") {
        return model.to_string();
    }

    // Default: return as-is
    model.to_string()
}

/// Check if a model string is valid
pub fn is_valid_model(model: &str) -> bool {
    // Check direct matches
    if AVAILABLE_MODELS.contains(&model) {
        return true;
    }

    // Check aliases
    for (alias, _) in MODEL_ALIASES {
        if model.eq_ignore_ascii_case(alias) {
            return true;
        }
    }

    // Accept any anthropic model pattern
    model.contains("anthropic.claude")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_model() {
        assert!(is_valid_model("us.anthropic.claude-sonnet-4-5-20250929-v1:0"));
        assert!(is_valid_model("global.anthropic.claude-opus-4-5-20251101-v1:0"));
        assert!(is_valid_model("sonnet"));
        assert!(is_valid_model("opus-4.5"));
        assert!(!is_valid_model("gpt-4"));
    }

    #[test]
    fn test_resolve_model_id() {
        assert_eq!(
            resolve_model_id("sonnet"),
            "us.anthropic.claude-sonnet-4-5-20250929-v1:0"
        );
        assert_eq!(
            resolve_model_id("opus-4.5"),
            "global.anthropic.claude-opus-4-5-20251101-v1:0"
        );
        assert_eq!(
            resolve_model_id("us.anthropic.claude-sonnet-4-5-20250929-v1:0"),
            "us.anthropic.claude-sonnet-4-5-20250929-v1:0"
        );
    }
}
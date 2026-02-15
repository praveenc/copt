//! CLI module for copt (Claude Optimizer)
//!
//! Handles command-line argument processing and configuration.

pub mod config;
pub mod suggest;

// TODO: Wire resolve_model_id() into main.rs (Phase 2 quick win)
/// Short aliases for models
pub const MODEL_ALIASES: &[(&str, &str)] = &[
    ("sonnet", "us.anthropic.claude-sonnet-4-5-20250929-v1:0"),
    ("sonnet-4.5", "us.anthropic.claude-sonnet-4-5-20250929-v1:0"),
    ("opus", "us.anthropic.claude-opus-4-5-20251101-v1:0"),
    ("opus-4.5", "us.anthropic.claude-opus-4-5-20251101-v1:0"),
    ("opus-4.6", "us.anthropic.claude-opus-4-6-v1"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_model_id() {
        assert_eq!(
            resolve_model_id("sonnet"),
            "us.anthropic.claude-sonnet-4-5-20250929-v1:0"
        );
        assert_eq!(
            resolve_model_id("opus-4.5"),
            "us.anthropic.claude-opus-4-5-20251101-v1:0"
        );
        assert_eq!(
            resolve_model_id("opus-4.6"),
            "us.anthropic.claude-opus-4-6-v1"
        );
        assert_eq!(
            resolve_model_id("us.anthropic.claude-sonnet-4-5-20250929-v1:0"),
            "us.anthropic.claude-sonnet-4-5-20250929-v1:0"
        );
    }
}

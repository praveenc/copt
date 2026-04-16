//! CLI module for copt (Claude Optimizer)
//!
//! Handles command-line argument processing and configuration.

pub mod config;
pub mod suggest;

// TODO: Wire resolve_model_id() into main.rs (Phase 2 quick win)
/// Short aliases for models
///
/// Note: `sonnet-4.7` and `haiku-4.7` are reserved placeholders — as of
/// 2026-04-16 only Claude Opus 4.7 is released. Selecting either reserved
/// alias surfaces a clear "not yet released" error at runtime
/// (see `crate::llm::unreleased_model_error`).
pub const MODEL_ALIASES: &[(&str, &str)] = &[
    ("sonnet", "us.anthropic.claude-sonnet-4-5-20250929-v1:0"),
    ("sonnet-4.5", "us.anthropic.claude-sonnet-4-5-20250929-v1:0"),
    ("opus", "us.anthropic.claude-opus-4-5-20251101-v1:0"),
    ("opus-4.5", "us.anthropic.claude-opus-4-5-20251101-v1:0"),
    ("opus-4.6", "us.anthropic.claude-opus-4-6-v1"),
    ("opus-4.7", "global.anthropic.claude-opus-4-7-v1:0"),
    ("haiku", "us.anthropic.claude-haiku-4-5-20251001-v1:0"),
    ("haiku-4.5", "us.anthropic.claude-haiku-4-5-20251001-v1:0"),
    // Reserved — not yet released. Resolver returns the alias unchanged so
    // downstream guards can produce a "not yet released" error.
    ("sonnet-4.7", "sonnet-4.7"),
    ("haiku-4.7", "haiku-4.7"),
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
            resolve_model_id("opus-4.7"),
            "global.anthropic.claude-opus-4-7-v1:0"
        );
        assert_eq!(
            resolve_model_id("us.anthropic.claude-sonnet-4-5-20250929-v1:0"),
            "us.anthropic.claude-sonnet-4-5-20250929-v1:0"
        );
    }

    #[test]
    fn test_resolve_model_id_reserved_aliases() {
        // sonnet-4.7 / haiku-4.7 are reserved; the resolver returns the alias
        // unchanged so a runtime guard can produce a "not yet released" error.
        assert_eq!(resolve_model_id("sonnet-4.7"), "sonnet-4.7");
        assert_eq!(resolve_model_id("haiku-4.7"), "haiku-4.7");
    }
}

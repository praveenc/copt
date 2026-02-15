//! Text processing utilities
//!
//! Provides token counting and prompt slug generation.

use regex::Regex;
use std::sync::LazyLock;

/// Regex to strip XML tags
static XML_TAG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").unwrap());

/// Regex to strip common prompt prefixes (boilerplate only, not action verbs)
static PREFIX_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^(you are an?|act as an?|please|i want you to|help me|i need you to)\s+")
        .unwrap()
});

/// Regex for slugification — anything that isn't alphanumeric
static SLUG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^a-z0-9]+").unwrap());

/// Generate a short descriptive slug from prompt content for use in filenames.
///
/// Extracts the first meaningful phrase, strips boilerplate, and produces
/// a kebab-case slug truncated to ~50 characters at a word boundary.
///
/// # Examples
/// ```
/// use copt::utils::text::generate_prompt_slug;
/// assert_eq!(generate_prompt_slug("Create a dashboard for analytics"), "create-a-dashboard-for-analytics");
/// assert_eq!(generate_prompt_slug("You are an expert. Review this code for bugs"), "expert-review-this-code-for");
/// assert_eq!(generate_prompt_slug(""), "prompt");
/// ```
pub fn generate_prompt_slug(prompt: &str) -> String {
    // Strip XML tags
    let cleaned = XML_TAG_RE.replace_all(prompt, " ");

    // Take the first non-empty line as the most descriptive content
    let first_line = cleaned
        .lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty())
        .unwrap_or("");

    // Strip common prompt prefixes
    let stripped = PREFIX_RE.replace(first_line, "");
    let stripped = stripped.trim();

    // Take first ~6 words
    let words: Vec<&str> = stripped.split_whitespace().take(6).collect();
    let phrase = words.join(" ");

    // Slugify: lowercase, replace non-alphanumeric with hyphens
    let lowered = phrase.to_lowercase();
    let slug = SLUG_RE.replace_all(&lowered, "-");

    // Trim leading/trailing hyphens
    let slug = slug.trim_matches('-');

    if slug.is_empty() {
        return "prompt".to_string();
    }

    // Truncate to 50 chars at a hyphen boundary
    if slug.len() <= 50 {
        slug.to_string()
    } else {
        match slug[..50].rfind('-') {
            Some(pos) => slug[..pos].to_string(),
            None => slug[..50].to_string(),
        }
    }
}

/// Estimate token count for a string
///
/// Uses a simple heuristic: ~4 characters per token on average.
/// For more accurate counts, consider using tiktoken-rs.
pub fn count_tokens(text: &str) -> usize {
    // Simple estimation: average of ~4 characters per token
    // This is a reasonable approximation for English text with Claude models
    let char_count = text.chars().count();

    // Account for whitespace and punctuation which typically map to fewer tokens
    let word_count = text.split_whitespace().count();

    // Weighted average: prioritize word-based estimation
    // Roughly 1.3 tokens per word on average
    let word_based = (word_count as f64 * 1.3).ceil() as usize;
    let char_based = (char_count as f64 / 4.0).ceil() as usize;

    // Return the average of both methods, favoring word-based slightly
    ((word_based * 2 + char_based) / 3).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_tokens() {
        assert!(count_tokens("Hello world") > 0);
        assert!(count_tokens("This is a longer sentence with more words") > count_tokens("Hello"));
    }

    #[test]
    fn test_slug_basic() {
        assert_eq!(
            generate_prompt_slug("Create a dashboard for analytics"),
            "create-a-dashboard-for-analytics"
        );
    }

    #[test]
    fn test_slug_strips_prefix() {
        assert_eq!(
            generate_prompt_slug("You are an expert code reviewer. Review this PR"),
            "expert-code-reviewer-review-this-pr"
        );
        // "Please" stripped, rest kept
        assert_eq!(
            generate_prompt_slug("Please write a Python script for data processing"),
            "write-a-python-script-for-data"
        );
    }

    #[test]
    fn test_slug_strips_xml() {
        // XML tags stripped → "You are helpful" on first line
        // "You are an" prefix doesn't match (no "a/an"), so kept as-is
        assert_eq!(
            generate_prompt_slug("<system>You are helpful</system>\nAnalyze this code"),
            "you-are-helpful"
        );
    }

    #[test]
    fn test_slug_truncates_long_input() {
        let long_prompt = "Implement a comprehensive real-time analytics dashboard with user authentication session management data visualization export functionality and automated reporting";
        let slug = generate_prompt_slug(long_prompt);
        assert!(slug.len() <= 50);
        // "Implement" stripped as prefix? No — not in prefix list. Takes first 6 words.
        assert_eq!(slug, "implement-a-comprehensive-real-time-analytics");
    }

    #[test]
    fn test_slug_empty_input() {
        assert_eq!(generate_prompt_slug(""), "prompt");
        assert_eq!(generate_prompt_slug("   "), "prompt");
    }

    #[test]
    fn test_slug_special_characters() {
        // 6 words: "what's the best way to handle"
        assert_eq!(
            generate_prompt_slug("What's the best way to handle errors?"),
            "what-s-the-best-way-to-handle"
        );
    }
}

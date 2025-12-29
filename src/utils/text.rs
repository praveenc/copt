//! Text processing utilities
//!
//! Provides token counting and text manipulation functions.

#![allow(dead_code)]

/// Estimate token count for a string
///
/// Uses a simple heuristic: ~4 characters per token on average.
/// For more accurate counts, we'd use tiktoken-rs in production.
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

/// Count the number of words in a string
pub fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Count the number of lines in a string
pub fn line_count(text: &str) -> usize {
    if text.is_empty() {
        0
    } else {
        text.lines().count()
    }
}

/// Truncate text to a maximum length, adding ellipsis if needed
pub fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else if max_len > 3 {
        format!("{}...", &text[..max_len - 3])
    } else {
        text[..max_len].to_string()
    }
}

/// Truncate text to a maximum number of lines
pub fn truncate_lines(text: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = text.lines().take(max_lines).collect();
    let result = lines.join("\n");

    if text.lines().count() > max_lines {
        format!("{}\n...", result)
    } else {
        result
    }
}

/// Clean and normalize whitespace in text
pub fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Extract the first N characters, respecting word boundaries
pub fn extract_preview(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }

    // Find a good breaking point
    let slice = &text[..max_chars];

    // Try to break at last space
    if let Some(last_space) = slice.rfind(char::is_whitespace) {
        format!("{}...", &slice[..last_space])
    } else {
        format!("{}...", slice)
    }
}

/// Check if text contains any code-like patterns
pub fn contains_code(text: &str) -> bool {
    // Common code indicators
    let code_patterns = [
        "```",
        "function ",
        "def ",
        "class ",
        "const ",
        "let ",
        "var ",
        "import ",
        "export ",
        "pub fn",
        "fn ",
        "=>",
        "->",
        "async ",
        "await ",
    ];

    code_patterns.iter().any(|p| text.contains(p))
}

/// Check if text appears to be a system prompt
pub fn is_system_prompt(text: &str) -> bool {
    let lower = text.to_lowercase();
    let indicators = [
        "you are",
        "you're",
        "your role",
        "your task",
        "your job",
        "as an assistant",
        "as a",
        "<system>",
        "system prompt",
        "instructions:",
        "guidelines:",
    ];

    indicators.iter().any(|i| lower.contains(i))
}

/// Extract XML-like tags from text
pub fn extract_xml_tags(text: &str) -> Vec<String> {
    use regex::Regex;

    let re = Regex::new(r"<([a-zA-Z_][a-zA-Z0-9_]*)(?:\s|>)").unwrap();

    re.captures_iter(text)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

/// Calculate text similarity (simple Jaccard index on words)
pub fn text_similarity(a: &str, b: &str) -> f64 {
    let words_a: std::collections::HashSet<_> = a.split_whitespace().collect();
    let words_b: std::collections::HashSet<_> = b.split_whitespace().collect();

    if words_a.is_empty() && words_b.is_empty() {
        return 1.0;
    }

    let intersection = words_a.intersection(&words_b).count();
    let union = words_a.union(&words_b).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Calculate the change percentage between two strings
pub fn calculate_change_percent(original: &str, modified: &str) -> f64 {
    let orig_len = original.len() as f64;
    let mod_len = modified.len() as f64;

    if orig_len == 0.0 {
        if mod_len == 0.0 {
            0.0
        } else {
            100.0
        }
    } else {
        ((mod_len - orig_len) / orig_len * 100.0).abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_tokens() {
        assert!(count_tokens("Hello world") > 0);
        assert!(
            count_tokens("This is a longer sentence with more words") > count_tokens("Hello")
        );
    }

    #[test]
    fn test_word_count() {
        assert_eq!(word_count("Hello world"), 2);
        assert_eq!(word_count("  spaced   out  "), 2);
        assert_eq!(word_count(""), 0);
    }

    #[test]
    fn test_line_count() {
        assert_eq!(line_count("one\ntwo\nthree"), 3);
        assert_eq!(line_count("single line"), 1);
        assert_eq!(line_count(""), 0);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
    }

    #[test]
    fn test_contains_code() {
        assert!(contains_code("```rust\nfn main() {}\n```"));
        assert!(contains_code("function test() {}"));
        assert!(!contains_code("This is just plain text"));
    }

    #[test]
    fn test_is_system_prompt() {
        assert!(is_system_prompt("You are a helpful assistant"));
        assert!(is_system_prompt("Your task is to help users"));
        assert!(!is_system_prompt("What is the weather today?"));
    }

    #[test]
    fn test_extract_xml_tags() {
        let tags = extract_xml_tags("<rules>test</rules><example>data</example>");
        assert!(tags.contains(&"rules".to_string()));
        assert!(tags.contains(&"example".to_string()));
    }

    #[test]
    fn test_text_similarity() {
        assert_eq!(text_similarity("hello world", "hello world"), 1.0);
        assert!(text_similarity("hello world", "hello there") > 0.0);
        assert!(text_similarity("hello world", "goodbye moon") < 0.5);
    }
}
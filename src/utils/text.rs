//! Text processing utilities
//!
//! Provides token counting functions.

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
}

//! Rules engine for prompt optimization
//!
//! This module contains all the optimization rules based on Claude 4.5
//! best practices. Rules are organized by category and detect common
//! anti-patterns in prompts.

#![allow(dead_code)]

use regex::Regex;
use std::sync::LazyLock;

/// Rule severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// A rule category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Explicitness,
    Style,
    Tools,
    Formatting,
    Verbosity,
    Agentic,
    LongHorizon,
    Frontend,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Explicitness => "Explicitness",
            Category::Style => "Style",
            Category::Tools => "Tool Usage",
            Category::Formatting => "Formatting",
            Category::Verbosity => "Verbosity",
            Category::Agentic => "Agentic Coding",
            Category::LongHorizon => "Long-Horizon",
            Category::Frontend => "Frontend Design",
        }
    }

    pub fn prefix(&self) -> &'static str {
        match self {
            Category::Explicitness => "EXP",
            Category::Style => "STY",
            Category::Tools => "TUL",
            Category::Formatting => "FMT",
            Category::Verbosity => "VRB",
            Category::Agentic => "AGT",
            Category::LongHorizon => "LHT",
            Category::Frontend => "FED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "explicitness" | "exp" => Some(Category::Explicitness),
            "style" | "sty" => Some(Category::Style),
            "tools" | "tul" | "tool" => Some(Category::Tools),
            "formatting" | "fmt" | "format" => Some(Category::Formatting),
            "verbosity" | "vrb" => Some(Category::Verbosity),
            "agentic" | "agt" => Some(Category::Agentic),
            "long_horizon" | "longhorizon" | "lht" | "horizon" => Some(Category::LongHorizon),
            "frontend" | "fed" | "design" => Some(Category::Frontend),
            _ => None,
        }
    }

    pub fn all() -> &'static [Category] {
        &[
            Category::Explicitness,
            Category::Style,
            Category::Tools,
            Category::Formatting,
            Category::Verbosity,
            Category::Agentic,
            Category::LongHorizon,
            Category::Frontend,
        ]
    }
}

/// A detected issue in a prompt
#[derive(Debug, Clone)]
pub struct Issue {
    /// Unique rule identifier (e.g., "EXP001")
    pub id: String,
    /// Rule category
    pub category: Category,
    /// Issue severity
    pub severity: Severity,
    /// Human-readable description of the issue
    pub message: String,
    /// Line number where the issue was detected (1-indexed)
    pub line: Option<usize>,
    /// Column position in the line
    pub column: Option<usize>,
    /// Matched text that triggered the rule
    pub matched_text: Option<String>,
    /// Suggested fix or improvement
    pub suggestion: Option<String>,
    /// Example of the improved version
    pub example: Option<String>,
}

impl Issue {
    pub fn new(id: &str, category: Category, severity: Severity, message: &str) -> Self {
        Self {
            id: id.to_string(),
            category,
            severity,
            message: message.to_string(),
            line: None,
            column: None,
            matched_text: None,
            suggestion: None,
            example: None,
        }
    }

    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    pub fn with_matched_text(mut self, text: &str) -> Self {
        self.matched_text = Some(text.to_string());
        self
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }

    pub fn with_example(mut self, example: &str) -> Self {
        self.example = Some(example.to_string());
        self
    }
}

/// Common regex patterns used across rules
pub mod patterns {
    use super::*;

    /// Matches "Can you...", "Could you...", etc.
    pub static INDIRECT_COMMAND: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)^(can|could|would|will)\s+(you\s+)?(please\s+)?").unwrap()
    });

    /// Matches negative instructions like "Don't", "Never", "Avoid"
    pub static NEGATIVE_INSTRUCTION: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)\b(don'?t|do\s+not|never|avoid|stop)\b").unwrap());

    /// Matches ALL CAPS text (excluding common acronyms)
    pub static ALL_CAPS_TEXT: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\b[A-Z]{4,}\b").unwrap());

    /// Matches the word "think" and variants
    pub static THINK_VARIANTS: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)\b(think(ing)?|thinks?)\b").unwrap());

    /// Matches suggestion phrases
    pub static SUGGESTION_PHRASE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\b(suggest|recommend|propose|advise|what\s+do\s+you\s+think)\b").unwrap()
    });

    /// Matches generic UI creation requests
    pub static GENERIC_UI_REQUEST: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\b(create|build|make|design)\s+(a\s+)?(simple\s+)?(ui|interface|page|form|dashboard|component)\b").unwrap()
    });

    /// Matches code modification requests
    pub static CODE_MODIFICATION: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\b(fix|update|change|modify|refactor|improve)\s+(the\s+)?(bug|code|function|method|class|implementation)\b").unwrap()
    });

    /// Matches vague action verbs
    pub static VAGUE_ACTION: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)^(create|build|make|write|implement)\s+[a-z]+\s*$").unwrap()
    });

    /// Matches aggressive emphasis
    pub static AGGRESSIVE_EMPHASIS: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\b(CRITICAL|IMPORTANT|MUST|ALWAYS|ESSENTIAL|MANDATORY|REQUIRED)\s*[:\-!]")
            .unwrap()
    });

    /// Matches multiple file operations
    pub static MULTI_FILE_OPERATION: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\b(all\s+)?(files?|directories?|folders?|configs?)\b").unwrap()
    });

    /// Matches large task indicators
    pub static LARGE_TASK_INDICATOR: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\b(entire|complete|full|whole|all)\s+(codebase|backend|frontend|system|project|application|api)\b").unwrap()
    });
}

/// Helper function to find line number for a match position
pub fn find_line_number(text: &str, position: usize) -> usize {
    text[..position.min(text.len())].matches('\n').count() + 1
}

/// Helper function to check if text is short (likely vague)
pub fn is_short_instruction(text: &str, threshold: usize) -> bool {
    let word_count = text.split_whitespace().count();
    word_count < threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_from_str() {
        assert_eq!(Category::from_str("exp"), Some(Category::Explicitness));
        assert_eq!(Category::from_str("STYLE"), Some(Category::Style));
        assert_eq!(Category::from_str("unknown"), None);
    }

    #[test]
    fn test_find_line_number() {
        let text = "line 1\nline 2\nline 3";
        assert_eq!(find_line_number(text, 0), 1);
        assert_eq!(find_line_number(text, 7), 2);
        assert_eq!(find_line_number(text, 14), 3);
    }

    #[test]
    fn test_is_short_instruction() {
        assert!(is_short_instruction("Create a dashboard", 5));
        assert!(!is_short_instruction(
            "Create a comprehensive analytics dashboard with multiple features",
            5
        ));
    }

    #[test]
    fn test_indirect_command_pattern() {
        assert!(patterns::INDIRECT_COMMAND.is_match("Can you help me?"));
        assert!(patterns::INDIRECT_COMMAND.is_match("Could you please fix this?"));
        assert!(!patterns::INDIRECT_COMMAND.is_match("Fix this bug"));
    }

    #[test]
    fn test_negative_instruction_pattern() {
        assert!(patterns::NEGATIVE_INSTRUCTION.is_match("Don't use markdown"));
        assert!(patterns::NEGATIVE_INSTRUCTION.is_match("Never include headers"));
        assert!(patterns::NEGATIVE_INSTRUCTION.is_match("Avoid using lists"));
    }
}
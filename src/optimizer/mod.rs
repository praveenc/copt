//! Optimizer module for prompt transformation
//!
//! This module provides both static (rule-based) and LLM-powered
//! optimization of prompts for Claude 4.5 models.

use anyhow::Result;
use regex::Regex;
use std::sync::LazyLock;

use crate::analyzer::{Issue, PromptType, Severity};
use crate::llm::{build_optimization_message, LlmClient, OPTIMIZER_SYSTEM_PROMPT};

/// Static optimization using rule-based transformations
///
/// This function applies known transformations without requiring API calls.
/// Useful for offline mode or quick fixes. Also appends applicable
/// enhancement hints based on prompt content analysis.
pub fn optimize_static(prompt: &str, issues: &[Issue]) -> Result<String> {
    let mut result = prompt.to_string();

    for issue in issues {
        result = apply_static_transformation(&result, issue);
    }

    // Append applicable enhancement hints
    let enhancements = get_applicable_enhancements(&result);
    if !enhancements.is_empty() {
        for enhancement in &enhancements {
            result.push_str(enhancement);
        }
    }

    Ok(result)
}

/// Apply a single static transformation based on an issue
fn apply_static_transformation(prompt: &str, issue: &Issue) -> String {
    match issue.id.as_str() {
        // Explicitness transformations
        "EXP003" => transform_indirect_commands(prompt),

        // Style transformations
        "STY002" => transform_aggressive_emphasis(prompt),
        "STY003" => transform_think_word(prompt),
        "STY004" => transform_overtriggering_language(prompt),

        // For other rules, return unchanged (require LLM for complex rewrites)
        _ => prompt.to_string(),
    }
}

/// Transform indirect commands like "Can you..." to direct commands
fn transform_indirect_commands(prompt: &str) -> String {
    static PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
        vec![
            (Regex::new(r"(?i)^can you\s+").unwrap(), ""),
            (Regex::new(r"(?i)^could you\s+").unwrap(), ""),
            (Regex::new(r"(?i)^would you mind\s+").unwrap(), ""),
            (Regex::new(r"(?i)^is it possible to\s+").unwrap(), ""),
            (
                Regex::new(r"(?i)^i was wondering if you could\s+").unwrap(),
                "",
            ),
            (Regex::new(r"(?i)^please\s+").unwrap(), ""),
        ]
    });

    let mut result = prompt.to_string();

    for (re, replacement) in PATTERNS.iter() {
        result = re.replace(&result, *replacement).to_string();
    }

    // Capitalize first letter if needed
    if let Some(first_char) = result.chars().next() {
        if first_char.is_lowercase() {
            result = first_char.to_uppercase().to_string() + &result[first_char.len_utf8()..];
        }
    }

    result
}

/// Transform aggressive ALL CAPS emphasis to normal case
fn transform_aggressive_emphasis(prompt: &str) -> String {
    static CAPS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\b([A-Z]{2,})\b").unwrap());

    // Match ALL CAPS words that aren't common acronyms
    let acronyms = [
        "API", "URL", "HTTP", "HTML", "CSS", "JSON", "XML", "SQL", "REST", "CLI", "UI", "UX",
        "AWS", "GCP", "ID",
    ];

    CAPS_RE
        .replace_all(prompt, |caps: &regex::Captures| {
            let word = &caps[1];
            if acronyms.contains(&word) {
                word.to_string()
            } else {
                // Convert to lowercase, capitalize first letter
                let lower = word.to_lowercase();
                if let Some(first) = lower.chars().next() {
                    first.to_uppercase().to_string() + &lower[first.len_utf8()..]
                } else {
                    lower
                }
            }
        })
        .to_string()
}

/// Transform "think" and variants to Claude 4.5 friendly alternatives
fn transform_think_word(prompt: &str) -> String {
    static PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
        vec![
            (Regex::new(r"(?i)\bthink about\b").unwrap(), "consider"),
            (
                Regex::new(r"(?i)\bthink through\b").unwrap(),
                "work through",
            ),
            (Regex::new(r"(?i)\bI think\b").unwrap(), "I believe"),
            (
                Regex::new(r"(?i)\bthinking about\b").unwrap(),
                "considering",
            ),
            (Regex::new(r"(?i)\bthinking\b").unwrap(), "evaluating"),
            (Regex::new(r"(?i)\bthink\b").unwrap(), "consider"),
        ]
    });

    let mut result = prompt.to_string();

    for (re, replacement) in PATTERNS.iter() {
        result = re.replace_all(&result, *replacement).to_string();
    }

    result
}

/// Tone down overtriggering language
fn transform_overtriggering_language(prompt: &str) -> String {
    static PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
        vec![
            (Regex::new(r"(?i)\bCRITICAL:\s*").unwrap(), ""),
            (Regex::new(r"(?i)\bIMPORTANT:\s*").unwrap(), ""),
            (Regex::new(r"(?i)\bYou MUST\b").unwrap(), "You should"),
            (Regex::new(r"(?i)\bMUST ALWAYS\b").unwrap(), "should"),
            (Regex::new(r"(?i)\bALWAYS MUST\b").unwrap(), "should"),
            (Regex::new(r"(?i)\bNEVER EVER\b").unwrap(), "avoid"),
            (Regex::new(r"(?i)!{2,}").unwrap(), "!"),
            (Regex::new(r"(?i)\bMANDATORY\b").unwrap(), "required"),
            (Regex::new(r"(?i)\bESSENTIAL\b").unwrap(), "important"),
            (Regex::new(r"(?i)\bCRUCIAL\b").unwrap(), "important"),
        ]
    });

    let mut result = prompt.to_string();

    for (re, replacement) in PATTERNS.iter() {
        result = re.replace_all(&result, *replacement).to_string();
    }

    result
}

/// Optimize a prompt using an LLM
pub async fn optimize_with_llm(
    prompt: &str,
    issues: &[Issue],
    client: &dyn LlmClient,
    model: &str,
    prompt_type: PromptType,
) -> Result<String> {
    // First apply static transformations for quick wins
    let partially_optimized = optimize_static(prompt, issues)?;

    // Build the user message with detected issues, enhancements, and prompt type
    let issues_summary = format_issues_for_llm(issues);
    let enhancements = get_applicable_enhancements(&partially_optimized);
    let enhancements_summary = if enhancements.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nSuggested enhancements to incorporate:\n{}",
            enhancements
                .iter()
                .map(|e| format!("- {}", e.trim()))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    let prompt_type_str = prompt_type_to_str(prompt_type);
    let user_message = build_optimization_message(
        &partially_optimized,
        &format!("{}{}", issues_summary, enhancements_summary),
        prompt_type_str,
    );

    // Call the LLM
    let optimized = client
        .complete(OPTIMIZER_SYSTEM_PROMPT, &user_message, model, 4096)
        .await?;

    // Clean up any accidental wrapping the LLM might add
    let optimized = clean_llm_output(&optimized);

    Ok(optimized)
}

/// Format issues for inclusion in the LLM prompt
fn format_issues_for_llm(issues: &[Issue]) -> String {
    if issues.is_empty() {
        return "No specific issues detected, but general optimization is requested.".to_string();
    }

    issues
        .iter()
        .map(|issue| {
            let severity = match issue.severity {
                Severity::Error => "ERROR",
                Severity::Warning => "WARNING",
                Severity::Info => "INFO",
            };
            format!(
                "- [{}] {}: {} {}",
                severity,
                issue.id,
                issue.message,
                issue.suggestion.as_deref().unwrap_or("")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Clean up LLM output that might have unwanted wrapping
fn clean_llm_output(output: &str) -> String {
    let mut result = output.trim().to_string();

    // Remove common LLM wrapping patterns
    let prefixes = [
        "Here is the optimized prompt:",
        "Here's the optimized prompt:",
        "Optimized prompt:",
        "Here is the improved prompt:",
        "```",
    ];

    for prefix in prefixes {
        if result.starts_with(prefix) {
            result = result[prefix.len()..].trim_start().to_string();
        }
    }

    // Remove trailing code fence if present
    if result.ends_with("```") {
        result = result[..result.len() - 3].trim_end().to_string();
    }

    result
}

/// Convert PromptType enum to string for LLM context
fn prompt_type_to_str(prompt_type: PromptType) -> &'static str {
    match prompt_type {
        PromptType::Coding => "coding",
        PromptType::QaAssistant => "qa_assistant",
        PromptType::Research => "research",
        PromptType::Creative => "creative",
        PromptType::LongHorizon => "long_horizon",
        PromptType::General => "general",
    }
}

/// Enhancement suggestions that can be appended to prompts based on detected patterns
pub struct Enhancement {
    #[allow(dead_code)] // Useful for debugging/logging, not read in current pipeline
    pub id: &'static str,
    pub condition: fn(&str) -> bool,
    pub template: &'static str,
}

/// Get applicable enhancements for a prompt
pub fn get_applicable_enhancements(prompt: &str) -> Vec<&'static str> {
    let enhancements: Vec<Enhancement> = vec![
        Enhancement {
            id: "parallel_tools",
            condition: |p| p.contains("files") || p.contains("multiple") || p.contains("each"),
            template: "\n\nIf you need to perform multiple independent operations, execute them in parallel for efficiency.",
        },
        Enhancement {
            id: "exploration",
            condition: |p| {
                let lower = p.to_lowercase();
                lower.contains("fix")
                    || lower.contains("bug")
                    || lower.contains("change")
                    || lower.contains("update")
            },
            template: "\n\nRead and understand the relevant code before making changes. Do not speculate about code you haven't inspected.",
        },
        Enhancement {
            id: "action_default",
            condition: |p| {
                let lower = p.to_lowercase();
                lower.contains("suggest") || lower.contains("recommend") || lower.contains("improve")
            },
            template: "\n\nImplement the changes directly rather than only suggesting them.",
        },
        Enhancement {
            id: "summary",
            condition: |p| p.len() > 500 || p.contains("refactor") || p.contains("update"),
            template: "\n\nAfter completing the changes, provide a brief summary of what was modified.",
        },
    ];

    enhancements
        .iter()
        .filter(|e| (e.condition)(prompt))
        .map(|e| e.template)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_indirect_commands() {
        assert_eq!(
            transform_indirect_commands("Can you fix this bug?"),
            "Fix this bug?"
        );
        assert_eq!(
            transform_indirect_commands("Could you refactor the code?"),
            "Refactor the code?"
        );
        assert_eq!(
            transform_indirect_commands("Would you mind reviewing this?"),
            "Reviewing this?"
        );
    }

    #[test]
    fn test_transform_think_word() {
        assert_eq!(
            transform_think_word("Think about the edge cases"),
            "consider the edge cases"
        );
        assert_eq!(
            transform_think_word("I think this approach is better"),
            "I believe this approach is better"
        );
    }

    #[test]
    fn test_transform_aggressive_emphasis() {
        let input = "CRITICAL: You MUST ALWAYS check the API response";
        let result = transform_aggressive_emphasis(input);
        assert!(!result.contains("CRITICAL"));
        assert!(result.contains("API")); // Acronym preserved
    }

    #[test]
    fn test_transform_overtriggering() {
        let input = "CRITICAL: You MUST ALWAYS validate input!!!";
        let result = transform_overtriggering_language(input);
        assert!(!result.contains("CRITICAL:"));
        assert!(result.contains("should"));
        assert!(!result.contains("!!!"));
    }

    #[test]
    fn test_clean_llm_output() {
        assert_eq!(
            clean_llm_output("Here is the optimized prompt:\n\nDo this task"),
            "Do this task"
        );
        assert_eq!(clean_llm_output("```\nCode here\n```"), "Code here");
    }

    #[test]
    fn test_prompt_type_to_str() {
        assert_eq!(prompt_type_to_str(PromptType::Coding), "coding");
        assert_eq!(prompt_type_to_str(PromptType::QaAssistant), "qa_assistant");
        assert_eq!(prompt_type_to_str(PromptType::Research), "research");
        assert_eq!(prompt_type_to_str(PromptType::Creative), "creative");
        assert_eq!(prompt_type_to_str(PromptType::LongHorizon), "long_horizon");
        assert_eq!(prompt_type_to_str(PromptType::General), "general");
    }
}

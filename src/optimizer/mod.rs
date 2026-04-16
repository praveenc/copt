//! Optimizer module for prompt transformation
//!
//! This module provides both static (rule-based) and LLM-powered
//! optimization of prompts for Claude 4.5 models.

use anyhow::Result;
use regex::Regex;
use std::sync::LazyLock;

use crate::analyzer::{Issue, PromptType, Severity};
use crate::llm::{build_optimization_message, system_prompt_for_family, LlmClient, ModelFamily};

/// Static optimization using rule-based transformations
///
/// This function applies known transformations without requiring API calls.
/// Useful for offline mode or quick fixes. Also appends applicable
/// enhancement hints based on prompt content analysis.
pub fn optimize_static(prompt: &str, issues: &[Issue]) -> Result<String> {
    let mut result = prompt.to_string();

    // Apply transforms in a fixed priority order to avoid ordering bugs.
    // STY004 (overtriggering) must run before STY002 (emphasis) because
    // STY002 lowercases "CRITICAL" → "Critical", making STY004's pattern miss.
    let transform_order = ["STY004", "STY002", "STY003", "STY001", "FMT002", "EXP003"];

    for rule_id in &transform_order {
        if issues.iter().any(|i| i.id == *rule_id) {
            result = apply_static_transformation(&result, rule_id);
        }
    }

    // Append applicable enhancement hints (family-neutral set).
    let enhancements = get_applicable_enhancements(&result);
    if !enhancements.is_empty() {
        for enhancement in &enhancements {
            result.push_str(enhancement);
        }
    }

    Ok(result)
}

/// Family-aware variant of `optimize_static` that additionally appends the
/// Claude 4.7-specific hints (literal scope, adaptive thinking, scratchpad,
/// vision) when targeting 4.7. Callers that don't care about the family can
/// keep using `optimize_static`.
pub fn optimize_static_for_family(
    prompt: &str,
    issues: &[Issue],
    family: ModelFamily,
) -> Result<String> {
    let mut result = optimize_static(prompt, issues)?;

    if family == ModelFamily::Claude47 {
        for hint in get_family_enhancements(&result, family) {
            result.push_str(hint);
        }
    }

    Ok(result)
}

/// Apply a single static transformation based on a rule ID
fn apply_static_transformation(prompt: &str, rule_id: &str) -> String {
    match rule_id {
        // Explicitness transformations
        "EXP003" => transform_indirect_commands(prompt),

        // Style transformations
        "STY001" => transform_negative_instructions(prompt),
        "STY002" => transform_aggressive_emphasis(prompt),
        "STY003" => transform_think_word(prompt),
        "STY004" => transform_overtriggering_language(prompt),

        // Formatting transformations
        "FMT002" => transform_negative_format(prompt),

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

/// Transform negative instructions to positive guidance
///
/// "Don't use global variables" → "Use local variables or dependency injection instead"
/// Only transforms common patterns where a positive alternative is clear.
fn transform_negative_instructions(prompt: &str) -> String {
    static PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
        vec![
            (
                Regex::new(
                    r"(?im)^(\s*)(?:don't|do not)\s+use\s+global\s+(?:variables?|state)\b[.]?",
                )
                .unwrap(),
                "${1}Use local variables or dependency injection instead of global state.",
            ),
            (
                Regex::new(r"(?im)^(\s*)(?:don't|do not)\s+use\s+var\b[.]?").unwrap(),
                "${1}Use const or let instead of var.",
            ),
            (
                Regex::new(r"(?im)^(\s*)(?:don't|do not)\s+use\s+any\b[.]?").unwrap(),
                "${1}Use specific types instead of any.",
            ),
            (
                Regex::new(r"(?im)^(\s*)(?:don't|do not)\s+hardcode\b[.]?").unwrap(),
                "${1}Use configuration or constants instead of hardcoded values.",
            ),
            (
                Regex::new(r"(?im)^(\s*)(?:don't|do not)\s+repeat\s+(?:yourself|code)\b[.]?")
                    .unwrap(),
                "${1}Extract shared logic into reusable functions.",
            ),
            (
                Regex::new(r"(?im)^(\s*)(?:don't|do not)\s+ignore\s+errors?\b[.]?").unwrap(),
                "${1}Handle all errors explicitly with appropriate error types.",
            ),
            (
                Regex::new(r"(?im)^(\s*)never\s+use\s+unwrap\b[.]?").unwrap(),
                "${1}Use proper error handling (? operator or match) instead of unwrap.",
            ),
        ]
    });

    let mut result = prompt.to_string();
    for (re, replacement) in PATTERNS.iter() {
        result = re.replace_all(&result, *replacement).to_string();
    }
    result
}

/// Transform negative format instructions to positive alternatives
///
/// "no markdown" → "write in flowing prose paragraphs"
/// "don't use bullet points" → "use flowing prose paragraphs"
fn transform_negative_format(prompt: &str) -> String {
    static PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
        vec![
            (
                Regex::new(r"(?i)\b(?:no|don't use|do not use|avoid)\s+markdown\b").unwrap(),
                "write in plain text without markdown formatting",
            ),
            (
                Regex::new(
                    r"(?i)\b(?:no|don't use|do not use|avoid)\s+bullet\s*(?:points?|lists?)\b",
                )
                .unwrap(),
                "use flowing prose paragraphs",
            ),
            (
                Regex::new(r"(?i)\b(?:no|don't use|do not use|avoid)\s+(?:bold|italic)\b").unwrap(),
                "use plain text emphasis through word choice",
            ),
            (
                Regex::new(r"(?i)\b(?:no|don't use|do not use|avoid)\s+(?:lists?|formatting)\b")
                    .unwrap(),
                "write in continuous prose",
            ),
        ]
    });

    let mut result = prompt.to_string();
    for (re, replacement) in PATTERNS.iter() {
        result = re.replace_all(&result, *replacement).to_string();
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
///
/// The `target_family` parameter controls WHICH Claude family's best-practices
/// the rewrite is optimized for. Pass `None` to infer the family from `model`
/// (historical behaviour) or `Some(family)` to decouple "rewriter model" from
/// "target family" — e.g., using Sonnet 4.5 to rewrite a prompt for Opus 4.7.
/// Unknown model IDs fall back to the 4.5 meta-prompt.
pub async fn optimize_with_llm(
    prompt: &str,
    issues: &[Issue],
    client: &dyn LlmClient,
    model: &str,
    prompt_type: PromptType,
    target_family: Option<ModelFamily>,
) -> Result<String> {
    let family = target_family.unwrap_or_else(|| ModelFamily::from_model_id(model));

    // First apply static transformations for quick wins (family-aware so 4.7
    // static hints make it into the user message as enhancement context).
    let partially_optimized = optimize_static_for_family(prompt, issues, family)?;

    // Build the user message with detected issues, enhancements, and prompt type
    let issues_summary = format_issues_for_llm(issues);
    let mut all_enhancements = get_applicable_enhancements(&partially_optimized);
    if family == ModelFamily::Claude47 {
        all_enhancements.extend(get_family_enhancements(&partially_optimized, family));
    }
    let enhancements_summary = if all_enhancements.is_empty() {
        String::new()
    } else {
        format!(
            "\n\nSuggested enhancements to incorporate:\n{}",
            all_enhancements
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
        family,
    );

    // Call the LLM using the family-specific meta-prompt.
    let system_prompt = system_prompt_for_family(family);
    let optimized = client
        .complete(system_prompt, &user_message, model, 4096)
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

/// Claude 4.7-specific enhancement hints.
///
/// These are gated behind `ModelFamily::Claude47` so 4.5/4.6 output is
/// unchanged. Each hint targets one of the eight 4.7 prompting rules.
pub fn get_family_enhancements(prompt: &str, family: ModelFamily) -> Vec<&'static str> {
    if family != ModelFamily::Claude47 {
        return Vec::new();
    }

    let lower = prompt.to_lowercase();
    let mut hints: Vec<&'static str> = Vec::new();

    // 1. Literal instruction following — state scope explicitly.
    if lower.contains(" each ")
        || lower.contains(" every ")
        || lower.contains(" all ")
        || lower.contains(" any ")
    {
        hints.push(
            "\n\nWhen an instruction should apply broadly, state the scope explicitly (e.g., \"apply to every section, not just the first\"). Claude 4.7 interprets prompts literally and will not silently generalize.",
        );
    }

    // 2. Adaptive thinking — nudge based on reasoning keywords.
    if lower.contains("think")
        || lower.contains("reason")
        || lower.contains("analyze")
        || lower.contains("plan")
    {
        hints.push(
            "\n\nIf the task genuinely benefits from reasoning, say so: \"This task involves multi-step reasoning. Think carefully through the problem before responding.\" Adaptive thinking is off by default on Claude 4.7.",
        );
    }

    // 3. Effort-level awareness for coding / agentic work.
    if lower.contains("code")
        || lower.contains("refactor")
        || lower.contains("implement")
        || lower.contains("agent")
        || lower.contains("tool")
    {
        hints.push(
            "\n\nFor coding and agentic workloads on Claude 4.7, xhigh effort is the recommended default; use at minimum high effort for intelligence-sensitive work.",
        );
    }

    // 4. Scratchpad / memory directives for long-horizon work.
    if prompt.len() > 500
        || lower.contains("long")
        || lower.contains("multi-step")
        || lower.contains("agent")
        || lower.contains("across turns")
    {
        hints.push(
            "\n\nMaintain a scratchpad of intermediate findings, open questions, and decisions. Consult it before each new step and update it after each tool call. Claude 4.7 is meaningfully better at writing and using file-system memory.",
        );
    }

    // 5. Condensed context — remove 4.5/4.6 scaffolding.
    if lower.contains("double-check")
        || lower.contains("status update")
        || lower.contains("summarize progress")
        || lower.contains("after every")
    {
        hints.push(
            "\n\nClaude 4.7 provides native progress updates and self-checking — remove forced interim-status or self-verification scaffolding left over from 4.5/4.6 prompts.",
        );
    }

    // 6. Tone specification when a voice is implied.
    if lower.contains("friendly")
        || lower.contains("warm")
        || lower.contains("helpful tone")
        || lower.contains("conversational")
    {
        hints.push(
            "\n\nClaude 4.7 defaults to a direct, opinionated tone with fewer emoji than 4.6. If a warmer voice is required, state it explicitly, e.g., \"Use a warm, collaborative tone. Acknowledge the user's framing before answering.\"",
        );
    }

    // 7. Response-length calibration.
    if lower.contains("concise")
        || lower.contains("brief")
        || lower.contains("short")
        || lower.contains("verbose")
        || lower.contains("thorough")
    {
        hints.push(
            "\n\nClaude 4.7 calibrates response length to perceived task complexity. Prefer a positive example of the target concision over \"don't over-explain\" negatives, e.g., \"Provide concise, focused responses. Skip non-essential context and keep examples minimal.\"",
        );
    }

    // 8. Vision-aware instructions.
    if lower.contains("image")
        || lower.contains("screenshot")
        || lower.contains("pixel")
        || lower.contains("bounding box")
        || lower.contains("coordinate")
        || lower.contains("vision")
    {
        hints.push(
            "\n\nClaude 4.7 supports high-resolution images up to 2576px / 3.75MP and returns pointing/bounding-box coordinates 1:1 with actual image pixels. Remove any scale-factor conversion logic. If pixel-level fidelity is not required, downsample images to 1080p before sending to control token usage.",
        );
    }

    hints
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
    fn test_transform_negative_instructions() {
        assert_eq!(
            transform_negative_instructions("Don't use global variables"),
            "Use local variables or dependency injection instead of global state."
        );
        assert_eq!(
            transform_negative_instructions("do not hardcode"),
            "Use configuration or constants instead of hardcoded values."
        );
        // Should not transform unrecognized patterns
        assert_eq!(
            transform_negative_instructions("Don't forget to test"),
            "Don't forget to test"
        );
    }

    #[test]
    fn test_transform_negative_format() {
        let result = transform_negative_format("no markdown please");
        assert!(result.contains("plain text"));
        assert!(!result.contains("no markdown"));

        let result = transform_negative_format("don't use bullet points");
        assert!(result.contains("flowing prose"));
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

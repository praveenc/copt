//! Prompt analyzer module
//!
//! This module analyzes prompts for common anti-patterns and issues
//! based on Claude 4.5 best practices.

use anyhow::Result;
use regex::Regex;

/// Issue severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// An issue detected in the prompt
#[derive(Debug, Clone)]
pub struct Issue {
    pub id: String,
    pub category: String,
    pub severity: Severity,
    pub message: String,
    pub line: Option<usize>,
    pub suggestion: Option<String>,
}

/// All available rule categories
pub const CATEGORIES: &[&str] = &[
    "explicitness",
    "style",
    "tools",
    "formatting",
    "verbosity",
    "agentic",
    "long_horizon",
    "frontend",
];

/// Analyze a prompt and return detected issues
pub fn analyze(prompt: &str, check_categories: Option<&[String]>) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();

    // Determine which categories to check
    let categories_to_check: Vec<&str> = match check_categories {
        Some(cats) => cats.iter().map(|s| s.as_str()).collect(),
        None => CATEGORIES.to_vec(),
    };

    // Run all applicable analyzers
    for category in categories_to_check {
        match category {
            "explicitness" => issues.extend(analyze_explicitness(prompt)),
            "style" => issues.extend(analyze_style(prompt)),
            "tools" => issues.extend(analyze_tools(prompt)),
            "formatting" => issues.extend(analyze_formatting(prompt)),
            "verbosity" => issues.extend(analyze_verbosity(prompt)),
            "agentic" => issues.extend(analyze_agentic(prompt)),
            "long_horizon" => issues.extend(analyze_long_horizon(prompt)),
            "frontend" => issues.extend(analyze_frontend(prompt)),
            _ => {} // Unknown category, skip
        }
    }

    Ok(issues)
}

/// Analyze for explicitness issues (EXP001-004)
fn analyze_explicitness(prompt: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = prompt.lines().collect();

    // EXP001: Vague instructions (short imperatives without detail)
    let vague_patterns = Regex::new(
        r"(?i)^(create|build|make|write|implement|design|develop|add|fix|update)\s+(?:a\s+|an\s+|the\s+)?[\w\s]{1,20}$"
    ).unwrap();

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if vague_patterns.is_match(trimmed) && trimmed.split_whitespace().count() < 8 {
            issues.push(Issue {
                id: "EXP001".to_string(),
                category: "explicitness".to_string(),
                severity: Severity::Warning,
                message: format!("Vague instruction: \"{}\"", trimmed),
                line: Some(idx + 1),
                suggestion: Some(
                    "Add specific details, features, and success criteria. \
                    For example: \"Include as many relevant features as possible. \
                    Go beyond the basics to create a fully-featured implementation.\""
                        .to_string(),
                ),
            });
        }
    }

    // EXP003: Indirect commands (Can you... / Could you...)
    let indirect_pattern =
        Regex::new(r"(?i)\b(can you|could you|would you|would you mind|is it possible to|i was wondering if)\b").unwrap();

    for (idx, line) in lines.iter().enumerate() {
        if indirect_pattern.is_match(line) {
            issues.push(Issue {
                id: "EXP003".to_string(),
                category: "explicitness".to_string(),
                severity: Severity::Warning,
                message: "Indirect command detected - Claude 4.5 may suggest rather than act"
                    .to_string(),
                line: Some(idx + 1),
                suggestion: Some(
                    "Use direct commands instead. Replace \"Can you...\" with imperative verbs."
                        .to_string(),
                ),
            });
        }
    }

    // EXP002: Missing context for bare prohibitions
    let bare_prohibition = Regex::new(r"(?i)^(always|never|don't|do not)\s+\w+[^.]*\.?$").unwrap();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if bare_prohibition.is_match(trimmed) && trimmed.split_whitespace().count() < 10 {
            // Check if there's explanation nearby
            let has_context = lines
                .get(idx + 1)
                .map(|l| l.contains("because") || l.contains("since") || l.contains("so that"))
                .unwrap_or(false);

            if !has_context && !trimmed.contains("because") {
                issues.push(Issue {
                    id: "EXP002".to_string(),
                    category: "explicitness".to_string(),
                    severity: Severity::Info,
                    message: "Prohibition without context or motivation".to_string(),
                    line: Some(idx + 1),
                    suggestion: Some(
                        "Add context explaining why this rule exists to help Claude generalize."
                            .to_string(),
                    ),
                });
            }
        }
    }

    // EXP004: Complex tasks without success criteria
    let complex_task_indicators = Regex::new(
        r"(?i)\b(research|analyze|investigate|explore|evaluate|review|implement|build|create)\s+(the|a|an)?\s*\w+"
    ).unwrap();

    if complex_task_indicators.is_match(prompt) {
        let has_criteria = prompt.contains("success")
            || prompt.contains("complete when")
            || prompt.contains("done when")
            || prompt.contains("criteria")
            || Regex::new(r"\d+\s*(steps?|items?|points?)")
                .unwrap()
                .is_match(prompt);

        if !has_criteria && prompt.len() > 100 {
            issues.push(Issue {
                id: "EXP004".to_string(),
                category: "explicitness".to_string(),
                severity: Severity::Info,
                message: "Complex task may benefit from explicit success criteria".to_string(),
                line: None,
                suggestion: Some(
                    "Define what constitutes successful completion of this task.".to_string(),
                ),
            });
        }
    }

    issues
}

/// Analyze for style issues (STY001-004)
fn analyze_style(prompt: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = prompt.lines().collect();

    // STY001: Negative instructions
    let negative_patterns =
        Regex::new(r"(?i)\b(don't|do not|never|avoid|stop|no\s+\w+ing)\b").unwrap();

    for (idx, line) in lines.iter().enumerate() {
        if negative_patterns.is_match(line) {
            // Check if it's a substantial negative instruction
            let negation_count = negative_patterns.find_iter(line).count();
            if negation_count > 0 {
                issues.push(Issue {
                    id: "STY001".to_string(),
                    category: "style".to_string(),
                    severity: Severity::Warning,
                    message: "Negative instruction detected".to_string(),
                    line: Some(idx + 1),
                    suggestion: Some(
                        "Reframe as positive guidance. Instead of \"Don't use X\", \
                        try \"Use Y instead\" or explain what to do."
                            .to_string(),
                    ),
                });
            }
        }
    }

    // STY002: Aggressive emphasis (instructional ALL CAPS words, multiple !)
    // Only flag instructional/emphatic words in ALL CAPS, not acronyms/abbreviations
    let instructional_caps = Regex::new(
        r"\b(DON'?T|DONT|NEVER|ALWAYS|MUST|IMPORTANT|CRUCIAL|REMEMBER|NOTE|WARNING|CAUTION|CRITICAL|ESSENTIAL|REQUIRED|MANDATORY|ABSOLUTELY|DEFINITELY|CERTAINLY|ENSURE|VERY|STOP|AVOID)\b"
    ).unwrap();
    let multi_exclaim = Regex::new(r"!{2,}").unwrap();

    for (idx, line) in lines.iter().enumerate() {
        // Only flag instructional words in ALL CAPS
        let caps_matches: Vec<_> = instructional_caps.find_iter(line).collect();

        if !caps_matches.is_empty() {
            issues.push(Issue {
                id: "STY002".to_string(),
                category: "style".to_string(),
                severity: Severity::Info,
                message: format!(
                    "Aggressive emphasis with ALL CAPS: {}",
                    caps_matches.iter().map(|m| m.as_str()).collect::<Vec<_>>().join(", ")
                ),
                line: Some(idx + 1),
                suggestion: Some(
                    "Claude 4.5 follows instructions precisely; aggressive emphasis may cause overtriggering. \
                    Use normal casing.".to_string()
                ),
            });
        }

        if multi_exclaim.is_match(line) {
            issues.push(Issue {
                id: "STY002".to_string(),
                category: "style".to_string(),
                severity: Severity::Info,
                message: "Multiple exclamation marks detected".to_string(),
                line: Some(idx + 1),
                suggestion: Some(
                    "Reduce emphasis; Claude 4.5 doesn't need emphatic punctuation.".to_string(),
                ),
            });
        }
    }

    // STY003: Word "think" (when extended thinking might be disabled)
    let think_pattern = Regex::new(r"(?i)\b(think|thinking|think about|think through)\b").unwrap();

    for (idx, line) in lines.iter().enumerate() {
        if think_pattern.is_match(line) {
            issues.push(Issue {
                id: "STY003".to_string(),
                category: "style".to_string(),
                severity: Severity::Warning,
                message: "Word \"think\" detected - sensitive in Claude Opus 4.5 without extended thinking".to_string(),
                line: Some(idx + 1),
                suggestion: Some(
                    "Replace with alternatives: \"consider\", \"evaluate\", \"reflect on\", \"work through\".".to_string()
                ),
            });
        }
    }

    // STY004: Over-triggering language (multiple emphatic triggers)
    let emphatic_triggers =
        Regex::new(r"(?i)\b(critical|must|mandatory|required|essential|always|never|important)\b")
            .unwrap();

    let trigger_count = emphatic_triggers.find_iter(prompt).count();
    if trigger_count > 3 {
        issues.push(Issue {
            id: "STY004".to_string(),
            category: "style".to_string(),
            severity: Severity::Info,
            message: format!(
                "Multiple emphatic triggers detected ({} instances) - may cause overtriggering",
                trigger_count
            ),
            line: None,
            suggestion: Some(
                "Claude 4.5 is more responsive; dial back aggressive language. \
                Simple instructions like \"Use this tool when...\" are sufficient."
                    .to_string(),
            ),
        });
    }

    issues
}

/// Analyze for tool usage issues (TUL001-003)
fn analyze_tools(prompt: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = prompt.lines().collect();

    // TUL001: Suggestion without action
    let suggestion_patterns = Regex::new(
        r"(?i)\b(suggest|recommend|what do you think|how would you|propose|advise)\b.*\b(changes?|improvements?|modifications?)\b"
    ).unwrap();

    for (idx, line) in lines.iter().enumerate() {
        if suggestion_patterns.is_match(line) {
            issues.push(Issue {
                id: "TUL001".to_string(),
                category: "tools".to_string(),
                severity: Severity::Warning,
                message: "Request for suggestions may result in advice rather than action"
                    .to_string(),
                line: Some(idx + 1),
                suggestion: Some(
                    "If you want changes implemented, use direct language: \
                    \"Make these changes\" or \"Implement improvements\"."
                        .to_string(),
                ),
            });
        }
    }

    // TUL002: Multiple operations without parallel guidance
    let multi_file_pattern = Regex::new(
        r"(?i)\b(all|every|each|multiple)\s+\w*\s*(files?|endpoints?|functions?|tests?)\b",
    )
    .unwrap();

    if multi_file_pattern.is_match(prompt) {
        let has_parallel_guidance = prompt.contains("parallel")
            || prompt.contains("simultaneously")
            || prompt.contains("sequential")
            || prompt.contains("one at a time");

        if !has_parallel_guidance {
            issues.push(Issue {
                id: "TUL002".to_string(),
                category: "tools".to_string(),
                severity: Severity::Info,
                message: "Multiple operations without parallel/sequential guidance".to_string(),
                line: None,
                suggestion: Some(
                    "Claude 4.5 excels at parallel tool calls. Consider adding: \
                    \"If independent, process in parallel for efficiency.\""
                        .to_string(),
                ),
            });
        }
    }

    // TUL003: Missing cleanup instructions
    let temp_file_indicators =
        Regex::new(r"(?i)\b(test|temp|temporary|helper|scratch|debug)\s*(script|file|code)\b")
            .unwrap();

    if temp_file_indicators.is_match(prompt) {
        let has_cleanup = prompt.contains("clean up")
            || prompt.contains("cleanup")
            || prompt.contains("remove")
            || prompt.contains("delete")
            || prompt.contains("after");

        if !has_cleanup {
            issues.push(Issue {
                id: "TUL003".to_string(),
                category: "tools".to_string(),
                severity: Severity::Info,
                message: "Temporary file creation without cleanup instructions".to_string(),
                line: None,
                suggestion: Some(
                    "Add: \"Clean up any temporary files created during this process.\""
                        .to_string(),
                ),
            });
        }
    }

    issues
}

/// Analyze for formatting issues (FMT001-003)
fn analyze_formatting(prompt: &str) -> Vec<Issue> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = prompt.lines().collect();

    // FMT001: Missing format specification for complex outputs
    let complex_output_indicators =
        Regex::new(r"(?i)\b(explain|describe|analyze|write|create|generate|produce)\b").unwrap();

    if complex_output_indicators.is_match(prompt) && prompt.len() > 50 {
        let has_format_spec = prompt.contains("format")
            || prompt.contains("structure")
            || prompt.contains("heading")
            || prompt.contains("section")
            || prompt.contains("bullet")
            || prompt.contains("paragraph")
            || prompt.contains("```")
            || prompt.contains("<");

        if !has_format_spec {
            issues.push(Issue {
                id: "FMT001".to_string(),
                category: "formatting".to_string(),
                severity: Severity::Info,
                message: "No explicit format specification for output".to_string(),
                line: None,
                suggestion: Some(
                    "Specify desired output format explicitly (prose, markdown, code blocks, etc.)."
                        .to_string(),
                ),
            });
        }
    }

    // FMT002: Negative format instructions
    let negative_format = Regex::new(
        r"(?i)\b(no|don't|do not|avoid|without)\s+(markdown|bullet|list|formatting|bold|italic)\b",
    )
    .unwrap();

    for (idx, line) in lines.iter().enumerate() {
        if negative_format.is_match(line) {
            issues.push(Issue {
                id: "FMT002".to_string(),
                category: "formatting".to_string(),
                severity: Severity::Warning,
                message: "Negative format instruction detected".to_string(),
                line: Some(idx + 1),
                suggestion: Some(
                    "Reframe positively: instead of \"no markdown\", \
                    use \"write in flowing prose paragraphs\"."
                        .to_string(),
                ),
            });
        }
    }

    // FMT003: Complex prompt without XML structure
    let has_multiple_sections =
        prompt.contains(":") && (prompt.matches(':').count() > 3) && prompt.len() > 300;

    let has_xml = prompt.contains('<') && prompt.contains('>');

    if has_multiple_sections && !has_xml {
        issues.push(Issue {
            id: "FMT003".to_string(),
            category: "formatting".to_string(),
            severity: Severity::Info,
            message: "Complex prompt may benefit from XML tag organization".to_string(),
            line: None,
            suggestion: Some(
                "Consider using semantic XML tags to structure sections: \
                <rules>, <examples>, <input>, <output_format>."
                    .to_string(),
            ),
        });
    }

    issues
}

/// Analyze for verbosity issues (VRB001-002)
fn analyze_verbosity(prompt: &str) -> Vec<Issue> {
    let mut issues = Vec::new();

    // VRB001: Missing verbosity guidance for complex tasks
    let complex_task =
        Regex::new(r"(?i)\b(refactor|implement|build|create|develop|migrate)\b").unwrap();

    if complex_task.is_match(prompt) && prompt.len() > 100 {
        let has_verbosity = prompt.contains("summary")
            || prompt.contains("brief")
            || prompt.contains("detailed")
            || prompt.contains("verbose")
            || prompt.contains("concise");

        if !has_verbosity {
            issues.push(Issue {
                id: "VRB001".to_string(),
                category: "verbosity".to_string(),
                severity: Severity::Info,
                message: "Complex task without verbosity guidance".to_string(),
                line: None,
                suggestion: Some(
                    "Claude 4.5 tends toward efficiency. Add: \"After completing, \
                    provide a brief summary of changes made.\""
                        .to_string(),
                ),
            });
        }
    }

    // VRB002: Multi-step without progress reporting
    let multi_step = prompt.contains("multiple")
        || prompt.contains("several")
        || prompt.contains("all")
        || Regex::new(r"\b\d+\s*(files?|steps?|items?)\b")
            .unwrap()
            .is_match(prompt);

    if multi_step && !prompt.contains("progress") && !prompt.contains("update") {
        issues.push(Issue {
            id: "VRB002".to_string(),
            category: "verbosity".to_string(),
            severity: Severity::Info,
            message: "Multi-step task without progress reporting guidance".to_string(),
            line: None,
            suggestion: Some(
                "Consider adding: \"Provide a quick update after each step.\"".to_string(),
            ),
        });
    }

    issues
}

/// Analyze for agentic coding issues (AGT001-004)
fn analyze_agentic(prompt: &str) -> Vec<Issue> {
    let mut issues = Vec::new();

    // AGT001: Code modification without exploration directive
    let code_mod_patterns = Regex::new(
        r"(?i)\b(fix|update|change|modify|edit|refactor)\b.*\b(code|function|class|file|module)\b",
    )
    .unwrap();

    if code_mod_patterns.is_match(prompt) {
        let has_exploration = prompt.contains("read")
            || prompt.contains("understand")
            || prompt.contains("inspect")
            || prompt.contains("review")
            || prompt.contains("look at")
            || prompt.contains("examine");

        if !has_exploration {
            issues.push(Issue {
                id: "AGT001".to_string(),
                category: "agentic".to_string(),
                severity: Severity::Warning,
                message: "Code modification without exploration directive".to_string(),
                line: None,
                suggestion: Some(
                    "Add: \"First, read and understand the relevant files before making changes.\""
                        .to_string(),
                ),
            });
        }
    }

    // AGT002: Questions about code without investigation requirement
    let code_question =
        Regex::new(r"(?i)\b(why|how|what)\b.*\b(code|function|bug|error|issue|failing)\b").unwrap();

    if code_question.is_match(prompt) {
        let has_investigation = prompt.contains("investigate")
            || prompt.contains("inspect")
            || prompt.contains("don't speculate")
            || prompt.contains("do not speculate");

        if !has_investigation {
            issues.push(Issue {
                id: "AGT002".to_string(),
                category: "agentic".to_string(),
                severity: Severity::Warning,
                message: "Code question without hallucination prevention".to_string(),
                line: None,
                suggestion: Some(
                    "Add: \"Investigate the relevant files before answering. \
                    Do not speculate about code you haven't read.\""
                        .to_string(),
                ),
            });
        }
    }

    // AGT003: Complex implementation without state tracking
    let complex_impl =
        Regex::new(r"(?i)\b(implement|build|create)\b.*\b(full|complete|entire|whole)\b").unwrap();

    if complex_impl.is_match(prompt) {
        let has_state_tracking = prompt.contains("progress")
            || prompt.contains("track")
            || prompt.contains("git")
            || prompt.contains("commit")
            || prompt.contains("checkpoint");

        if !has_state_tracking {
            issues.push(Issue {
                id: "AGT003".to_string(),
                category: "agentic".to_string(),
                severity: Severity::Info,
                message: "Complex implementation without state management guidance".to_string(),
                line: None,
                suggestion: Some(
                    "Add state tracking: \"Track progress in a progress.txt file. \
                    Use git commits to checkpoint your work.\""
                        .to_string(),
                ),
            });
        }
    }

    // AGT004: Open-ended implementation without anti-overengineering
    let open_ended = Regex::new(
        r"(?i)\b(build|create|implement|design)\s+(a|an)\s+\w+\s+(system|solution|service)\b",
    )
    .unwrap();

    if open_ended.is_match(prompt) {
        let has_simplicity = prompt.contains("simple")
            || prompt.contains("minimal")
            || prompt.contains("don't over")
            || prompt.contains("avoid over")
            || prompt.contains("only what");

        if !has_simplicity {
            issues.push(Issue {
                id: "AGT004".to_string(),
                category: "agentic".to_string(),
                severity: Severity::Info,
                message: "Open-ended implementation may lead to overengineering".to_string(),
                line: None,
                suggestion: Some(
                    "Add: \"Avoid over-engineering. Only implement what's directly needed.\""
                        .to_string(),
                ),
            });
        }
    }

    issues
}

/// Analyze for long-horizon task issues (LHT001-003)
fn analyze_long_horizon(prompt: &str) -> Vec<Issue> {
    let mut issues = Vec::new();

    // Detect indicators of long/complex tasks
    let long_task_indicators = prompt.len() > 500
        || prompt.contains("entire")
        || prompt.contains("all the")
        || prompt.contains("complete")
        || prompt.contains("full");

    if !long_task_indicators {
        return issues;
    }

    // LHT001: Long task without persistence strategy
    let has_persistence = prompt.contains("save")
        || prompt.contains("persist")
        || prompt.contains("file")
        || prompt.contains("git")
        || prompt.contains("commit")
        || prompt.contains("checkpoint");

    if !has_persistence {
        issues.push(Issue {
            id: "LHT001".to_string(),
            category: "long_horizon".to_string(),
            severity: Severity::Warning,
            message: "Long task without state persistence strategy".to_string(),
            line: None,
            suggestion: Some(
                "Add: \"If context runs low, save your progress and state before continuing.\""
                    .to_string(),
            ),
        });
    }

    // LHT002: Large scope without incremental guidance
    let has_incremental = prompt.contains("incremental")
        || prompt.contains("one at a time")
        || prompt.contains("step by step")
        || prompt.contains("iteratively");

    if !has_incremental {
        issues.push(Issue {
            id: "LHT002".to_string(),
            category: "long_horizon".to_string(),
            severity: Severity::Info,
            message: "Large task scope without incremental progress guidance".to_string(),
            line: None,
            suggestion: Some(
                "Add: \"Work incrementally, completing one component before moving to the next.\""
                    .to_string(),
            ),
        });
    }

    // LHT003: Extended task without context awareness
    let has_context_awareness = prompt.contains("context")
        || prompt.contains("budget")
        || prompt.contains("token")
        || prompt.contains("limit");

    if !has_context_awareness && prompt.len() > 800 {
        issues.push(Issue {
            id: "LHT003".to_string(),
            category: "long_horizon".to_string(),
            severity: Severity::Info,
            message: "Extended task without context window awareness".to_string(),
            line: None,
            suggestion: Some(
                "Consider adding context awareness instructions for very long tasks.".to_string(),
            ),
        });
    }

    issues
}

/// Analyze for frontend design issues (FED001-002)
fn analyze_frontend(prompt: &str) -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check if this is a frontend-related prompt
    let frontend_indicators = Regex::new(
        r"(?i)\b(ui|frontend|page|component|dashboard|form|button|layout|design|css|html|react|vue|web)\b"
    ).unwrap();

    if !frontend_indicators.is_match(prompt) {
        return issues;
    }

    // FED001: Generic UI request without aesthetic guidance
    let ui_creation =
        Regex::new(r"(?i)\b(create|build|make|design)\b.*\b(ui|page|component|form|dashboard)\b")
            .unwrap();

    if ui_creation.is_match(prompt) {
        let has_aesthetics = prompt.contains("aesthetic")
            || prompt.contains("design")
            || prompt.contains("style")
            || prompt.contains("beautiful")
            || prompt.contains("creative")
            || prompt.contains("distinctive");

        if !has_aesthetics {
            issues.push(Issue {
                id: "FED001".to_string(),
                category: "frontend".to_string(),
                severity: Severity::Info,
                message: "UI request without aesthetic guidance may result in generic design"
                    .to_string(),
                line: None,
                suggestion: Some(
                    "Add design guidance: \"Create a distinctive, creative design. \
                    Avoid generic 'AI slop' aesthetics.\""
                        .to_string(),
                ),
            });
        }
    }

    // FED002: Missing typography/color/motion guidance
    let has_design_details = prompt.contains("font")
        || prompt.contains("typography")
        || prompt.contains("color")
        || prompt.contains("palette")
        || prompt.contains("animation")
        || prompt.contains("motion");

    if ui_creation.is_match(prompt) && !has_design_details {
        issues.push(Issue {
            id: "FED002".to_string(),
            category: "frontend".to_string(),
            severity: Severity::Info,
            message: "Frontend request without specific design guidance".to_string(),
            line: None,
            suggestion: Some(
                "Consider specifying typography, color scheme, and motion preferences.".to_string(),
            ),
        });
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_vague_instruction() {
        let issues = analyze("Create a dashboard", None).unwrap();
        assert!(issues.iter().any(|i| i.id == "EXP001"));
    }

    #[test]
    fn test_detect_indirect_command() {
        let issues = analyze("Can you fix this bug?", None).unwrap();
        assert!(issues.iter().any(|i| i.id == "EXP003"));
    }

    #[test]
    fn test_detect_negative_instruction() {
        let issues = analyze("Don't use markdown in your response", None).unwrap();
        assert!(issues.iter().any(|i| i.id == "STY001"));
    }

    #[test]
    fn test_detect_think_word() {
        let issues = analyze("Think about the edge cases", None).unwrap();
        assert!(issues.iter().any(|i| i.id == "STY003"));
    }

    #[test]
    fn test_detect_suggestion_language() {
        let issues = analyze("Can you suggest some changes to improve this?", None).unwrap();
        assert!(issues.iter().any(|i| i.id == "TUL001"));
    }

    #[test]
    fn test_category_filtering() {
        let prompt = "Can you suggest some changes? Don't use markdown.";

        // Only check style
        let style_issues = analyze(prompt, Some(&["style".to_string()])).unwrap();
        assert!(style_issues.iter().all(|i| i.category == "style"));

        // Only check tools
        let tool_issues = analyze(prompt, Some(&["tools".to_string()])).unwrap();
        assert!(tool_issues.iter().all(|i| i.category == "tools"));
    }
}

//! Prompt analyzer module
//!
//! This module analyzes prompts for common anti-patterns and issues
//! based on Claude 4.5 best practices.

use anyhow::Result;
use regex::Regex;

/// Prompt type for context-aware rule application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptType {
    Coding,
    QaAssistant,
    Research,
    Creative,
    LongHorizon,
    General,
}

/// Classify prompt type for context-aware analysis
pub fn classify_prompt(prompt: &str) -> PromptType {
    let lower = prompt.to_lowercase();

    // Check for long-horizon indicators first (most specific)
    if lower.contains("multi-step")
        || lower.contains("over multiple")
        || lower.contains("long-running")
        || lower.contains("complex project")
        || (lower.contains("implement") && lower.contains("full"))
    {
        return PromptType::LongHorizon;
    }

    // Coding task indicators
    if lower.contains("code")
        || lower.contains("function")
        || lower.contains("refactor")
        || lower.contains("debug")
        || lower.contains("implement")
        || lower.contains("fix the bug")
        || lower.contains("write a program")
    {
        return PromptType::Coding;
    }

    // Q&A Assistant indicators
    if lower.contains("answer question")
        || lower.contains("answer any")
        || lower.contains("assist with")
        || lower.contains("help with")
        || lower.contains("you are a") && (lower.contains("assistant") || lower.contains("agent"))
    {
        return PromptType::QaAssistant;
    }

    // Research indicators
    if lower.contains("research")
        || lower.contains("investigate")
        || lower.contains("analyze")
        || lower.contains("find information")
    {
        return PromptType::Research;
    }

    // Creative indicators
    if lower.contains("write a story")
        || lower.contains("create content")
        || lower.contains("generate")
        || lower.contains("creative")
    {
        return PromptType::Creative;
    }

    PromptType::General
}

/// Get applicable rule categories for a prompt type
pub fn get_applicable_categories(prompt_type: PromptType) -> Vec<&'static str> {
    match prompt_type {
        PromptType::Coding => vec!["explicitness", "style", "tools", "formatting", "agentic"],
        PromptType::QaAssistant => vec!["explicitness", "style", "formatting"],
        PromptType::Research => vec!["explicitness", "style", "agentic", "verbosity"],
        PromptType::Creative => vec!["explicitness", "style", "formatting", "frontend"],
        PromptType::LongHorizon => vec![
            "explicitness",
            "style",
            "tools",
            "formatting",
            "verbosity",
            "agentic",
            "long_horizon",
            "frontend",
        ],
        PromptType::General => vec!["explicitness", "style", "formatting"],
    }
}

/// XML block preserved during analysis (fields used for future reconstruction)
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct XmlBlock {
    pub tag: String,
    pub content: String,
    pub start: usize,
    pub end: usize,
}

/// Extract XML blocks (examples, instructions, etc.) to prevent false positives
pub fn extract_xml_blocks(prompt: &str) -> (String, Vec<XmlBlock>) {
    let mut blocks = Vec::new();
    let mut cleaned = prompt.to_string();

    // Tags to extract and preserve (not analyze)
    let preserve_tags = [
        "examples",
        "example",
        "input",
        "output",
        "context",
        "background",
    ];

    for tag in preserve_tags {
        let pattern = format!(r"(?s)<{}>(.*?)</{}>", tag, tag);
        if let Ok(re) = Regex::new(&pattern) {
            for cap in re.captures_iter(prompt) {
                if let (Some(full_match), Some(content)) = (cap.get(0), cap.get(1)) {
                    blocks.push(XmlBlock {
                        tag: tag.to_string(),
                        content: content.as_str().to_string(),
                        start: full_match.start(),
                        end: full_match.end(),
                    });
                }
            }
            // Remove the matched blocks from cleaned text for analysis
            cleaned = re.replace_all(&cleaned, "").to_string();
        }
    }

    (cleaned, blocks)
}

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

/// All available rule categories (used when explicit category check is requested)
#[allow(dead_code)]
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

    // Extract XML blocks to prevent false positives from examples
    let (cleaned_prompt, _xml_blocks) = extract_xml_blocks(prompt);

    // Classify prompt type for context-aware analysis
    let prompt_type = classify_prompt(prompt);

    // Determine which categories to check
    let categories_to_check: Vec<&str> = match check_categories {
        Some(cats) => cats.iter().map(|s| s.as_str()).collect(),
        None => {
            // Use context-aware categories based on prompt type
            get_applicable_categories(prompt_type)
        }
    };

    // Run all applicable analyzers on cleaned prompt (without XML blocks)
    for category in categories_to_check {
        match category {
            "explicitness" => issues.extend(analyze_explicitness(&cleaned_prompt, prompt_type)),
            "style" => issues.extend(analyze_style(&cleaned_prompt)),
            "tools" => issues.extend(analyze_tools(&cleaned_prompt)),
            "formatting" => issues.extend(analyze_formatting(&cleaned_prompt)),
            "verbosity" => issues.extend(analyze_verbosity(&cleaned_prompt)),
            "agentic" => issues.extend(analyze_agentic(&cleaned_prompt)),
            "long_horizon" => issues.extend(analyze_long_horizon(&cleaned_prompt)),
            "frontend" => issues.extend(analyze_frontend(&cleaned_prompt)),
            _ => {} // Unknown category, skip
        }
    }

    Ok(issues)
}

/// Analyze for explicitness issues (EXP001-006)
fn analyze_explicitness(prompt: &str, prompt_type: PromptType) -> Vec<Issue> {
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

    // EXP005: Role-only prompt without specific actions
    let role_pattern = Regex::new(r"(?i)^you are\s+(a|an)\s+").unwrap();
    let task_pattern = Regex::new(r"(?i)your task is to\s+").unwrap();

    if role_pattern.is_match(prompt) {
        // Check if there are specific action directives
        let has_specific_actions = prompt.contains("When the user")
            || prompt.contains("If the user")
            || prompt.contains("For each")
            || prompt.contains("Always respond with")
            || prompt.contains("Format your response")
            || Regex::new(r"(?i)\b(first|then|finally|step \d)\b")
                .unwrap()
                .is_match(prompt);

        let has_passive_task = task_pattern.is_match(prompt);

        if !has_specific_actions && (has_passive_task || prompt_type == PromptType::QaAssistant) {
            issues.push(Issue {
                id: "EXP005".to_string(),
                category: "explicitness".to_string(),
                severity: Severity::Warning,
                message: "Role-only prompt without specific action directives".to_string(),
                line: Some(1),
                suggestion: Some(
                    "Add explicit actions: 'When the user asks about X, respond with Y format.' \
                    Claude 4.5 follows instructions precisely - be specific about what you want."
                        .to_string(),
                ),
            });
        }
    }

    // EXP006: Open-ended instructions
    let open_ended_pattern = Regex::new(
        r"(?i)\b(answer any|respond to any|help with any|handle any|assist with any)\s+\w+",
    )
    .unwrap();

    if open_ended_pattern.is_match(prompt) {
        let has_boundaries = prompt.contains("format")
            || prompt.contains("limit")
            || prompt.contains("only")
            || prompt.contains("scope")
            || prompt.contains("boundaries");

        if !has_boundaries {
            issues.push(Issue {
                id: "EXP006".to_string(),
                category: "explicitness".to_string(),
                severity: Severity::Warning,
                message: "Open-ended instruction without boundaries or format specification"
                    .to_string(),
                line: None,
                suggestion: Some(
                    "Specify: What format should responses use? What topics are in scope? \
                    How detailed should answers be? How to handle unknown information?"
                        .to_string(),
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
    let complex_output_indicators = Regex::new(
        r"(?i)\b(explain|describe|analyze|write|create|generate|produce|answer|respond|reply)\b",
    )
    .unwrap();

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
        let issues = analyze_tools("Please suggest some changes to improve the code");
        assert!(issues.iter().any(|i| i.id == "TUL001"));
    }

    #[test]
    fn test_detect_role_only_prompt() {
        let prompt = "You are an experienced travel assistant. Your task is to answer questions about flights.";
        let issues = analyze_explicitness(prompt, PromptType::QaAssistant);
        assert!(issues.iter().any(|i| i.id == "EXP005"));
    }

    #[test]
    fn test_detect_open_ended_instructions() {
        let prompt = "Answer any questions the user might have about the product.";
        let issues = analyze_explicitness(prompt, PromptType::QaAssistant);
        assert!(issues.iter().any(|i| i.id == "EXP006"));
    }

    #[test]
    fn test_xml_extraction() {
        let prompt = "Do this task.\n<examples>\n<example>Example 1</example>\n</examples>";
        let (cleaned, blocks) = extract_xml_blocks(prompt);
        assert!(cleaned.contains("Do this task"));
        assert!(!cleaned.contains("<examples>"));
        assert!(!blocks.is_empty());
    }

    #[test]
    fn test_prompt_classifier() {
        assert_eq!(
            classify_prompt("You are an assistant. Answer any questions."),
            PromptType::QaAssistant
        );
        assert_eq!(
            classify_prompt("Fix the bug in this function"),
            PromptType::Coding
        );
        assert_eq!(
            classify_prompt("Research the history of AI"),
            PromptType::Research
        );
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

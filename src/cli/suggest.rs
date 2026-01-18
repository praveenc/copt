//! Interactive suggestion module for vague prompt improvement
//!
//! When prompts trigger EXP005 (role-only) or EXP006 (open-ended),
//! this module offers interactive suggestions to improve them.

use crate::analyzer::Issue;
use anyhow::Result;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};

/// Suggestion templates for improving vague prompts
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub template: &'static str,
}

/// Available suggestions for role-only prompts (EXP005)
pub const ROLE_SUGGESTIONS: &[Suggestion] = &[
    Suggestion {
        id: "response_format",
        label: "Response format specification",
        description: "Define how responses should be structured",
        template: r#"
<response_format>
Structure your responses as follows:
- Start with a brief summary (1-2 sentences)
- Provide detailed explanation with relevant context
- Use bullet points for lists of items
- End with any caveats or additional considerations
</response_format>"#,
    },
    Suggestion {
        id: "source_citation",
        label: "Source citation requirements",
        description: "Require citing sources for answers",
        template: r#"
<citation_requirements>
When answering questions:
- Reference the specific document or section where you found the information
- Use phrases like "According to [document name]..." or "Based on [section]..."
- If information is not found in the provided materials, clearly state this
</citation_requirements>"#,
    },
    Suggestion {
        id: "unknown_handling",
        label: "Unknown information handling",
        description: "How to handle questions without answers",
        template: r#"
<unknown_handling>
If you cannot find the answer in the provided documentation:
- Clearly state that the specific information is not available
- Do not speculate or make up information
- Suggest where the user might find the answer (e.g., "Contact support for...")
</unknown_handling>"#,
    },
    Suggestion {
        id: "response_length",
        label: "Response length guidance",
        description: "Set expectations for response verbosity",
        template: r#"
<response_length>
Adjust response length based on query complexity:
- Simple factual questions: 1-3 sentences
- Explanatory questions: 1-2 paragraphs
- Complex comparisons or analyses: Detailed response with sections
</response_length>"#,
    },
    Suggestion {
        id: "action_directive",
        label: "Action directive (default to action)",
        description: "Make Claude take action rather than suggest",
        template: r#"
<default_to_action>
When the user asks for help, provide direct answers rather than asking clarifying questions unless absolutely necessary. Infer the most useful response based on context.
</default_to_action>"#,
    },
];

/// Available suggestions for open-ended prompts (EXP006)
pub const OPENENDED_SUGGESTIONS: &[Suggestion] = &[
    Suggestion {
        id: "scope_boundaries",
        label: "Topic scope boundaries",
        description: "Define what topics are in/out of scope",
        template: r#"
<scope>
In-scope topics:
- [List specific topics this assistant should handle]

Out-of-scope topics (politely decline):
- [List topics to avoid or redirect]
</scope>"#,
    },
    Suggestion {
        id: "expertise_level",
        label: "Expertise level assumption",
        description: "Set the assumed user expertise level",
        template: r#"
<expertise_level>
Assume the user has [beginner/intermediate/expert] knowledge. Adjust explanations accordingly:
- Avoid unnecessary jargon for beginners
- Skip basic explanations for experts
- Define technical terms when first used
</expertise_level>"#,
    },
    Suggestion {
        id: "interaction_style",
        label: "Interaction style",
        description: "Define the conversation tone and style",
        template: r#"
<interaction_style>
Maintain a [professional/friendly/casual] tone. Be:
- Concise but thorough
- Helpful without being verbose
- Direct in providing information
</interaction_style>"#,
    },
];

/// Check if issues warrant interactive suggestions
pub fn should_suggest(issues: &[Issue]) -> bool {
    issues.iter().any(|i| i.id == "EXP005" || i.id == "EXP006")
}

/// Get relevant suggestions based on detected issues
pub fn get_suggestions_for_issues(issues: &[Issue]) -> Vec<&'static Suggestion> {
    let mut suggestions = Vec::new();

    let has_exp005 = issues.iter().any(|i| i.id == "EXP005");
    let has_exp006 = issues.iter().any(|i| i.id == "EXP006");

    if has_exp005 {
        suggestions.extend(ROLE_SUGGESTIONS.iter());
    }

    if has_exp006 {
        suggestions.extend(OPENENDED_SUGGESTIONS.iter());
    }

    // Deduplicate by id (in case of overlap)
    suggestions.sort_by_key(|s| s.id);
    suggestions.dedup_by_key(|s| s.id);

    suggestions
}

/// Run interactive suggestion flow
/// Returns the enhanced prompt or None if user skips
pub fn run_interactive_suggestions(
    original_prompt: &str,
    issues: &[Issue],
) -> Result<Option<String>> {
    if !should_suggest(issues) {
        return Ok(None);
    }

    println!();
    println!(
        "  {}  {}",
        "⚠".yellow(),
        "Vague prompt detected".yellow().bold()
    );
    println!();

    // Show which issues were detected
    for issue in issues
        .iter()
        .filter(|i| i.id == "EXP005" || i.id == "EXP006")
    {
        println!(
            "     {} {}: {}",
            "•".bright_black(),
            issue.id.cyan(),
            issue.message
        );
        if let Some(ref suggestion) = issue.suggestion {
            println!(
                "       {} {}",
                "→".bright_black(),
                suggestion.bright_black()
            );
        }
    }
    println!();

    // Ask if user wants suggestions
    let wants_suggestions = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Would you like to add specific improvements to this prompt?")
        .default(true)
        .interact()?;

    if !wants_suggestions {
        return Ok(None);
    }

    // Get relevant suggestions
    let suggestions = get_suggestions_for_issues(issues);

    if suggestions.is_empty() {
        return Ok(None);
    }

    // Build selection items
    let items: Vec<String> = suggestions
        .iter()
        .map(|s| format!("{} - {}", s.label, s.description))
        .collect();

    println!();
    println!(
        "  {}  {}",
        "📝".cyan(),
        "Select improvements to add:".white().bold()
    );
    println!(
        "     {}",
        "(Space to toggle, Enter to confirm)".bright_black()
    );
    println!();

    // Multi-select dialog
    let selected_indices = MultiSelect::with_theme(&ColorfulTheme::default())
        .items(&items)
        .defaults(&vec![false; items.len()])
        .interact()?;

    if selected_indices.is_empty() {
        println!(
            "  {}  {}",
            "ℹ".blue(),
            "No improvements selected, using original prompt.".bright_black()
        );
        return Ok(None);
    }

    // Build enhanced prompt
    let mut enhanced = original_prompt.trim().to_string();
    enhanced.push('\n');

    for idx in &selected_indices {
        let suggestion = suggestions[*idx];
        enhanced.push_str(suggestion.template);
        enhanced.push('\n');
    }

    // Show preview
    println!();
    println!(
        "  {}  {}",
        "✨".green(),
        "Additions that will be appended:".white().bold()
    );
    println!("  {}", "─".repeat(60).bright_black());

    for idx in &selected_indices {
        let suggestion = suggestions[*idx];
        println!("     {} {}", "✓".green(), suggestion.label);
    }

    println!("  {}", "─".repeat(60).bright_black());
    println!();

    // Confirm
    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Apply these improvements?")
        .default(true)
        .interact()?;

    if confirm {
        println!(
            "  {}  {}",
            "✓".green(),
            "Prompt enhanced with selected improvements.".green()
        );
        Ok(Some(enhanced))
    } else {
        Ok(None)
    }
}

/// Non-interactive suggestion: just show what could be improved
pub fn print_suggestions(issues: &[Issue]) {
    if !should_suggest(issues) {
        return;
    }

    let suggestions = get_suggestions_for_issues(issues);

    println!();
    println!(
        "  {}  {}",
        "💡".cyan(),
        "Suggested improvements for this prompt:".white().bold()
    );
    println!();

    for suggestion in suggestions {
        println!("     {} {}", "•".cyan(), suggestion.label.white());
        println!("       {}", suggestion.description.bright_black());
    }

    println!();
    println!(
        "     {}",
        "Run with --suggest to interactively add these improvements.".bright_black()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::Severity;

    fn make_issue(id: &str) -> Issue {
        Issue {
            id: id.to_string(),
            category: "explicitness".to_string(),
            severity: Severity::Warning,
            message: "Test issue".to_string(),
            line: None,
            suggestion: None,
        }
    }

    #[test]
    fn test_should_suggest_exp005() {
        let issues = vec![make_issue("EXP005")];
        assert!(should_suggest(&issues));
    }

    #[test]
    fn test_should_suggest_exp006() {
        let issues = vec![make_issue("EXP006")];
        assert!(should_suggest(&issues));
    }

    #[test]
    fn test_should_not_suggest_other() {
        let issues = vec![make_issue("EXP001"), make_issue("STY001")];
        assert!(!should_suggest(&issues));
    }

    #[test]
    fn test_get_suggestions_exp005() {
        let issues = vec![make_issue("EXP005")];
        let suggestions = get_suggestions_for_issues(&issues);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.id == "response_format"));
    }

    #[test]
    fn test_get_suggestions_exp006() {
        let issues = vec![make_issue("EXP006")];
        let suggestions = get_suggestions_for_issues(&issues);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.id == "scope_boundaries"));
    }
}

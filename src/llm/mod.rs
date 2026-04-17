//! LLM Client Module
//!
//! Provides unified interface for Claude 4.x API access via:
//! - Anthropic API (direct)
//! - AWS Bedrock
//!
//! Also hosts the model-family classifier and family-specific optimizer
//! meta-prompts. The classifier selects which meta-prompt is used when an
//! LLM-powered optimization is requested:
//!
//! - `ModelFamily::Claude45` / `Claude46` → the 4.5/4.6 meta-prompt (unchanged)
//! - `ModelFamily::Claude47` → the 4.7 meta-prompt covering literal instruction
//!   following, adaptive thinking guidance, effort-level awareness,
//!   scratchpad/memory directives, condensed context, tone specification,
//!   response-length calibration, and vision-aware instructions.

mod anthropic;
mod bedrock;

pub use anthropic::AnthropicClient;
pub use bedrock::BedrockApiKeyClient;
pub use bedrock::BedrockClient;

use anyhow::{anyhow, Result};
use async_trait::async_trait;

/// Unified LLM client interface
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Send a completion request to the LLM
    async fn complete(
        &self,
        system: &str,
        user_message: &str,
        model: &str,
        max_tokens: u32,
    ) -> Result<String>;

    /// Get the provider name
    #[allow(dead_code)]
    fn provider_name(&self) -> &str;
}

/// Claude model family — drives which optimizer meta-prompt is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFamily {
    /// Claude 4.5 (Sonnet 4.5, Opus 4.5, Haiku 4.5). Current default.
    Claude45,
    /// Claude 4.6 (Opus 4.6, Sonnet 4.6). Shares the 4.5 prompting style.
    Claude46,
    /// Claude 4.7 (Opus 4.7 as of 2026-04-16). New prompting contract:
    /// literal instruction following, adaptive thinking, effort awareness,
    /// native memory/scratchpad use, condensed context, direct tone, calibrated
    /// response length, high-resolution vision with 1:1 pixel coordinates.
    Claude47,
}

impl ModelFamily {
    /// Classify a model ID or alias into a family.
    ///
    /// The match is deliberately loose — any string containing `-4-7`, `-4.7`,
    /// or `4-7-` counts as 4.7. Unknown inputs fall back to `Claude45` (the
    /// historical default), which keeps existing behaviour for anything the
    /// classifier does not recognise.
    pub fn from_model_id(model: &str) -> Self {
        let lower = model.to_lowercase();
        if lower.contains("-4-7")
            || lower.contains("-4.7")
            || lower.contains("4-7-")
            || lower.contains("opus-4.7")
            || lower.contains("sonnet-4.7")
            || lower.contains("haiku-4.7")
        {
            return ModelFamily::Claude47;
        }
        if lower.contains("-4-6") || lower.contains("-4.6") || lower.contains("4-6-") {
            return ModelFamily::Claude46;
        }
        ModelFamily::Claude45
    }

    /// Human-readable label used in user messages and the optimizer prompt.
    pub fn display_label(&self) -> &'static str {
        match self {
            ModelFamily::Claude45 => "Claude 4.5",
            ModelFamily::Claude46 => "Claude 4.6",
            ModelFamily::Claude47 => "Claude 4.7",
        }
    }
}

/// Return a descriptive error if the caller asked for a 4.7 model that is not
/// yet released (as of 2026-04-16 only Claude Opus 4.7 is generally available).
///
/// Called by `run_optimization`, `run_interactive_mode`, and the Bedrock
/// connectivity check so that offline/static paths are still usable while
/// online paths fail fast with a clear message.
pub fn unreleased_model_error(model: &str) -> Option<anyhow::Error> {
    let lower = model.to_lowercase();
    // Only Opus 4.7 is GA — reject explicit sonnet/haiku 4.7 aliases.
    let asks_sonnet_47 = lower == "sonnet-4.7"
        || lower.contains("claude-sonnet-4-7")
        || lower.contains("claude-sonnet-4.7");
    let asks_haiku_47 = lower == "haiku-4.7"
        || lower.contains("claude-haiku-4-7")
        || lower.contains("claude-haiku-4.7");

    if asks_sonnet_47 {
        return Some(anyhow!(
            "Claude Sonnet 4.7 is not yet released (as of 2026-04-16). \
             Only Claude Opus 4.7 is generally available.\n\
             Use `-m opus-4.7` for Claude 4.7, or `-m sonnet` for the current Sonnet 4.5 default."
        ));
    }
    if asks_haiku_47 {
        return Some(anyhow!(
            "Claude Haiku 4.7 is not yet released (as of 2026-04-16). \
             Only Claude Opus 4.7 is generally available.\n\
             Use `-m opus-4.7` for Claude 4.7, or `-m haiku` for the current Haiku 4.5 default."
        ));
    }
    None
}

/// The meta-prompt used to optimize prompts for Claude 4.5 / 4.6 (unchanged
/// from prior releases).
pub const OPTIMIZER_SYSTEM_PROMPT_4_5: &str = r#"You are an expert prompt engineer specializing in optimizing prompts for Claude 4.5 models.

Your task is to improve the given prompt according to Anthropic's official best practices:

<optimization_rules>
1. EXPLICITNESS: Convert vague instructions to specific, actionable ones. Add detail about desired output.
2. CONTEXT: Add motivation/reasoning when it helps Claude understand intent (explain "why").
3. POSITIVE FRAMING: Replace negative instructions ("Don't...", "Never...") with positive guidance about what TO do.
4. TOOL USAGE: Add explicit directives for tool use when the intent is to take action, not just suggest.
5. FORMAT: Include clear format specifications. Use XML tags for complex prompts.
6. MODIFIERS: Add quality/detail modifiers where beneficial (e.g., "fully-featured", "comprehensive").
7. WORD CHOICE: Replace "think" with "consider", "evaluate", or "reflect" when appropriate.
8. TONE: Remove aggressive emphasis (ALL CAPS, excessive !!!) - Claude 4.5 follows instructions well without it.
</optimization_rules>

<prompt_type_awareness>
Tailor optimization based on the prompt's purpose:

- Q&A ASSISTANT prompts: Add response format specs, source citation guidance, handling for unknown information, and clear scope boundaries. Convert role-only definitions into actionable conditional handlers ("When the user asks about X, respond with Y").

- CODING prompts: Add exploration directives ("Read and understand code before modifying"), tool usage guidance, and anti-hallucination instructions ("Do not speculate about code you haven't inspected").

- RESEARCH prompts: Add structured approach guidance, hypothesis tracking, source evaluation criteria, and progress reporting instructions.

- CREATIVE prompts: Add style/tone specifications, audience awareness, and format flexibility while preserving creative intent.

- LONG-HORIZON prompts: Add state persistence strategies, incremental progress checkpoints, context window awareness, and clear milestone definitions.
</prompt_type_awareness>

<preserve_structure>
If the prompt contains XML blocks like <examples>, <example>, <instructions>, <context>, <rules>, <format>, or <schema>:
- PRESERVE these blocks and their content
- ENHANCE the content within blocks rather than removing them
- Maintain the XML structure as it provides clear semantic organization
- Add complementary XML blocks if they would improve clarity (e.g., <response_format>, <constraints>)
</preserve_structure>

<output_requirements>
- Return ONLY the optimized prompt text
- No explanations, no preamble, no markdown formatting around the output
- Preserve the original intent and meaning
- Keep the prompt practical and focused
- Do not over-engineer or add unnecessary complexity
- STRUCTURE: Wrap the rewrite in semantic XML tags at the top level — at minimum <task>, and whichever of <requirements>, <response_format>, <constraints>, <examples> apply. Do NOT wrap the entire output in a single outer <prompt> or <rewrite> tag; emit sibling top-level tags instead.
- LENGTH: The rewrite MUST NOT exceed ~3x the original word count. Prefer removing scaffolding over adding it. Omit sections that are not strictly useful for this particular prompt.
</output_requirements>"#;

/// Back-compat re-export — legacy name used by callers predating the 4.7 split.
#[allow(dead_code)]
pub const OPTIMIZER_SYSTEM_PROMPT: &str = OPTIMIZER_SYSTEM_PROMPT_4_5;

/// The meta-prompt used to optimize prompts for the Claude 4.7 model family.
///
/// 4.7 changes the prompting contract in eight user-visible ways, each called
/// out explicitly below so the optimizer can apply the right transformations
/// instead of relying on 4.5-era scaffolding.
pub const OPTIMIZER_SYSTEM_PROMPT_4_7: &str = r#"You are an expert prompt engineer specializing in optimizing prompts for the Claude 4.7 model family (Claude Opus 4.7 and, when released, Claude Sonnet 4.7 / Haiku 4.7).

Claude 4.7 changes the prompting contract relative to 4.5/4.6. Your rewrite MUST apply the eight rules below in addition to the general 4.x best practices.

<claude_4_7_rules>
1. LITERAL INSTRUCTION FOLLOWING
   Claude 4.7 does not silently generalize one instruction to adjacent items,
   especially at low/medium effort. When an instruction should apply broadly,
   state the scope explicitly (e.g., "Apply this formatting to every section,
   not just the first one"; "Return this field for every record in the list").

2. ADAPTIVE THINKING GUIDANCE
   Extended thinking with budget_tokens is removed in 4.7. The only thinking-on
   mode is adaptive thinking, and it is OFF by default. If the task genuinely
   benefits from reasoning, say so in the prompt:
   "This task involves multi-step reasoning. Think carefully through the
   problem before responding." If the task is a simple lookup, say so:
   "Thinking adds latency and should only be used when it will meaningfully
   improve answer quality. When in doubt, respond directly."

3. EFFORT-LEVEL AWARENESS
   Effort (low / medium / high / xhigh / max) is respected strictly. For coding
   and agentic workloads, recommend xhigh; for most intelligence-sensitive
   work, recommend at minimum high. If the prompt pins the task to a low
   effort level for latency reasons, add a targeted "think carefully" line
   rather than prompting around under-thinking.

4. SCRATCHPAD / MEMORY DIRECTIVES
   4.7 is meaningfully better at writing and using file-system memory. For
   long-horizon, multi-turn, or agentic prompts, add explicit scratchpad
   directives: "Maintain a scratchpad of intermediate findings, open
   questions, and decisions. Consult it before each new step and update it
   after each tool call."

5. CONDENSED CONTEXT (remove 4.5/4.6 scaffolding)
   Remove scaffolding that 4.7 no longer needs: forced interim status
   messages ("After every 3 tool calls, summarize progress"), redundant
   self-checks ("Double-check the slide layout before returning"), and
   verbose length-control boilerplate. 4.7 provides these natively.

6. TONE SPECIFICATION
   4.7 defaults to a direct, opinionated tone with less validation-forward
   phrasing and fewer emoji than 4.6. If the product voice should be warmer
   or more conversational, say so explicitly: "Use a warm, collaborative
   tone. Acknowledge the user's framing before answering." If the default is
   acceptable, no tone instruction is needed.

7. RESPONSE-LENGTH CALIBRATION
   4.7 calibrates length to perceived task complexity instead of defaulting
   to a fixed verbosity. Drop blanket "be thorough" / "be concise" boilerplate.
   If a specific length is required, use a POSITIVE example of the target
   concision rather than "don't over-explain" negatives. Example rewrite:
   "Provide concise, focused responses. Skip non-essential context and keep
   examples minimal."

8. VISION-AWARE INSTRUCTIONS
   4.7 supports high-resolution images (up to 2576px / 3.75MP) and returns
   pointing/bounding-box coordinates 1:1 with actual image pixels. Remove
   any scale-factor conversion language from image-aware prompts. If image
   fidelity is not needed, add: "Downsample images to 1080p before sending
   unless pixel-level detail is required." For computer-use workloads,
   1080p is a good default; 720p or 1366x768 for cost-sensitive flows.
</claude_4_7_rules>

<optimization_rules>
Also apply the general 4.x prompting best practices:
1. EXPLICITNESS: Convert vague instructions to specific, actionable ones.
2. CONTEXT: Add motivation/reasoning when it helps Claude understand intent.
3. POSITIVE FRAMING: Replace negative instructions with positive guidance.
4. TOOL USAGE: Be explicit about when and why tools should be used — 4.7
   uses tools LESS by default, so tool-heavy flows need explicit triggers.
5. FORMAT: Clear format specs; XML tags for complex prompts.
6. MODIFIERS: Add quality modifiers where beneficial; avoid filler.
7. WORD CHOICE: "consider" / "evaluate" / "reflect" read better than bare
   "think" in instructional copy.
8. TONE: Remove aggressive emphasis (ALL CAPS, excessive !!!) — 4.7 follows
   instructions well without it.
</optimization_rules>

<prompt_type_awareness>
- Q&A ASSISTANT: Add response format specs, citation guidance, unknown-info
  handling, scope boundaries. Convert role-only definitions into conditional
  handlers ("When the user asks about X, respond with Y"). At low effort,
  add a "think carefully" nudge for multi-step questions.
- CODING: Recommend xhigh effort. Add exploration directives ("Read and
  understand the relevant code before modifying"), explicit tool triggers
  ("Use grep to locate callers before refactoring"), and
  anti-hallucination instructions. Add scratchpad directives for long
  sessions.
- RESEARCH: Structured approach, hypothesis tracking, source evaluation
  criteria. Add explicit subagent guidance if fan-out is desired — 4.7
  spawns fewer subagents by default.
- CREATIVE: Specify tone and audience explicitly; 4.7's default is direct
  and less warm. If the product uses generic aesthetics, specify concrete
  alternatives (palette hex codes, typography) rather than negatives.
- LONG-HORIZON: Scratchpad / memory directives, incremental checkpoints,
  explicit milestone definitions, and guidance on when user-facing progress
  updates should be emitted.
</prompt_type_awareness>

<preserve_structure>
If the prompt contains XML blocks like <examples>, <example>, <instructions>,
<context>, <rules>, <format>, or <schema>:
- PRESERVE these blocks and their content
- ENHANCE the content within blocks rather than removing them
- Maintain the XML structure — it provides clear semantic organization
- Add complementary XML blocks if they improve clarity (e.g.,
  <response_format>, <constraints>, <scratchpad_policy>)
</preserve_structure>

<output_requirements>
- Return ONLY the optimized prompt text
- No explanations, no preamble, no markdown fences around the output
- Preserve the original intent and meaning
- Keep the prompt practical and focused; do not over-engineer
- Prefer positive examples over negative prohibitions
- STRUCTURE: Wrap the rewrite in semantic XML tags at the top level — at minimum <task>, and whichever of <requirements>, <response_format>, <constraints>, <examples>, <scratchpad_policy> apply. Do NOT wrap the entire output in a single outer <prompt> or <rewrite> tag; emit sibling top-level tags instead.
- LENGTH: The rewrite MUST NOT exceed ~3x the original word count. Prefer removing scaffolding over adding it. Omit sections that are not strictly useful for this particular prompt.
</output_requirements>"#;

/// Select the optimizer meta-prompt for a given family.
pub fn system_prompt_for_family(family: ModelFamily) -> &'static str {
    match family {
        ModelFamily::Claude45 | ModelFamily::Claude46 => OPTIMIZER_SYSTEM_PROMPT_4_5,
        ModelFamily::Claude47 => OPTIMIZER_SYSTEM_PROMPT_4_7,
    }
}

/// Build the user message for optimization, tagged with the target family.
pub fn build_optimization_message(
    original_prompt: &str,
    issues_json: &str,
    prompt_type: &str,
    family: ModelFamily,
) -> String {
    format!(
        r#"Optimize this prompt for {target}:

<prompt_type>{prompt_type}</prompt_type>

<original_prompt>
{original_prompt}
</original_prompt>

<detected_issues>
{issues_json}
</detected_issues>

Return the optimized prompt only."#,
        target = family.display_label()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_optimization_message_4_5() {
        let message = build_optimization_message(
            "Create a dashboard",
            r#"[{"id": "EXP001", "message": "Vague instruction"}]"#,
            "coding",
            ModelFamily::Claude45,
        );

        assert!(message.contains("Create a dashboard"));
        assert!(message.contains("EXP001"));
        assert!(message.contains("<prompt_type>coding</prompt_type>"));
        assert!(message.contains("Claude 4.5"));
    }

    #[test]
    fn test_build_optimization_message_4_7() {
        let message =
            build_optimization_message("Create a dashboard", "[]", "coding", ModelFamily::Claude47);
        assert!(message.contains("Claude 4.7"));
    }

    #[test]
    fn test_model_family_classifier_4_7() {
        assert_eq!(
            ModelFamily::from_model_id("global.anthropic.claude-opus-4-7-v1:0"),
            ModelFamily::Claude47
        );
        assert_eq!(
            ModelFamily::from_model_id("claude-opus-4-7"),
            ModelFamily::Claude47
        );
        assert_eq!(
            ModelFamily::from_model_id("opus-4.7"),
            ModelFamily::Claude47
        );
        assert_eq!(
            ModelFamily::from_model_id("sonnet-4.7"),
            ModelFamily::Claude47
        );
    }

    #[test]
    fn test_model_family_classifier_4_6() {
        assert_eq!(
            ModelFamily::from_model_id("us.anthropic.claude-opus-4-6-v1"),
            ModelFamily::Claude46
        );
        assert_eq!(
            ModelFamily::from_model_id("opus-4.6"),
            ModelFamily::Claude46
        );
    }

    #[test]
    fn test_model_family_classifier_default_is_4_5() {
        assert_eq!(
            ModelFamily::from_model_id("us.anthropic.claude-sonnet-4-5-20250929-v1:0"),
            ModelFamily::Claude45
        );
        assert_eq!(ModelFamily::from_model_id("sonnet"), ModelFamily::Claude45);
        // Unknown → falls back to 4.5 so existing callers keep working.
        assert_eq!(
            ModelFamily::from_model_id("some-unknown-model-id"),
            ModelFamily::Claude45
        );
    }

    #[test]
    fn test_system_prompt_for_family() {
        let p45 = system_prompt_for_family(ModelFamily::Claude45);
        let p46 = system_prompt_for_family(ModelFamily::Claude46);
        let p47 = system_prompt_for_family(ModelFamily::Claude47);

        assert!(p45.contains("Claude 4.5 models"));
        // 4.5 and 4.6 share the same meta-prompt for now.
        assert!(std::ptr::eq(p45, p46));
        assert!(p47.contains("Claude 4.7"));
        assert!(p47.contains("LITERAL INSTRUCTION FOLLOWING"));
        assert!(p47.contains("ADAPTIVE THINKING"));
        assert!(p47.contains("SCRATCHPAD"));
        assert!(p47.contains("VISION-AWARE"));
    }

    #[test]
    fn test_unreleased_model_error_sonnet_4_7() {
        let err = unreleased_model_error("sonnet-4.7").expect("should flag sonnet-4.7");
        let msg = format!("{err}");
        assert!(msg.contains("Sonnet 4.7"));
        assert!(msg.contains("not yet released"));
        assert!(msg.contains("opus-4.7"));
    }

    #[test]
    fn test_unreleased_model_error_haiku_4_7() {
        let err = unreleased_model_error("haiku-4.7").expect("should flag haiku-4.7");
        let msg = format!("{err}");
        assert!(msg.contains("Haiku 4.7"));
        assert!(msg.contains("not yet released"));
    }

    #[test]
    fn test_unreleased_model_error_opus_4_7_is_fine() {
        assert!(unreleased_model_error("opus-4.7").is_none());
        assert!(unreleased_model_error("global.anthropic.claude-opus-4-7-v1:0").is_none());
    }

    #[test]
    fn test_unreleased_model_error_existing_models_unaffected() {
        assert!(unreleased_model_error("sonnet").is_none());
        assert!(unreleased_model_error("opus-4.5").is_none());
        assert!(unreleased_model_error("opus-4.6").is_none());
        assert!(unreleased_model_error("haiku-4.5").is_none());
    }
}

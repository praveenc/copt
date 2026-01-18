//! LLM Client Module
//!
//! Provides unified interface for Claude 4.5 API access via:
//! - Anthropic API (direct)
//! - AWS Bedrock

#![allow(dead_code)]

mod anthropic;
mod bedrock;

pub use anthropic::AnthropicClient;
pub use bedrock::BedrockClient;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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
    fn provider_name(&self) -> &str;
}

/// A completion request (for future use with generic clients)
#[derive(Debug, Clone, Serialize)]
pub struct CompletionRequest {
    pub model: String,
    pub system: Option<String>,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    pub temperature: Option<f32>,
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// Message role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

/// A completion response
#[derive(Debug, Clone, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    pub model: String,
    pub stop_reason: Option<String>,
    pub usage: Option<Usage>,
}

/// Token usage statistics
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// The meta-prompt used to optimize prompts
pub const OPTIMIZER_SYSTEM_PROMPT: &str = r#"You are an expert prompt engineer specializing in optimizing prompts for Claude 4.5 models.

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
</output_requirements>"#;

/// Build the user message for optimization
pub fn build_optimization_message(
    original_prompt: &str,
    issues_json: &str,
    prompt_type: &str,
) -> String {
    format!(
        r#"Optimize this prompt for Claude 4.5:

<prompt_type>{prompt_type}</prompt_type>

<original_prompt>
{original_prompt}
</original_prompt>

<detected_issues>
{issues_json}
</detected_issues>

Return the optimized prompt only."#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_optimization_message() {
        let message = build_optimization_message(
            "Create a dashboard",
            r#"[{"id": "EXP001", "message": "Vague instruction"}]"#,
            "coding",
        );

        assert!(message.contains("Create a dashboard"));
        assert!(message.contains("EXP001"));
        assert!(message.contains("<prompt_type>coding</prompt_type>"));
    }
}

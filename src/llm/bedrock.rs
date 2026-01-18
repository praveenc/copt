//! AWS Bedrock client implementation
//!
//! Provides access to Claude models via AWS Bedrock using inference profile IDs.

use anyhow::{Context, Result};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_bedrockruntime::primitives::Blob;
use aws_sdk_bedrockruntime::Client as BedrockRuntimeClient;
use serde::{Deserialize, Serialize};

use super::LlmClient;

/// AWS Bedrock client
pub struct BedrockClient {
    client: BedrockRuntimeClient,
    region: String,
}

impl BedrockClient {
    /// Create a new Bedrock client for the specified region
    pub async fn new(region: &str) -> Result<Self> {
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_config::Region::new(region.to_string()))
            .load()
            .await;

        let client = BedrockRuntimeClient::new(&config);

        Ok(Self {
            client,
            region: region.to_string(),
        })
    }

    /// Check connectivity to AWS Bedrock
    ///
    /// This performs a lightweight check to verify:
    /// 1. AWS credentials are properly configured
    /// 2. The credentials have access to Bedrock in the specified region
    ///
    /// Returns Ok(()) if the connection is successful, or an error with
    /// a helpful message if something is wrong.
    pub async fn check_connectivity(&self, model_id: &str) -> Result<()> {
        // We'll make a minimal request to test connectivity
        // Using a tiny prompt to minimize cost/latency
        let test_request = BedrockRequest {
            anthropic_version: "bedrock-2023-05-31".to_string(),
            max_tokens: 1,
            temperature: None, // Use defaults for connectivity check
            top_p: None,
            system: None,
            messages: vec![BedrockMessage {
                role: "user".to_string(),
                content: "hi".to_string(),
            }],
        };

        let model_id = Self::get_bedrock_model_id(model_id);
        let body_bytes =
            serde_json::to_vec(&test_request).context("Failed to serialize test request")?;

        let result = self
            .client
            .invoke_model()
            .model_id(&model_id)
            .content_type("application/json")
            .accept("application/json")
            .body(Blob::new(body_bytes))
            .send()
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                let error_str = format!("{:?}", e);

                // Provide helpful error messages based on common failure modes
                if error_str.contains("credentials")
                    || error_str.contains("NoCredentialsError")
                    || error_str.contains("ExpiredToken")
                    || error_str.contains("InvalidIdentityToken")
                {
                    anyhow::bail!(
                        "AWS credentials not found or invalid.\n\n\
                        Please ensure you have valid AWS credentials configured:\n\
                        • Set AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY environment variables, or\n\
                        • Configure credentials in ~/.aws/credentials, or\n\
                        • Use AWS SSO: run 'aws sso login'\n\n\
                        Region: {}\n\
                        Error: {}",
                        self.region,
                        e
                    );
                } else if error_str.contains("AccessDenied")
                    || error_str.contains("UnauthorizedAccess")
                {
                    anyhow::bail!(
                        "Access denied to AWS Bedrock.\n\n\
                        Your AWS credentials are valid but don't have permission to access Bedrock.\n\
                        Please ensure:\n\
                        • Your IAM user/role has the 'bedrock:InvokeModel' permission\n\
                        • You have requested access to Claude models in the Bedrock console\n\n\
                        Region: {}\n\
                        Model: {}\n\
                        Error: {}",
                        self.region,
                        model_id,
                        e
                    );
                } else if error_str.contains("ResourceNotFoundException")
                    || error_str.contains("ValidationException")
                    || error_str.contains("model")
                {
                    anyhow::bail!(
                        "Model not available in AWS Bedrock.\n\n\
                        The specified model may not be available in your region or account.\n\
                        Please ensure:\n\
                        • You have enabled the model in AWS Bedrock console\n\
                        • The model is available in the '{}' region\n\
                        • You're using the correct model ID\n\n\
                        Model: {}\n\
                        Error: {}",
                        self.region,
                        model_id,
                        e
                    );
                } else if error_str.contains("timeout")
                    || error_str.contains("connect")
                    || error_str.contains("network")
                {
                    anyhow::bail!(
                        "Network error connecting to AWS Bedrock.\n\n\
                        Please check your internet connection and try again.\n\n\
                        Region: {}\n\
                        Error: {}",
                        self.region,
                        e
                    );
                } else if error_str.contains("ThrottlingException") {
                    // Throttling actually means we connected successfully!
                    // The credentials work, we just hit a rate limit
                    Ok(())
                } else {
                    anyhow::bail!(
                        "Failed to connect to AWS Bedrock.\n\n\
                        Region: {}\n\
                        Model: {}\n\
                        Error: {}",
                        self.region,
                        model_id,
                        e
                    );
                }
            }
        }
    }

    /// Get the configured region
    pub fn region(&self) -> &str {
        &self.region
    }

    /// Convert Anthropic model ID to Bedrock inference profile model ID
    fn get_bedrock_model_id(model: &str) -> String {
        match model {
            // Direct inference profile IDs - pass through
            m if m.starts_with("us.anthropic.") || m.starts_with("global.anthropic.") => {
                m.to_string()
            }

            // Map short names to inference profile IDs
            "claude-sonnet-4-5-20250929" | "claude-sonnet-4.5" | "sonnet-4.5" | "sonnet" => {
                "us.anthropic.claude-sonnet-4-5-20250929-v1:0".to_string()
            }
            "claude-haiku-4-5-20251001" | "claude-haiku-4.5" | "haiku-4.5" | "haiku" => {
                "us.anthropic.claude-haiku-4-5-20251001-v1:0".to_string()
            }
            "claude-opus-4-5-20251101" | "claude-opus-4.5" | "opus-4.5" | "opus" => {
                "global.anthropic.claude-opus-4-5-20251101-v1:0".to_string()
            }

            // Legacy model ID format - convert to inference profile
            "anthropic.claude-sonnet-4-5-20250929-v1:0" => {
                "us.anthropic.claude-sonnet-4-5-20250929-v1:0".to_string()
            }
            "anthropic.claude-haiku-4-5-20251001-v1:0" => {
                "us.anthropic.claude-haiku-4-5-20251001-v1:0".to_string()
            }
            "anthropic.claude-opus-4-5-20251101-v1:0" => {
                "global.anthropic.claude-opus-4-5-20251101-v1:0".to_string()
            }

            // Default: assume it's already a valid model ID
            other => other.to_string(),
        }
    }
}

#[async_trait]
impl LlmClient for BedrockClient {
    async fn complete(
        &self,
        system: &str,
        user_message: &str,
        model: &str,
        max_tokens: u32,
    ) -> Result<String> {
        let model_id = Self::get_bedrock_model_id(model);

        // Build the request body in Anthropic's Messages API format
        // (which Bedrock uses for Claude models)
        let request_body = BedrockRequest {
            anthropic_version: "bedrock-2023-05-31".to_string(),
            max_tokens,
            temperature: Some(0.3),
            top_p: Some(0.95),
            system: Some(system.to_string()),
            messages: vec![BedrockMessage {
                role: "user".to_string(),
                content: user_message.to_string(),
            }],
        };

        let body_bytes =
            serde_json::to_vec(&request_body).context("Failed to serialize request body")?;

        let response = self
            .client
            .invoke_model()
            .model_id(&model_id)
            .content_type("application/json")
            .accept("application/json")
            .body(Blob::new(body_bytes))
            .send()
            .await
            .context("Failed to invoke Bedrock model")?;

        let response_bytes = response.body.as_ref();
        let api_response: BedrockResponse =
            serde_json::from_slice(response_bytes).context("Failed to parse Bedrock response")?;

        // Extract text from the first content block
        let text = api_response
            .content
            .into_iter()
            .filter_map(|block| {
                if block.content_type == "text" {
                    Some(block.text)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");

        Ok(text)
    }

    fn provider_name(&self) -> &str {
        "bedrock"
    }
}

/// Request body for Bedrock (Anthropic Claude format)
#[derive(Debug, Serialize)]
struct BedrockRequest {
    anthropic_version: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<BedrockMessage>,
}

/// A message in the Bedrock request format
#[derive(Debug, Serialize)]
struct BedrockMessage {
    role: String,
    content: String,
}

/// Response from Bedrock (Anthropic Claude format)
#[derive(Debug, Deserialize)]
struct BedrockResponse {
    content: Vec<ContentBlock>,
    #[allow(dead_code)]
    stop_reason: Option<String>,
    #[allow(dead_code)]
    usage: Option<BedrockUsage>,
}

/// A content block in the response
#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(default)]
    text: String,
}

/// Usage statistics from Bedrock
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BedrockUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_id_conversion_inference_profiles() {
        // Direct inference profile IDs should pass through
        assert_eq!(
            BedrockClient::get_bedrock_model_id("us.anthropic.claude-sonnet-4-5-20250929-v1:0"),
            "us.anthropic.claude-sonnet-4-5-20250929-v1:0"
        );
        assert_eq!(
            BedrockClient::get_bedrock_model_id("global.anthropic.claude-opus-4-5-20251101-v1:0"),
            "global.anthropic.claude-opus-4-5-20251101-v1:0"
        );
    }

    #[test]
    fn test_model_id_conversion_short_names() {
        assert_eq!(
            BedrockClient::get_bedrock_model_id("claude-sonnet-4-5-20250929"),
            "us.anthropic.claude-sonnet-4-5-20250929-v1:0"
        );
        assert_eq!(
            BedrockClient::get_bedrock_model_id("sonnet-4.5"),
            "us.anthropic.claude-sonnet-4-5-20250929-v1:0"
        );
        assert_eq!(
            BedrockClient::get_bedrock_model_id("sonnet"),
            "us.anthropic.claude-sonnet-4-5-20250929-v1:0"
        );
        assert_eq!(
            BedrockClient::get_bedrock_model_id("opus-4.5"),
            "global.anthropic.claude-opus-4-5-20251101-v1:0"
        );
        assert_eq!(
            BedrockClient::get_bedrock_model_id("haiku-4.5"),
            "us.anthropic.claude-haiku-4-5-20251001-v1:0"
        );
    }

    #[test]
    fn test_model_id_conversion_legacy() {
        assert_eq!(
            BedrockClient::get_bedrock_model_id("anthropic.claude-sonnet-4-5-20250929-v1:0"),
            "us.anthropic.claude-sonnet-4-5-20250929-v1:0"
        );
    }
}

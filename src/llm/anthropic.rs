//! Anthropic API client implementation
//!
//! Provides direct access to the Anthropic Claude API.

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use super::LlmClient;

/// Anthropic API base URL
const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";

/// Current API version
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic API client
pub struct AnthropicClient {
    client: reqwest::Client,
    api_key: String,
}

impl AnthropicClient {
    /// Create a new Anthropic client with the given API key
    pub fn new(api_key: String) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, api_key })
    }
}

#[async_trait]
impl LlmClient for AnthropicClient {
    async fn complete(
        &self,
        system: &str,
        user_message: &str,
        model: &str,
        max_tokens: u32,
    ) -> Result<String> {
        let request = AnthropicRequest {
            model: model.to_string(),
            max_tokens,
            system: Some(system.to_string()),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: user_message.to_string(),
            }],
        };

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&self.api_key).context("Invalid API key format")?,
        );
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static(ANTHROPIC_VERSION),
        );

        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Anthropic API request failed with status {}: {}",
                status,
                error_text
            );
        }

        let api_response: AnthropicResponse = response
            .json()
            .await
            .context("Failed to parse Anthropic API response")?;

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
        "anthropic"
    }
}

/// Request body for Anthropic Messages API
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<AnthropicMessage>,
}

/// A message in the Anthropic format
#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Response from Anthropic Messages API
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    #[allow(dead_code)]
    model: String,
    #[allow(dead_code)]
    stop_reason: Option<String>,
    #[allow(dead_code)]
    usage: Option<AnthropicUsage>,
}

/// A content block in the response
#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(default)]
    text: String,
}

/// Usage statistics
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = AnthropicClient::new("test-api-key".to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_provider_name() {
        let client = AnthropicClient::new("test-api-key".to_string()).unwrap();
        assert_eq!(client.provider_name(), "anthropic");
    }
}
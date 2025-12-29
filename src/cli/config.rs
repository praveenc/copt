//! Configuration module for copt (Claude Optimizer)
//!
//! Handles loading configuration from files and environment variables.

#![allow(dead_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Default settings
    pub default: DefaultConfig,
    /// Anthropic API settings
    pub anthropic: AnthropicConfig,
    /// AWS Bedrock settings
    pub bedrock: BedrockConfig,
    /// Output settings
    pub output: OutputConfig,
    /// Rules settings
    pub rules: RulesConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default: DefaultConfig::default(),
            anthropic: AnthropicConfig::default(),
            bedrock: BedrockConfig::default(),
            output: OutputConfig::default(),
            rules: RulesConfig::default(),
        }
    }
}

/// Default configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DefaultConfig {
    /// Default provider (anthropic or bedrock)
    pub provider: String,
    /// Default model to use
    pub model: String,
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            provider: "bedrock".to_string(),
            model: "us.anthropic.claude-sonnet-4-5-20250929-v1:0".to_string(),
        }
    }
}

/// Anthropic API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AnthropicConfig {
    /// Environment variable name for API key
    pub api_key_env: String,
    /// Maximum tokens for requests
    pub max_tokens: u32,
    /// API base URL (for custom endpoints)
    pub base_url: Option<String>,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_key_env: "ANTHROPIC_API_KEY".to_string(),
            max_tokens: 4096,
            base_url: None,
        }
    }
}

/// AWS Bedrock configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BedrockConfig {
    /// AWS region
    pub region: String,
    /// AWS profile name
    pub profile: Option<String>,
    /// Maximum tokens for requests
    pub max_tokens: u32,
}

impl Default for BedrockConfig {
    fn default() -> Self {
        Self {
            region: "us-west-2".to_string(),
            profile: None,
            max_tokens: 4096,
        }
    }
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputConfig {
    /// Enable colored output
    pub color: bool,
    /// Output format (pretty, json, quiet)
    pub format: String,
    /// Show diff by default
    pub show_diff: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            color: true,
            format: "pretty".to_string(),
            show_diff: false,
        }
    }
}

/// Rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RulesConfig {
    /// Enabled rule categories ("all" or specific categories)
    pub enabled_categories: Vec<String>,
    /// Disabled specific rules
    pub disabled: Vec<String>,
    /// Disabled categories
    pub disabled_categories: Vec<String>,
    /// Severity overrides (rule_id -> severity)
    #[serde(default)]
    pub severity_overrides: std::collections::HashMap<String, String>,
}

impl Default for RulesConfig {
    fn default() -> Self {
        Self {
            enabled_categories: vec!["all".to_string()],
            disabled: Vec::new(),
            disabled_categories: Vec::new(),
            severity_overrides: std::collections::HashMap::new(),
        }
    }
}

/// Provider configuration enum for runtime use
#[derive(Debug, Clone)]
pub enum ProviderConfig {
    Anthropic {
        api_key: String,
        base_url: Option<String>,
        max_tokens: u32,
    },
    Bedrock {
        region: String,
        profile: Option<String>,
        max_tokens: u32,
    },
}

/// Load configuration from the default config file
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path();

    if config_path.exists() {
        load_config_from_path(&config_path)
    } else {
        Ok(Config::default())
    }
}

/// Load configuration from a specific path
pub fn load_config_from_path(path: &PathBuf) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

    Ok(config)
}

/// Get the default configuration file path
pub fn get_config_path() -> PathBuf {
    // Check XDG_CONFIG_HOME first, then fall back to ~/.config
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg_config).join("copt").join("config.toml")
    } else if let Some(home) = dirs_home() {
        // Try ~/.copt.toml first (legacy location)
        let legacy_path = home.join(".copt.toml");
        if legacy_path.exists() {
            return legacy_path;
        }
        // Otherwise use ~/.config/copt/config.toml
        home.join(".config").join("copt").join("config.toml")
    } else {
        PathBuf::from(".copt.toml")
    }
}

/// Get home directory
fn dirs_home() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| std::env::var("USERPROFILE").ok().map(PathBuf::from))
}

/// Create a default configuration file
pub fn create_default_config() -> Result<PathBuf> {
    let config_path = get_config_path();

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
    }

    let default_config = Config::default();
    let content =
        toml::to_string_pretty(&default_config).context("Failed to serialize default config")?;

    std::fs::write(&config_path, content)
        .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

    Ok(config_path)
}

/// Validate configuration
impl Config {
    pub fn validate(&self) -> Result<()> {
        // Validate provider
        let valid_providers = ["anthropic", "bedrock"];
        if !valid_providers.contains(&self.default.provider.as_str()) {
            anyhow::bail!(
                "Invalid provider '{}'. Valid options: {:?}",
                self.default.provider,
                valid_providers
            );
        }

        // Validate output format
        let valid_formats = ["pretty", "json", "quiet"];
        if !valid_formats.contains(&self.output.format.as_str()) {
            anyhow::bail!(
                "Invalid output format '{}'. Valid options: {:?}",
                self.output.format,
                valid_formats
            );
        }

        Ok(())
    }

    /// Get the effective API key for Anthropic
    pub fn get_anthropic_api_key(&self) -> Result<String> {
        std::env::var(&self.anthropic.api_key_env).with_context(|| {
            format!(
                "API key not found. Set the {} environment variable.",
                self.anthropic.api_key_env
            )
        })
    }

    /// Check if a rule is enabled
    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        // Check if explicitly disabled
        if self.rules.disabled.contains(&rule_id.to_string()) {
            return false;
        }

        // Check if category is disabled
        let category_prefix = &rule_id[..3]; // e.g., "EXP" from "EXP001"
        let category = category_from_prefix(category_prefix);
        if let Some(cat) = category {
            if self.rules.disabled_categories.contains(&cat.to_string()) {
                return false;
            }
        }

        // Check if enabled
        if self.rules.enabled_categories.contains(&"all".to_string()) {
            return true;
        }

        if let Some(cat) = category {
            self.rules.enabled_categories.contains(&cat.to_string())
        } else {
            true
        }
    }

    /// Get severity override for a rule
    pub fn get_severity_override(&self, rule_id: &str) -> Option<&String> {
        self.rules.severity_overrides.get(rule_id)
    }
}

/// Map rule prefix to category name
fn category_from_prefix(prefix: &str) -> Option<&'static str> {
    match prefix.to_uppercase().as_str() {
        "EXP" => Some("explicitness"),
        "STY" => Some("style"),
        "TUL" => Some("tools"),
        "FMT" => Some("formatting"),
        "VRB" => Some("verbosity"),
        "AGT" => Some("agentic"),
        "LHT" => Some("long_horizon"),
        "FED" => Some("frontend"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.default.provider, "bedrock");
        assert_eq!(
            config.default.model,
            "us.anthropic.claude-sonnet-4-5-20250929-v1:0"
        );
    }

    #[test]
    fn test_config_validation() {
        let config = Config::default();
        assert!(config.validate().is_ok());

        let mut invalid = Config::default();
        invalid.default.provider = "invalid".to_string();
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_rule_enabled() {
        let config = Config::default();
        assert!(config.is_rule_enabled("EXP001"));

        let mut config = Config::default();
        config.rules.disabled.push("EXP001".to_string());
        assert!(!config.is_rule_enabled("EXP001"));

        let mut config = Config::default();
        config
            .rules
            .disabled_categories
            .push("explicitness".to_string());
        assert!(!config.is_rule_enabled("EXP001"));
    }

    #[test]
    fn test_category_from_prefix() {
        assert_eq!(category_from_prefix("EXP"), Some("explicitness"));
        assert_eq!(category_from_prefix("STY"), Some("style"));
        assert_eq!(category_from_prefix("XXX"), None);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.default.provider, config.default.provider);
    }
}

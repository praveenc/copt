//! copt - Claude Optimizer
//!
//! A beautiful CLI tool to optimize prompts for Claude 4.5 family of models.
//! Analyzes prompts and rewrites them according to Anthropic's official best practices.

use anyhow::{Context, Result};
use chrono::Local;
use clap::{Parser, ValueEnum};
use colored::Colorize;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;

mod analyzer;
mod cli;
mod llm;
mod optimizer;
mod rules;
mod tui;
mod utils;

// Re-export types from analyzer for use throughout the crate
pub use analyzer::{Issue, Severity};

/// Claude Optimizer - A beautiful CLI tool to optimize prompts for Claude 4.5 models
#[derive(Parser, Debug)]
#[command(
    name = "copt",
    version,
    about = "⚡ Optimize prompts for Claude 4.5 models",
    after_help = "Examples:\n  copt \"Your prompt here\"\n  copt -f prompt.txt\n  copt -f prompt.txt --offline\n  cat prompt.txt | copt"
)]
struct Cli {
    /// Prompt text to optimize
    #[arg(value_name = "PROMPT")]
    prompt: Option<String>,

    /// Read prompt from file
    #[arg(short, long, value_name = "FILE")]
    file: Option<PathBuf>,

    /// Save optimized prompt to file
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Output directory for auto-save
    #[arg(long, value_name = "DIR", default_value = "copt-output", hide_default_value = true)]
    output_dir: PathBuf,

    /// Disable auto-save
    #[arg(long)]
    no_save: bool,

    /// Provider: anthropic, bedrock
    #[arg(short, long, value_enum, default_value = "bedrock", hide_default_value = true)]
    provider: Provider,

    /// Model ID or alias
    #[arg(short, long, hide_default_value = true, default_value = "us.anthropic.claude-sonnet-4-5-20250929-v1:0")]
    model: String,

    /// AWS region for Bedrock
    #[arg(long, default_value = "us-west-2", hide_default_value = true)]
    region: String,

    /// Output format: pretty, json, quiet
    #[arg(long, value_enum, default_value = "pretty", hide_default_value = true)]
    format: OutputFormat,

    /// Show before/after diff
    #[arg(long)]
    diff: bool,

    /// Display optimized prompt
    #[arg(long)]
    show_prompt: bool,

    /// Quiet mode (prompt only)
    #[arg(short, long)]
    quiet: bool,

    /// Analyze only, no optimization
    #[arg(long)]
    analyze: bool,

    /// Offline mode (no API calls)
    #[arg(long)]
    offline: bool,

    /// Check specific categories
    #[arg(long, value_delimiter = ',', value_name = "CAT")]
    check: Option<Vec<String>>,

    /// Interactive multi-line input
    #[arg(short, long)]
    interactive: bool,

    /// Skip connectivity check
    #[arg(long)]
    skip_connectivity_check: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Provider {
    Anthropic,
    Bedrock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    Pretty,
    Json,
    Quiet,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    if std::env::var("RUST_LOG").is_ok() {
        tracing_subscriber::fmt::init();
    }

    // Parse CLI arguments
    let cli = Cli::parse();

    // Check provider connectivity on first use (unless offline or skipped)
    if !cli.offline && !cli.skip_connectivity_check {
        check_provider_connectivity(&cli).await?;
    }

    // Get the input prompt
    let prompt = get_input_prompt(&cli).await?;

    if prompt.trim().is_empty() {
        eprintln!(
            "{} No prompt provided. Use --help for usage information.",
            "Error:".red().bold()
        );
        std::process::exit(1);
    }

    // Run the optimization
    let result = run_optimization(&cli, &prompt).await?;

    // Handle output
    handle_output(&cli, &result).await?;

    Ok(())
}

/// Check connectivity to the configured provider
async fn check_provider_connectivity(cli: &Cli) -> Result<()> {
    match cli.provider {
        Provider::Bedrock => {
            if !cli.quiet && cli.format != OutputFormat::Quiet {
                print!(
                    "{} Checking AWS Bedrock connectivity ({})... ",
                    "⚡".cyan(),
                    cli.region.bright_black()
                );
                // Flush to show the message immediately
                use std::io::Write;
                let _ = std::io::stdout().flush();
            }

            let client = llm::BedrockClient::new(&cli.region).await?;

            match client.check_connectivity(&cli.model).await {
                Ok(()) => {
                    if !cli.quiet && cli.format != OutputFormat::Quiet {
                        println!("{}", "✓ Connected".green());
                        println!();
                    }
                    Ok(())
                }
                Err(e) => {
                    if !cli.quiet && cli.format != OutputFormat::Quiet {
                        println!("{}", "✗ Failed".red());
                        println!();
                    }
                    Err(e)
                }
            }
        }
        Provider::Anthropic => {
            // Check if API key is set
            if std::env::var("ANTHROPIC_API_KEY").is_err() {
                anyhow::bail!(
                    "ANTHROPIC_API_KEY environment variable not set.\n\n\
                    Please set your Anthropic API key:\n\
                    export ANTHROPIC_API_KEY=\"your-api-key-here\"\n\n\
                    Or switch to AWS Bedrock provider:\n\
                    copt --provider bedrock \"your prompt\""
                );
            }

            if !cli.quiet && cli.format != OutputFormat::Quiet {
                println!(
                    "{} Using Anthropic API (API key configured)",
                    "✓".green()
                );
                println!();
            }
            Ok(())
        }
    }
}

/// Get the input prompt from various sources
async fn get_input_prompt(cli: &Cli) -> Result<String> {
    // Priority: direct argument > file > stdin > interactive

    if let Some(ref prompt) = cli.prompt {
        return Ok(prompt.clone());
    }

    if let Some(ref file_path) = cli.file {
        let content = tokio::fs::read_to_string(file_path)
            .await
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
        return Ok(content);
    }

    // Check if stdin has data (not a terminal)
    if !io::stdin().is_terminal() {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;
        return Ok(buffer);
    }

    if cli.interactive {
        return interactive_input().await;
    }

    // No input provided
    Ok(String::new())
}

/// Interactive multi-line input mode
async fn interactive_input() -> Result<String> {
    use dialoguer::Editor;

    println!(
        "\n{} Opening editor for multi-line input...\n",
        "📝".to_string()
    );

    let prompt = Editor::new()
        .edit("# Enter your prompt below (save and close to continue)\n\n")?
        .unwrap_or_default();

    // Remove the comment line if present
    let prompt = prompt
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    Ok(prompt)
}

/// Main optimization result structure
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub original: String,
    pub optimized: String,
    pub issues: Vec<Issue>,
    pub stats: OptimizationStats,
}

/// Statistics about the optimization
#[derive(Debug, Clone, Default)]
pub struct OptimizationStats {
    pub original_chars: usize,
    pub optimized_chars: usize,
    pub original_tokens: usize,
    pub optimized_tokens: usize,
    pub rules_applied: usize,
    pub categories_improved: usize,
    pub processing_time_ms: u64,
    pub provider: String,
    pub model: String,
}

/// Run the optimization process
async fn run_optimization(cli: &Cli, prompt: &str) -> Result<OptimizationResult> {
    let start_time = std::time::Instant::now();

    // Show header unless quiet mode
    if !cli.quiet && cli.format != OutputFormat::Quiet {
        tui::print_header();
    }

    // Show offline mode banner if applicable
    if cli.offline && !cli.quiet && cli.format != OutputFormat::Quiet {
        tui::renderer::print_offline_banner();
    }

    // Show input info
    if !cli.quiet && cli.format != OutputFormat::Quiet {
        tui::print_input_info(prompt, &cli.file);
    }

    // Analyze the prompt
    let issues = analyzer::analyze(prompt, cli.check.as_deref())?;

    // Show analysis results
    if !cli.quiet && cli.format != OutputFormat::Quiet {
        tui::print_analysis(&issues);
    }

    // If analyze-only or no issues, return early
    if cli.analyze || (issues.is_empty() && !cli.offline) {
        if issues.is_empty() && !cli.quiet {
            println!(
                "\n{} Your prompt looks great! No optimization needed.\n",
                "✓".green().bold()
            );
        }

        let stats = OptimizationStats {
            original_chars: prompt.len(),
            optimized_chars: prompt.len(),
            original_tokens: utils::count_tokens(prompt),
            optimized_tokens: utils::count_tokens(prompt),
            processing_time_ms: start_time.elapsed().as_millis() as u64,
            provider: format!("{:?}", cli.provider).to_lowercase(),
            model: cli.model.clone(),
            ..Default::default()
        };

        return Ok(OptimizationResult {
            original: prompt.to_string(),
            optimized: prompt.to_string(),
            issues,
            stats,
        });
    }

    // Perform optimization
    let optimized = if cli.offline {
        // Static rules only (no spinner needed - just analysis)
        optimizer::optimize_static(prompt, &issues)?
    } else {
        // Start optimization spinner for LLM mode
        let spinner = if !cli.quiet && cli.format != OutputFormat::Quiet {
            Some(tui::renderer::start_optimizing_spinner(&cli.model))
        } else {
            None
        };

        // LLM-powered optimization
        let client: Box<dyn llm::LlmClient> = match cli.provider {
            Provider::Anthropic => Box::new(llm::AnthropicClient::new(
                std::env::var("ANTHROPIC_API_KEY")
                    .context("ANTHROPIC_API_KEY environment variable not set")?,
            )?),
            Provider::Bedrock => Box::new(llm::BedrockClient::new(&cli.region).await?),
        };

        let result = optimizer::optimize_with_llm(prompt, &issues, client.as_ref(), &cli.model).await?;
        if let Some(s) = spinner {
            tui::renderer::stop_optimizing_spinner(s);
        }
        result
    };

    let processing_time = start_time.elapsed().as_millis() as u64;

    // Calculate stats
    let stats = OptimizationStats {
        original_chars: prompt.len(),
        optimized_chars: optimized.len(),
        original_tokens: utils::count_tokens(prompt),
        optimized_tokens: utils::count_tokens(&optimized),
        rules_applied: issues.len(),
        categories_improved: issues
            .iter()
            .map(|i| i.category.as_str())
            .collect::<std::collections::HashSet<_>>()
            .len(),
        processing_time_ms: processing_time,
        provider: format!("{:?}", cli.provider).to_lowercase(),
        model: cli.model.clone(),
    };

    Ok(OptimizationResult {
        original: prompt.to_string(),
        optimized,
        issues,
        stats,
    })
}

/// Handle output based on CLI options
async fn handle_output(cli: &Cli, result: &OptimizationResult) -> Result<()> {
    match cli.format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "original": result.original,
                "optimized": result.optimized,
                "issues": result.issues.iter().map(|i| serde_json::json!({
                    "id": i.id,
                    "category": i.category,
                    "severity": format!("{:?}", i.severity).to_lowercase(),
                    "message": i.message,
                    "line": i.line,
                    "suggestion": i.suggestion,
                })).collect::<Vec<_>>(),
                "stats": {
                    "original_chars": result.stats.original_chars,
                    "optimized_chars": result.stats.optimized_chars,
                    "original_tokens": result.stats.original_tokens,
                    "optimized_tokens": result.stats.optimized_tokens,
                    "rules_applied": result.stats.rules_applied,
                    "categories_improved": result.stats.categories_improved,
                    "processing_time_ms": result.stats.processing_time_ms,
                    "provider": result.stats.provider,
                    "model": result.stats.model,
                }
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        OutputFormat::Quiet => {
            println!("{}", result.optimized);
        }
        OutputFormat::Pretty => {
            if cli.diff {
                tui::print_diff(&result.original, &result.optimized);
            }

            // In offline mode, skip stats (nothing was optimized) and show helpful message
            if cli.offline {
                println!();
                println!("  {}", "─".repeat(70).bright_black());
                println!(
                    "  {}  {}",
                    "💡".cyan(),
                    "To optimize this prompt with an LLM, run without --offline".white()
                );
                println!("  {}", "─".repeat(70).bright_black());
                println!();
            } else {
                tui::print_stats(&result.stats);

                if !cli.diff && !cli.quiet && cli.show_prompt {
                    tui::renderer::print_optimized_prompt(&result.optimized);
                }
            }
        }
    }

    // Determine the output path
    // In offline mode, don't auto-save unless user explicitly specifies -o
    let output_path = if let Some(ref explicit_output) = cli.output {
        // User specified explicit output path (always respect this)
        Some(explicit_output.clone())
    } else if !cli.no_save && !cli.offline && cli.format != OutputFormat::Json {
        // Auto-save to output directory (only when not in offline mode)
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("optimized_{}.txt", timestamp);
        Some(cli.output_dir.join(filename))
    } else {
        None
    };

    // Save the optimized prompt
    if let Some(ref path) = output_path {
        // Create output directory if it doesn't exist
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
        }

        // Write the optimized prompt
        tokio::fs::write(path, &result.optimized)
            .await
            .with_context(|| format!("Failed to write to: {}", path.display()))?;

        // Also write metadata JSON alongside
        let metadata_path = path.with_extension("json");
        let metadata = serde_json::json!({
            "timestamp": Local::now().to_rfc3339(),
            "original_length": result.stats.original_chars,
            "optimized_length": result.stats.optimized_chars,
            "original_tokens": result.stats.original_tokens,
            "optimized_tokens": result.stats.optimized_tokens,
            "rules_applied": result.stats.rules_applied,
            "categories_improved": result.stats.categories_improved,
            "processing_time_ms": result.stats.processing_time_ms,
            "provider": result.stats.provider,
            "model": result.stats.model,
            "issues": result.issues.iter().map(|i| serde_json::json!({
                "id": i.id,
                "category": i.category,
                "severity": format!("{:?}", i.severity).to_lowercase(),
                "message": i.message,
            })).collect::<Vec<_>>(),
        });
        
        tokio::fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)
            .await
            .with_context(|| format!("Failed to write metadata: {}", metadata_path.display()))?;

        if !cli.quiet && cli.format != OutputFormat::Quiet {
            tui::stats::print_save_success(&path.display().to_string(), false);
        }
    }

    Ok(())
}
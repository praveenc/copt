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
    #[arg(
        long,
        value_name = "DIR",
        default_value = "copt-output",
        hide_default_value = true
    )]
    output_dir: PathBuf,

    /// Disable auto-save
    #[arg(long)]
    no_save: bool,

    /// Provider: anthropic, bedrock
    #[arg(
        short,
        long,
        value_enum,
        default_value = "bedrock",
        hide_default_value = true
    )]
    provider: Provider,

    /// Model ID or alias
    #[arg(
        short,
        long,
        hide_default_value = true,
        default_value = "us.anthropic.claude-sonnet-4-5-20250929-v1:0"
    )]
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

    /// Interactively suggest improvements for vague prompts (default when TTY)
    #[arg(long, hide = true)]
    suggest: bool,

    /// Disable auto-suggestions for vague prompts
    #[arg(long)]
    no_suggest: bool,

    /// Launch full-screen interactive TUI mode
    #[arg(short, long)]
    interactive: bool,

    /// Open editor for multi-line input
    #[arg(short = 'e', long)]
    editor: bool,

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

    // Interactive mode requires TTY
    if cli.interactive && !io::stdout().is_terminal() {
        eprintln!(
            "{} Interactive mode requires a terminal. Use without -i for piped output.",
            "Error:".red().bold()
        );
        std::process::exit(1);
    }

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

    // Run in interactive TUI mode or standard mode
    if cli.interactive {
        run_interactive_mode(&cli, &prompt).await?;
    } else {
        // Standard mode
        let result = run_optimization(&cli, &prompt).await?;
        handle_output(&cli, &result).await?;
    }

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
                println!("{} Using Anthropic API (API key configured)", "✓".green());
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

    if cli.editor {
        return editor_input().await;
    }

    // No input provided
    Ok(String::new())
}

/// Editor-based multi-line input mode
async fn editor_input() -> Result<String> {
    println!("\n📝 Opening editor for multi-line input...\n");

    // Create a temporary file with initial content
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("copt_prompt_{}.txt", std::process::id()));

    let initial_content = "# Enter your prompt below (save and close to continue)\n\n";
    std::fs::write(&temp_path, initial_content)
        .with_context(|| format!("Failed to create temp file: {}", temp_path.display()))?;

    // Get the editor from environment or use sensible defaults
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            // Try to find a common editor
            if cfg!(target_os = "macos") {
                "nano".to_string()
            } else if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });

    // Build editor command with appropriate wait flags for GUI editors
    // GUI editors fork and return immediately unless told to wait
    let (editor_cmd, editor_args) = build_editor_command(&editor, &temp_path);

    // Clone for the blocking task
    let editor_cmd_clone = editor_cmd.clone();
    let editor_args_clone = editor_args.clone();

    // Use spawn_blocking to properly wait for the editor process
    let status = tokio::task::spawn_blocking(move || {
        std::process::Command::new(&editor_cmd_clone)
            .args(&editor_args_clone)
            .status()
    })
    .await
    .context("Failed to spawn editor task")?
    .with_context(|| format!("Failed to execute editor: {}", editor_cmd))?;

    if !status.success() {
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);
        anyhow::bail!("Editor exited with non-zero status: {:?}", status.code());
    }

    // Read the edited content
    let content = std::fs::read_to_string(&temp_path)
        .with_context(|| format!("Failed to read temp file: {}", temp_path.display()))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    // Remove comment lines and trim
    let prompt = content
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    Ok(prompt)
}

/// Build editor command with appropriate wait flags for GUI editors
///
/// GUI editors fork and return immediately unless given a --wait flag.
/// We only add flags for editors we've verified support them.
fn build_editor_command(editor: &str, file_path: &std::path::Path) -> (String, Vec<String>) {
    let editor_lower = editor.to_lowercase();
    let file_arg = file_path.to_string_lossy().to_string();

    // Extract just the binary name for matching (handle full paths)
    let editor_name = std::path::Path::new(editor)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(editor)
        .to_lowercase();

    // VSCode: `code --wait` (verified)
    if editor_name.contains("code") || editor_lower.contains("visual studio code") {
        return (editor.to_string(), vec!["--wait".to_string(), file_arg]);
    }

    // Zed: `zed --wait` or `/path/to/Zed.app/.../cli --wait` (verified)
    if editor_name == "cli" && editor_lower.contains("zed") {
        return (editor.to_string(), vec!["--wait".to_string(), file_arg]);
    }
    if editor_name.contains("zed") {
        return (editor.to_string(), vec!["--wait".to_string(), file_arg]);
    }

    // Default: terminal editors (vim, nano, emacs, etc.) block by default
    (editor.to_string(), vec![file_arg])
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
    use tui::model::{AppPhase, Model};

    let start_time = std::time::Instant::now();
    let use_new_renderer = !cli.quiet && cli.format == OutputFormat::Pretty;

    // Build model for new renderer
    let mut model = if use_new_renderer {
        let mut m = Model::new();
        m.offline_mode = cli.offline;
        m.original_prompt = prompt.to_string();
        m.input_file = cli.file.as_ref().map(|p| p.display().to_string());
        m.phase = AppPhase::Analyzing;
        Some(m)
    } else {
        None
    };

    // Analyze the prompt
    let issues = analyzer::analyze(prompt, cli.check.as_deref())?;

    // Classify prompt type for context-aware LLM optimization
    let prompt_type = analyzer::classify_prompt(prompt);

    // Update model with issues
    if let Some(ref mut m) = model {
        m.set_issues(&issues);
    }

    // Auto-suggest improvements for vague prompts (EXP005/EXP006)
    // Triggers automatically when: TTY + vague prompt + not --no-suggest
    let is_tty = io::stdout().is_terminal();
    let should_auto_suggest =
        (cli.suggest || is_tty) && !cli.no_suggest && cli::suggest::should_suggest(&issues);

    let prompt = if should_auto_suggest {
        // Render header/analysis first so user sees context
        if let Some(ref mut m) = model {
            m.phase = AppPhase::AnalysisDone;
            tui::linear::render(m)?;
        }

        // Run interactive suggestion flow
        match cli::suggest::run_interactive_suggestions(prompt, &issues) {
            Ok(Some(enhanced)) => {
                println!();
                enhanced
            }
            Ok(None) => prompt.to_string(),
            Err(e) => {
                eprintln!("  {} Suggestion prompt failed: {}", "⚠".yellow(), e);
                prompt.to_string()
            }
        }
    } else {
        prompt.to_string()
    };
    let prompt = prompt.as_str();

    // If analyze-only mode, return early without optimization
    // In LLM mode, we still optimize even if no static rules triggered
    // (the LLM can enhance prompts beyond what static rules detect)
    if cli.analyze {
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

        // Update model phase and render
        if let Some(ref mut m) = model {
            m.phase = AppPhase::AnalysisDone;
            tui::linear::render(m)?;
        }

        return Ok(OptimizationResult {
            original: prompt.to_string(),
            optimized: prompt.to_string(),
            issues,
            stats,
        });
    }

    // Show header and analysis before optimization starts
    if let Some(ref mut m) = model {
        m.phase = AppPhase::Optimizing;
        // Render header, input info, and analysis
        tui::linear::render(m)?;
    }

    // Show suggestion hint in offline mode if vague prompt detected (only if suggestions were skipped)
    if cli.offline && !should_auto_suggest && cli::suggest::should_suggest(&issues) && !cli.quiet {
        cli::suggest::print_suggestions(&issues);
    }

    // Perform optimization
    let optimized = if cli.offline {
        // Static rules only
        optimizer::optimize_static(prompt, &issues)?
    } else {
        // Start optimization spinner for LLM mode
        let spinner = if use_new_renderer {
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

        let result =
            optimizer::optimize_with_llm(prompt, &issues, client.as_ref(), &cli.model, prompt_type)
                .await?;
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
    use tui::model::{AppPhase, Model};

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
            // Use new linear renderer for stats
            if !cli.offline && !result.issues.is_empty() {
                let mut model = Model::new();
                model.offline_mode = cli.offline;
                model.original_prompt = result.original.clone();
                model.input_file = cli.file.as_ref().map(|p| p.display().to_string());
                model.set_issues(&result.issues);
                model.set_optimization_result(result.optimized.clone(), result.stats.clone());
                model.phase = AppPhase::Done;

                // Render stats section only (header/analysis already shown)
                tui::linear::render_stats_only(&model)?;
            }

            if cli.diff {
                tui::diff::print_diff(&result.original, &result.optimized);
            }

            // In offline mode, show helpful message
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
            } else if !cli.diff && cli.show_prompt {
                tui::renderer::print_optimized_prompt(&result.optimized);
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

    // Save the optimized prompt and original prompt for comparison
    if let Some(ref path) = output_path {
        // Create output directory if it doesn't exist
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!("Failed to create output directory: {}", parent.display())
            })?;
        }

        // Derive original prompt path from optimized path
        let original_path = {
            let filename = path.file_name().unwrap().to_string_lossy();
            let original_filename = filename.replace("optimized_", "original_");
            path.with_file_name(original_filename)
        };

        // Write the optimized prompt
        tokio::fs::write(path, &result.optimized)
            .await
            .with_context(|| format!("Failed to write to: {}", path.display()))?;

        // Write the original prompt for comparison
        tokio::fs::write(&original_path, &result.original)
            .await
            .with_context(|| format!("Failed to write original: {}", original_path.display()))?;

        // Also write metadata JSON alongside
        let metadata_path = path.with_extension("json");
        let metadata = serde_json::json!({
            "timestamp": Local::now().to_rfc3339(),
            "files": {
                "original": original_path.file_name().unwrap().to_string_lossy(),
                "optimized": path.file_name().unwrap().to_string_lossy(),
            },
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

/// Run the full-screen interactive TUI mode
async fn run_interactive_mode(cli: &Cli, prompt: &str) -> Result<()> {
    use tui::model::{AppPhase, ErrorState, Model, RenderMode};

    let start_time = std::time::Instant::now();

    // Create the model
    let mut model = Model::new();
    model.render_mode = RenderMode::Interactive;
    model.offline_mode = cli.offline;
    model.original_prompt = prompt.to_string();
    model.input_file = cli.file.as_ref().map(|p| p.display().to_string());

    // Analyze the prompt
    model.phase = AppPhase::Analyzing;
    let issues = analyzer::analyze(prompt, cli.check.as_deref())?;
    model.set_issues(&issues);

    // If not offline, optimize with LLM (even if no static rules triggered,
    // the LLM can enhance prompts beyond what static rules detect)
    if !cli.offline && !cli.analyze {
        model.phase = AppPhase::Optimizing;

        // Run LLM optimization
        let client: Box<dyn llm::LlmClient> = match cli.provider {
            Provider::Anthropic => Box::new(llm::AnthropicClient::new(
                std::env::var("ANTHROPIC_API_KEY")
                    .context("ANTHROPIC_API_KEY environment variable not set")?,
            )?),
            Provider::Bedrock => Box::new(llm::BedrockClient::new(&cli.region).await?),
        };

        let prompt_type = analyzer::classify_prompt(prompt);
        match optimizer::optimize_with_llm(
            prompt,
            &issues,
            client.as_ref(),
            &cli.model,
            prompt_type,
        )
        .await
        {
            Ok(optimized) => {
                let processing_time = start_time.elapsed().as_millis() as u64;

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

                model.set_optimization_result(optimized, stats);
            }
            Err(e) => {
                model.set_error(ErrorState::new(format!("Optimization failed: {}", e)));
            }
        }
    } else {
        // In offline/analyze mode, just show analysis results
        model.phase = AppPhase::AnalysisDone;
    }

    // Run the interactive TUI
    tui::app::run_interactive(&mut model)?;

    // After TUI exits, handle auto-save if we have results
    if let Some(ref optimized) = model.optimized_prompt {
        if !cli.no_save && !cli.offline {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let filename = format!("optimized_{}.txt", timestamp);
            let output_path = cli.output_dir.join(filename);

            // Create output directory if it doesn't exist
            if let Some(parent) = output_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }

            // Write the optimized prompt
            tokio::fs::write(&output_path, optimized).await?;

            // Print save message after TUI exits
            println!(
                "\n{} Saved to: {}\n",
                "✓".green(),
                output_path.display().to_string().white().bold()
            );
        }
    }

    Ok(())
}

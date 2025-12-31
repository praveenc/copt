# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`copt` is a Rust CLI tool that optimizes prompts for Claude 4.5 models. It analyzes prompts for anti-patterns based on Anthropic's best practices and rewrites them using either static rules or LLM-powered optimization.

## Build and Development Commands

```bash
# Build
cargo build              # Debug build
cargo build --release    # Release build (optimized)

# Test
cargo test               # Run all tests
cargo test test_name     # Run specific test
cargo test -- --nocapture  # Show test output
cargo test analyzer::    # Test specific module

# Lint
cargo fmt --check        # Check formatting
cargo fmt                # Fix formatting
cargo clippy -- -D warnings  # Run clippy with warnings as errors

# Run locally
cargo run -- --help
cargo run -- "Your prompt here"
cargo run -- -f prompt.txt --offline   # Analyze without API calls
RUST_LOG=debug cargo run -- -f test.txt --offline  # With debug logging
```

## Environment Variables

For LLM-powered optimization (not needed for `--offline` mode):

```bash
# AWS Bedrock (default provider)
export AWS_ACCESS_KEY_ID="..."
export AWS_SECRET_ACCESS_KEY="..."
export AWS_REGION="us-west-2"

# Anthropic API
export ANTHROPIC_API_KEY="sk-ant-..."
```

## Architecture

```bash
src/
├── main.rs           # CLI entry point, argument parsing (clap), orchestration
├── analyzer/         # Rule-based prompt analysis (25 rules across 8 categories)
│   └── mod.rs        # analyze() function, rule implementations (EXP, STY, TUL, FMT, VRB, AGT, LHT, FED)
├── optimizer/        # Optimization logic
│   └── mod.rs        # optimize_static() and optimize_with_llm()
├── llm/              # LLM client implementations
│   ├── mod.rs        # LlmClient trait, OPTIMIZER_SYSTEM_PROMPT
│   ├── anthropic.rs  # Anthropic API client
│   └── bedrock.rs    # AWS Bedrock client
├── tui/              # Terminal UI (ratatui-based, Elm MVU architecture)
│   ├── mod.rs        # Module exports and legacy compatibility
│   ├── app.rs        # Main event loop for interactive mode
│   ├── model.rs      # State definitions (Model) - MVU pattern
│   ├── update.rs     # Event handling (Update) - MVU pattern
│   ├── view.rs       # Render dispatch (View) - MVU pattern
│   ├── linear.rs     # Non-interactive enhanced output (default)
│   ├── terminal.rs   # Terminal init/restore with panic hooks
│   ├── theme.rs      # Single theme for dark/light terminals
│   ├── icons.rs      # Nerd Font icons with Unicode/ASCII fallback
│   ├── widgets/      # Modular UI components
│   │   ├── header.rs     # ASCII art banner
│   │   ├── analysis.rs   # Collapsible issue tree
│   │   ├── progress.rs   # Optimization gauge
│   │   ├── dashboard.rs  # Stats with bar charts
│   │   ├── diff.rs       # Side-by-side comparison
│   │   ├── status_bar.rs # Keyboard hints
│   │   ├── help.rs       # Full keyboard shortcuts
│   │   ├── error_modal.rs # Modal error dialog
│   │   └── minimal.rs    # Small terminal fallback
│   └── (legacy)      # renderer.rs, spinner.rs, stats.rs, diff.rs, components.rs
├── rules/            # Rule definitions
└── utils/            # Utilities (token counting via tiktoken-rs, file handling)
```

### TUI Modes

- **Default (Linear)**: Enhanced scrolling output with ASCII art banner
- **Interactive (`-i`)**: Full-screen ratatui TUI with keyboard navigation
- **Quiet (`-q`)**: Prompt output only
- **JSON (`--format json`)**: Machine-readable output

## Key Data Flow

1. Input: CLI args, file, or stdin → `get_input_prompt()`
2. Analysis: `analyzer::analyze()` → returns `Vec<Issue>` with detected anti-patterns
3. Optimization: Either `optimize_static()` (offline) or `optimize_with_llm()` (uses LLM)
4. Output: Results displayed via `tui::` functions, saved to `copt-output/` directory

## Analysis Rule Categories

Rules are identified by prefix (e.g., `EXP001`, `STY003`):

- **EXP** (Explicitness): Vague instructions, indirect commands, missing context
- **STY** (Style): Negative framing, aggressive caps, "think" word sensitivity
- **TUL** (Tools): Suggestion vs action, parallel execution guidance
- **FMT** (Formatting): Output format specs, XML structure suggestions
- **VRB** (Verbosity): Progress reporting, summary guidance
- **AGT** (Agentic): Exploration directives, hallucination prevention
- **LHT** (Long-Horizon): State persistence, incremental progress
- **FED** (Frontend): UI aesthetic guidance

See `docs/RULES.md` for complete rule documentation.

## Adding New Analysis Rules

1. Choose category and create rule ID (e.g., `EXP005`)
2. Add detection logic in the corresponding `analyze_*()` function in `src/analyzer/mod.rs`
3. Create `Issue` with id, category, severity, message, line, suggestion
4. Add tests in the `#[cfg(test)]` module
5. Document in `docs/RULES.md`

## Adding New LLM Providers

1. Create `src/llm/newprovider.rs`
2. Implement the `LlmClient` trait (see `anthropic.rs` or `bedrock.rs`)
3. Add to `src/llm/mod.rs` exports
4. Add CLI option in `src/main.rs` `Provider` enum

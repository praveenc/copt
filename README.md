# ⚡ copt — Claude Prompt Optimizer

> Migrate your prompts from Claude 3.x to Claude 4.5 with confidence

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/github/v/release/praveenc/copt?include_prereleases)](https://github.com/praveenc/copt/releases)
[![Build](https://img.shields.io/github/actions/workflow/status/praveenc/copt/ci.yml?branch=main)](https://github.com/praveenc/copt/actions)

---

## Why copt?

**Claude 4.5 models** (Opus, Sonnet, Haiku) are trained for **precise instruction following** — they do exactly what you ask. This is different from Claude 3.x, which was more forgiving of vague or implicit instructions.
Refer to [prompt engineering Claude 4 best practices](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-4-best-practices) for more details.

**copt** helps you:

- 🔍 **Analyze** prompts for Claude 3.x patterns that don't work well with 4.5
- 🛠️ **Identify** anti-patterns like negative instructions, vague commands, and missing context
- ✨ **Optimize** prompts using Claude 4.5 itself for best results
- 📊 **Track** improvements with token counts and metrics

---

## Installation

### Pre-built Binaries (Recommended)

Download the latest release for your platform:

| Platform              | Download                                                                 |
| --------------------- | ------------------------------------------------------------------------ |
| macOS (Apple Silicon) | [copt-macos-arm64](https://github.com/praveenc/copt/releases/latest)     |
| macOS (Intel)         | [copt-macos-x64](https://github.com/praveenc/copt/releases/latest)       |
| Linux (x64)           | [copt-linux-x64](https://github.com/praveenc/copt/releases/latest)       |
| Windows (x64)         | [copt-windows-x64.exe](https://github.com/praveenc/copt/releases/latest) |

```bash
# macOS/Linux: Make executable and move to PATH
chmod +x copt-*
sudo mv copt-* /usr/local/bin/copt

# Verify installation
copt --version
```

### From Source

```bash
# Requires Rust 1.75+
cargo install --git https://github.com/praveenc/copt

# Or build locally
git clone https://github.com/praveenc/copt
cd copt
cargo build --release
./target/release/copt --help
```

---

## Quick Start

### 1. Set Up Credentials

```bash
# Option A: AWS Bedrock (default)
export AWS_ACCESS_KEY_ID="..."
export AWS_SECRET_ACCESS_KEY="..."
export AWS_REGION="us-west-2"

# Option B: Anthropic API
export ANTHROPIC_API_KEY="sk-ant-..."
```

### 2. Optimize a Prompt

```bash
# Direct input
copt "Create a dashboard"

# From file
copt -f my-prompt.txt

# Pipe from stdin
cat prompt.txt | copt

# Analyze only (no API calls)
copt -f prompt.txt --offline
```

---

## Example Output

```
  ⚡  CLAUDE PROMPT OPTIMIZER
     v0.1.1 • Optimize prompts for Claude 4.5

  📥  Input: prompt.txt (847 chars, 215 tokens)

  ──────────────────────────────────────────────────────────────────────
  📊  Analysis Results
  ──────────────────────────────────────────────────────────────────────

  Found 2 warnings, 1 info across 2 categories

  ●  Explicitness (2 issues)
     ⚠ EXP001 Vague instruction detected
     ⚠ EXP003 Indirect command - Claude 4.5 prefers direct commands

  ●  Style (1 issue)
     ℹ STY001 Negative instruction detected (3 lines)

  ✓ Optimization complete [00:01:42]
  ──────────────────────────────────────────────────────────────────────
  📊  Optimization Results
  ──────────────────────────────────────────────────────────────────────

  TOKEN ANALYSIS

  Original:          ███████████████░░░░░░░░░░░░░░░░ 215
  Optimized:         █████████████████████████████░░ 287
  Change:            +33%

  PERFORMANCE

  Processing time:   1.42s
  Rules applied:     3
  Categories fixed:  2

  PROVIDER

  Service:           Bedrock
  Model:             us.anthropic.claude-sonnet-4-5-20250929-v1:0

  ──────────────────────────────────────────────────────────────────────
  ✓  Saved to: copt-output/optimized_20251229_143052.txt
  ──────────────────────────────────────────────────────────────────────
```

---

## Claude 3.x → 4.5 Migration Guide

### What Changes?

| Claude 3.x Pattern   | Problem in 4.5       | copt Fix                                    |
| -------------------- | -------------------- | ------------------------------------------- |
| `Don't use X`        | Negative framing     | Reframe as positive guidance                |
| `Can you help me...` | Indirect command     | Convert to direct instruction               |
| `NEVER do X`         | Aggressive caps      | Normal casing, Claude 4.5 follows precisely |
| `Create something`   | Vague instruction    | Add explicit requirements                   |
| Missing format spec  | Unpredictable output | Add format specifications                   |

### Before & After Examples

**Vague → Explicit**

```diff
- Create a dashboard
+ Create an analytics dashboard with:
+ - User metrics visualization
+ - Date range filtering
+ - Export functionality
+ Include comprehensive features beyond the basics.
```

**Negative → Positive**

```diff
- Don't use placeholder data. Never hardcode values.
+ Use real data from the provided API. Generate dynamic values
+ based on user input and configuration.
```

**Indirect → Direct**

```diff
- Can you help me refactor this code?
+ Refactor this code to improve readability and performance.
+ Make the changes directly using the available tools.
```

**Aggressive → Normal**

```diff
- CRITICAL: You MUST ALWAYS validate input!!!
+ Validate all user input before processing.
```

---

## CLI Reference

```
⚡ Optimize prompts for Claude 4.5 models

Usage: copt [OPTIONS] [PROMPT]

Arguments:
  [PROMPT]  Prompt text to optimize

Options:
  -f, --file <FILE>              Read prompt from file
  -o, --output <FILE>            Save optimized prompt to file
      --output-dir <DIR>         Output directory for auto-save
      --no-save                  Disable auto-save
  -p, --provider <PROVIDER>      Provider: anthropic, bedrock
  -m, --model <MODEL>            Model ID or alias
      --region <REGION>          AWS region for Bedrock
      --format <FORMAT>          Output format: pretty, json, quiet
      --diff                     Show before/after diff
      --show-prompt              Display optimized prompt
  -q, --quiet                    Quiet mode (prompt only)
      --analyze                  Analyze only, no optimization
      --offline                  Offline mode (no API calls)
      --check <CAT>              Check specific categories
  -i, --interactive              Interactive multi-line input
      --skip-connectivity-check  Skip connectivity check
  -v, --verbose                  Verbose output
  -h, --help                     Print help
  -V, --version                  Print version
```

### Common Workflows

```bash
# Analyze prompt without making changes
copt -f prompt.txt --offline

# Optimize and see the diff
copt -f prompt.txt --diff --show-prompt

# Optimize with Anthropic API
copt -f prompt.txt -p anthropic

# JSON output for scripting
copt -f prompt.txt --format json

# Batch process multiple files
for f in prompts/*.txt; do
  copt -f "$f" -o "optimized/$(basename $f)"
done
```

---

## Analysis Rules

copt analyzes prompts across 8 categories with 25+ rules:

| Category         | Rules      | What It Catches                                 |
| ---------------- | ---------- | ----------------------------------------------- |
| **Explicitness** | EXP001-004 | Vague instructions, missing context             |
| **Style**        | STY001-004 | Negative framing, aggressive caps, "think" word |
| **Tools**        | TUL001-003 | Missing action directives                       |
| **Formatting**   | FMT001-003 | Missing output format specs                     |
| **Verbosity**    | VRB001-002 | Missing progress guidance                       |
| **Agentic**      | AGT001-004 | Hallucination risks, exploration hints          |
| **Long-Horizon** | LHT001-003 | State management, incremental progress          |
| **Frontend**     | FED001-002 | UI/design aesthetic guidance                    |

See [docs/RULES.md](docs/RULES.md) for detailed documentation.

---

## Providers

### AWS Bedrock (Default)

```bash
export AWS_ACCESS_KEY_ID="..."
export AWS_SECRET_ACCESS_KEY="..."
export AWS_REGION="us-west-2"

copt -f prompt.txt -p bedrock
```

Supported models:

- `us.anthropic.claude-sonnet-4-5-20250929-v1:0` (default)
- `us.anthropic.claude-opus-4-5-20250929-v1:0`
- `us.anthropic.claude-haiku-4-5-20250929-v1:0`

### Anthropic API

```bash
export ANTHROPIC_API_KEY="sk-ant-..."

copt -f prompt.txt -p anthropic
```

---

## Output Files

When optimizing (not in offline mode), copt automatically saves results:

```
copt-output/
├── optimized_20251229_143052.txt   # Optimized prompt
└── optimized_20251229_143052.json  # Metadata
```

**Metadata JSON:**

```json
{
  "timestamp": "2025-12-29T14:30:52-08:00",
  "original_tokens": 215,
  "optimized_tokens": 287,
  "rules_applied": 3,
  "categories_improved": 2,
  "processing_time_ms": 1420,
  "provider": "bedrock",
  "model": "us.anthropic.claude-sonnet-4-5-20250929-v1:0",
  "issues": [...]
}
```

Use `--no-save` to disable auto-saving, or `-o <file>` to specify a custom path.

---

## Contributing

Contributions welcome! Please read [CONTRIBUTING.md](docs/CONTRIBUTING.md) first.

```bash
# Development setup
git clone https://github.com/praveenc/copt
cd copt
cargo build
cargo test

# Run with debug output
RUST_LOG=debug cargo run -- -f test.txt --offline
```

---

## License

[MIT License](LICENSE) — Copyright (c) 2025 Praveen Chamarthi

---

## Acknowledgments

- [Anthropic](https://www.anthropic.com/) — Claude models and [prompt engineering best practices](https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-4-best-practices)
- [clap](https://github.com/clap-rs/clap) — CLI argument parsing
- [colored](https://github.com/mackwic/colored) — Terminal colors
- [indicatif](https://github.com/console-rs/indicatif) — Progress indicators

---

If you are a Python developer then I recommend reading [Python - Rust comparision](docs/RUSTY_THINGS.md)

<div align="center">
  <b>Built for the Claude developer community with ❤️</b><br>
  <sub>⭐ Star this repo if you find it useful!</sub>
</div>

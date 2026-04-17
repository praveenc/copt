# ⚡ copt — Claude Prompt Optimizer

> Optimize your prompts for Claude 4.x models (4.5, 4.6, and 4.7)

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/github/v/release/praveenc/copt?include_prereleases)](https://github.com/praveenc/copt/releases)
[![Build](https://img.shields.io/github/actions/workflow/status/praveenc/copt/ci.yml?branch=main)](https://github.com/praveenc/copt/actions)

**Claude 4.5 models** do exactly what you ask — no more, no less. Prompts that worked with Claude 3.x may need adjustment. **copt** analyzes your prompts for anti-patterns and optimizes them using Claude 4.5 itself. With Claude 4.7, prompts also need to account for literal instruction following, adaptive thinking, and calibrated response length — **copt** handles these automatically when you target an `opus-4.7` model.

---

## Installation

### Pre-built Binaries

| Platform              | Download                                                                 |
| --------------------- | ------------------------------------------------------------------------ |
| macOS (Apple Silicon) | [copt-macos-arm64](https://github.com/praveenc/copt/releases/latest)     |
| macOS (Intel)         | [copt-macos-x64](https://github.com/praveenc/copt/releases/latest)       |
| Linux (x64)           | [copt-linux-x64](https://github.com/praveenc/copt/releases/latest)       |
| Windows (x64)         | [copt-windows-x64.exe](https://github.com/praveenc/copt/releases/latest) |

```bash
# macOS/Linux quick install
curl -L https://github.com/praveenc/copt/releases/latest/download/copt-macos-arm64 -o copt
chmod +x copt && sudo mv copt /usr/local/bin/
```

<details>
<summary>⚠️ macOS Gatekeeper note</summary>

If macOS blocks the binary, either:
1. Go to **System Settings → Privacy & Security** and click **"Allow Anyway"**
2. Or run: `xattr -d com.apple.quarantine /usr/local/bin/copt`

</details>

### From Source

```bash
cargo install --git https://github.com/praveenc/copt
```

---

## Quick Start

### 1. Set Up Credentials

```bash
# Bedrock API key (recommended — simplest setup)
export AWS_BEARER_TOKEN_BEDROCK="your-bedrock-api-key"

# Or Anthropic API
export ANTHROPIC_API_KEY="sk-ant-..."

# Or AWS credential chain (SigV4 fallback — no API key needed)
# Uses ~/.aws/credentials, AWS_PROFILE, IAM roles, SSO, etc.
```

To generate a Bedrock API key: Bedrock console → API keys → Generate long-term key (up to 1 year).

### 2. Create a Config File (optional)

```bash
copt --config-init    # Creates ~/.config/copt/config.toml
```

Edit the config to set your preferred defaults (including API key):

```toml
[default]
provider = "bedrock"
model = "us.anthropic.claude-sonnet-4-5-20250929-v1:0"

[bedrock]
region = "us-west-2"
api_key = "your-bedrock-api-key"   # or use AWS_BEARER_TOKEN_BEDROCK env var

[output]
format = "pretty"
show_diff = false
```

CLI arguments always take precedence over config values.

### 3. Optimize a Prompt

```bash
copt "Create a dashboard"           # Direct input
copt -f my-prompt.txt               # From file
copt -f prompt.txt --offline        # Analyze only (no API calls)
copt -f prompt.txt -i               # Interactive TUI mode
```

---

## What It Does

copt detects Claude 3.x patterns and rewrites them for Claude 4.5:

| Pattern | Problem | Fix |
|---------|---------|-----|
| `Don't use X` | Negative framing | Reframe positively |
| `Can you help...` | Indirect command | Direct instruction |
| `NEVER do X` | Aggressive caps | Normal casing |
| `Create something` | Vague instruction | Add explicit requirements |

**Example transformation:**

```diff
- Don't use placeholder data. Can you help me create a dashboard?
+ Use real data from the API. Create an analytics dashboard with:
+ - User metrics visualization
+ - Date range filtering
+ - Export functionality
```

See [docs/RULES.md](docs/RULES.md) for the full list of 25 analysis rules across 8 categories.

---

## CLI Reference

<details>
<summary>View full <code>--help</code> output</summary>

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
  -t, --target <FAMILY>          Target Claude family: auto (default), 4.5, 4.6, 4.7
      --region <REGION>          AWS region for Bedrock
      --format <FORMAT>          Output format: pretty, json, quiet
      --diff                     Show before/after diff
      --show-prompt              Display optimized prompt
  -q, --quiet                    Quiet mode (prompt only)
      --analyze                  Analyze only, no optimization
      --offline                  Offline mode (no API calls)
      --check <CAT>              Check specific categories
      --no-suggest               Disable auto-suggestions for vague prompts
  -i, --interactive              Launch full-screen interactive TUI mode
  -e, --editor                   Open editor for multi-line input
      --skip-connectivity-check  Skip connectivity check
      --config-init              Create default config file and exit
  -v, --verbose                  Verbose output
  -h, --help                     Print help
  -V, --version                  Print version
```

</details>

### Model Aliases

Use short aliases instead of full Bedrock ARNs:

| Alias | Model |
|-------|-------|
| `sonnet` / `sonnet-4.5` | `us.anthropic.claude-sonnet-4-5-20250929-v1:0` |
| `opus` / `opus-4.5` | `us.anthropic.claude-opus-4-5-20251101-v1:0` |
| `opus-4.6` | `global.anthropic.claude-opus-4-6-v1` |
| `sonnet-4.6` | `global.anthropic.claude-sonnet-4-6` |
| `opus-4.7` | `global.anthropic.claude-opus-4-7` |
| `haiku` / `haiku-4.5` | `us.anthropic.claude-haiku-4-5-20251001-v1:0` |

```bash
copt -f prompt.txt -m opus            # Use Claude Opus 4.5
copt -f prompt.txt -m haiku           # Use Claude Haiku 4.5
```

### Claude 4.7 Support

**Opus 4.7** is generally available as of 2026-04-16. Sonnet 4.7 and Haiku 4.7 are not yet released.

When you target `opus-4.7`, **copt** routes optimization through a Claude 4.7-specific meta-prompt that applies eight 4.7 behaviors: **literal instruction following** (the model does exactly what you say), **adaptive thinking guidance** (explicit reasoning directives), **effort-level awareness** (xhigh/high/medium/low/max markers), **scratchpad and memory directives** (working-memory tags), **condensed context** (drops 4.5/4.6 scaffolding that 4.7 doesn't need), **tone specification** (explicit tone markers), **response-length calibration** (explicit length bounds), and **vision-aware instructions** (high-resolution image handling up to 2576px/3.75MP with 1:1 pixel coordinates).

Selecting `sonnet-4.7` or `haiku-4.7` produces a clear "not yet released" error unless you run in `--offline` mode (offline mode still works for analysis).

**Usage:**

```bash
copt -f prompt.txt -m opus-4.7
```

### Targeting a family explicitly (`-t` / `--target`)

By default `copt` infers the target Claude family from `--model` (`-m`). To decouple the rewriter model from the target family — for example, to use a fast, cheap Sonnet 4.5 to produce a prompt tailored for Opus 4.7 — pass `-t`:

```bash
# Opus 4.7 rewrites for Opus 4.7 (auto)
copt -f prompt.txt -m opus-4.7

# Sonnet 4.5 rewrites for Opus 4.7 family best-practices
copt -f prompt.txt -m sonnet -t 4.7

# Opus 4.7 rewrites a prompt that will run on Sonnet 4.5
copt -f prompt.txt -m opus-4.7 -t 4.5
```

Accepted values for `-t`: `auto` (default), `4.5`, `4.6`, `4.7`.

### Common Examples

```bash
copt -f prompt.txt --offline          # Analyze without API calls
copt -f prompt.txt --diff             # Show before/after diff
copt -f prompt.txt -p anthropic       # Use Anthropic API
copt -f prompt.txt --format json      # JSON output for scripting
copt -f prompt.txt --no-suggest       # Disable suggestions (for CI)
copt --config-init                    # Create default config file
```

---

## Interactive Mode

Launch with `-i` for a full-screen TUI:

```bash
copt -f prompt.txt -i
```

**Keyboard shortcuts:**

| Key | Action |
|-----|--------|
| `q` | Quit |
| `d` | Toggle diff view |
| `c` | Copy to clipboard |
| `s` / `e` | Save & open in editor |
| `?` | Help |

---

## Prompt Storage

Optimized prompts are auto-saved to `~/.copt/prompts/YYYY-MM-DD/` with descriptive filenames:

```
~/.copt/prompts/
└── 2026-02-15/
    ├── dashboard-analytics-api_143022_original.txt
    ├── dashboard-analytics-api_143022_optimized.txt
    └── dashboard-analytics-api_143022.json
```

Use `--output-dir <DIR>` to override, or `--no-save` to disable.

---

## Raycast Integration

Optimize prompts directly from your clipboard. See [scripts/raycast/README.md](scripts/raycast/README.md) for setup.

---

## Documentation

- [Analysis Rules](docs/RULES.md) — All 27 rules across 8 categories
- [Migration Guide](docs/MIGRATION.md) — Claude 3.x → 4.5 patterns
- [Contributing](docs/CONTRIBUTING.md) — Development setup
- [Rust for Pythonistas](docs/RUSTY_THINGS.md) — If you're coming from Python

---

## License

[MIT](LICENSE) — Copyright (c) 2026 Praveen Chamarthi

---

<div align="center">
  <sub>Built for the Claude developer community ❤️</sub>
</div>
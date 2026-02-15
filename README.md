# ⚡ copt — Claude Prompt Optimizer

> Optimize your prompts for Claude 4.5 models

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/github/v/release/praveenc/copt?include_prereleases)](https://github.com/praveenc/copt/releases)
[![Build](https://img.shields.io/github/actions/workflow/status/praveenc/copt/ci.yml?branch=main)](https://github.com/praveenc/copt/actions)

**Claude 4.5 models** do exactly what you ask — no more, no less. Prompts that worked with Claude 3.x may need adjustment. **copt** analyzes your prompts for anti-patterns and optimizes them using Claude 4.5 itself.

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
# AWS Bedrock (default)
export AWS_ACCESS_KEY_ID="..."
export AWS_SECRET_ACCESS_KEY="..."
export AWS_REGION="us-west-2"

# Or Anthropic API
export ANTHROPIC_API_KEY="sk-ant-..."
```

### 2. Create a Config File (optional)

```bash
copt --config-init    # Creates ~/.config/copt/config.toml
```

Edit the config to set your preferred defaults:

```toml
provider = "bedrock"
model = "us.anthropic.claude-sonnet-4-5-20250929-v1:0"
region = "us-west-2"
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

See [docs/RULES.md](docs/RULES.md) for the full list of 27 analysis rules across 8 categories.

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
      --output-dir <DIR>         Output directory [default: copt-output]
      --no-save                  Disable auto-save
  -p, --provider <PROVIDER>      Provider: anthropic, bedrock [default: bedrock]
  -m, --model <MODEL>            Model ID or alias
      --region <REGION>          AWS region for Bedrock
      --format <FORMAT>          Output format: pretty, json, quiet
      --diff                     Show before/after diff
      --show-prompt              Display optimized prompt
  -q, --quiet                    Quiet mode (prompt only)
      --analyze                  Analyze only, no optimization
      --offline                  Offline mode (no API calls)
      --check <CAT>              Check specific categories
      --no-suggest               Disable auto-suggestions
  -i, --interactive              Full-screen interactive TUI
      --config-init              Create default config file and exit
      --skip-connectivity-check  Skip connectivity check
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
| `opus-4.6` | `us.anthropic.claude-opus-4-6-v1` |
| `haiku` / `haiku-4.5` | `us.anthropic.claude-haiku-4-5-20251001-v1:0` |

```bash
copt -f prompt.txt -m opus            # Use Claude Opus 4.5
copt -f prompt.txt -m haiku           # Use Claude Haiku 4.5
```

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
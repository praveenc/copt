# Raycast Integration for copt

Optimize prompts directly from your clipboard using Raycast.

## How It Works

1. Copy a prompt to your clipboard
2. Trigger "Optimize Prompt" from Raycast
3. Wait a few seconds while Claude optimizes it via Bedrock
4. Your clipboard is replaced with the optimized prompt — paste it anywhere

Both the original and optimized prompts are auto-saved to `~/.copt/prompts/YYYY-MM-DD/` with descriptive filenames.

## Setup

### Prerequisites

- [Raycast](https://raycast.com) installed
- `copt` binary in your PATH (verify with `copt --version`)
- AWS credentials configured for Bedrock access

### Install the Script Command

1. Open Raycast Settings → Extensions → Script Commands
2. Click "Add Script Directory"
3. Select the `scripts/raycast/` folder from this repo
4. The "Optimize Prompt" command appears in Raycast immediately

Alternatively, create a symlink from your existing Raycast script directory:

```bash
ln -s /path/to/copt/scripts/raycast/optimize-prompt.sh ~/raycast-scripts/optimize-prompt.sh
```

### Configuration

The script uses your existing `copt` config at `~/.config/copt/config.toml` (if present). No additional configuration needed.

If `copt` is not in your PATH, set the `COPT_BIN` environment variable in Raycast Settings → Extensions → Script Commands → Optimize Prompt:

```bash
COPT_BIN=/usr/local/bin/copt
```

## Prompt Storage

Optimized prompts are auto-saved by `copt` to:

```
~/.copt/prompts/
└── 2026-02-15/
    ├── dashboard-analytics-api_143022_original.txt
    ├── dashboard-analytics-api_143022_optimized.txt
    ├── dashboard-analytics-api_143022.json
    ├── code-review-rust-pr_150510_original.txt
    ├── code-review-rust-pr_150510_optimized.txt
    └── code-review-rust-pr_150510.json
```

Each optimization creates a timestamped triplet (original + optimized + metadata) with a descriptive name derived from the prompt content, grouped by date.

## Tips

- Assign a hotkey in Raycast for quick access (e.g., `⌥⇧O`)
- Works with any text in your clipboard — prompt files, chat messages, system prompts
- The script uses `--format json` to cleanly extract the optimized prompt — copt handles saving automatically to `~/.copt/prompts/`

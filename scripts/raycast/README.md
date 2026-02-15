# Raycast Integration for copt

Optimize prompts directly from your clipboard using Raycast.

## How It Works

1. Copy a prompt to your clipboard
2. Trigger "Optimize Prompt" from Raycast
3. Wait a few seconds while Claude optimizes it via Bedrock
4. Your clipboard is replaced with the optimized prompt — paste it anywhere

Both the original and optimized prompts are saved to `~/.copt/prompts/YYYY-MM-DD/` for reference.

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

Optimized prompts are saved to:

```bash
~/.copt/prompts/
└── 2026-02-15/
    ├── original_143022.txt
    ├── optimized_143022.txt
    ├── original_150510.txt
    └── optimized_150510.txt
```

Each optimization creates a timestamped pair (original + optimized) grouped by date.

## Tips

- Assign a hotkey in Raycast for quick access (e.g., `⌥⇧O`)
- Works with any text in your clipboard — prompt files, chat messages, system prompts
- The script uses `--quiet --no-save` flags so copt only outputs the optimized text and skips its default save behavior

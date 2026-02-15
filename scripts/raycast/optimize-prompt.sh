#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title Optimize Prompt
# @raycast.mode compact
# @raycast.packageName copt

# Optional parameters:
# @raycast.icon ⚡
# @raycast.description Optimize clipboard prompt using Claude via Bedrock
# @raycast.author Praveen Chamarthi
# @raycast.authorURL https://github.com/praveenc

# --- Configuration ---
# Raycast runs in a minimal shell — source your profile for PATH and AWS env vars
if [ -f "$HOME/.zshrc" ]; then
  source "$HOME/.zshrc" 2>/dev/null
elif [ -f "$HOME/.zprofile" ]; then
  source "$HOME/.zprofile" 2>/dev/null
fi

export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/local/bin:/opt/homebrew/bin:$PATH"

COPT_BIN="${COPT_BIN:-copt}"
PROMPTS_DIR="$HOME/.copt/prompts"

# Ensure AWS credentials are available
# Set these if your .zshrc doesn't export them, or override via Raycast env vars
# AWS_PROFILE="${AWS_PROFILE:-default}"
# AWS_REGION="${AWS_REGION:-us-west-2}"

# --- Preflight ---
if ! command -v "$COPT_BIN" &>/dev/null; then
  echo "copt not found. Install it or set COPT_BIN."
  exit 1
fi

clipboard="$(pbpaste)"
if [ -z "$clipboard" ]; then
  echo "Clipboard is empty"
  exit 1
fi

# --- Optimize ---
# Use JSON format to cleanly extract the optimized prompt
error_output=$("$COPT_BIN" "$clipboard" --format json --no-save 2>/tmp/copt_raycast_err.log)
exit_code=$?

if [ $exit_code -ne 0 ] || [ -z "$error_output" ]; then
  error_msg=$(cat /tmp/copt_raycast_err.log 2>/dev/null | head -3)
  echo "Optimization failed: ${error_msg:-unknown error}"
  exit 1
fi

# Extract just the optimized prompt from JSON output
optimized=$(echo "$error_output" | /usr/bin/python3 -c "import sys,json; print(json.load(sys.stdin).get('optimized',''))" 2>/dev/null)

if [ -z "$optimized" ]; then
  echo "Could not extract optimized prompt from output"
  exit 1
fi

# --- Save to ~/.copt/prompts/YYYY-MM-DD/ ---
today=$(date +%Y-%m-%d)
timestamp=$(date +%H%M%S)
day_dir="$PROMPTS_DIR/$today"
mkdir -p "$day_dir"

echo "$clipboard"  > "$day_dir/original_${timestamp}.txt"
echo "$optimized"  > "$day_dir/optimized_${timestamp}.txt"

# --- Update clipboard ---
echo "$optimized" | pbcopy

echo "Prompt optimized ✓"

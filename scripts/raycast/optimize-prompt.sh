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
# copt auto-saves to ~/.copt/prompts/YYYY-MM-DD/ with smart naming
json_output=$("$COPT_BIN" "$clipboard" --format json 2>/tmp/copt_raycast_err.log)
exit_code=$?

if [ $exit_code -ne 0 ] || [ -z "$json_output" ]; then
  error_msg=$(cat /tmp/copt_raycast_err.log 2>/dev/null | head -3)
  echo "Optimization failed: ${error_msg:-unknown error}"
  exit 1
fi

# Extract the optimized prompt and name from JSON output
optimized=$(echo "$json_output" | /usr/bin/python3 -c "import sys,json; print(json.load(sys.stdin).get('optimized',''))" 2>/dev/null)
name=$(echo "$json_output" | /usr/bin/python3 -c "import sys,json; print(json.load(sys.stdin).get('name','prompt'))" 2>/dev/null)

if [ -z "$optimized" ]; then
  echo "Could not extract optimized prompt from output"
  exit 1
fi

# --- Update clipboard ---
echo "$optimized" | pbcopy

echo "✓ ${name}"

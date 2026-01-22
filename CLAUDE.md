# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`copt` is a Rust CLI tool that optimizes prompts for Claude 4.5 models. It analyzes prompts for anti-patterns based on Anthropic's best practices and rewrites them using either static rules or LLM-powered optimization.

## Git Operations

**All git operations (commit, push, tag) must run inside the git-workspace container.**

There are two ways to work with the container, depending on your IDE:

| Environment | When to Use | Git Command Style |
|-------------|-------------|-------------------|
| **Zed Dev Container** | Running inside the container (via Zed) | Direct: `git commit -m "..."` |
| **From Host (docker exec)** | Running outside the container (other IDEs, CI, terminal) | Prefixed: `docker exec my-git-workspace git -C /workspace/repos/copt commit -m "..."` |

### Zed Dev Container (Recommended for Zed Users)

When working in Zed with the dev container connected, git commands run directly without `docker exec` prefixes.

#### Detecting Dev Container Environment

Check if you're running inside the dev container:
```bash
# If this shows "Docker Debug" hints, you're inside the dev container
echo $HOSTNAME  # Shows container ID like "7e1040108a40"

# Or check for the container indicator
test -f /.dockerenv && echo "Inside container" || echo "On host"
```

#### Verify Setup
```bash
gh auth status                    # Should show: ✓ Logged in to github.com account praveenc
git config user.name              # Should show: Praveen Chamarthi  
git config user.email             # Should show: 1090396+praveenc@users.noreply.github.com
```

#### Git Commands (Dev Container)
```bash
git add -A
git commit -m "feat: your message"
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin main --tags
gh release create vX.Y.Z --repo praveenc/copt --title "vX.Y.Z" --notes "Release notes"
```

#### Opening in Dev Container
1. Open `copt` folder in Zed
2. Click "Open in Dev Container" when toast appears (or use `project: open remote` → "Connect Dev Container")
3. Wait for container to start and Zed Remote Server to connect
4. Title bar indicates dev container connection

See `docs/DEV_CONTAINERS.md` for detailed setup and troubleshooting.

### From Host (docker exec)

Use this approach when running **outside** the container:
- Working from a host terminal (not inside Zed dev container)
- Running CI/CD scripts
- Using IDEs without dev container support

#### Setup
1. **Check if running**: `docker ps --filter name=my-git-workspace --format '{{.Names}}'`
2. **Start only if not running**: `docker compose run -d --rm --name my-git-workspace git-workspace`

#### Verify Git Identity
Before any commit, verify git config (must run from repo directory):
```bash
docker exec -w /workspace/repos/copt my-git-workspace git-test
```
Expected: username `praveenc`, email `1090396+praveenc@users.noreply.github.com`

**Note**: Running from `/workspace` root will show `not set` for user.name/email since git config is set at repo level.

#### Git Commands (From Host)
```bash
docker exec my-git-workspace git -C /workspace/repos/copt <command>
docker exec my-git-workspace gh <command>
```

Repo path inside container: `/workspace/repos/copt`

## Build and Development Commands

### Makefile (Preferred)

```bash
make ci          # CI pipeline (debug): fmt-check → lint → build → test
make ci-debug    # Same as `make ci` (explicit debug build)
make ci-release  # CI pipeline (release): fmt-check → lint → release → test
make check       # Local dev: fmt → lint → test (auto-fixes formatting)
make build       # Debug build
make release     # Release build (optimized)
make test        # Run all tests
make lint        # Clippy with warnings as errors
make fmt         # Auto-fix formatting
make clean       # Clean build artifacts
```

### Direct Cargo Commands

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
cargo run -- -f prompt.txt -i          # Full-screen interactive TUI mode
cargo run -- -f prompt.txt --no-suggest  # Disable auto-suggestions for scripting
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

## LLM Inference Configuration

The LLM clients use these inference parameters for prompt optimization:

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `temperature` | 0.3 | Low for consistent, deterministic rewrites |
| `max_tokens` | 4096 | Sufficient for most prompt optimizations |

**Note**: Claude 4.5 models do not allow `temperature` and `top_p` to be specified together. We use `temperature` only.

These settings prioritize **reproducibility** over creativity — prompt optimization is a precision task where users expect similar inputs to produce similar outputs.

## Architecture

```bash
src/
├── main.rs           # CLI entry point, argument parsing (clap), orchestration
├── analyzer/         # Rule-based prompt analysis (27 rules across 8 categories)
│   └── mod.rs        # analyze() function, XML-aware parsing, prompt type classifier, rule implementations
├── cli/              # CLI modules
│   ├── mod.rs        # CLI argument definitions
│   ├── config.rs     # Configuration file support
│   └── suggest.rs    # Interactive suggestions for vague prompts (EXP005/EXP006)
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

- **EXP** (Explicitness): Vague instructions, indirect commands, missing context, role-only prompts (EXP005), open-ended instructions (EXP006)
- **STY** (Style): Negative framing, aggressive caps, "think" word sensitivity
- **TUL** (Tools): Suggestion vs action, parallel execution guidance
- **FMT** (Formatting): Output format specs, XML structure suggestions
- **VRB** (Verbosity): Progress reporting, summary guidance
- **AGT** (Agentic): Exploration directives, hallucination prevention
- **LHT** (Long-Horizon): State persistence, incremental progress
- **FED** (Frontend): UI aesthetic guidance

See `docs/RULES.md` for complete rule documentation.

### Hybrid Analyzer Features

- **XML-Aware Parsing**: Extracts `<examples>`, `<example>`, `<instructions>` blocks before analysis to prevent false positives
- **Prompt Type Classifier**: Detects prompt type (Coding, QaAssistant, Research, Creative, LongHorizon, General) for context-aware rule application
- **Auto-Suggest for Vague Prompts**: When EXP005/EXP006 detected in a TTY, automatically offers multi-select dialog to enhance prompts (use `--no-suggest` to disable)
- **TUI Suggest Modal**: In interactive mode (`-i`), shows a modal dialog with checkbox selection and keyboard navigation (↑/↓/Space/Enter/Esc)

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

## Release Process

When bumping the version for a new release:

1. **Update version in `Cargo.toml`**
   ```bash
   # Edit Cargo.toml: version = "X.Y.Z"
   ```

2. **Update snapshot tests** (critical - CI will fail otherwise!)
   ```bash
   # Option A: Auto-accept all snapshot changes
   cargo insta test --accept
   
   # Option B: Manually update snapshots
   sed -i '' 's/vOLD_VERSION/vNEW_VERSION/g' src/tui/snapshots/*.snap
   ```
   
   The TUI snapshots in `src/tui/snapshots/` contain the version string in the header.
   Forgetting this step will cause 6-7 snapshot tests to fail in CI.

3. **Update CHANGELOG.md**
   - Change `[Unreleased]` to `[X.Y.Z] - YYYY-MM-DD`
   - Add new `[Unreleased]` section at top
   - Update comparison links at bottom

4. **Run full test suite**
   ```bash
   make ci
   ```

5. **Commit, tag, and push**

   **In Zed Dev Container:**
   ```bash
   git add -A
   git commit -m "chore: bump version to X.Y.Z"
   git tag -a vX.Y.Z -m "Release vX.Y.Z - Description"
   git push origin main --tags
   ```

   **From Host (docker exec):**
   ```bash
   docker exec my-git-workspace git -C /workspace/repos/copt add -A
   docker exec my-git-workspace git -C /workspace/repos/copt commit -m "chore: bump version to X.Y.Z"
   docker exec my-git-workspace git -C /workspace/repos/copt tag -a vX.Y.Z -m "Release vX.Y.Z - Description"
   docker exec my-git-workspace git -C /workspace/repos/copt push origin main --tags
   ```

6. **Create GitHub release**

   **In Zed Dev Container:**
   ```bash
   gh release create vX.Y.Z \
     --repo praveenc/copt \
     --title "vX.Y.Z - Release Title" \
     --notes "Release notes here"
   ```

   **From Host (docker exec):**
   ```bash
   docker exec my-git-workspace gh release create vX.Y.Z \
     --repo praveenc/copt \
     --title "vX.Y.Z - Release Title" \
     --notes "Release notes here"
   ```

## CI Tips

### Skip CI for Non-Code Changes

To skip CI for documentation-only commits (README, CHANGELOG, etc.), add `[skip ci]` to the commit message:

```bash
git commit -m "docs: update README [skip ci]"
```

Supported skip patterns:
- `[skip ci]` or `[ci skip]`
- `[skip actions]` or `[actions skip]`
- `[no ci]`

Use this for:
- Documentation updates
- README changes
- Changelog tweaks
- Comment-only changes
- Other non-code modifications

**Note**: Don't skip CI for any changes that affect code, tests, or build configuration.

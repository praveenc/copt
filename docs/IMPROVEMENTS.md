# COPT Improvement Recommendations

## Executive Summary

COPT is a well-structured Rust CLI with a clear purpose and solid architecture (Elm MVU for the TUI, clean separation of analyzer/optimizer/LLM layers). However, the project carries significant dead weight: an entire `rules` module that duplicates the analyzer, a `cli/config` module that's never wired into the application, ~6 unused Cargo dependencies inflating compile times, and pervasive `#![allow(dead_code)]` annotations masking the problem. The static optimization rules are genuinely useful but limited to 4 of 25 detected issues — the gap between what the analyzer detects and what the static optimizer can fix is the project's biggest missed opportunity.

## Feature Improvements

### 1. Wire the config system into the application — ✅ DONE (v0.3.0)

`src/cli/config.rs` defines a complete configuration system (provider defaults, rule disabling, severity overrides, API key storage) but it is never called from `main.rs`. The `load_config()` function exists, `Config` has validation, but zero integration. This means:
- Users cannot disable noisy rules (e.g., STY003 "think" word) without `--check` flags every invocation
- The `--disable-rules` and `--disable-categories` flags documented in `docs/RULES.md` don't actually exist in the CLI
- Provider defaults must be passed via CLI args every time

Wire `load_config()` into `main()` and merge config values with CLI args (CLI takes precedence). This is the single highest-impact feature gap.

### 2. Use the model alias system — ✅ DONE (v0.3.0)

`src/cli/mod.rs` defines `resolve_model_id()` and `MODEL_ALIASES` (e.g., "sonnet" → full Bedrock ARN) but these are never called. The `--model` flag in `main.rs` passes the raw string directly to the LLM client. Users must type `us.anthropic.claude-sonnet-4-5-20250929-v1:0` instead of just `sonnet`. Call `resolve_model_id()` on `cli.model` before passing it downstream.

### 3. Use tiktoken-rs for accurate token counting — ✅ DONE (v0.3.0)

Decided to keep `chars/4` heuristic (display-only; providers handle tokenization). Removed `tiktoken-rs` dependency.

### 4. Expand static optimization coverage — ✅ DONE (v0.3.0)

The analyzer detects 25 rules but `optimize_static()` only handles 4 (EXP003, STY002, STY003, STY004). Several more rules have straightforward static transformations:
- **STY001** (negative instructions): "Don't use X" → "Use Y instead" — regex-replaceable for common patterns
- **FMT002** (negative format): "No markdown" → "Write in prose paragraphs" — similar pattern
- **EXP002** (bare prohibitions): Could append "because [reason]" placeholder

This would make `--offline` mode substantially more useful.

### 5. Add `--config init` subcommand — ✅ DONE (v0.3.0)

Exposed as `copt --config-init`. Creates `~/.config/copt/config.toml` with commented defaults.

### 6. Streaming LLM output — Nice to Have (High)

Both LLM clients wait for the full response before displaying anything. For large prompts, the optimization step can take 10+ seconds with no feedback beyond a spinner. The Bedrock client already uses `invoke_model` (not streaming) — switching to `invoke_model_with_response_stream` would let the TUI show incremental output. The Anthropic client could use SSE streaming similarly.

### 7. Connectivity check is blocking UX — ✅ DONE (v0.3.0)

Successful checks now cached with 5-minute TTL in `/tmp/copt_connectivity_<provider>_<region>.cache`.

### 8. Smart prompt naming — ✅ DONE (v0.3.1)

`generate_prompt_slug()` extracts a descriptive slug from prompt content for filenames (e.g., `dashboard-analytics-api_143022_optimized.txt`). JSON output and metadata include a `name` field.

### 9. Standardize prompt storage to `~/.copt/prompts/` — ✅ DONE (v0.3.1)

Default save location changed from `./copt-output/` to `~/.copt/prompts/YYYY-MM-DD/`. Date-bucketed, centralized. `--output-dir` still works for overrides.

### 10. Bedrock API Key authentication

Replace the current AWS SDK SigV4 credential chain (requires AWS CLI, `aws configure`, IAM roles, `AWS_PROFILE`, etc.) with Bedrock API Key Bearer token authentication. Long-term keys support up to 1-year expiry — effectively "set and forget" for a developer CLI tool.

Key changes:
- Replace `aws-sdk-bedrockruntime` + `aws-config` with direct `reqwest` HTTP calls using `Authorization: Bearer <key>` header
- Read key from `AWS_BEARER_TOKEN_BEDROCK` env var, `--bedrock-api-key` CLI flag, or `~/.config/copt/config.toml`
- Drop 3 heavy AWS SDK crates → faster compile times, smaller binary
- Fall back to existing SigV4 credential chain when no API key is present (backward compatible)
- Update config file, Raycast script, README, error messages

## Code Efficiency & Robustness

### 1. Duplicated `build_editor_command` function — ✅ DONE (v0.3.0)

Extracted to `src/utils/editor.rs`.

### 2. Duplicated `Issue` type definitions — ✅ DONE (v0.3.0)

Removed `src/rules/mod.rs` entirely.

### 3. Regex recompilation in optimizer transforms — ✅ DONE (v0.3.0)

All optimizer transforms now use `LazyLock<Regex>`.

### 4. `format!("{:?}", cli.provider).to_lowercase()` for provider name — ✅ DONE (v0.3.0)

Added `Provider::as_str()` method.

### 5. Inconsistent `Issue` field types between analyzer and rules — ✅ DONE (v0.3.0)

No longer applicable — `src/rules/mod.rs` was removed entirely.

### 6. `handle_output` rebuilds Model from scratch

`src/main.rs:handle_output()` (line ~570) creates a new `Model` to render stats, even though `run_optimization()` already built one. The model from `run_optimization` should be returned and reused.

### 7. Token counting called redundantly

`count_tokens()` is called on the same prompt string in multiple places: once in `run_optimization()` for stats, again in `handle_output()` when building the Model for rendering, and again inside `render_header()` / `render_input_info()`. Cache the token count in the Model or OptimizationStats.

## Static Styles Assessment

The "static styles" in COPT serve two distinct purposes that should be evaluated separately:

**The TUI theme system (`src/tui/theme.rs`)** is well-designed. It provides a single `Theme` struct with consistent color assignments, accessed via a global `OnceLock` singleton. The color choices (cyan primary, green success, yellow warning, red error, dark gray muted) are sensible defaults that work on both dark and light terminals. The theme is used consistently across all ratatui widgets. This is solid and should be kept as-is.

**The static optimization rules (`src/optimizer/mod.rs:optimize_static`)** are the area of concern. Currently, only 4 of 25 detected issues have static transformations:
- `EXP003`: Strips "Can you..." / "Could you..." prefixes — works well
- `STY002`: Lowercases ALL CAPS non-acronyms — works but aggressive (converts "CRITICAL" to "Critical" rather than removing the word)
- `STY003`: Replaces "think" variants — works correctly
- `STY004`: Tones down emphatic language — works but can produce awkward phrasing ("You should ALWAYS" → "You should should")

The interaction between STY002 and STY004 is fragile. Both transforms are applied sequentially per-issue in `optimize_static()`. If STY002 fires first, it lowercases "CRITICAL" to "Critical", so STY004's `\bCRITICAL:\s*` regex (case-insensitive) still matches — but the colon may or may not be present depending on context. More importantly, the STY004 replacements don't compose cleanly: "You MUST ALWAYS validate" triggers the "MUST ALWAYS" → "should" rule, but if the issues arrive in a different order or the prompt has "ALWAYS" without "MUST", the standalone "ALWAYS" has no replacement rule at all — it's only handled by STY002's caps lowering. The transforms need a single-pass approach or explicit ordering guarantees.

**Recommendation**: The static rules are genuinely valuable for `--offline` mode and as a pre-processing step before LLM optimization. But they need:
1. Fix the STY004 double-replacement bug
2. Expand coverage to more rules (STY001, FMT002, EXP002 are good candidates)
3. Add integration tests that verify the full `optimize_static()` pipeline, not just individual transforms

The `Enhancement` struct and `get_applicable_enhancements()` function in `src/optimizer/mod.rs` (lines 249-296) represent an interesting idea — context-aware prompt appendages — but they are never called from anywhere. Either wire them into the optimization pipeline or remove them.

## Dead Code Identified

### Entire modules never used externally

| Location | What | Why it's dead |
|----------|------|---------------|
| `src/rules/mod.rs` (entire file, ~250 lines) | Parallel `Issue`, `Severity`, `Category` types, regex `patterns` module, helper functions | Nothing imports from `rules::`. The analyzer has its own implementations of everything this module provides. The `mod rules;` declaration in `main.rs` compiles it, but no code path reaches it. |
| `src/cli/config.rs` (entire file, ~360 lines) | `Config`, `load_config()`, `create_default_config()`, provider/rules config structs | Never called from `main.rs` or anywhere else. The config system is fully implemented but completely unwired. |
| `src/utils/file.rs` (entire file, ~170 lines) | `read_prompt_file()`, `write_prompt_file()`, `file_exists()`, `is_prompt_file()`, `format_file_size()`, `read_prompts_from_dir()`, async variants | Zero external callers. `main.rs` uses `tokio::fs::read_to_string` and `tokio::fs::write` directly instead of these wrappers. |

### Unused functions and types

| Location | What | Why it's dead |
|----------|------|---------------|
| `src/cli/mod.rs`: `DEFAULT_MODEL`, `DEFAULT_MAX_TOKENS`, `AVAILABLE_MODELS`, `MODEL_ALIASES`, `resolve_model_id()`, `is_valid_model()` | Model alias resolution system | Never called. `main.rs` hardcodes the default model in the clap attribute and passes `cli.model` directly to LLM clients. |
| `src/optimizer/mod.rs:249-296`: `Enhancement` struct, `get_applicable_enhancements()` | Context-aware prompt enhancement system | Never called from any code path. |
| `src/llm/mod.rs`: `CompletionRequest`, `Message`, `Role`, `CompletionResponse`, `Usage` | Generic LLM request/response types | Marked "for future use" in comments. Both `AnthropicClient` and `BedrockClient` define their own request/response structs internally. |
| `src/utils/text.rs`: `word_count()`, `line_count()`, `truncate()`, `truncate_lines()`, `normalize_whitespace()`, `extract_preview()`, `contains_code()`, `is_system_prompt()`, `extract_xml_tags()`, `text_similarity()`, `calculate_change_percent()` | Text utility functions | Only `count_tokens()` is used (re-exported via `utils/mod.rs`). The other 11 functions have zero callers. |
| `src/tui/mod.rs`: `pad_right()`, `center()`, `draw_line()`, `draw_box_top()`, `draw_box_bottom()`, `draw_box_line()`, `colors` module | Legacy box-drawing and color utilities | Superseded by ratatui widgets. Only `terminal_width()`, `truncate()`, `chars` module, and `legacy_icons` are still used (by `diff.rs`). |
| `src/tui/renderer.rs`: `print_header()`, `print_offline_banner()`, `print_input_info()`, `print_analysis()`, `print_optimizing()`, `print_success()`, `print_error()`, `print_warning()`, `print_separator()` | Legacy renderer functions | Only `start_optimizing_spinner()`, `stop_optimizing_spinner()`, and `print_optimized_prompt()` are still called from `main.rs`. The other 9 functions are dead. |
| `src/tui/stats.rs`: `print_stats()`, `print_stats_compact()` | Legacy stats display | Only `print_save_success()` is called. The other 2 functions are dead. |
| `src/tui/diff.rs`: `print_unified_diff()`, `diff_stats()`, `DiffStats` | Legacy diff utilities | Only `print_diff()` is called from `main.rs`. The unified diff and stats functions are dead. |
| `src/tui/app.rs`: `run_linear()`, `detect_render_mode()`, `run()` | TUI entry points | Only `run_interactive()` is called. The other 3 functions are dead. |
| `src/analyzer/mod.rs:164`: `CATEGORIES` constant | Category list | Never referenced outside the module. The analyzer uses hardcoded match arms instead. |
| `src/analyzer/mod.rs:99`: `XmlBlock.start`, `XmlBlock.end` fields | Position tracking in XML blocks | Extracted but never read. The `extract_xml_blocks()` function returns them but callers only use the cleaned text. |

### Unused Cargo dependencies

| Dependency | Why it's unused |
|------------|----------------|
| `tiktoken-rs` | Imported in `Cargo.toml` but never `use`d. `count_tokens()` uses a hand-rolled heuristic instead. Adds significant compile time (pulls in BPE data). |
| `dotenvy` | Listed in `Cargo.toml` but never imported or called anywhere. Presumably intended for `.env` file loading. |
| `textwrap` | Listed in `Cargo.toml` but never imported. `print_optimized_prompt()` in `renderer.rs` does manual word wrapping instead. |
| `unicode-segmentation` | Listed in `Cargo.toml` but never imported. |
| `directories` | Listed in `Cargo.toml` but never imported. `config.rs` uses `std::env::var("HOME")` directly instead of `directories::ProjectDirs`. |
| `futures` | Listed in `Cargo.toml` but never imported. Tokio is used directly for all async operations. |

### Blanket `#![allow(dead_code)]` suppression

20 source files use `#![allow(dead_code)]` at the module level, which masks all of the above from compiler warnings. This is the root cause of dead code accumulation. Remove these annotations and address warnings individually.

## Quick Wins

1. ✅ **Remove 6 unused Cargo dependencies** — DONE (v0.3.0). Removed `tiktoken-rs`, `dotenvy`, `textwrap`, `unicode-segmentation`, `directories`, `futures`.

2. ✅ **Wire `resolve_model_id()` into `main.rs`** — DONE (v0.3.0). Users can type `copt -m sonnet`.

3. ✅ **Fix STY004 double-replacement bug** — DONE (v0.3.0). Static transforms now apply in fixed priority order.

4. ✅ **Remove blanket `#![allow(dead_code)]`** — DONE (v0.3.0). All blanket annotations removed, targeted allows added where needed.

5. ✅ **Delete `src/rules/mod.rs`** — DONE (v0.3.0). Module and `mod rules;` declaration removed.

6. ✅ **Extract `build_editor_command` to shared utility** — DONE (v0.3.0). Now in `src/utils/editor.rs`.

7. ✅ **Add `--editor` to the help output** — DONE (v0.3.1). `-e, --editor` visible in `--help` and README CLI reference.

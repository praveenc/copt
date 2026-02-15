# Handoff Prompt — COPT Improvement Session

## Context

You are continuing work on **COPT** (Claude Prompt Optimizer), a Rust CLI tool that analyzes and rewrites prompts for Claude 4.5 models. The project is at v0.2.3, functional, and in active use.

A comprehensive code review was completed and documented in `docs/IMPROVEMENTS.md`. **Phase 1 (Dead Code Cleanup) is now complete** on the `feature/dead-code-cleanup` branch. Phases 2-4 remain.

## What Was Done (Phase 1 — Complete)

Branch: `feature/dead-code-cleanup` (4 commits ahead of main)

### Commit 1: Remove 6 unused Cargo dependencies
Removed `tiktoken-rs`, `dotenvy`, `textwrap`, `unicode-segmentation`, `directories`, `futures` from `Cargo.toml`.

### Commit 2: Delete dead modules
- Deleted `src/rules/mod.rs` (entire module, duplicated analyzer)
- Deleted `src/utils/file.rs` (entire module, all functions unused)
- Removed `mod rules;` from `main.rs`, `mod file;` from `utils/mod.rs`
- Kept `cli/config.rs` with targeted `#![allow(dead_code)]` + TODO for Phase 3

### Commit 3: Remove ~30 dead functions across 11 files
- `llm/mod.rs`: Removed CompletionRequest, Message, Role, CompletionResponse, Usage
- `cli/mod.rs`: Removed DEFAULT_MODEL, DEFAULT_MAX_TOKENS, AVAILABLE_MODELS, is_valid_model()
- `utils/text.rs`: Removed 11 functions (kept count_tokens only)
- `tui/mod.rs`: Removed pad_right, center, draw_line, draw_box_*, colors module, truncate
- `tui/renderer.rs`: Removed 9 functions (kept spinner + print_optimized_prompt)
- `tui/stats.rs`: Removed print_stats, print_stats_compact (kept print_save_success)
- `tui/diff.rs`: Removed print_unified_diff, diff_stats, DiffStats (kept print_diff)
- `tui/app.rs`: Removed run_linear, detect_render_mode, run (kept run_interactive)
- Added targeted `#[allow(dead_code)]` + TODO for items kept for Phase 2/3:
  - `cli/mod.rs`: MODEL_ALIASES, resolve_model_id() (Phase 2 wiring)
  - `optimizer/mod.rs`: Enhancement, get_applicable_enhancements() (Phase 3 wiring)
  - `llm/mod.rs`: provider_name() trait method
  - `llm/bedrock.rs`: region() getter

### Commit 4: Strip blanket allow(dead_code) from TUI modules
- Removed `#![allow(dead_code)]` from linear.rs, model.rs, theme.rs, icons.rs, update.rs
- Added targeted `#[allow(dead_code)]` for 8 TUI architecture items

### Results
- ~1,800 lines removed
- Tests: 121 → 89 (32 tests were for dead code)
- `make ci` passes clean (fmt, clippy, build, test)
- App tested in both offline and LLM modes

## Current Project Structure (Post-Cleanup)

```
src/
├── main.rs              # CLI entry point, clap args, orchestration (~800 lines)
├── analyzer/mod.rs      # Rule-based prompt analysis, 25 rules across 8 categories
├── optimizer/mod.rs     # Static transforms (4 rules) + LLM-powered optimization
├── llm/
│   ├── mod.rs           # LlmClient trait, OPTIMIZER_SYSTEM_PROMPT
│   ├── anthropic.rs     # Anthropic API client
│   └── bedrock.rs       # AWS Bedrock client (default provider)
├── cli/
│   ├── mod.rs           # Model aliases, resolve_model_id() (TODO: wire in Phase 2)
│   ├── config.rs        # Full config system (TODO: wire in Phase 3)
│   └── suggest.rs       # Interactive suggestion flow for vague prompts
├── utils/
│   ├── mod.rs           # Re-exports count_tokens only
│   └── text.rs          # count_tokens() only
└── tui/                 # Elm MVU architecture (ratatui)
    ├── app.rs           # run_interactive() only
    ├── model.rs         # State definitions
    ├── update.rs        # Event handling
    ├── view.rs          # Render dispatch
    ├── linear.rs        # Non-interactive enhanced output
    ├── renderer.rs      # Spinner + print_optimized_prompt only
    ├── stats.rs         # print_save_success only
    ├── diff.rs          # print_diff only
    ├── theme.rs         # Theme singleton
    ├── icons.rs         # Nerd Font / Unicode / ASCII icon detection
    ├── terminal.rs      # Terminal init/restore with panic hooks
    └── widgets/         # 11 modular ratatui widgets (all active)
```

## Remaining Work

### Phase 2: Quick Wins (small behavior improvements)
1. Wire `resolve_model_id()` into `main.rs` — one-line change, lets users type `copt -m sonnet`
2. Extract `build_editor_command` to shared utility (duplicated in main.rs and tui/update.rs)
3. Use `LazyLock<Regex>` in optimizer transform functions (4 functions recompile regex every call)
4. Add `Provider::as_str()` method (replace `format!("{:?}", cli.provider).to_lowercase()`)

### Phase 3: Feature Wiring (connect existing code)
1. Wire `cli/config.rs` into `main.rs` (merge config with CLI args, CLI takes precedence)
2. Wire `get_applicable_enhancements()` into optimization pipeline
3. Decide: use tiktoken-rs properly or keep the heuristic (dep was removed in Phase 1)

### Phase 4: New Features
1. Expand static optimization coverage (STY001, FMT002, EXP002)
2. Cache connectivity check results (currently adds 2-5s latency every invocation)
3. Add `--config init` subcommand
4. Fix STY004 double-replacement bug in transform_overtriggering_language()

### Code Quality (can be done alongside any phase)
- `handle_output()` rebuilds Model from scratch instead of reusing
- Token counting called redundantly on same prompt in multiple places
- Static transforms STY002/STY004 have fragile ordering interactions

## Build & Test Commands

```bash
make ci          # Full CI: fmt-check → lint → build → test
make check       # Local dev: fmt → lint → test
cargo test       # Run all tests
cargo run -- -f docs/SAMPLE_PROMPT.txt --offline          # Test offline mode
cargo run -- -f docs/SAMPLE_PROMPT.txt --model <model>    # Test LLM mode
```

## Important Notes

- Git operations require the dev container: `docker exec my-git-workspace git -C /workspace/repos/copt <cmd>`
- Container check: `docker ps --filter name=my-git-workspace --format '{{.Names}}'`
- Project uses `bd` (beads) for issue tracking — run `bd ready` to see open work
- Snapshot tests in `src/tui/snapshots/` contain version strings — update after version bumps
- The `worktrees/` directory is a separate git worktree — ignore it
- `docs/tmp/` is gitignored — don't force-add files there without asking
- `docs/SAMPLE_PROMPT.txt` is a good test prompt for verifying the app works

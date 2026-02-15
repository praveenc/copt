# Handoff Prompt — COPT Improvement Session

## Context

You are continuing work on **COPT** (Claude Prompt Optimizer), a Rust CLI tool that analyzes and rewrites prompts for Claude 4.5 models. The project is at v0.2.3, functional, and in active use.

A comprehensive code review was just completed. Every Rust source file was read and evaluated. The findings are documented in `docs/IMPROVEMENTS.md`. Read that file first — it is the source of truth for this session.

## Project Structure (Key Files)

```bash
src/
├── main.rs              # CLI entry point, clap args, orchestration (~800 lines)
├── analyzer/mod.rs      # Rule-based prompt analysis, 25 rules across 8 categories
├── optimizer/mod.rs     # Static transforms (4 rules) + LLM-powered optimization
├── llm/
│   ├── mod.rs           # LlmClient trait, OPTIMIZER_SYSTEM_PROMPT, unused generic types
│   ├── anthropic.rs     # Anthropic API client
│   └── bedrock.rs       # AWS Bedrock client (default provider)
├── cli/
│   ├── mod.rs           # Model aliases, resolve_model_id() — NEVER CALLED
│   ├── config.rs        # Full config system — NEVER WIRED INTO APP
│   └── suggest.rs       # Interactive suggestion flow for vague prompts
├── rules/mod.rs         # ENTIRE MODULE IS DEAD CODE (duplicates analyzer)
├── utils/
│   ├── mod.rs           # Re-exports count_tokens only
│   ├── text.rs          # 12 functions, only count_tokens() is used
│   └── file.rs          # ENTIRE MODULE IS DEAD CODE
└── tui/                 # Elm MVU architecture (ratatui)
    ├── app.rs           # Event loop (3 of 4 pub functions are dead)
    ├── model.rs         # State definitions
    ├── update.rs        # Event handling
    ├── view.rs          # Render dispatch
    ├── linear.rs        # Non-interactive enhanced output
    ├── renderer.rs      # Legacy — only 3 of 12 functions still used
    ├── stats.rs         # Legacy — only 1 of 3 functions still used
    ├── diff.rs          # Legacy — only 1 of 4 functions still used
    ├── theme.rs         # Theme singleton (keep as-is)
    ├── icons.rs         # Nerd Font / Unicode / ASCII icon detection
    ├── terminal.rs      # Terminal init/restore with panic hooks
    └── widgets/         # 11 modular ratatui widgets (all active)
```

## What Was Found (Summary)

### Dead Code (highest priority to clean up)
- **3 entire modules** with zero external callers: `rules/mod.rs`, `utils/file.rs`, `cli/config.rs`
- **6 unused Cargo dependencies**: `tiktoken-rs`, `dotenvy`, `textwrap`, `unicode-segmentation`, `directories`, `futures`
- **~30 dead functions** across `cli/mod.rs`, `optimizer/mod.rs`, `llm/mod.rs`, `utils/text.rs`, `tui/mod.rs`, `tui/renderer.rs`, `tui/stats.rs`, `tui/diff.rs`, `tui/app.rs`
- **20 files** use blanket `#![allow(dead_code)]` masking all warnings

### Unwired Features (implemented but never connected)
- `cli/config.rs`: Complete config system (load/save/validate) — never called from main
- `cli/mod.rs`: Model alias resolution (`sonnet` → full ARN) — never called
- `optimizer/mod.rs`: `Enhancement` / `get_applicable_enhancements()` — never called
- `docs/RULES.md` documents `--disable-rules` and `--disable-categories` flags that don't exist

### Code Quality Issues
- `build_editor_command()` duplicated in `main.rs` and `tui/update.rs`
- Regex recompilation on every call in all 4 optimizer transform functions
- `format!("{:?}", cli.provider).to_lowercase()` used instead of proper Display impl
- `handle_output()` rebuilds Model from scratch instead of reusing
- Static transforms STY002/STY004 have fragile ordering interactions

### Feature Gaps
- Only 4 of 25 detected rules have static transforms (offline mode underserves)
- Connectivity check adds 2-5s latency on every invocation
- No streaming LLM output
- Token counting uses rough heuristic despite tiktoken-rs being in deps

## Recommended Work Order

### Phase 1: Clean Sweep (no behavior changes)
1. Remove 6 unused Cargo dependencies from `Cargo.toml`
2. Delete `src/rules/mod.rs`, remove `mod rules;` from `main.rs`
3. Remove blanket `#![allow(dead_code)]` from all files
4. Delete confirmed dead functions (see Dead Code tables in IMPROVEMENTS.md)
5. Run `cargo build` and `cargo test` to verify nothing breaks

### Phase 2: Quick Wins (small behavior improvements)
1. Wire `resolve_model_id()` into `main.rs` (one-line change)
2. Extract `build_editor_command` to shared utility
3. Use `LazyLock<Regex>` in optimizer transform functions
4. Add `Provider::as_str()` method

### Phase 3: Feature Wiring (connect existing code)
1. Wire `cli/config.rs` into `main.rs` (merge config with CLI args)
2. Wire `get_applicable_enhancements()` into optimization pipeline
3. Either use tiktoken-rs properly or remove it and keep the heuristic

### Phase 4: New Features
1. Expand static optimization coverage (STY001, FMT002, EXP002)
2. Cache connectivity check results
3. Add `--config init` subcommand

## Build & Test Commands

```bash
make ci          # Full CI: fmt-check → lint → build → test
make check       # Local dev: fmt → lint → test
cargo test       # Run all tests
cargo clippy -- -D warnings  # Lint
```

## Important Notes

- Git operations require the dev container or docker exec (see CLAUDE.md)
- Project uses `bd` (beads) for issue tracking — run `bd ready` to see open work
- Snapshot tests in `src/tui/snapshots/` contain version strings — update after version bumps
- The `worktrees/` directory contains a separate git worktree for a ratatui migration branch — ignore it

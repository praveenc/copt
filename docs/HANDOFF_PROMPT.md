# Handoff Prompt — COPT Improvement Session

## Context

You are continuing work on **COPT** (Claude Prompt Optimizer), a Rust CLI tool that analyzes and rewrites prompts for Claude 4.5 models. The project is at v0.2.3, functional, and in active use.

A comprehensive code review was completed and documented in `docs/IMPROVEMENTS.md`. **Phase 1 (Dead Code Cleanup) and Phase 2 (Quick Wins) are complete** on the `feature/dead-code-cleanup` branch. Phases 3-4 remain.

## What Was Done

Branch: `feature/dead-code-cleanup` (8 commits ahead of main, all pushed)

### Phase 1: Dead Code Cleanup (4 commits)

**Commit 1** (`89a5e26`): Remove 6 unused Cargo dependencies
- Removed `tiktoken-rs`, `dotenvy`, `textwrap`, `unicode-segmentation`, `directories`, `futures`

**Commit 2** (`d063fb3`): Delete dead modules
- Deleted `src/rules/mod.rs` (entire module, duplicated analyzer)
- Deleted `src/utils/file.rs` (entire module, all functions unused)
- Removed `mod rules;` from `main.rs`, `mod file;` from `utils/mod.rs`
- Kept `cli/config.rs` with targeted `#![allow(dead_code)]` + TODO for Phase 3

**Commit 3** (`43f3f97`): Remove ~30 dead functions across 11 files
- `llm/mod.rs`: Removed CompletionRequest, Message, Role, CompletionResponse, Usage
- `cli/mod.rs`: Removed DEFAULT_MODEL, DEFAULT_MAX_TOKENS, AVAILABLE_MODELS, is_valid_model()
- `utils/text.rs`: Removed 11 functions (kept count_tokens only)
- `tui/mod.rs`: Removed pad_right, center, draw_line, draw_box_*, colors module, truncate
- `tui/renderer.rs`: Removed 9 functions (kept spinner + print_optimized_prompt)
- `tui/stats.rs`: Removed print_stats, print_stats_compact (kept print_save_success)
- `tui/diff.rs`: Removed print_unified_diff, diff_stats, DiffStats (kept print_diff)
- `tui/app.rs`: Removed run_linear, detect_render_mode, run (kept run_interactive)
- Added targeted `#[allow(dead_code)]` + TODO for items kept for Phase 2/3 wiring

**Commit 4** (`da74e86`): Strip blanket allow(dead_code) from TUI modules
- Removed `#![allow(dead_code)]` from linear.rs, model.rs, theme.rs, icons.rs, update.rs
- Added targeted `#[allow(dead_code)]` for 8 TUI architecture items

**Phase 1 Results**: ~1,800 lines removed, tests 121→89, zero compiler warnings

### Phase 2: Quick Wins (1 commit)

**Commit 5** (`400e1c1`): All 4 quick wins in one commit
- Wired `resolve_model_id()` into `main.rs` — users can now type `-m sonnet` instead of full ARN
- Extracted `build_editor_command` to `src/utils/editor.rs` with `wait: bool` parameter
- Converted all 4 optimizer transform functions to `LazyLock<Regex>` (compile-once)
- Added `Provider::as_str()` method, replaced 3 occurrences of `format!("{:?}", cli.provider).to_lowercase()`

### Housekeeping commits
- **Commit 6** (`06cd276`): Updated handoff prompt after Phase 1
- **Commit 7** (`fbb3b24`): Added `ci-quiet` Makefile target

## Current Project Structure (Post Phase 1+2)

```
src/
├── main.rs              # CLI entry point, clap args, orchestration (~800 lines)
├── analyzer/mod.rs      # Rule-based prompt analysis, 25 rules across 8 categories
├── optimizer/mod.rs     # Static transforms (4 rules, LazyLock regex) + LLM optimization
├── llm/
│   ├── mod.rs           # LlmClient trait, OPTIMIZER_SYSTEM_PROMPT
│   ├── anthropic.rs     # Anthropic API client
│   └── bedrock.rs       # AWS Bedrock client (default provider)
├── cli/
│   ├── mod.rs           # Model aliases + resolve_model_id() (NOW WIRED)
│   ├── config.rs        # Full config system (TODO: wire in Phase 3)
│   └── suggest.rs       # Interactive suggestion flow for vague prompts
├── utils/
│   ├── mod.rs           # Re-exports count_tokens + editor
│   ├── text.rs          # count_tokens() only
│   └── editor.rs        # build_editor_command() shared utility (NEW in Phase 2)
└── tui/                 # Elm MVU architecture (ratatui)
    ├── app.rs           # run_interactive() only
    ├── model.rs         # State definitions
    ├── update.rs        # Event handling (uses shared editor utility)
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

### Phase 3: Feature Wiring (connect existing code)
1. Wire `cli/config.rs` into `main.rs` (merge config with CLI args, CLI takes precedence)
   - `config.rs` has a complete load/save/validate system that's never called
   - Remove the blanket `#![allow(dead_code)]` from config.rs once wired
2. Wire `get_applicable_enhancements()` into optimization pipeline
   - `optimizer/mod.rs` has `Enhancement` struct and `get_applicable_enhancements()` that maps analysis findings to specific improvements
   - Currently the static optimizer only runs 4 hardcoded transforms; this would make it data-driven
3. Decide: re-add tiktoken-rs or keep heuristic token counting
   - tiktoken-rs was removed in Phase 1 as unused dep, but `count_tokens()` uses a rough `chars/4` heuristic
   - If accuracy matters, re-add tiktoken-rs and wire it properly

### Phase 4: New Features
1. Expand static optimization coverage (STY001, FMT002, EXP002)
   - Only 4 of 25 detected rules have static transforms — offline mode underserves
2. Cache connectivity check results
   - Currently adds 2-5s latency on every invocation checking if API is reachable
3. Add `--config init` subcommand (depends on Phase 3 config wiring)
4. Fix STY004 double-replacement bug in `transform_overtriggering_language()`
   - Static transforms STY002/STY004 have fragile ordering interactions

### Code Quality (can be done alongside any phase)
- `handle_output()` in main.rs rebuilds Model from scratch instead of reusing
- Token counting called redundantly on same prompt in multiple places
- Consider streaming LLM output for better UX

## Build & Test Commands

```bash
# PREFERRED for agent use (concise output, saves context window):
make ci-quiet    # 4 lines of output: fmt-check, clippy, build, test result

# Full output versions:
make ci          # Full CI: fmt-check → lint → build → test
make check       # Local dev: auto-fix formatting, then lint and test

# Individual:
cargo test                              # Run all tests (89 currently)
cargo clippy -- -D warnings             # Lint
cargo run -- -f docs/SAMPLE_PROMPT.txt --offline           # Test offline mode
cargo run -- -f docs/SAMPLE_PROMPT.txt -m sonnet           # Test with model alias (Phase 2 feature)
```

## Critical Rules for Agents

### Git Operations
- All git ops via container: `docker exec my-git-workspace git -C /workspace/repos/copt <cmd>`
- Container check: `docker ps --filter name=my-git-workspace --format '{{.Names}}'`
- Use single quotes for commit messages in zsh (avoids bracket expansion)
- Atomic commits — one logical change per commit
- Always test before committing

### Files to Watch
- `docs/SAMPLE_PROMPT.txt` — test prompt, gets staged by `git add -A` — always unstage it
- `CONTEXT_MANAGEMENT.md` — untracked, don't stage without asking
- `docs/tmp/` — gitignored, never force-add files there
- `docs/tmp/feature-dead-code-cleanup/progress.txt` — update as work progresses (local only)

### Testing Protocol
- Always run `make ci-quiet` before committing
- Test the app with `cargo run -- -f docs/SAMPLE_PROMPT.txt --offline` after code changes
- For LLM mode: `cargo run -- -f docs/SAMPLE_PROMPT.txt -m sonnet`

### Context Management
- Use `make ci-quiet` instead of `make ci` to save context window
- Use subagents for parallel tasks and to extend context limits
- The Kiro IDE supports subagents: context-gatherer (explore codebases) and general-task-execution (parallel work)
- Delegate independent tasks to subagents to keep main context clean

## Key Files to Read First
1. This file (`docs/HANDOFF_PROMPT.md`) — you're reading it
2. `docs/IMPROVEMENTS.md` — detailed analysis with specific recommendations
3. `CLAUDE.md` — build commands, architecture, git workflow
4. `Makefile` — build targets including ci-quiet
5. `src/main.rs` — CLI entry point with Phase 2 changes
6. `src/optimizer/mod.rs` — LazyLock regex transforms, Enhancement struct for Phase 3
7. `src/cli/config.rs` — unwired config system for Phase 3

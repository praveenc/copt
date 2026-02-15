# Handoff Prompt — COPT

## Context

**COPT** (Claude Prompt Optimizer) is a Rust CLI tool that analyzes and rewrites prompts for Claude 4.5 models. The project is at v0.3.1, functional, and in active use.

## Current State

Branch: `feature/smart-prompt-storage` (based on `main` at v0.3.0)

### v0.3.0 (merged to main)

All improvements from `docs/IMPROVEMENTS.md` Phases 1-4 are complete:

- Dead code cleanup: removed 6 unused Cargo deps, 2 dead modules, ~30 dead functions, blanket `#![allow(dead_code)]`
- Wired config file (`~/.config/copt/config.toml`), model aliases (`-m sonnet`), `--config-init`
- Enhancement pipeline, connectivity caching (5-min TTL), STY004/STY002 ordering fix
- 2 new static transforms (STY001, FMT002), `LazyLock<Regex>`, shared editor utility, `Provider::as_str()`
- Net: -954 lines, tests 121→91

### v0.3.1 (current branch)

- Smart prompt naming: `generate_prompt_slug()` in `src/utils/text.rs` — descriptive filenames from prompt content
- Centralized storage: default save to `~/.copt/prompts/YYYY-MM-DD/<slug>_<HHMMSS>_optimized.txt`
- `name` field added to JSON output and metadata
- TUI save handler updated to use new paths/naming
- Raycast script command (`scripts/raycast/optimize-prompt.sh`) — clipboard optimization via Bedrock
- Tests: 97 passing

## Project Structure

```text
src/
├── main.rs              # CLI entry, clap args, orchestration
├── analyzer/mod.rs      # 25 analysis rules across 8 categories
├── optimizer/mod.rs     # 6 static transforms (LazyLock regex) + enhancements + LLM optimization
├── llm/
│   ├── mod.rs           # LlmClient trait, OPTIMIZER_SYSTEM_PROMPT
│   ├── anthropic.rs     # Anthropic API client
│   └── bedrock.rs       # AWS Bedrock client (default)
├── cli/
│   ├── mod.rs           # Model aliases + resolve_model_id()
│   ├── config.rs        # Config file system (~/.config/copt/config.toml)
│   └── suggest.rs       # Interactive suggestion flow
├── utils/
│   ├── mod.rs           # Re-exports
│   ├── text.rs          # count_tokens(), generate_prompt_slug()
│   └── editor.rs        # build_editor_command() shared utility
├── tui/                 # Elm MVU architecture (ratatui)
│   ├── app.rs           # run_interactive()
│   ├── model.rs, update.rs, view.rs, linear.rs
│   ├── renderer.rs, stats.rs, diff.rs
│   ├── theme.rs, icons.rs, terminal.rs
│   └── widgets/         # 11 modular ratatui widgets
└── scripts/raycast/     # Raycast script command
```

## Remaining Work

- **Bedrock API Key auth** (v0.3.2, in progress) — Replace AWS SDK SigV4 with Bearer token auth using Bedrock API keys. Drop `aws-config`, `aws-sdk-bedrockruntime`, `aws-credential-types` deps. Use `reqwest` HTTP calls with `Authorization: Bearer <key>`. Fall back to SigV4 when no API key present.
- Streaming LLM output (Nice to Have — High effort)
- Wire `is_rule_enabled()` / `get_severity_override()` from config into analyzer
- `handle_output()` rebuilds Model from scratch (minor refactor)
- Token counting called redundantly (minor optimization)

## Build & Test

```bash
make ci-quiet                                    # Preferred: 4-line CI output
cargo run -- -f test_prompt.txt --offline         # Smoke test
cargo run -- -f test_prompt.txt -m sonnet         # LLM test
```

## Agent Rules

- Git via container: `docker exec my-git-workspace git -C /workspace/repos/copt <cmd>`
- Single quotes for commit messages in zsh
- Atomic commits, always `make ci-quiet` before committing
- `docs/SAMPLE_PROMPT.txt` gets staged by `git add -A` — always unstage
- `docs/tmp/` is gitignored — never force-add

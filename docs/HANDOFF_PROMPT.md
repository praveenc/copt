# Handoff Prompt — COPT

## Context

**COPT** (Claude Prompt Optimizer) is a Rust CLI tool that analyzes and rewrites prompts for Claude 4.5 models. The project is at v0.3.2, functional, and in active use.

## Current State

Branch: `feature/bedrock-api-keys` (based on `main` at v0.3.1)

### v0.3.2 (current branch, not yet merged)

- **Bedrock API Key auth**: `BedrockApiKeyClient` in `src/llm/bedrock.rs` — uses `reqwest` + Converse API with `Authorization: Bearer <key>` header
- Key resolution order: `--bedrock-api-key` CLI flag → `AWS_BEARER_TOKEN_BEDROCK` env var → `api_key` in `~/.config/copt/config.toml` → SigV4 fallback
- Connectivity check shows auth method ("API key" vs "AWS credentials")
- `--config-init` generates hand-crafted template with comments and API key guidance
- Error messages suggest API keys as simplest fix for credential issues
- AWS SDK deps kept for backward-compatible SigV4 fallback
- Shared `get_bedrock_model_id()` function (extracted from `BedrockClient` method)
- Tests: 101 passing (4 new: client creation, provider name, request/response serialization)

### v0.3.1 (merged to main)

- Smart prompt naming: `generate_prompt_slug()` in `src/utils/text.rs`
- Centralized storage: `~/.copt/prompts/YYYY-MM-DD/<slug>_<HHMMSS>_optimized.txt`
- Raycast script command (`scripts/raycast/optimize-prompt.sh`)
- `name` field in JSON output and metadata

### v0.3.0 (merged to main)

- Dead code cleanup, config file, model aliases, `--config-init`
- Enhancement pipeline, connectivity caching, STY004/STY002 fix
- 2 new static transforms (STY001, FMT002), `LazyLock<Regex>`

## Project Structure

```text
src/
├── main.rs              # CLI entry, clap args, orchestration
├── analyzer/mod.rs      # 25 analysis rules across 8 categories
├── optimizer/mod.rs     # 6 static transforms + enhancements + LLM optimization
├── llm/
│   ├── mod.rs           # LlmClient trait, OPTIMIZER_SYSTEM_PROMPT
│   ├── anthropic.rs     # Anthropic API client (reqwest + x-api-key header)
│   └── bedrock.rs       # BedrockClient (SigV4/AWS SDK) + BedrockApiKeyClient (Bearer token/Converse API)
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

- Version bump to v0.3.2 + CHANGELOG entry (do before merging)
- Streaming LLM output (Nice to Have — High effort)
- Wire `is_rule_enabled()` / `get_severity_override()` from config into analyzer
- `handle_output()` rebuilds Model from scratch (minor refactor)
- Token counting called redundantly (minor optimization)
- Consider gating AWS SDK deps behind a cargo feature flag (future)

## Build & Test

```bash
make ci-quiet                                    # Preferred: 4-line CI output
cargo run -- -f test_prompt.txt --offline         # Smoke test
cargo run -- -f test_prompt.txt -m sonnet         # LLM test (needs API key or AWS creds)
```

## Agent Rules

- Git via container: `docker exec my-git-workspace git -C /workspace/repos/copt <cmd>`
- Single quotes for commit messages in zsh
- Atomic commits, always `make ci-quiet` before committing
- `docs/tmp/` is gitignored — never force-add
- Always run `cargo fmt` before committing

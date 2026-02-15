---
inclusion: auto
description: Rust idioms and patterns specific to this copt project for .rs file edits
---

# Copt Rust Patterns

## Error Handling

- Application errors: `anyhow::Result<T>` with `?` operator
- Library/domain errors: `thiserror::Error` derive macro
- Never `.unwrap()` in production paths — use `?` or `.unwrap_or_default()`

```rust
// Application function
pub async fn optimize(prompt: &str) -> Result<String> { ... }

// Domain error type
#[derive(Debug, thiserror::Error)]
pub enum AnalyzerError {
    #[error("invalid rule ID: {0}")]
    InvalidRule(String),
}
```

## Async Patterns

- Runtime: `tokio` (full features)
- Trait async methods: `#[async_trait]`
- The `LlmClient` trait is the core async interface

```rust
#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, system: &str, user_message: &str, model: &str, max_tokens: u32) -> Result<String>;
    fn provider_name(&self) -> &str;
}
```

## Module Structure

- `mod.rs` = module entry (declares sub-modules, re-exports public types)
- Use `pub use` re-exports for public API: `pub use suggest_modal::SuggestModalState;`
- Always reference types via their re-exported path, not private module paths
- Tests go in `#[cfg(test)] mod tests` at bottom of file

## Formatting Rules (cargo fmt)

- Break long `format!()` calls across multiple lines when >1 argument
- Imports must be alphabetically ordered (especially in test functions)
- Comments on their own line above code, not inline after statements
- After refactoring, scan for unused imports (clippy -D warnings catches these)

## CLI Patterns (clap)

- `#[derive(Parser)]` on the `Cli` struct in `src/main.rs`
- `#[derive(ValueEnum)]` for enum CLI args (e.g., `Provider`)
- `#[arg(short, long)]` for flag definitions
- Features in `Cargo.toml` gate optional providers: `anthropic`, `bedrock`, `offline`

## Testing Patterns

- Unit tests: `#[cfg(test)] mod tests` inline
- Integration tests: `assert_cmd` + `predicates` for CLI testing
- Snapshot tests: `insta` crate — update with `cargo insta test --accept`
- Mock HTTP: `wiremock` for LLM client tests
- Test helpers: `create_test_issues()` pattern for shared test fixtures

## Version Bumping

When version changes in `Cargo.toml`:
1. Update `Cargo.toml` version field
2. Run `cargo insta test --accept` to update TUI snapshots (they contain the version in the header)
3. Update `CHANGELOG.md`
4. Grep for old version: `grep -r "vOLD" --include="*.md" --include="*.snap"`

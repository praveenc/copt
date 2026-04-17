# Rust Code Quality Learnings

Real-world learnings from code generation that required fixes to pass `cargo fmt`, `cargo clippy`, and `cargo test`.

This document serves as a reference to minimize second-pass fixes.

---

## Formatting (`cargo fmt`)

### 1. Long `format!()` Macro Calls

**Problem:** Long format strings on a single line fail `cargo fmt --check`.

```rust
// ❌ WRONG - too long
model.set_status_message(
    format!("✓ Saved to {} and opened in {}", output_path.display(), editor_cmd),
    Duration::from_secs(5),
);
```

```rust
// ✅ CORRECT - break across lines
model.set_status_message(
    format!(
        "✓ Saved to {} and opened in {}",
        output_path.display(),
        editor_cmd
    ),
    Duration::from_secs(5),
);
```

**Rule:** If a `format!()` call has multiple arguments, break it across lines proactively.

---

### 2. Import Ordering in Test Functions

**Problem:** Imports inside test functions must be alphabetically ordered.

```rust
// ❌ WRONG - not alphabetical
#[test]
fn test_something() {
    use crate::Issue;
    use crate::analyzer::Severity;
    use crate::tui::widgets::SuggestModalState;
}
```

```rust
// ✅ CORRECT - alphabetical order
#[test]
fn test_something() {
    use crate::analyzer::Severity;
    use crate::tui::widgets::SuggestModalState;
    use crate::Issue;
}
```

**Rule:** `cargo fmt` sorts imports alphabetically. When adding imports, insert them in alphabetical order by full path.

---

### 3. Inline Comments After Assertions

**Problem:** Comments on the same line as assertions can cause weird alignment.

```rust
// ❌ PROBLEMATIC - fmt may align oddly
assert!(!model.suggest_modal.visible); // Modal should be dismissed
// Prompt should be enhanced
assert!(model.original_prompt.len() > original_len);
```

The formatter may produce:
```rust
assert!(!model.suggest_modal.visible); // Modal should be dismissed
                                       // Prompt should be enhanced  <- weird indent
```

**Rule:** Put explanatory comments on their own line above the assertion, not inline after it.

```rust
// ✅ BETTER
// Modal should be dismissed
assert!(!model.suggest_modal.visible);
// Prompt should be enhanced
assert!(model.original_prompt.len() > original_len);
```

---

## Linting (`cargo clippy`)

### 4. Unused Imports After Refactoring

**Problem:** When removing or refactoring code, imports that were used by that code become unused.

```rust
// ❌ WRONG - Write is no longer used after refactoring
use std::io::Write;

fn handle_save(model: &mut Model) -> bool {
    // Refactored to use std::fs::write() instead of file.write_all()
    std::fs::write(&output_path, optimized)?;
}
```

**Rule:** After refactoring, scan for imports that may no longer be needed. Clippy with `-D warnings` will catch these.

---

## Test Compilation

### 5. Private vs Re-exported Module Paths

**Problem:** Using the full path to a type in a private module fails, even in tests.

```rust
// ❌ WRONG - suggest_modal module is private
use crate::tui::widgets::suggest_modal::SuggestModalState;
```

```rust
// ✅ CORRECT - use the re-exported path
use crate::tui::widgets::SuggestModalState;
```

**Rule:** Check `mod.rs` for `pub use` re-exports. Use the shortest public path to a type, not the full internal module path.

**How to check:** Look at the module's `mod.rs`:
```rust
// In widgets/mod.rs
mod suggest_modal;  // Private module
pub use suggest_modal::SuggestModalState;  // Public re-export
```

---

## Pre-Commit Checklist

Before considering code complete:

1. [ ] Long `format!()` calls broken across multiple lines
2. [ ] Imports in alphabetical order (especially in test functions)
3. [ ] No unused imports from refactored code
4. [ ] Using re-exported paths, not private module paths
5. [ ] Comments on their own lines, not inline after complex statements

---

## Release Process

### 6. GitHub Release Titles

**Problem:** Adding descriptive suffixes to release titles makes them inconsistent.

```bash
# ❌ WRONG - extra description in title
gh release create v0.2.3 --title "v0.2.3 - TUI Bug Fixes"
```

```bash
# ✅ CORRECT - version only
gh release create v0.2.3 --title "v0.2.3"
```

**Rule:** Release titles should be the version number only. Put descriptions in the release notes body.

---

### 7. Version References in Documentation

**Problem:** Version numbers get hardcoded in multiple places and become stale.

Places to check when bumping versions:
- `Cargo.toml` — The source of truth
- `src/tui/snapshots/*.snap` — TUI snapshot tests contain version in header
- `README.md` — Example outputs, curl download URLs
- `CHANGELOG.md` — Release notes

**Rule:** After bumping `Cargo.toml`, grep for the old version:
```bash
grep -r "v0\.2\.2" --include="*.md" --include="*.snap"
```

---

### 8. README Conciseness

**Problem:** READMEs that try to document everything become overwhelming.

**Principles:**
- README goal: Get user from zero to running in <2 minutes
- Move detailed content to `docs/` and link to it
- Use collapsible `<details>` for reference material (e.g., `--help` output)
- Avoid large code output blocks — they dominate the page

**Structure:**
1. One-line description
2. Installation (brief)
3. Quick start (3-4 commands max)
4. Core features (brief)
5. Links to detailed docs

---

## Pre-Release Checklist

Before tagging a release:

1. [ ] Version bumped in `Cargo.toml`
2. [ ] Snapshot tests updated (`cargo insta test --accept` or manual sed)
3. [ ] `CHANGELOG.md` updated with release date
4. [ ] Version references in `README.md` updated
5. [ ] `make ci` passes
6. [ ] Release title is just the version (e.g., "v0.2.3")

---

### 9. Back-Compat Re-Exports and Dead-Code Warnings

**Problem:** When introducing a new name for an existing public constant (e.g., splitting `OPTIMIZER_SYSTEM_PROMPT` into `OPTIMIZER_SYSTEM_PROMPT_4_5` and keeping `OPTIMIZER_SYSTEM_PROMPT` as a re-export), `cargo build` emits `constant ... is never used` because the internal codebase has already been migrated to the new name.

```rust
// ❌ WARNING - dead code even though this is public
pub const OPTIMIZER_SYSTEM_PROMPT: &str = OPTIMIZER_SYSTEM_PROMPT_4_5;
```

```rust
// ✅ CORRECT - explicit allow for back-compat re-exports
#[allow(dead_code)]
pub const OPTIMIZER_SYSTEM_PROMPT: &str = OPTIMIZER_SYSTEM_PROMPT_4_5;
```

**Rule:** `pub` alone does not silence `dead_code` for binary crates. For library-style back-compat re-exports inside a binary crate, add `#[allow(dead_code)]` with a comment explaining the re-export is intentional.

---

### 10. Use project `make` targets, not raw `cargo` commands

**Problem:** The project has a `Makefile` with curated build/test/lint targets (`make ci`, `make ci-quiet`, `make ci-release`, `make release`, `make check`). Bypassing them with raw `cargo build --release` or `cargo test` skips project conventions (e.g., the `-D warnings` clippy flag, the coordinated fmt→lint→build→test order) and produces inconsistent CI/local behaviour.

```bash
# ❌ WRONG - bypasses project conventions, may pass locally and fail CI
cargo build --release
cargo test
```

```bash
# ✅ CORRECT - use project targets
make release        # release build
make ci-quiet       # concise CI gate (agent-friendly)
make ci             # full debug CI gate
make ci-release     # full release CI gate
make check          # auto-fix formatting, then lint and test
```

**Rule:** Before running ANY `cargo` command, check `Makefile` for a target that covers the same intent. Raw `cargo` is only appropriate for one-off things the Makefile doesn't cover (e.g., `cargo run -- <specific args>` during live testing, or `cargo insta test --accept` for snapshot updates).

---

### 11. `String::replace` silently no-ops when the needle is absent

**Problem:** Deriving a sibling filename from a "primary" path with `replace` is only safe if the sentinel substring is guaranteed to be present. When it's absent, `replace` returns the input unchanged — which can produce a silent filename collision that then gets overwritten by a later write.

This was the root cause of a `-o <path>` bug in `src/main.rs` where explicit user-supplied paths (e.g. `-o /tmp/foo.txt`) clobbered the optimized output with the original prompt:

```rust
// ❌ WRONG - returns the same path unchanged when the sentinel isn't there
let original_path = {
    let filename = path.file_name().unwrap().to_string_lossy();
    // When `filename` is "foo.txt" (no "_optimized.txt" suffix),
    // `original_filename` is STILL "foo.txt" — same path as `path`.
    let original_filename = filename.replace("_optimized.txt", "_original.txt");
    path.with_file_name(original_filename)
};

// Then:
tokio::fs::write(path, &result.optimized).await?;        // writes optimized
tokio::fs::write(&original_path, &result.original).await?; // OVERWRITES same file with original
```

```rust
// ✅ CORRECT - check the sentinel is actually there, otherwise derive a
// different sibling name
let original_path = {
    let filename = path.file_name().unwrap().to_string_lossy();
    if filename.contains("_optimized.txt") {
        let original_filename = filename.replace("_optimized.txt", "_original.txt");
        path.with_file_name(original_filename)
    } else {
        // Fallback: `<stem>.original.<ext>` so we never collide with `path`.
        let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let ext  = path.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
        let fallback = if ext.is_empty() {
            format!("{stem}.original")
        } else {
            format!("{stem}.original.{ext}")
        };
        path.with_file_name(fallback)
    }
};
```

**Rule:** When deriving a sibling filename via `replace`, either (a) assert the needle is present first, or (b) use an explicit conditional with a fallback that is provably different from the input. Any two consecutive writes in a pipeline must target demonstrably different paths.

---

### 12. Don't guess Bedrock inference profile IDs; list them

**Problem:** Bedrock inference profile IDs don't follow a single naming convention. Some end in `-v1:0`, some in `-v1`, some have no version suffix at all:

| Model | Actual profile ID |
|-------|-------------------|
| Opus 4.5 | `global.anthropic.claude-opus-4-5-20251101-v1:0` |
| Opus 4.6 | `global.anthropic.claude-opus-4-6-v1` |
| Sonnet 4.6 | `global.anthropic.claude-sonnet-4-6` |
| Opus 4.7 | `global.anthropic.claude-opus-4-7` |

Guessing `global.anthropic.claude-opus-4-7-v1:0` produces a convincing-looking but wrong string that returns `HTTP 400 {"message":"The provided model identifier is invalid."}` at runtime — not caught by tests, only by a live call.

```rust
// ❌ WRONG - plausible-looking extrapolation
"opus-4.7" => "global.anthropic.claude-opus-4-7-v1:0".to_string(),
```

```rust
// ✅ CORRECT - value confirmed via `aws bedrock list-inference-profiles`
"opus-4.7" => "global.anthropic.claude-opus-4-7".to_string(),
```

**Rule:** Before hard-coding a Bedrock inference profile ID, run:

```bash
aws bedrock list-inference-profiles --region us-west-2 2>&1 \
  | grep '"inferenceProfileId"' | sort -u
```

Copy the exact string. Do NOT extrapolate a pattern from neighbouring models.

---

### 13. Family-aware LLM request parameters

**Problem:** Anthropic's API contract for sampling parameters drifts across model families. Claude 4.5 and 4.6 accept a non-default `temperature`. Claude Opus 4.7 returns HTTP 400 on any non-default `temperature`, `top_p`, or `top_k`:

```json
{"error":{"type":"invalid_request_error","message":"`temperature` is deprecated for this model."}}
```

A single hard-coded `temperature: Some(0.3)` works for older models but breaks on 4.7.

```rust
// ❌ WRONG - hard-coded temperature works for 4.5/4.6 but HTTP 400 on 4.7
let request = BedrockRequest {
    temperature: Some(0.3),
    // ...
};
```

```rust
// ✅ CORRECT - derive from model family; drop the field entirely for 4.7
let model_id = get_bedrock_model_id(model);
let family = crate::llm::ModelFamily::from_model_id(&model_id);
let temperature = if family == crate::llm::ModelFamily::Claude47 {
    None
} else {
    Some(0.3)
};
let request = BedrockRequest {
    temperature,
    // ...
};
```

Combined with `#[serde(skip_serializing_if = "Option::is_none")]` on the field, `None` cleanly drops the key from the JSON body — exactly what the 4.7 contract requires.

**Rule:** Any request parameter whose acceptance varies across model families should be sourced from a `ModelFamily`-aware helper, not hard-coded. Classify the model at the top of the `complete()` call and let the family decide. Use `Option<T>` + `skip_serializing_if = "Option::is_none"` so dropping a field is a one-line change.

---

*Last updated: 2026-04-16*
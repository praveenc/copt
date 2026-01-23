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

*Last updated: 2026-01-23*
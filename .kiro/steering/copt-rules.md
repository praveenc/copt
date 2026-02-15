---
inclusion: auto
description: Analysis rule taxonomy, naming conventions, and add-rule checklist for copt analyzer
---

# Copt Analysis Rule Reference

## Rule Naming Convention

```
{CAT}{NNN}  e.g., EXP001, STY003
  |    |
  |    +-- Sequential number (001, 002, ...)
  +------- 3-letter category prefix
```

## Categories (27 rules total)

| Prefix | Category | Count | Severity Mix |
|--------|----------|-------|--------------|
| EXP | Explicitness — clear, specific instructions | 6 | 3 warning, 3 info |
| STY | Style — tone, framing, emphasis | 4 | 2 warning, 2 info |
| TUL | Tool Usage — action directives, parallel ops | 3 | 1 warning, 2 info |
| FMT | Formatting — output format, XML structure | 3 | 1 warning, 2 info |
| VRB | Verbosity — response length, progress | 2 | 2 info |
| AGT | Agentic Coding — exploration, hallucination | 4 | 2 warning, 2 info |
| LHT | Long-Horizon — state persistence, incremental | 3 | 1 warning, 2 info |
| FED | Frontend Design — UI aesthetics | 2 | 2 info |

## Severity Levels

| Level | When to Use |
|-------|-------------|
| Error | Critical issue likely to cause poor results (currently 0 rules) |
| Warning | Issue that may degrade output quality |
| Info | Suggestion for improvement |

## Issue Struct

```rust
Issue {
    id: String,          // e.g., "EXP001"
    category: String,    // e.g., "explicitness" (lowercase)
    severity: Severity,  // Error | Warning | Info
    message: String,     // Human-readable description
    line: Option<usize>, // Line number in prompt (if applicable)
    suggestion: Option<String>, // Recommended fix
}
```

## Adding a New Rule — Checklist

1. Choose category prefix + next sequential number (e.g., `EXP007`)
2. Add detection logic in `src/analyzer/mod.rs` in the `analyze_{category}()` function
3. Create `Issue` with all fields populated
4. If the rule has a static transformation, add it in `src/optimizer/mod.rs` → `apply_static_transformation()`
5. Add tests in `src/analyzer/mod.rs` → `#[cfg(test)]` module
6. Document in `docs/RULES.md` following the existing pattern (before/after examples)

## Key Implementation Details

- **XML-Aware Parsing**: Analyzer extracts `<examples>`, `<example>`, `<instructions>` blocks before analysis to prevent false positives inside XML content
- **Prompt Type Classifier**: Detects prompt type (Coding, QaAssistant, Research, Creative, LongHorizon, General) for context-aware rule application
- **Category names in code are lowercase**: `"explicitness"`, `"style"`, `"tools"`, `"formatting"`, `"verbosity"`, `"agentic"`, `"long_horizon"`, `"frontend"`

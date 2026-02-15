---
inclusion: auto
description: TUI module architecture, MVU pattern, ratatui widget conventions for src/tui/
---

# TUI Architecture — Elm MVU Pattern

The TUI uses the **Elm Model-View-Update** architecture with `ratatui` (v0.28) and `crossterm` (v0.28).

## Strict Boundaries

| Layer | File(s) | Responsibility | Rules |
|-------|---------|----------------|-------|
| **Model** | `model.rs` | ALL application state | No rendering. No side effects. No I/O. |
| **Update** | `update.rs` | Event handling, state mutation | Receives events, mutates Model. No rendering. |
| **View** | `view.rs` | Render dispatch | Read-only access to Model. Delegates to widgets. |
| **Widgets** | `widgets/*.rs` | Pure render functions | Take `&Model` or subset, draw to `Frame`. No mutation. |

## State Types (model.rs)

- `Model` — Root state struct (all fields `pub`)
- `View` — Current view enum: `Main | Diff | Help`
- `RenderMode` — `Interactive | Linear | Plain | Json | Quiet`
- `AppPhase` — `Ready | Analyzing | AnalysisDone | Optimizing | Done | Error`
- `IssueTree` — Collapsible tree with `CategoryNode` items
- `SuggestModalState` — Modal dialog state for vague prompt suggestions
- `ErrorState` — Error message + optional details

## Widget Files (widgets/)

| File | Widget | Renders |
|------|--------|---------|
| `header.rs` | ASCII art banner | Top of screen |
| `analysis.rs` | Collapsible issue tree | Analysis results |
| `progress.rs` | Optimization gauge | Progress bar during LLM call |
| `dashboard.rs` | Stats with bar charts | Token counts, severity breakdown |
| `diff.rs` | Side-by-side comparison | Original vs optimized prompt |
| `status_bar.rs` | Keyboard hints | Bottom bar with available keys |
| `help.rs` | Full keyboard shortcuts | Help overlay |
| `error_modal.rs` | Modal error dialog | Error display overlay |
| `minimal.rs` | Small terminal fallback | When terminal < 60 cols |
| `suggest_modal.rs` | Multi-select dialog | Vague prompt improvement options |

## Adding a New Widget

1. Create `src/tui/widgets/new_widget.rs`
2. Implement a render function: `pub fn render_new_widget(f: &mut Frame, area: Rect, model: &Model)`
3. Export in `src/tui/widgets/mod.rs`
4. Call from `view.rs` in the appropriate view branch

## Conventions

- Widgets are **pure functions** — same Model always produces same output
- Use `ratatui::layout::{Layout, Constraint, Direction}` for layout
- Use `crate::tui::theme` for consistent colors (supports both dark and light terminals)
- Use `crate::tui::icons` for Nerd Font icons with Unicode/ASCII fallback
- Event loop lives in `app.rs` (interactive mode) and `linear.rs` (default scrolling output)

## Snapshot Testing

- Tests in `src/tui/snapshot_tests.rs`
- Snapshots stored in `src/tui/snapshots/*.snap`
- Update with: `cargo insta test --accept`
- Snapshots contain the version string — update after version bumps

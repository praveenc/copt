# Changelog

All notable changes to `copt` (Claude Prompt Optimizer) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Fixed

---

## [0.2.1] - 2025-01-18

### Added

- **Save original prompt for comparison**: When optimizing, both original and optimized prompts are now saved
    - `original_{timestamp}.txt` - The original prompt text
    - `optimized_{timestamp}.txt` - The optimized prompt text
    - Metadata JSON now includes `files` object with paths to both files
- **Makefile for development workflow**:
    - `make ci` - CI pipeline: fmt-check → lint → build → test (strict, no auto-fix)
    - `make check` - Local dev: fmt → lint → test (auto-fixes formatting)
    - `make build`, `make release`, `make test`, `make lint`, `make fmt`, `make clean`
- **Hybrid Analyzer Enhancement** (Phases 1-6):
    - **XML-Aware Parsing**: Analyzer now extracts and preserves XML blocks (`<examples>`, `<example>`, `<instructions>`, etc.) before analysis, preventing false positives from example content
    - **Prompt Type Classifier**: New `PromptType` enum (Coding, QaAssistant, Research, Creative, LongHorizon, General) with context-aware rule application - LHT rules no longer fire on simple Q&A prompts
    - **EXP005 — Role-Only Prompt**: Detects prompts that define a role ("You are...") without specific action directives
    - **EXP006 — Open-Ended Instructions**: Detects overly open-ended instructions ("answer any questions") without boundaries or format specs
    - **Auto-Suggest for Vague Prompts**: Interactive suggestions now trigger automatically when running in a TTY and EXP005/EXP006 detected (no `--suggest` flag required)
    - **TUI Suggest Modal** (Phase 5): Full-screen interactive mode (`-i`) now shows a modal dialog for vague prompts with checkbox selection, keyboard navigation (↑/↓/Space/Enter/Esc), and real-time selection count
    - **LLM Prompt Type Awareness** (Phase 6): `OPTIMIZER_SYSTEM_PROMPT` now includes prompt-type-specific optimization guidance (Q&A → response format/citations, Coding → exploration directives, Research → structured approach, etc.)
    - **XML Structure Preservation** (Phase 6): LLM optimizer now preserves and enhances existing XML blocks rather than removing them
- **`--no-suggest` flag**: Disable auto-suggestions for scripting and CI/CD pipelines

### Changed

- **Documentation**: Updated CLAUDE.md with Makefile section and container instructions
- **Release workflow**: Added `/release` slash command for Claude Code with full automation
- **FMT001 rule**: Now also triggers on `answer`, `respond`, `reply`, `address` keywords (expanded from just `write`/`generate`)

### Technical

- Test suite expanded to 114 tests (from 95)
- New `src/cli/suggest.rs` module for interactive suggestion flow
- New `src/tui/widgets/suggest_modal.rs` for TUI modal with `SuggestModalState`
- Prompt classifier integrated into analyzer pipeline
- `optimize_with_llm()` now accepts `PromptType` parameter for context-aware optimization
- `build_optimization_message()` includes `<prompt_type>` tag for LLM context
- TTY detection for smart auto-suggest behavior (`std::io::IsTerminal`)
- LLM inference config: `temperature: 0.3` for consistent, deterministic prompt rewrites (Claude 4.5 doesn't allow both `temperature` and `top_p`)

## [0.2.0] - 2025-01-10

### Added

- **Full-screen interactive TUI mode** (`-i` / `--interactive`):
    - Built with ratatui for professional terminal UI
    - Elm-style MVU (Model-View-Update) architecture
    - Collapsible issue tree with keyboard navigation
    - Side-by-side diff view for comparing original and optimized prompts
    - Real-time progress indicators during optimization
    - Dashboard with visual bar charts for token statistics
    - Modal error dialogs and help overlay
    - Keyboard shortcuts: `q` quit, `d` diff, `?` help, `c` copy, `↑/↓` navigate
- **ASCII art logo banner** in header for enhanced branding
- **Modular widget system** in `tui/widgets/`:
    - `header.rs` - ASCII art banner
    - `analysis.rs` - Collapsible issue tree
    - `progress.rs` - Optimization gauge
    - `dashboard.rs` - Stats with bar charts
    - `diff.rs` - Side-by-side comparison
    - `status_bar.rs` - Keyboard hints
    - `help.rs` - Full keyboard shortcuts overlay
    - `error_modal.rs` - Modal error dialog
    - `minimal.rs` - Small terminal fallback

### Changed

- **Default output mode**: Now uses enhanced linear renderer with ASCII art banner
- **TUI architecture**: Migrated from ad-hoc rendering to structured ratatui framework
- **Theme system**: Single unified theme for both dark and light terminals
- **Icon system**: Nerd Font icons with Unicode/ASCII fallback detection

### Technical

- Added ratatui 0.28 and crossterm 0.28 dependencies
- Test suite expanded to 95 tests
- Added snapshot tests for TUI rendering

## [0.1.1] - 2025-12-28

### Added

- **Auto-save feature**: Optimized prompts are now automatically saved to `./copt-output/` by default
    - Each optimization creates two files:
        - `optimized_{timestamp}.txt` - The optimized prompt text
        - `optimized_{timestamp}.json` - Metadata with stats, issues, and processing info
- **New CLI flags**:
    - `--output-dir <DIR>` - Customize the output directory (default: `copt-output`)
    - `--no-save` - Disable automatic saving of optimized prompts
    - `--show-prompt` - Display optimized prompt in output (default: off)
- **Enhanced TUI output**:
    - Clean header with branding and version info
    - Visual progress bars for token comparison (using `█` and `░` blocks)
    - Organized statistics sections (Token Analysis, Performance, Provider)
    - Simple bullet-style analysis results with aggregation
    - Proper text wrapping for optimized prompt display
    - Success message when prompts are saved
- **Animated spinner**: Shows optimization progress with elapsed time counter
    - Uses indicatif for smooth animation
    - Displays elapsed time in `[HH:MM:SS]` format
    - Shows "Optimization complete" when finished
- **Aggregated issue display**: Issues are now grouped by rule ID
    - Shows count when same issue occurs on multiple lines: `(6 lines)`
    - Shows single line number for single occurrences: `(L5)`
    - Reduces visual clutter for prompts with many similar issues
- **Offline mode improvements**: Cleaner, more honest feedback in offline mode
    - Yellow "OFFLINE MODE" banner at the top of output
    - Shows analysis results (detected issues) only
    - Removes misleading "Optimization Results" panel (nothing is optimized)
    - No spinner in offline mode (no LLM call happening)
    - Helpful tip: "To optimize this prompt with an LLM, run without --offline"

### Changed

- **Complete visual redesign**: Box-free design for guaranteed alignment
    - Removed all surrounding boxes to eliminate border alignment issues
    - Uses simple horizontal separators (`─`) instead of box borders
    - Clean indentation-based hierarchy
    - Clean bullet lists (`●`) for category groupings
    - Consistent cyan color theming for headers and icons
- **Analysis display**: Simple indented bullet lists with aggregation (no boxes)
- **Statistics display**: Clean labeled rows with visual bar graphs
- **Output formatting**: Long lines in optimized prompts are properly word-wrapped
- **Header**: Simplified to icon + title + subtitle (no surrounding box)
- **Optimized prompt display**: Now hidden by default, use `--show-prompt` to display
- **STY002 rule**: Now only flags instructional ALL CAPS words, not acronyms/abbreviations
    - Flags: `DON'T, NEVER, ALWAYS, MUST, IMPORTANT, CRUCIAL, REMEMBER, WARNING`, etc.
    - Ignores: `NAMER, CSAT, EMEA, APAC, API, JSON`, and other abbreviations
- **Offline mode behavior**: Reframed as analysis-only mode
    - No auto-save in offline mode (nothing is optimized, nothing to save)
    - Can still save explicitly with `-o <file>` flag if needed
    - No "Optimization Results" panel shown (would be misleading)
    - Offline mode is now honestly presented as "analysis only"

### Fixed

- **Border alignment issues**: Eliminated entirely by removing boxes
    - Previous attempts to calculate emoji/ANSI widths were unreliable
    - Box-free design guarantees consistent display across all terminals
- **STY002 false positives**: No longer flags common abbreviations as aggressive emphasis
    - Previously flagged any 4+ character uppercase word
    - Now only flags specific instructional/emphatic words
- Removed unused imports in TUI modules
- Fixed type ambiguity errors for integer calculations

## [0.1.0] - 2025-12-27

### Added

- Initial release of `copt` (Claude Prompt Optimizer)
- **Core functionality**:
    - Analyze prompts against 25 rules across 8 categories
    - Static optimization without API calls (`--offline` mode)
    - LLM-powered optimization using Claude models
- **Provider support**:
    - Anthropic API integration
    - AWS Bedrock integration with connectivity checks
- **Input methods**:
    - Direct text input
    - File input (`-f/--file`)
    - Stdin piping
    - Interactive mode with editor support (`--interactive`)
- **Output formats**:
    - Pretty terminal output (default)
    - JSON output for automation (`--format json`)
    - Quiet mode (`--quiet`)
    - File output (`-o/--output`)
    - Side-by-side diff view (`--diff`)
- **Analysis categories**:
    - Explicitness (EXP001-004)
    - Style (STY001-004)
    - Tool Usage (TUL001-003)
    - Formatting (FMT001-003)
    - Verbosity (VRB001-002)
    - Agentic Coding (AGT001-004)
    - Long-Horizon Tasks (LHT001-003)
    - Frontend Design (FED001-002)
- **CLI features**:
    - Model selection with short aliases
    - Region configuration for Bedrock
    - Category filtering (`--check`)
    - Connectivity verification (skippable with `--skip-connectivity-check`)
    - Verbose mode (`--verbose`)
- **Documentation**:
    - Comprehensive README
    - Implementation plan
    - Rules reference documentation

### Technical

- Built with Rust for single-binary distribution
- Async runtime using Tokio
- Beautiful TUI with `colored` and `console` crates
- Token counting approximation
- Comprehensive test suite (60 tests)

---

[Unreleased]: https://github.com/praveenc/copt/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/praveenc/copt/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/praveenc/copt/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/praveenc/copt/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/praveenc/copt/releases/tag/v0.1.0

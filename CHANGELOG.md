# Changelog

All notable changes to `copt` (Claude Prompt Optimizer) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/your-org/copt/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/your-org/copt/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/your-org/copt/releases/tag/v0.1.0

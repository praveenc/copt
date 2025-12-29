# Contributing to copt

Thank you for your interest in contributing to copt! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful and constructive in all interactions. We welcome contributors of all experience levels.

## How to Contribute

### Reporting Bugs

1. **Check existing issues** to avoid duplicates
2. **Use the bug report template** when creating a new issue
3. **Include**:
   - copt version (`copt --version`)
   - Operating system and version
   - Steps to reproduce
   - Expected vs actual behavior
   - Relevant logs or error messages

### Suggesting Features

1. **Check existing issues/discussions** first
2. **Describe the use case** - what problem does it solve?
3. **Provide examples** of how it would work

### Submitting Code

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Run lints (`cargo clippy` and `cargo fmt`)
6. Commit with a clear message
7. Push and create a Pull Request

## Development Setup

### Prerequisites

- Rust 1.75 or later
- Git

### Getting Started

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/copt.git
cd copt

# Add upstream remote
git remote add upstream https://github.com/praveenc/copt.git

# Build
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- --help
```

### Project Structure

```
copt/
├── src/
│   ├── main.rs           # Entry point, CLI definitions
│   ├── analyzer/         # Prompt analysis rules
│   ├── optimizer/        # Optimization logic
│   ├── llm/              # LLM client implementations
│   │   ├── anthropic.rs  # Anthropic API
│   │   └── bedrock.rs    # AWS Bedrock
│   ├── tui/              # Terminal UI rendering
│   └── utils/            # Utilities
├── docs/                 # Documentation
└── tests/                # Integration tests
```

## Code Style

### Rust Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`
- Write documentation comments for public APIs

### Commit Messages

Use clear, descriptive commit messages:

```
feat: add support for custom rule categories
fix: correct token counting for unicode
docs: update installation instructions
refactor: simplify LLM client interface
test: add tests for offline mode
```

## Testing

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture

# Specific module
cargo test analyzer::
```

### Writing Tests

- Add unit tests in the same file using `#[cfg(test)]` module
- Add integration tests in `tests/` directory
- Aim for good coverage of edge cases

Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_works() {
        let result = my_function("input");
        assert_eq!(result, "expected");
    }
}
```

## Pull Request Process

1. **Update documentation** if needed
2. **Add tests** for new functionality
3. **Ensure CI passes** (tests, lints, formatting)
4. **Keep PRs focused** - one feature/fix per PR
5. **Respond to feedback** promptly

### PR Checklist

- [ ] Code compiles without warnings
- [ ] Tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Clippy passes (`cargo clippy`)
- [ ] Documentation updated if needed
- [ ] CHANGELOG.md updated for notable changes

## Adding New Analysis Rules

Rules are defined in `src/analyzer/mod.rs`. To add a new rule:

1. Choose an appropriate category (explicitness, style, tools, etc.)
2. Define the rule ID (e.g., `EXP005`)
3. Implement detection logic
4. Add appropriate severity (Error, Warning, Info)
5. Include a helpful suggestion
6. Add tests
7. Document in `docs/RULES.md`

Example:

```rust
// In analyze_explicitness function
let my_pattern = Regex::new(r"pattern").unwrap();
if my_pattern.is_match(line) {
    issues.push(Issue {
        id: "EXP005".to_string(),
        category: "explicitness".to_string(),
        severity: Severity::Warning,
        message: "Description of the issue".to_string(),
        line: Some(line_number),
        suggestion: Some("How to fix it".to_string()),
    });
}
```

## Adding New LLM Providers

1. Create a new file in `src/llm/` (e.g., `google.rs`)
2. Implement the `LlmClient` trait
3. Add to `src/llm/mod.rs` exports
4. Add CLI option in `src/main.rs`
5. Document usage in README

## Questions?

- Open a [GitHub Discussion](https://github.com/praveenc/copt/discussions)
- Check existing issues and PRs

Thank you for contributing! 🎉

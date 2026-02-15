.PHONY: build release test lint fmt fmt-check clean run ci ci-debug ci-release check ci-quiet

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

lint:
	cargo clippy -- -D warnings

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

clean:
	cargo clean

run:
	cargo run -- --help

# CI with debug build: verify formatting, lint, build, test (for development/testing)
ci-debug: fmt-check lint build test

# CI with release build: verify formatting, lint, release build, test (for releases)
ci-release: fmt-check lint release test

# Default CI target (alias for ci-debug)
ci: ci-debug

# Local dev: auto-fix formatting, then lint and test
check: fmt lint test

# Quiet CI: concise output for agent/automated use (saves context window)
ci-quiet:
	@echo "=== fmt-check ===" && cargo fmt --check 2>&1 | tail -1
	@echo "=== clippy ===" && cargo clippy -- -D warnings 2>&1 | tail -1
	@echo "=== build ===" && cargo build 2>&1 | tail -1
	@echo "=== test ===" && cargo test 2>&1 | grep "^test result"

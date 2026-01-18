.PHONY: build release test lint fmt fmt-check clean run ci check

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

# CI: verify formatting, lint, build, test (correct order, no auto-fix)
ci: fmt-check lint build test

# Local dev: auto-fix formatting, then lint and test
check: fmt lint test
.PHONY: build release test lint fmt check clean run

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

check: fmt lint test
	@echo "All checks passed"

clean:
	cargo clean

run:
	cargo run -- --help
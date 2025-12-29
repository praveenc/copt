# Rusty Things: A Python Developer's Guide to This Rust Project

> **For Python developers who want to understand how this Rust project works**

Welcome! If you're coming from Python and want to understand how this Rust CLI tool is built, this guide is for you. We'll explain everything by relating it to Python concepts you already know.

---

## Table of Contents

1. [Project Structure Overview](#1-project-structure-overview)
2. [Model Provider Connections](#2-model-provider-connections)
3. [The Optimization Engine](#3-the-optimization-engine)
4. [CLI Options and Argument Parsing](#4-cli-options-and-argument-parsing)
5. [Build and Release Process](#5-build-and-release-process)
6. [Installation and Distribution](#6-installation-and-distribution)
7. [Cargo.toml vs pyproject.toml](#7-cargotoml-vs-pyprojecttoml)
8. [Prerequisites for Users](#8-prerequisites-for-users)
9. [Quick Reference: Python ↔ Rust](#9-quick-reference-python--rust)

---

## 1. Project Structure Overview

### Python Project (typical)
```
my_python_project/
├── pyproject.toml          # Project config & dependencies
├── uv.lock / poetry.lock   # Locked dependency versions
├── src/
│   └── my_package/
│       ├── __init__.py
│       ├── main.py
│       └── utils.py
└── tests/
    └── test_main.py
```

### This Rust Project
```
copt/
├── Cargo.toml              # Project config & dependencies (like pyproject.toml)
├── Cargo.lock              # Locked dependency versions (like uv.lock)
├── src/
│   ├── main.rs             # Entry point (like __main__.py)
│   ├── analyzer/
│   │   └── mod.rs          # mod.rs = __init__.py for the module
│   ├── llm/
│   │   ├── mod.rs          # Module definition
│   │   ├── anthropic.rs    # Anthropic API client
│   │   └── bedrock.rs      # AWS Bedrock client
│   ├── optimizer/
│   │   └── mod.rs          # Optimization logic
│   ├── tui/
│   │   ├── mod.rs
│   │   ├── renderer.rs     # Terminal UI rendering
│   │   └── stats.rs        # Statistics display
│   └── cli/
│       └── mod.rs          # CLI helpers
└── target/                 # Build output (like __pycache__ but bigger!)
    ├── debug/              # Debug builds
    └── release/            # Optimized release builds
```

### Key Differences

| Python | Rust | Notes |
|--------|------|-------|
| `__init__.py` | `mod.rs` | Defines a module |
| `__main__.py` | `main.rs` | Entry point |
| `import x` | `mod x;` + `use x::Thing;` | Two steps: declare then use |
| `venv/` | N/A | No virtual env needed! |
| `__pycache__/` | `target/` | Compiled artifacts |

---

## 2. Model Provider Connections

**Question: What files implement connection to the model provider (e.g., Bedrock)?**

### Files Involved

```
src/llm/
├── mod.rs          # 🔑 Defines the LlmClient trait (like an ABC in Python)
├── bedrock.rs      # 🔑 AWS Bedrock implementation
└── anthropic.rs    # 🔑 Anthropic API implementation
```

### How It Works

#### `src/llm/mod.rs` - The Interface (Like Python's ABC)

```rust
// This is like Python's ABC (Abstract Base Class)
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Send a completion request to the LLM
    async fn complete(
        &self,
        system: &str,
        user_message: &str,
        model: &str,
        max_tokens: u32,
    ) -> Result<String>;

    /// Get the provider name
    fn provider_name(&self) -> &str;
}
```

**Python equivalent:**
```python
from abc import ABC, abstractmethod

class LlmClient(ABC):
    @abstractmethod
    async def complete(self, system: str, user_message: str, model: str, max_tokens: int) -> str:
        pass
    
    @abstractmethod
    def provider_name(self) -> str:
        pass
```

#### `src/llm/bedrock.rs` - AWS Bedrock Client

This file contains:
- `BedrockClient` struct (like a Python class)
- `new()` constructor (like `__init__`)
- `check_connectivity()` method to verify AWS credentials
- `complete()` implementation that sends requests to AWS Bedrock

**Key code pattern:**
```rust
pub struct BedrockClient {
    client: BedrockRuntimeClient,  // AWS SDK client
    region: String,
}

impl BedrockClient {
    pub async fn new(region: &str) -> Result<Self> {
        // Load AWS config and create client
    }
}

#[async_trait]
impl LlmClient for BedrockClient {
    async fn complete(&self, ...) -> Result<String> {
        // Make API call to AWS Bedrock
    }
}
```

#### `src/llm/anthropic.rs` - Anthropic API Client

Similar structure, but calls Anthropic's API directly instead of AWS.

---

## 3. The Optimization Engine

**Question: Where is the actual optimization prompt implemented?**

### Files Involved

```
src/
├── llm/mod.rs           # Contains OPTIMIZER_SYSTEM_PROMPT constant
├── optimizer/mod.rs     # 🔑 Main optimization logic
└── analyzer/mod.rs      # Analyzes prompts for issues
```

### The Flow

```
┌─────────────────┐     ┌──────────────┐     ┌─────────────────┐     ┌───────────┐
│  User's Prompt  │ ──▶ │   Analyzer   │ ──▶ │    Optimizer    │ ──▶ │ LLM Client│
│                 │     │ (find issues)│     │ (apply fixes)   │     │ (API call)│
└─────────────────┘     └──────────────┘     └─────────────────┘     └───────────┘
```

### Where Each Piece Lives

#### 1. The System Prompt (`src/llm/mod.rs`)

```rust
pub const OPTIMIZER_SYSTEM_PROMPT: &str = r#"You are an expert prompt engineer...
<optimization_rules>
1. EXPLICITNESS: Convert vague instructions to specific, actionable ones...
2. CONTEXT: Add motivation/reasoning when it helps Claude understand intent...
...
</optimization_rules>
"#;
```

This is the "meta-prompt" that tells Claude how to optimize prompts.

#### 2. The Optimization Logic (`src/optimizer/mod.rs`)

Two functions:

**`optimize_static()`** - Rule-based optimization (no API call):
```rust
pub fn optimize_static(prompt: &str, issues: &[Issue]) -> Result<String> {
    let mut result = prompt.to_string();
    for issue in issues {
        result = apply_static_transformation(&result, issue);
    }
    Ok(result)
}
```

**`optimize_with_llm()`** - Uses Claude to optimize:
```rust
pub async fn optimize_with_llm(
    prompt: &str,
    issues: &[Issue],
    client: &dyn LlmClient,
    model: &str,
) -> Result<String> {
    // 1. Apply static transformations first
    let partially_optimized = optimize_static(prompt, issues)?;
    
    // 2. Build message with detected issues
    let user_message = build_optimization_message(&partially_optimized, &issues_summary);
    
    // 3. Call the LLM
    let optimized = client.complete(OPTIMIZER_SYSTEM_PROMPT, &user_message, model, 4096).await?;
    
    Ok(optimized)
}
```

---

## 4. CLI Options and Argument Parsing

**Question: What files control the CLI options?**

### File: `src/main.rs`

All CLI options are defined in a single struct using the `clap` crate (Rust's equivalent to `argparse` or `click`).

### The Pattern

```rust
use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "copt", about = "⚡ Claude Optimizer")]
struct Cli {
    /// The prompt text to optimize
    #[arg(value_name = "PROMPT")]
    prompt: Option<String>,

    /// Read prompt from a file
    #[arg(short, long, value_name = "FILE")]
    file: Option<PathBuf>,

    /// LLM provider to use
    #[arg(short, long, value_enum, default_value = "bedrock")]
    provider: Provider,

    /// Run in offline mode
    #[arg(long)]
    offline: bool,
    
    // ... more options
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Provider {
    Anthropic,
    Bedrock,
}
```

### Python Equivalent (using argparse)

```python
import argparse

parser = argparse.ArgumentParser(prog='copt', description='⚡ Claude Optimizer')
parser.add_argument('prompt', nargs='?', help='The prompt text to optimize')
parser.add_argument('-f', '--file', type=Path, help='Read prompt from a file')
parser.add_argument('-p', '--provider', choices=['anthropic', 'bedrock'], default='bedrock')
parser.add_argument('--offline', action='store_true', help='Run in offline mode')
```

### Key Differences

| Python (argparse/click) | Rust (clap) |
|------------------------|-------------|
| `parser.add_argument()` | `#[arg(...)]` attribute |
| Runtime parsing | Compile-time verification |
| Dynamic | Type-safe |
| `args.file` | `cli.file` |

### How Clap Derive Macros Work

The `#[derive(Parser)]` macro automatically generates argument parsing code at compile time:

```rust
// What you write:
#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    verbose: bool,
}

// What Rust generates (simplified):
impl Cli {
    fn parse() -> Self {
        // All the argparse logic is generated!
    }
}
```

---

## 5. Build and Release Process

**Question: What is the build and release process in full detail?**

### Quick Comparison

| Python | Rust | Purpose |
|--------|------|---------|
| `python script.py` | `cargo run` | Run directly |
| `pip install -e .` | `cargo build` | Development build |
| `pip wheel .` | `cargo build --release` | Production build |
| `dist/*.whl` | `target/release/copt` | Built artifact |

### Step-by-Step Build Process

#### 1. Development Build (Fast, Unoptimized)

```bash
cargo build
```

- Creates: `target/debug/copt`
- Fast compilation (~5-10 seconds)
- Includes debug symbols
- Slower runtime performance
- Use for development/testing

#### 2. Release Build (Slow, Optimized)

```bash
cargo build --release
```

- Creates: `target/release/copt`
- Slower compilation (~1-2 minutes)
- Highly optimized (LTO, stripping)
- ~10x faster runtime than debug
- Use for distribution

#### 3. Run Without Building Separately

```bash
# Debug mode
cargo run -- --help

# Release mode
cargo run --release -- --help
```

The `--` separates cargo arguments from your program's arguments.

### Build Configuration (`Cargo.toml`)

```toml
[profile.release]
opt-level = 3      # Maximum optimization
lto = true         # Link-time optimization (smaller binary)
codegen-units = 1  # Single codegen unit (slower build, faster binary)
strip = true       # Remove debug symbols (smaller binary)
```

### What Happens During Build

```
Source Files (.rs)
        │
        ▼
   ┌─────────┐
   │  rustc  │  ← Rust compiler
   └────┬────┘
        │
        ▼
   ┌─────────┐
   │  LLVM   │  ← Optimization & code generation
   └────┬────┘
        │
        ▼
  Native Binary
  (no runtime needed!)
```

### Full Release Workflow

```bash
# 1. Make sure tests pass
cargo test

# 2. Check for warnings
cargo clippy

# 3. Format code
cargo fmt

# 4. Build release
cargo build --release

# 5. The binary is ready!
ls -lh target/release/copt
# -rwxr-xr-x  1 user  staff  8.5M Dec 29 10:00 target/release/copt
```

---

## 6. Installation and Distribution

**Question: How do I install and use this binary across the system?**

### Local Installation (Your Machine)

#### Option 1: Copy to PATH (Simple)

```bash
# macOS/Linux
sudo cp target/release/copt /usr/local/bin/

# Now use from anywhere
copt --help
```

#### Option 2: Cargo Install (From Source)

```bash
# Install from current directory
cargo install --path .

# Binary goes to ~/.cargo/bin/copt
# Make sure ~/.cargo/bin is in your PATH
```

#### Option 3: Symlink (Development)

```bash
# Create a symlink (updates automatically when you rebuild)
ln -sf $(pwd)/target/release/copt /usr/local/bin/copt
```

### Distributing to Others

#### Via GitHub Releases (Recommended)

1. **Build for your platform:**
```bash
cargo build --release
```

2. **Create a release on GitHub** and upload:
   - `copt-macos-arm64` (Apple Silicon)
   - `copt-macos-x64` (Intel Mac)
   - `copt-linux-x64`
   - `copt-windows-x64.exe`

3. **Users download and install:**
```bash
# Download
curl -L https://github.com/your-org/copt/releases/latest/download/copt-macos-arm64 -o copt

# Make executable
chmod +x copt

# Move to PATH
sudo mv copt /usr/local/bin/
```

#### Via crates.io (Rust's PyPI)

1. **Publish:**
```bash
# Login (one-time)
cargo login

# Publish
cargo publish
```

2. **Users install:**
```bash
cargo install copt
```

#### Via Homebrew (macOS)

Create a formula and users can:
```bash
brew install your-org/tap/copt
```

### Cross-Platform Building

To build for different platforms:

```bash
# Add target
rustup target add x86_64-unknown-linux-gnu

# Build for Linux from macOS
cargo build --release --target x86_64-unknown-linux-gnu
```

Or use GitHub Actions for CI/CD:

```yaml
# .github/workflows/release.yml
jobs:
  build:
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: windows-latest
            target: x86_64-pc-windows-msvc
```

---

## 7. Cargo.toml vs pyproject.toml

**Question: Is Cargo.toml similar to pyproject.toml?**

### Yes! They're Very Similar

#### Side-by-Side Comparison

**Cargo.toml (Rust):**
```toml
[package]
name = "copt"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
description = "A CLI tool to optimize prompts"
license = "MIT"
repository = "https://github.com/your-org/copt"
keywords = ["cli", "prompt", "llm"]

[dependencies]
clap = { version = "4.5", features = ["derive"] }
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }

[dev-dependencies]
assert_cmd = "2.0"
tempfile = "3.12"

[profile.release]
opt-level = 3
lto = true
```

**pyproject.toml (Python):**
```toml
[project]
name = "copt"
version = "0.1.0"
authors = [{name = "Your Name", email = "you@example.com"}]
description = "A CLI tool to optimize prompts"
license = {text = "MIT"}
keywords = ["cli", "prompt", "llm"]

[project.urls]
Repository = "https://github.com/your-org/copt"

[project.dependencies]
click = ">=8.0"
httpx = ">=0.25"
pydantic = ">=2.0"

[project.optional-dependencies]
dev = ["pytest", "black"]

[tool.setuptools]
# build config
```

### Mapping the Sections

| Cargo.toml | pyproject.toml | Purpose |
|------------|----------------|---------|
| `[package]` | `[project]` | Project metadata |
| `[dependencies]` | `[project.dependencies]` | Runtime dependencies |
| `[dev-dependencies]` | `[project.optional-dependencies]` | Test/dev dependencies |
| `[features]` | `[project.optional-dependencies]` | Optional features |
| `[profile.release]` | N/A | Build optimization settings |

### Cargo.lock vs uv.lock / poetry.lock

**Yes, they're the same concept!**

| Rust | Python | Purpose |
|------|--------|---------|
| `Cargo.lock` | `uv.lock` / `poetry.lock` | Pins exact versions |
| Auto-generated | Auto-generated | Don't edit manually |
| Commit for binaries | Commit for apps | Ensures reproducibility |
| Don't commit for libraries | Don't commit for libraries | Allow flexibility |

**What's inside:**

```toml
# Cargo.lock (simplified)
[[package]]
name = "clap"
version = "4.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "abc123..."
dependencies = [
 "clap_builder",
 "clap_derive",
]
```

---

## 8. Prerequisites for Users

**Question: What do users need to install to use this CLI tool?**

### The Beautiful Thing About Rust

**Users need NOTHING to run the compiled binary!** 🎉

Unlike Python where you need:
- Python runtime
- pip
- Virtual environment
- Dependencies

Rust compiles to a **single static binary** with everything included.

### For Running the Pre-built Binary

#### macOS
```bash
# Just download and run!
chmod +x copt
./copt --help

# Optional: Move to PATH
sudo mv copt /usr/local/bin/
```

#### Windows
```powershell
# Just download and run!
.\copt.exe --help

# Optional: Add to PATH via System Properties
```

#### Linux
```bash
# Just download and run!
chmod +x copt
./copt --help
```

### For Building from Source

Users need to install the Rust toolchain:

#### macOS
```bash
# Install Rust (includes cargo, rustc, rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart terminal or run:
source $HOME/.cargo/env

# Verify
rustc --version
cargo --version
```

#### Windows
```powershell
# Download and run rustup-init.exe from https://rustup.rs
# Or use winget:
winget install Rustlang.Rustup

# Restart terminal
rustc --version
cargo --version
```

#### Linux
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Application-Specific Requirements

For **this specific tool** (copt), users also need:

| Provider | Requirement |
|----------|-------------|
| AWS Bedrock | AWS credentials configured (`aws configure` or env vars) |
| Anthropic | `ANTHROPIC_API_KEY` environment variable |

```bash
# For AWS Bedrock
export AWS_ACCESS_KEY_ID="..."
export AWS_SECRET_ACCESS_KEY="..."
export AWS_REGION="us-west-2"

# For Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."
```

---

## 9. Quick Reference: Python ↔ Rust

### Commands

| Task | Python | Rust |
|------|--------|------|
| Run program | `python main.py` | `cargo run` |
| Run tests | `pytest` | `cargo test` |
| Install dependencies | `pip install -r requirements.txt` | `cargo build` (auto) |
| Add dependency | `pip install X` | Add to Cargo.toml, run `cargo build` |
| Format code | `black .` | `cargo fmt` |
| Lint | `ruff check .` | `cargo clippy` |
| Build package | `pip wheel .` | `cargo build --release` |
| Publish | `twine upload dist/*` | `cargo publish` |

### File Extensions

| Python | Rust | Purpose |
|--------|------|---------|
| `.py` | `.rs` | Source files |
| `.pyi` | N/A | Type stubs (Rust has built-in types) |
| `.pyc` | N/A | Bytecode (Rust compiles to native) |

### Concepts

| Python | Rust | Notes |
|--------|------|-------|
| `class` | `struct` + `impl` | Data and methods separate |
| `ABC` | `trait` | Interface definition |
| `async def` | `async fn` | Async functions |
| `await` | `.await` | Await syntax |
| `None` | `None` / `Option<T>` | Explicit optional types |
| `try/except` | `Result<T, E>` | Error handling |
| `list` | `Vec<T>` | Dynamic array |
| `dict` | `HashMap<K, V>` | Hash map |
| `str` | `String` / `&str` | Owned vs borrowed strings |
| `typing.Optional[X]` | `Option<X>` | Maybe a value |
| `decorator` | `#[attribute]` | Metaprogramming |
| GC (automatic) | Ownership system | Memory management |

### Error Handling

**Python:**
```python
try:
    result = risky_operation()
except SomeError as e:
    print(f"Error: {e}")
```

**Rust:**
```rust
match risky_operation() {
    Ok(result) => println!("Success: {}", result),
    Err(e) => println!("Error: {}", e),
}

// Or with ? operator (like Python's "let it crash")
let result = risky_operation()?;
```

### Async/Await

**Python:**
```python
async def fetch_data():
    response = await client.get(url)
    return response.json()
```

**Rust:**
```rust
async fn fetch_data() -> Result<Data> {
    let response = client.get(url).await?;
    let data = response.json().await?;
    Ok(data)
}
```

---

## Summary

### What Makes Rust Different from Python

1. **Compiled, not interpreted** - No runtime needed
2. **Statically typed** - All types known at compile time
3. **No garbage collector** - Memory managed by ownership rules
4. **No virtual environments** - Dependencies compiled into binary
5. **Faster execution** - Native machine code

### When to Use Rust vs Python

| Use Case | Python | Rust |
|----------|--------|------|
| Quick scripts | ✅ | ❌ |
| Data science | ✅ | ❌ |
| CLI tools | ✅ | ✅✅ |
| Web APIs | ✅ | ✅ |
| System tools | ❌ | ✅✅ |
| Performance-critical | ❌ | ✅✅ |
| Distribution simplicity | ❌ | ✅✅ |

### This Project's Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point + CLI definition |
| `src/llm/mod.rs` | LLM client interface |
| `src/llm/bedrock.rs` | AWS Bedrock connection |
| `src/optimizer/mod.rs` | Prompt optimization logic |
| `src/analyzer/mod.rs` | Issue detection |
| `src/tui/renderer.rs` | Terminal output |
| `Cargo.toml` | Project config |

---

## Getting Started

```bash
# Clone the repo
git clone https://github.com/your-org/copt
cd copt

# Build (debug mode, fast)
cargo build

# Run tests
cargo test

# Run the tool
cargo run -- --help

# Build release
cargo build --release

# Install globally
cargo install --path .
```

Happy Rusting! 🦀

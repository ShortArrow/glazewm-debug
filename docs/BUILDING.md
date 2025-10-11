# Building glazewm-debug

This document covers building glazewm-debug from source and setting up the development environment.

## Overview

glazewm-debug follows standard Rust conventions with **minimal external dependencies**. The CLI+JSON approach eliminates complex platform-specific integrations.

**Key Benefits:**

- **Platform-Agnostic Core**: JSON parsing identical across platforms
- **Minimal Dependencies**: No IPC libraries or native bindings
- **Simple Testing**: Easy JSON response mocking
- **Cross-Compilation**: No platform-specific code to handle

## Prerequisites

### System Requirements

**Current Platform:**

- Windows 10 (1903+) or 11
- x86_64 architecture

**Hardware:**

- Memory: 256MB minimum, 1GB for development
- Storage: 50MB for binary, 200MB for development

### Required Software

#### Rust Toolchain

```bash
# Install Rust (all platforms)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows alternative
# Download from https://rustup.rs/

# Verify installation
rustc --version  # Requires 1.70.0+
cargo --version

# Install components
rustup component add rustfmt clippy
```

#### glazewm

**Windows:**

```bash
# Via winget (recommended)
winget install glzr-io.glazewm

# Verify installation
glazewm --version
glazewm query monitors  # Test JSON output
```

## Building

### Quick Build

```bash
git clone https://github.com/username/glazewm-debug.git
cd glazewm-debug
cargo build --release
./target/release/glazewm-debug.exe
```

### Development Build

```bash
# Clone and setup
git clone https://github.com/username/glazewm-debug.git
cd glazewm-debug

# Verify environment
cargo check

# Run tests (all should pass without glazewm)
cargo test

# Development build
cargo build

# Run with debug logging
RUST_LOG=debug cargo run
```

### Build Variants

**Debug Build (Default):**

- Fast compilation, debug symbols, larger binary

```bash
cargo build
```

**Release Build:**

- Full optimization, smaller binary

```bash
cargo build --release
```

**Development Pipeline:**

```bash
cargo fmt --check    # Format check
cargo clippy         # Lint check  
cargo test          # Test execution
cargo build         # Compilation
```

## Configuration

### Environment Variables

**Rust Compiler:**

```bash
# Windows (PowerShell)
$env:RUSTFLAGS="-C target-cpu=native"

# Unix (bash/zsh)  
export RUSTFLAGS="-C target-cpu=native"
```

**Application Runtime:**

```bash
# Logging level
set RUST_LOG="glazewm_debug=debug"

# Disable colors
set NO_COLOR="1"
```

### Cross Compilation

The JSON-based architecture enables simple cross-compilation:

```bash
# Windows from Linux/macOS
rustup target add x86_64-pc-windows-msvc
cargo build --target x86_64-pc-windows-msvc

# Linux from Windows/macOS  
rustup target add x86_64-unknown-linux-gnu
cargo build --target x86_64-unknown-linux-gnu

# macOS from Linux/Windows
rustup target add x86_64-apple-darwin
cargo build --target x86_64-apple-darwin
```

## Testing

### Test Categories

**Unit Tests**: Domain logic, JSON parsing

```bash
cargo test --lib
```

**Integration Tests**: CLI client, end-to-end workflows

```bash
cargo test --test integration
```

**Live Tests**: Real glazewm integration (optional)

```bash
cargo test --test glazewm_live
```

### Development Testing

```bash
# Quick test cycle
cargo test domain::           # Fast unit tests
cargo test --test json_parse  # JSON parsing tests

# Full test suite
cargo test

# Test with coverage
cargo test -- --nocapture
```

## Troubleshooting

### Common Build Issues

**Rust Toolchain:**

```bash
# Update toolchain
rustup update

# Clean and rebuild
cargo clean && cargo build
```

**glazewm Integration:**

```bash
# Verify glazewm works
where glazewm           # Windows
which glazewm           # Unix
glazewm --version
glazewm query windows

# Add to PATH if needed
$env:PATH += ";C:\Program Files\glazewm"  # Windows
export PATH=$PATH:/usr/local/bin          # Unix
```

**Compilation Errors:**

```bash
# Common fixes
cargo update          # Update dependencies
cargo clean          # Clear build cache
rustup update         # Update Rust version
```

### Runtime Issues

**Application Startup:**

```bash
# Debug mode
RUST_LOG=debug cargo run

# Test glazewm connectivity
glazewm query monitors
```

**Terminal Display:**

```bash
# UTF-8 encoding (Windows)
chcp 65001

# Terminal compatibility
$env:TERM="xterm-256color"
```

## Packaging

### Release Build

```bash
# Optimized release
cargo build --release

# Verify binary
./target/release/glazewm-debug.exe --version

# Size optimization (optional)
strip target/release/glazewm-debug  # Unix only
```

### Distribution

```bash
# Windows installer (optional)
cargo install cargo-wix
cargo wix

# Cross-platform packages
cargo install cargo-dist
cargo dist build
```

## CI/CD

### GitHub Actions

```yaml
name: Build and Test

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [windows-latest, ubuntu-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
    
    - run: cargo fmt --check
    - run: cargo clippy -- -D warnings  
    - run: cargo test
    - run: cargo build --release
```

### Local Development

```bash
# Pre-commit checks
cargo fmt && cargo clippy && cargo test

# Performance check
cargo build --release
time ./target/release/glazewm-debug --help
```

## Development Environment

### Recommended Tools

**Editor**: VS Code with rust-analyzer
**Terminal**: Windows Terminal (Windows) or Alacritty (cross-platform)
**Debugging**: Built-in Rust debugging with VS Code

### Project Setup

```bash
# Setup workspace
git clone https://github.com/username/glazewm-debug.git
cd glazewm-debug

# Install pre-commit hooks (optional)
pip install pre-commit
pre-commit install

# Verify setup
cargo check
cargo test
```

## Related Documentation

- **[← Back to README](../README.md)** - Project overview
- **[Usage Guide](USAGE.md)** - CLI options and controls
- **[API Integration](API.md)** - glazewm integration details
- **[Architecture](ARCHITECTURE.md)** - Design principles
- **[Contributing](CONTRIBUTE.md)** - Development workflow

## External References

- [Rust Programming Language](https://doc.rust-lang.org/book/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [glazewm Documentation](https://github.com/glzr-io/glazewm)

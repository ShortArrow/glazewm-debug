# Building glazewm-debug

This article covers building glazewm-debug from source, including development environment setup and troubleshooting common issues.

## Overview

glazewm-debug follows standard Rust conventions with **minimal external dependencies**. The CLI+JSON approach eliminates complex platform-specific integrations, making the build process straightforward and the codebase highly portable.

**Key Architectural Benefits:**

- **Platform-Agnostic Core**: JSON parsing is identical across all platforms
- **Minimal Dependencies**: No IPC libraries, native bindings, or complex integrations
- **Simple Testing**: Easy to mock JSON responses for testing
- **Future Extensibility**: Adding support for other window managers requires only JSON schema mapping

## Prerequisites

### System Requirements

**Current Target Platform:**

- **Windows**: 10 (1903+) or 11 (for glazewm compatibility)
- **Architecture**: x86_64 (amd64)

**Hardware:**

- **Memory**: 256MB minimum, 1GB recommended for development
- **Storage**: 50MB for binary, 200MB for full development environment

**Future Platform Support:**
> The JSON-based architecture is inherently platform-agnostic. The core parsing and UI logic will work on any platform that can:
>
> 1. Execute command-line tools
> 2. Parse JSON responses
> 3. Run terminal applications
>
> This includes Linux, macOS, and other Unix-like systems when paired with compatible window managers.

### Required Software

#### Rust Toolchain

The project requires Rust 1.70.0 or newer with the stable toolchain.

**Installation:**

```powershell
# Download and run rustup installer
Invoke-WebRequest -Uri "https://win.rustup.rs/" -OutFile "rustup-init.exe"
.\rustup-init.exe

# Verify installation
rustc --version
cargo --version
```

**Required Components:**

```bash
rustup component add rustfmt clippy
```

#### glazewm

glazewm must be installed and available in PATH.

**Via Package Manager (Recommended):**

```powershell
winget install glzr-io.glazewm
```

**Manual Installation:**

1. Download binary from [glazewm releases](https://github.com/glzr-io/glazewm/releases)
2. Extract to directory in PATH (e.g., `C:\Program Files\glazewm\`)
3. Verify installation:

   ```cmd
   glazewm --version
   ```

**From Source:**

```bash
git clone https://github.com/glzr-io/glazewm.git
cd glazewm
# Follow glazewm's build instructions
```

### Optional Dependencies

#### Development Tools

**Git:**

```powershell
winget install Git.Git
```

**Text Editor:**

```powershell
# VS Code with Rust extension
winget install Microsoft.VisualStudioCode

# Alternative: Notepad++
winget install Notepad++.Notepad++
```

## Building

### Quick Build

For users who want to build and run immediately:

```bash
git clone https://github.com/username/glazewm-debug.git
cd glazewm-debug
cargo build --release
./target/release/glazewm-debug.exe
```

### Development Build

For contributors and developers:

```bash
# Clone repository
git clone https://github.com/username/glazewm-debug.git
cd glazewm-debug

# Verify dependencies
cargo check

# Run test suite
cargo test

# Build in debug mode
cargo build

# Run from source
cargo run
```

### Build Variants

**Debug Build (Default):**

- Fast compilation
- Debug symbols included
- Optimizations disabled
- Larger binary size

```bash
cargo build
```

**Release Build:**

- Slower compilation
- Debug symbols stripped
- Full optimizations
- Smaller binary size

```bash
cargo build --release
```

**Development Build with Checks:**

```bash
# Complete development pipeline
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all
cargo test
cargo build
```

## Configuration

### Build Configuration

The build can be customized through `Cargo.toml` and environment variables.

#### Cargo Features

Currently, no optional features are defined. Future versions may include:

- `gui-mode` - Optional GUI interface
- `extended-logging` - Verbose logging support
- `plugin-system` - Plugin architecture

#### Environment Variables

**Rust Compiler:**

```bash
# Enable debug assertions in release builds
set RUSTFLAGS="-C debug-assertions"

# Target-specific compilation
set CARGO_BUILD_TARGET="x86_64-pc-windows-msvc"
```

**Application Runtime:**

```bash
# Logging configuration
set RUST_LOG="glazewm_debug=debug"

# Disable color output
set NO_COLOR="1"
```

### Cross Compilation

The JSON-based architecture makes cross-compilation much simpler since there are **no platform-specific dependencies** in the core logic.

**Supported Cross-Compilation:**

```bash
# Windows from Linux/macOS
rustup target add x86_64-pc-windows-msvc
cargo build --target x86_64-pc-windows-msvc

# Linux from Windows/macOS
rustup target add x86_64-unknown-linux-gnu
cargo build --target x86_64-unknown-linux-gnu

# macOS from Linux/Windows (requires macOS SDK)
rustup target add x86_64-apple-darwin
cargo build --target x86_64-apple-darwin
```

**Benefits of JSON Approach:**

- No native libraries to cross-compile
- No platform-specific APIs
- Only standard Rust dependencies
- Terminal UI works across all platforms

## Testing

### Test Categories

**Unit Tests:**

```bash
# Test individual components
cargo test --lib

# Test specific module
cargo test domain::monitor
```

**Integration Tests:**

```bash
# Test glazewm integration
cargo test --test integration

# Requires running glazewm instance
cargo test --test glazewm_live
```

**Documentation Tests:**

```bash
# Test code examples in documentation
cargo test --doc
```

### Test Configuration

**Running Specific Tests:**

```bash
# Filter by name pattern
cargo test monitor_should_

# Run ignored tests
cargo test -- --ignored

# Single-threaded execution
cargo test -- --test-threads=1
```

**Test Output:**

```bash
# Capture stdout/stderr
cargo test -- --nocapture

# Show test execution time
cargo test -- --report-time
```

## Static Analysis

### Code Quality

**Linting:**

```bash
# Basic linting
cargo clippy

# Strict linting for CI
cargo clippy --all-targets --all-features -- -D warnings

# Fix automatic issues
cargo clippy --fix
```

**Formatting:**

```bash
# Check formatting
cargo fmt --check

# Apply formatting
cargo fmt

# Format with options
cargo fmt -- --config hard_tabs=true
```

### Security Analysis

**Dependency Audit:**

```bash
# Install audit tool
cargo install cargo-audit

# Check for known vulnerabilities
cargo audit

# Generate audit report
cargo audit --format json > audit-report.json
```

**License Compliance:**

```bash
# Install license checker
cargo install cargo-license

# Check dependency licenses
cargo license
```

## Performance

### Binary Size Optimization

**Release Profile Tuning:**

Add to `Cargo.toml`:

```toml
[profile.release]
opt-level = "z"    # Optimize for size
lto = true         # Enable link-time optimization
codegen-units = 1  # Reduce parallel code generation
panic = "abort"    # Reduce panic handling overhead
strip = true       # Strip debug symbols
```

**Runtime Performance:**

```bash
# Profile release build
cargo build --release
perf record ./target/release/glazewm-debug.exe
perf report
```

### Memory Usage

**Heap Profiling:**

```bash
# Using application verifier (Windows)
# Or custom memory tracking in debug builds
set RUST_LOG="glazewm_debug::memory=trace"
cargo run
```

## Troubleshooting

### Common Build Issues

#### Rust Toolchain Problems

**Symptom**: `rustc` not found

```bash
# Verify PATH includes Cargo bin directory
echo $env:PATH | Select-String ".cargo"

# Reinstall toolchain
rustup self uninstall
# Reinstall rustup
```

**Symptom**: Compilation errors with dependencies

```bash
# Update toolchain
rustup update

# Clean build cache
cargo clean

# Force dependency update
cargo update
```

#### glazewm Integration Issues

**Symptom**: glazewm not found in PATH

```bash
# Check glazewm installation
where glazewm
glazewm --version

# Add to PATH if needed
$env:PATH += ";C:\Program Files\glazewm"
```

**Symptom**: Permission denied accessing glazewm

```bash
# Run as administrator
# Or check glazewm service status
sc query GlazeWM
```

#### Windows-Specific Issues

**Symptom**: Windows Defender blocking compilation

```bash
# Add exclusion for project directory
Add-MpPreference -ExclusionPath "C:\path\to\glazewm-debug"
```

**Symptom**: Long path issues

```powershell
# Enable long path support (requires admin)
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" `
  -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD
```

### Build Performance Issues

**Slow Compilation:**

```bash
# Parallel compilation
set CARGO_BUILD_JOBS=4

# Use faster linker
cargo install lld
set RUSTFLAGS="-C link-arg=-fuse-ld=lld"

# Incremental compilation (debug builds)
set CARGO_INCREMENTAL=1
```

**High Memory Usage:**

```bash
# Reduce parallel units
export CARGO_BUILD_JOBS=1

# Disable incremental compilation
export CARGO_INCREMENTAL=0
```

### Runtime Issues

#### Application Startup

**Symptom**: glazewm-debug fails to start

```bash
# Check dependencies
ldd target/release/glazewm-debug.exe  # On WSL
dumpbin /dependents target/release/glazewm-debug.exe  # Windows

# Enable debug logging
set RUST_LOG=debug
cargo run
```

**Symptom**: Cannot connect to glazewm

```bash
# Verify glazewm is running
tasklist | findstr glazewm

# Test manual query
glazewm query windows
```

#### Terminal Compatibility

**Symptom**: Display corruption or encoding issues

```bash
# Set UTF-8 encoding
chcp 65001

# Use Windows Terminal (recommended)
winget install Microsoft.WindowsTerminal

# Set environment variables
$env:TERM="xterm-256color"
```

## Packaging

### Binary Distribution

**Standalone Executable:**

```bash
# Build optimized release
cargo build --release

# Verify binary
./target/release/glazewm-debug.exe --version

# Package with dependencies (if any)
# Currently no external runtime dependencies
```

**Installer Creation:**

```bash
# Using cargo-wix (Windows Installer)
cargo install cargo-wix
cargo wix init
cargo wix
```

### Development Environment

**Portable Setup:**

```bash
# Create development bundle
mkdir glazewm-debug-dev
cp -r src/ glazewm-debug-dev/
cp Cargo.toml glazewm-debug-dev/
cp README.md glazewm-debug-dev/

# Include build script
echo "cargo build && cargo test" > glazewm-debug-dev/build.bat
```

## CI/CD Integration

### GitHub Actions

**Multi-Platform Pipeline:**

```yaml
name: Build and Test

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
    
    - name: Format Check
      run: cargo fmt --check
    
    - name: Lint
      run: cargo clippy -- -D warnings
    
    - name: Test (JSON parsing is platform-agnostic)
      run: cargo test
    
    - name: Build Release
      run: cargo build --release

  cross-compile:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target: [x86_64-pc-windows-msvc, x86_64-apple-darwin, x86_64-unknown-linux-gnu]
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: ${{ matrix.target }}
    
    - name: Cross Build
      run: cargo build --release --target ${{ matrix.target }}
```

### Local Development

**Pre-commit Hooks:**

```bash
# Install pre-commit framework
pip install pre-commit

# Add to .pre-commit-config.yaml
# Then install hooks
pre-commit install
```

## Related Documentation

- **[← Back to README](../README.md)** - Project overview and quick start
- **[Usage Guide](USAGE.md)** - Detailed CLI options and controls
- **[API Integration](API.md)** - glazewm CLI integration details
- **[Architecture](ARCHITECTURE.md)** - Design principles and structure
- **[Contributing](CONTRIBUTE.md)** - Development workflow

## External References

- [The Rust Programming Language](https://doc.rust-lang.org/book/) - Rust fundamentals
- [Cargo Book](https://doc.rust-lang.org/cargo/) - Build system documentation
- [glazewm Documentation](https://github.com/glzr-io/glazewm) - Target application
- [Windows Terminal](https://docs.microsoft.com/en-us/windows/terminal/) - Recommended terminal

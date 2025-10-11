# Contributing to glazewm-debug

Welcome to the glazewm-debug project! This guide covers contributing to a **CLI+JSON based window manager debugging tool** that prioritizes simplicity, testability, and UNIX philosophy.

## Overview

glazewm-debug uses a **simplified architecture** that makes contributing straightforward:

- **JSON-First**: All data flows through well-defined JSON schemas
- **Platform-Agnostic Core**: Business logic works on any platform
- **Minimal Dependencies**: Easy to set up and test
- **CLI-Based Integration**: Simple command execution, no complex APIs

## Development Environment

### Prerequisites

**Required Software:**

- Rust 1.70.0+ with `cargo`, `rustfmt`, `clippy`
- glazewm 3.0.0+ (for integration testing)
- Git

**Recommended Tools:**

- Windows Terminal or equivalent
- VS Code with rust-analyzer extension

### Quick Setup

```bash
# Clone and setup
git clone https://github.com/username/glazewm-debug.git
cd glazewm-debug

# Verify dependencies
cargo check

# Run test suite
cargo test

# Start development
cargo run
```

## CLI+JSON Architecture Benefits for Contributors

### Simplified Testing

**JSON Fixture Testing:**

```rust
// Easy to create test data
let monitor_json = r#"
{
  "data": {
    "monitors": [{
      "id": "monitor-1",
      "width": 1920,
      "height": 1080,
      "hasFocus": true,
      "children": []
    }]
  },
  "success": true
}
"#;

let monitors: Vec<Monitor> = parse_monitors_response(monitor_json)?;
assert_eq!(monitors.len(), 1);
```

**Mock CLI Client:**

```rust
pub struct MockGlazewmClient {
    responses: HashMap<Vec<String>, String>,
}

impl MockGlazewmClient {
    pub fn with_monitor_response(json: &str) -> Self {
        let mut responses = HashMap::new();
        responses.insert(vec!["query".to_string(), "monitors".to_string()], json.to_string());
        Self { responses }
    }
}
```

### Platform-Independent Development

**Core Logic Testing:**

- Domain logic works the same on all platforms
- JSON parsing is platform-agnostic
- UI rendering is terminal-based (cross-platform)
- No platform-specific APIs to mock

## Contributing Guidelines

### Code Standards

**UNIX Philosophy:**

- Do one thing well (glazewm state visualization)
- Compose tools (pipe-friendly output modes)
- Simple, readable code

**Domain-Driven Design:**

- Business logic in `domain/` modules
- Clear entity boundaries (Monitor, Workspace, Window)
- Value objects for immutable data

**Test-Driven Development:**

- Write tests first for new features
- Test JSON schemas thoroughly
- Mock external dependencies (CLI calls)

### Commit Message Format

Use [Conventional Commits](https://www.conventionalcommits.org/):

```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Examples:**

```bash
feat(cli): add timeout handling for glazewm commands
fix(json): handle missing fields in window response
docs(architecture): update CLI client documentation
test(domain): add workspace focus state tests
```

## Development Workflow

### 1. Issue-Driven Development

**Before Starting:**

1. Check existing issues for duplicates
2. Create or comment on relevant issue
3. Discuss approach if feature is significant

**Issue Template:**

```markdown
## Problem
Brief description of the issue

## Expected Behavior
What should happen

## Current Behavior
What actually happens

## Environment
- OS: Windows 10/11
- glazewm version: `glazewm --version`
- glazewm-debug version: `cargo run -- --version`

## glazewm Output
```json
# Output from `glazewm query monitors` and/or `glazewm query windows`
```

```text

### 2. Feature Development

**TDD Workflow:**
```bash
# 1. Write failing test
cargo test new_feature_test -- --nocapture

# 2. Implement minimal code to pass
# ...

# 3. Refactor and improve
cargo clippy
cargo fmt

# 4. Verify all tests pass
cargo test
```

**JSON Schema Testing:**

```rust
#[test]
fn should_parse_glazewm_monitor_response() {
    let json = include_str!("../fixtures/monitors_response.json");
    let result = parse_monitors_response(json);
    
    assert!(result.is_ok());
    let monitors = result.unwrap();
    assert_eq!(monitors.len(), 2);
    assert_eq!(monitors[0].width, 1920);
}
```

### 3. Integration Testing

**glazewm CLI Testing:**

```rust
#[tokio::test]
async fn should_execute_glazewm_query() {
    let client = GlazewmClient::new("glazewm".into());
    
    // This test requires glazewm to be running
    if client.is_available().await {
        let monitors = client.query_monitors().await.unwrap();
        assert!(!monitors.is_empty());
    }
}
```

**Error Handling Testing:**

```rust
#[tokio::test]
async fn should_handle_invalid_command() {
    let client = GlazewmClient::new("nonexistent-command".into());
    
    let result = client.query_monitors().await;
    assert!(matches!(result, Err(ClientError::CommandNotFound)));
}
```

## Testing Strategy

### Test Categories

**Unit Tests** (`cargo test`):

- Domain logic (Monitor, Workspace, Window entities)
- JSON parsing and validation
- Value object behavior
- Error handling

**Integration Tests** (`cargo test --test integration`):

- CLI client with mock responses
- End-to-end JSON processing
- Error recovery scenarios

**Live Tests** (`cargo test --test live` - requires glazewm):

- Real glazewm integration
- Command execution testing
- Response format validation

### Test Fixtures

**Create JSON Fixtures:**

```bash
# Capture real glazewm responses for testing
glazewm query monitors > tests/fixtures/monitors_response.json
glazewm query windows > tests/fixtures/windows_response.json
```

**Use in Tests:**

```rust
#[test]
fn should_parse_real_glazewm_response() {
    let json = include_str!("../fixtures/monitors_response.json");
    let monitors = parse_monitors_response(json).unwrap();
    
    // Test against real data structure
    assert!(!monitors.is_empty());
}
```

## Adding New Features

### Example: Adding Window Focus History

**1. Domain Model:**

```rust
// domain/window.rs
pub struct Window {
    // existing fields...
    focus_history: Vec<Instant>,
}

impl Window {
    pub fn record_focus(&mut self) {
        self.focus_history.push(Instant::now());
    }
    
    pub fn last_focus_time(&self) -> Option<Instant> {
        self.focus_history.last().copied()
    }
}
```

**2. JSON Schema Extension:**

```rust
// cli/types.rs
#[derive(Debug, Deserialize)]
pub struct RawWindow {
    // existing fields...
    #[serde(rename = "lastFocusTime")]
    last_focus_time: Option<String>, // ISO 8601 timestamp
}
```

**3. CLI Client Update:**

```rust
// cli/client.rs
impl From<RawWindow> for Window {
    fn from(raw: RawWindow) -> Self {
        let focus_time = raw.last_focus_time
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.into());
            
        Window::new(
            // existing fields...
            focus_time,
        )
    }
}
```

**4. Tests:**

```rust
#[test]
fn should_parse_window_focus_time() {
    let json = r#"
    {
      "data": {
        "windows": [{
          "id": "window-1",
          "title": "main.rs - Visual Studio Code",
          "lastFocusTime": "2023-12-01T10:30:00Z"
        }]
      }
    }
    "#;
    
    let windows = parse_windows_response(json).unwrap();
    assert!(windows[0].last_focus_time().is_some());
}

#[test]
fn should_identify_focused_window() {
    let json = r#"
    {
      "data": {
        "windows": [
          {
            "id": "window-1", 
            "hasFocus": false,
            "title": "Background - Chrome",
            "processName": "chrome"
          },
          {
            "id": "window-2",
            "hasFocus": true, 
            "title": "main.rs - Visual Studio Code",
            "processName": "Code"
          }
        ]
      },
      "success": true
    }
    "#;
    
    let windows = parse_windows_response(json).unwrap();
    
    // Find focused window
    let focused = windows.iter().find(|w| w.has_focus);
    assert!(focused.is_some());
    assert_eq!(focused.unwrap().title, "main.rs - Visual Studio Code");
    
    // Verify focus exclusivity (only one window focused)
    let focused_count = windows.iter().filter(|w| w.has_focus).count();
    assert_eq!(focused_count, 1);
}
```

## Debugging and Troubleshooting

### CLI Debugging

**Manual Testing:**

```bash
# Test glazewm CLI directly
glazewm query monitors | jq '.'
glazewm query windows | jq '.data.windows[0]'

# Test with glazewm-debug
RUST_LOG=debug cargo run
```

**JSON Validation:**

```bash
# Validate JSON schema
glazewm query monitors | jq 'type'
glazewm query monitors | jq '.data.monitors | length'
```

### Common Issues

**glazewm Not Found:**

```bash
# Check PATH
echo $PATH | grep glazewm
where glazewm

# Verify glazewm works
glazewm --version
glazewm query monitors
```

**JSON Parsing Errors:**

```rust
// Add debug logging
#[derive(Debug, Deserialize)]
struct DebugResponse {
    #[serde(flatten)]
    data: serde_json::Value,
}

let debug: DebugResponse = serde_json::from_str(&json)?;
eprintln!("Raw JSON structure: {:#?}", debug.data);
```

## Pull Request Process

### Before Submitting

**Pre-submission Checklist:**

```bash
# Format and lint
cargo fmt
cargo clippy -- -D warnings

# Run all tests
cargo test
cargo test --test integration

# Test with real glazewm (if available)
cargo test --test live

# Check documentation
cargo doc --open
```

**Performance Check:**

```bash
# Ensure CLI calls are efficient
cargo build --release
time ./target/release/glazewm-debug --help

# Memory usage check
cargo run --release &
RUST_LOG=debug cargo run 2>&1 | grep "memory"
```

### PR Requirements

**Essential Elements:**

- Clear description of changes
- Tests for new functionality
- Documentation updates (if needed)
- JSON schema compatibility verification
- Performance impact assessment

**PR Template:**

```markdown
## Changes
Brief description of what this PR accomplishes

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Live tests pass (with glazewm running)
- [ ] JSON parsing works with real glazewm output

## Compatibility
- [ ] No breaking changes to JSON schemas
- [ ] CLI interface remains stable
- [ ] Cross-platform compatibility maintained

## Performance
- [ ] No significant performance regression
- [ ] Memory usage remains stable
- [ ] CLI call frequency is appropriate
```

## Release Process

### Version Management

**Semantic Versioning:**

- **Major** (1.0.0): Breaking changes to CLI interface or JSON schemas
- **Minor** (0.1.0): New features, backward-compatible
- **Patch** (0.0.1): Bug fixes, no API changes

**Release Checklist:**

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Tag release: `git tag v1.0.0`
4. Build release binaries for all platforms
5. Create GitHub release with binaries

## Community

### Getting Help

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and general discussion
- **Code Review**: PR feedback and architectural discussions

### Code of Conduct

- Be respectful and inclusive
- Focus on technical merit
- Help newcomers learn the CLI+JSON architecture
- Document decisions and trade-offs

## Future Extensions

The CLI+JSON architecture makes several extensions straightforward:

### New Window Managers

Adding support for other window managers (i3, sway, yabai) only requires:

1. New CLI client implementation
2. JSON schema mapping
3. Command execution logic

### Output Formats

```bash
# JSON output for scripting
glazewm-debug --output json

# CSV for data analysis
glazewm-debug --output csv

# Tree format for debugging
glazewm-debug --output tree
```

### Configuration

```toml
# ~/.config/glazewm-debug.toml
[cli]
command_path = "glazewm"
timeout_ms = 5000
retry_attempts = 3

[display]
refresh_rate_ms = 1000
compact_mode = false
```

## Related Documentation

- **[← Back to README](../README.md)** - Project overview and quick start
- **[Usage Guide](USAGE.md)** - CLI options and keyboard controls
- **[Display Format](DISPLAY.md)** - Output interpretation guide
- **[API Integration](API.md)** - glazewm JSON API reference
- **[Building](BUILDING.md)** - Build instructions and environment setup
- **[Architecture](ARCHITECTURE.md)** - Design principles and implementation

---

The CLI+JSON approach makes glazewm-debug an ideal project for contributors of all levels. The simplified architecture, comprehensive testing strategy, and platform-agnostic design create opportunities for meaningful contributions without complex setup requirements.

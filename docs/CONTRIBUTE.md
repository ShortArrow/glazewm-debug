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

### Testable Design Strategy

**Interface-Driven Architecture:**
All external dependencies (glazewm CLI, terminal I/O, keyboard input, configuration) are abstracted through trait interfaces, enabling comprehensive testing with mock implementations.

**Key Mockable Components:**

- `WindowManagerClient` trait → Mock with JSON fixtures
- `Terminal` trait → Mock with captured output verification
- `EventSource` trait → Mock with scripted user input sequences
- `ConfigProvider` trait → Mock with test-specific settings

**Testing Approach:**

- **Builder Pattern**: Fluent APIs for creating complex test scenarios
- **JSON Fixtures**: Real glazewm responses for accurate integration testing
- **State Simulation**: Time-based testing for lifecycle scenarios  
- **Error Injection**: Mock failures for resilience testing
- **Performance Validation**: Large dataset handling with controlled timing

> **Implementation Details**: See `docs/TESTING.md` for concrete examples of mock implementations and test builder patterns.

#### Behavior Testing with Mocks

**User Workflow Testing:**

```rust
#[tokio::test]
async fn should_complete_typical_user_workflow() {
    // Given - User opens app, navigates, then quits
    let mock_events = MockEventSource::user_workflow(vec![
        "app_startup",
        "press_r",      // Force refresh
        "press_down",   // Navigate down  
        "press_up",     // Navigate up
        "press_q",      // Quit
    ]);

    let mock_terminal = MockTerminal::new(80, 24)
        .capture_all_draws()
        .expect_no_errors();

    let app = create_test_app(mock_client, mock_terminal, mock_events);

    // When
    app.run_complete_session().await.unwrap();

    // Then - Verify expected UI updates
    let draw_history = mock_terminal.draw_history();
    assert!(draw_history.len() >= 4); // At least 4 screen updates
    
    // Verify navigation worked
    assert!(draw_history[1].contains("Workspace")); // After refresh
    assert!(draw_history[2] != draw_history[1]);    // Navigation changed display
    
    // Verify clean shutdown
    assert_eq!(mock_events.remaining_events(), 0);
}
```

#### Error Scenario Testing

**System Failure Simulation:**

```rust
#[tokio::test]
async fn should_handle_glazewm_timeout() {
    // Given - glazewm responds slowly
    let slow_client = MockWindowManagerClient::new()
        .with_response_delay(Duration::from_secs(10))  // Very slow
        .with_timeout(Duration::from_secs(5));         // App timeout

    let mock_terminal = MockTerminal::new(80, 24);
    let app = App::new(slow_client, mock_terminal, ...);

    // When
    let result = app.update_state().await;

    // Then
    assert!(result.is_err());
    assert!(mock_terminal.contains_text("Connection timeout"));
    assert!(mock_terminal.contains_text("Check glazewm status"));
}

#[test]
fn should_recover_from_terminal_resize() {
    // Given - Terminal is resized during operation
    let mock_terminal = MockTerminal::new(80, 24)
        .schedule_resize_to(120, 30) // Simulate resize
        .then_resize_to(100, 25);

    let app = App::new(..., mock_terminal, ...);

    // When
    app.handle_resize_events().unwrap();

    // Then
    let final_content = mock_terminal.last_draw_content();
    assert!(content_fits_in_size(&final_content, Size::new(100, 25)));
}
```

#### JSON Schema Evolution Testing

**Backward Compatibility Testing:**

```rust
#[test]
fn should_handle_glazewm_api_changes() {
    // Test with different JSON schemas
    let test_cases = vec![
        ("glazewm_v3.0.json", true),   // Current version
        ("glazewm_v3.1.json", true),   // Future version (should work)
        ("glazewm_v2.9.json", false),  // Old version (may fail gracefully)
    ];

    for (fixture, should_succeed) in test_cases {
        let json = include_str!(fixture);
        let result = parse_monitors_response(json);
        
        if should_succeed {
            assert!(result.is_ok(), "Failed to parse {}", fixture);
        } else {
            // Should fail gracefully with helpful error
            assert!(result.is_err());
            assert!(result.unwrap_err().is_version_incompatible());
        }
    }
}
```

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

### Mock Strategy Best Practices

#### Test Environment Isolation

**Principle**: Every test should run in complete isolation without external dependencies.

**Implementation:**

```rust
// Create test environment factory
pub struct TestEnvironment {
    pub client: MockWindowManagerClient,
    pub terminal: MockTerminal,
    pub events: MockEventSource,
    pub config: MockConfig,
}

impl TestEnvironment {
    // Standard scenarios
    pub fn single_monitor_setup() -> Self { /* ... */ }
    pub fn multi_monitor_setup() -> Self { /* ... */ }
    pub fn empty_desktop_setup() -> Self { /* ... */ }
    
    // Error scenarios  
    pub fn glazewm_unavailable() -> Self { /* ... */ }
    pub fn network_timeout() -> Self { /* ... */ }
    pub fn invalid_json_response() -> Self { /* ... */ }
    
    // Performance scenarios
    pub fn large_window_count() -> Self { /* ... */ }
    pub fn rapid_state_changes() -> Self { /* ... */ }
}

// Use in tests
#[tokio::test] 
async fn test_multi_monitor_navigation() {
    let env = TestEnvironment::multi_monitor_setup();
    let app = App::from_test_env(env);
    // ... test logic
}
```

#### State Transition Testing

**Test State Changes Over Time:**

```rust
#[tokio::test]
async fn should_track_window_lifecycle() {
    // Given - Simulate window creation → focus → minimize → close
    let mock_client = MockWindowManagerClient::new()
        .at_time(0, empty_desktop_json())
        .at_time(1000, window_created_json("VS Code"))
        .at_time(2000, window_focused_json("VS Code"))  
        .at_time(3000, window_minimized_json("VS Code"))
        .at_time(4000, window_closed_json("VS Code"));

    let mock_events = MockEventSource::new()
        .every_second(Event::Timer)
        .until_time(5000);

    let app = App::new(mock_client, mock_terminal, mock_events, config);

    // When - Run through time sequence
    let state_history = app.run_with_time_simulation().await.unwrap();

    // Then - Verify state transitions
    assert_eq!(state_history[0].total_windows(), 0);  // Empty
    assert_eq!(state_history[1].total_windows(), 1);  // Created
    assert!(state_history[2].has_focused_window());   // Focused
    assert!(state_history[3].has_minimized_windows()); // Minimized
    assert_eq!(state_history[4].total_windows(), 0);  // Closed
}
```

#### Performance Testing with Mocks

**Response Time Testing:**

```rust
#[tokio::test]
async fn should_meet_performance_requirements() {
    // Given - Large dataset
    let large_state = TestStateBuilder::new()
        .with_monitors(4)
        .with_workspaces_per_monitor(8)
        .with_windows_per_workspace(10)  // 320 total windows
        .build();

    let mock_client = MockWindowManagerClient::from_state(large_state)
        .with_response_delay(Duration::from_millis(50)); // Realistic delay

    let start_time = std::time::Instant::now();

    // When
    let app = App::new(mock_client, mock_terminal, mock_events, config);
    app.update_state().await.unwrap();
    app.render().unwrap();

    let elapsed = start_time.elapsed();

    // Then - Should handle large datasets efficiently
    assert!(elapsed < Duration::from_millis(200)); // Total under 200ms
    assert_eq!(mock_client.call_count(), 2); // Only 2 API calls needed
}
```

#### Edge Case Testing

**Boundary Condition Testing:**

```rust
#[test]
fn should_handle_edge_cases() {
    let edge_cases = vec![
        // Empty responses
        ("empty_monitors.json", |state| assert_eq!(state.monitors.len(), 0)),
        
        // Single item responses  
        ("single_monitor.json", |state| assert_eq!(state.monitors.len(), 1)),
        
        // Maximum realistic sizes
        ("large_monitor_4k.json", |state| {
            assert!(state.monitors[0].geometry.size.width >= 3840);
        }),
        
        // Unicode in window titles
        ("unicode_titles.json", |state| {
            assert!(state.windows().any(|w| w.title().contains("🚀")));
        }),
        
        // Very long window titles
        ("long_titles.json", |state| {
            let longest_title = state.windows()
                .map(|w| w.title().len())
                .max()
                .unwrap_or(0);
            assert!(longest_title > 100);
        }),
    ];

    for (fixture, assertion) in edge_cases {
        let json = include_str!(fixture);
        let state = parse_complete_state(json).unwrap();
        assertion(state);
    }
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

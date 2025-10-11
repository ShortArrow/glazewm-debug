# Contributing to glazewm-debug

Welcome to glazewm-debug! This guide covers contributing to a **CLI+JSON based window manager debugging tool** following UNIX philosophy and modern development practices.

## Project Overview

glazewm-debug uses a **simplified architecture** that makes contributing straightforward:

- **JSON-First**: All data flows through well-defined JSON schemas
- **Platform-Agnostic Core**: Business logic works on any platform
- **Minimal Dependencies**: Easy to set up and test
- **Interface-Driven**: All external systems abstracted via traits

## Development Environment

### Prerequisites

**Required:**

- Rust 1.70.0+ with `rustfmt`, `clippy`
- glazewm 3.0.0+ (for integration testing)
- Git

### Quick Setup

```bash
git clone https://github.com/username/glazewm-debug.git
cd glazewm-debug
cargo test          # Verify environment
cargo run           # Start development
```

## Architectural Benefits for Contributors

### Simplified Testing

**JSON Fixture Testing:**

- Mock glazewm responses with JSON files
- Test domain logic without external dependencies
- Create complex scenarios declaratively

**Interface Mocking:**

- All I/O operations abstracted via traits
- Easy to mock terminal rendering, keyboard input, configuration
- Complete test isolation

### Platform-Independent Development

- Domain logic identical across platforms
- JSON parsing platform-agnostic
- Terminal UI cross-platform compatible
- No platform-specific APIs to handle

## Contributing Guidelines

### Code Standards

**UNIX Philosophy:**

- Do one thing well (glazewm state visualization)
- Compose with other tools (pipe-friendly)
- Simple, readable code

**Domain-Driven Design:**

- Business logic in `domain/` modules
- Clear entity boundaries (Monitor, Workspace, Window)
- Value objects for immutable data

**Test-Driven Development:**

- Write tests first for new features
- Test JSON schemas thoroughly
- Mock external dependencies

### Development Workflow

#### 1. Issue-Driven Development

**Before Starting:**

1. Check existing issues for duplicates
2. Create or comment on relevant issue
3. Discuss approach for significant features

#### 2. TDD Workflow

```bash
# 1. Write failing test
cargo test new_feature_test -- --nocapture

# 2. Implement minimal code to pass
# ... code changes ...

# 3. Refactor and improve
cargo clippy && cargo fmt

# 4. Verify all tests pass
cargo test
```

#### 3. Integration Testing

**Test with Real Data:**

```bash
# Create JSON fixtures from real glazewm
glazewm query monitors > tests/fixtures/monitors.json
glazewm query windows > tests/fixtures/windows.json

# Use in tests
#[test]
fn should_parse_real_response() {
    let json = include_str!("../fixtures/monitors.json");
    let result = parse_monitors(json);
    assert!(result.is_ok());
}
```

## Testable Design Strategy

### Interface-Driven Architecture

All external dependencies abstracted through trait interfaces:

**Key Mockable Components:**

- `WindowManagerClient` trait → Mock with JSON fixtures
- `Terminal` trait → Mock with output capture
- `EventSource` trait → Mock with scripted input sequences  
- `ConfigProvider` trait → Mock with test settings

**Testing Benefits:**

- **Builder Pattern**: Fluent APIs for complex test scenarios
- **JSON Fixtures**: Real glazewm responses for integration testing
- **Error Injection**: Mock failures for resilience testing
- **State Simulation**: Time-based lifecycle testing
- **Performance Validation**: Large dataset handling

### Testing Strategy Summary

**Test Categories:**

- **Unit**: Domain logic, JSON parsing, individual components
- **Integration**: Component interaction, mock-based workflows  
- **Behavior**: End-to-end user scenarios with full mock stack
- **Performance**: Large datasets, timing requirements
- **Edge Cases**: Boundary conditions, error recovery

**Key Principles:**

- Tests never require real glazewm installation
- Complete test isolation via dependency injection
- JSON-based mocks use real response formats
- Builder patterns for scenario creation
- Error scenarios as important as success paths

## Adding New Features

### Example Process: Window Focus History

**1. Write Domain Test:**

```rust
#[test]
fn should_track_window_focus_history() {
    let mut window = Window::new(/* ... */);
    window.record_focus();
    assert!(window.last_focus_time().is_some());
}
```

**2. Implement Domain Logic:**

```rust
impl Window {
    pub fn record_focus(&mut self) {
        self.focus_history.push(Instant::now());
    }
}
```

**3. Update JSON Schema:**

```rust
#[derive(Deserialize)]
struct RawWindow {
    #[serde(rename = "lastFocusTime")]
    last_focus_time: Option<String>,
    // ...
}
```

**4. Add Integration Test:**

```rust
#[test]
fn should_parse_focus_time_from_json() {
    let json = r#"{"lastFocusTime": "2023-12-01T10:30:00Z"}"#;
    let window = parse_window(json).unwrap();
    assert!(window.last_focus_time().is_some());
}
```

## Code Quality

### Standards

```bash
# Format and lint
cargo fmt
cargo clippy -- -D warnings

# Test execution
cargo test

# Documentation
cargo doc
```

### Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```text
feat(domain): add window focus history tracking
fix(cli): handle glazewm timeout gracefully  
docs(api): clarify JSON schema requirements
test(integration): add multi-monitor scenarios
```

## Pull Request Process

### Requirements

**Essential Elements:**

- Clear description of changes
- Tests for new functionality
- JSON schema compatibility verification
- Performance impact assessment

**Checklist:**

- [ ] Tests pass: `cargo test`
- [ ] Linting clean: `cargo clippy -- -D warnings`
- [ ] Formatted: `cargo fmt --check`
- [ ] Documentation updated (if needed)
- [ ] JSON schemas remain compatible

### Performance Guidelines

**Response Time Requirements:**

- CLI queries: < 100ms
- UI updates: < 50ms
- Test execution: < 10s for full suite

**Memory Usage:**

- Normal operation: < 50MB
- Large datasets: < 100MB
- No memory leaks over time

## Release Process

### Version Management

**Semantic Versioning:**

- **Major**: Breaking CLI interface or JSON schema changes
- **Minor**: New features, backward-compatible
- **Patch**: Bug fixes, no API changes

### Release Steps

1. Update `Cargo.toml` version
2. Update `CHANGELOG.md`  
3. Tag release: `git tag v1.0.0`
4. Build cross-platform binaries
5. Create GitHub release

## Community

### Getting Help

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and architecture discussions
- **Code Review**: PR feedback and design input

### Code of Conduct

- Be respectful and inclusive
- Focus on technical merit
- Help newcomers understand the CLI+JSON architecture
- Document architectural decisions

## Future Extensions

The CLI+JSON architecture makes several extensions straightforward:

**New Window Managers**: Only requires JSON schema mapping and CLI client implementation
**Output Formats**: JSON, CSV, tree formats via command-line flags
**Plugin System**: Runtime-loaded window manager clients
**Configuration**: TOML-based user preferences

## Related Documentation

- **[← Back to README](../README.md)** - Project overview
- **[Usage Guide](USAGE.md)** - CLI options and controls
- **[Display Format](DISPLAY.md)** - Output interpretation
- **[API Integration](API.md)** - glazewm JSON API details
- **[Architecture](ARCHITECTURE.md)** - Design principles and structure
- **[Building](BUILDING.md)** - Build instructions and environment

---

The CLI+JSON approach creates an ideal contributor experience: simple setup, comprehensive testing, and platform-agnostic development.

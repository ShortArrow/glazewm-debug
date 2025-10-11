# Architecture

This document describes the software architecture of glazewm-debug, focusing on design principles and strategic decisions.

## Overview

glazewm-debug follows **Domain-Driven Design (DDD)**, **Test-Driven Development (TDD)**, and **UNIX philosophy**. The **CLI+JSON architecture** eliminates platform-specific complexity while maintaining modularity and testability.

## Design Principles

### UNIX Philosophy

- **Do One Thing Well**: Focus exclusively on glazewm state visualization
- **Composition over Monoliths**: Small, focused components
- **Text-Based Interface**: Human-readable output
- **Tool Cooperation**: Pipe-friendly design

### Domain-Driven Design

- **Ubiquitous Language**: Code uses window management terminology
- **Bounded Contexts**: Clear boundaries between functionality areas
- **Rich Domain Model**: Business logic in domain entities
- **Anti-Corruption Layer**: Protection from external system changes

### Test-Driven Development

- **Test-First**: Tests drive interface design
- **Dependency Injection**: External dependencies injected for testability
- **Pure Functions**: Side-effect-free where possible
- **Behavior-Driven Tests**: Business-readable test descriptions

### CLI+JSON Architecture Benefits

- **Universal Interface**: JSON as platform-agnostic data contract
- **Schema Stability**: Well-defined JSON provides API stability
- **Testing Simplicity**: Easy test fixture creation
- **Extensibility**: New window managers via JSON schema mapping
- **Debugging**: Human-readable responses

## Component Architecture

### High-Level Structure

```text
┌─────────────────────────────────────────────────┐
│                   main.rs                       │
│          CLI Args + Composition Root            │
└─────────────┬───────────────────────────────────┘
              │
┌─────────────▼───────────────────────────────────┐
│              Application Layer                  │
│         State Management & Update Loop          │
└─────┬─────────────────────────────────────┬─────┘
      │                                     │
┌─────▼─────┐     ┌──────────┐     ┌────────▼─────┐
│ CLI Layer │────▶│   JSON   │────▶│   Domain     │
│(glazewm)  │     │ Parser   │     │    Layer     │
└───────────┘     └──────────┘     └──────┬───────┘
                                          │
                                   ┌──────▼───────┐
                                   │  TUI Layer   │
                                   │  (ratatui)   │
                                   └──────────────┘
```

### Layer Responsibilities

#### CLI Layer

- Execute `glazewm query` commands via `tokio::process`
- Handle command failures, timeouts, invalid responses
- Provide JSON validation and error recovery

#### JSON Parser Layer  

- Deserialize JSON to Rust structs using `serde`
- Validate response format and handle missing fields
- Convert external data to domain models

#### Domain Layer

- **Platform-agnostic** business logic for window management
- Entity definitions: Monitor, Workspace, Window
- Domain services: Layout calculation, focus management

#### Application Layer

- State management from JSON updates
- Periodic refresh via CLI queries (1-second polling)
- Event handling and data transformation

#### TUI Layer

- Display hierarchical window state using `ratatui`
- Handle keyboard input and navigation
- Calculate screen layouts for multi-monitor displays

## Module Structure

### Directory Layout

```text
src/
├── main.rs               # Bootstrap & CLI arguments
├── cli/                  # glazewm CLI client (platform-agnostic)
│   ├── client.rs        # Command execution via tokio::process
│   ├── types.rs         # glazewm JSON response types
│   └── parser.rs        # JSON deserialization & validation
├── domain/              # Core business logic (pure, platform-agnostic)
│   ├── monitor.rs      # Monitor aggregate root
│   ├── workspace.rs    # Workspace entity  
│   ├── window.rs       # Window entity
│   └── values.rs       # Value objects
├── app/                # Application coordination
│   ├── state.rs       # State management
│   ├── update.rs      # Update loop
│   └── events.rs      # Event handling
├── tui/               # Terminal interface (platform-agnostic)
│   ├── ui.rs         # UI rendering with ratatui
│   ├── input.rs      # Keyboard input
│   └── layout.rs     # Layout calculation
└── config.rs         # Configuration
```

**Dependency Rules:**

- Domain layer has **no external dependencies**
- Application layer depends **only on domain**  
- Infrastructure (CLI/TUI) can depend on domain and application
- **No circular dependencies**

## Data Flow

### Query Flow

```text
Timer → CLI Client → tokio::process → glazewm → JSON → serde → Domain → TUI → Display
```

**1-Second Update Cycle:**

1. Timer triggers state refresh
2. Execute `glazewm query monitors` and `glazewm query windows`
3. Parse JSON responses to domain models
4. Update application state
5. Render changes to terminal

### Error Flow

```text
CLI Error → Error Recovery → Fallback State → User-Friendly Message → TUI Display
```

**Error Scenarios:**

- **Command Not Found**: Display setup instructions
- **Invalid JSON**: Retry with exponential backoff
- **Timeout**: Kill process, show connection error
- **Permission Denied**: Display permission guidance

## Interface-Driven Design

### External System Abstractions

All I/O operations use trait interfaces for testability:

```rust
// Window manager communication
#[async_trait]
pub trait WindowManagerClient: Send + Sync {
    async fn query_monitors(&self) -> Result<Vec<Monitor>>;
    async fn query_windows(&self) -> Result<Vec<Window>>;
}

// Terminal rendering
pub trait Terminal: Send + Sync {
    fn draw(&mut self, content: &str) -> Result<()>;
    fn size(&self) -> Result<Size>;
}

// User input events
#[async_trait]
pub trait EventSource: Send + Sync {
    async fn next_event(&mut self) -> Result<Event>;
}
```

### Dependency Injection

```rust
// High-level modules depend on abstractions
pub struct App<WM, Term, Events> 
where
    WM: WindowManagerClient,
    Term: Terminal,
    Events: EventSource,
{
    window_manager: WM,
    terminal: Term,
    event_source: Events,
    state: ApplicationState,
}
```

## Testing Architecture

### Test Strategy

**Unit Tests**: Domain logic with pure functions
**Integration Tests**: Component interaction via mocks
**Behavior Tests**: End-to-end scenarios with full mock stack

### Mock Implementations

```rust
// JSON-based mocking
pub struct MockWindowManagerClient {
    json_fixtures: HashMap<String, String>,
}

// Terminal output capture  
pub struct MockTerminal {
    rendered_content: Vec<String>,
    terminal_size: Size,
}

// Scripted user input
pub struct MockEventSource {
    event_sequence: VecDeque<Event>,
}
```

### Test Data Strategy

- **JSON Fixtures**: Real glazewm responses for accuracy
- **Builder Pattern**: Fluent APIs for complex scenarios
- **Error Injection**: Mock failures for resilience testing
- **State Simulation**: Time-based lifecycle testing

## Performance Design

### Memory Management

- **Immutable Structures**: Prevent aliasing issues
- **Arc/Rc Sharing**: Efficient data sharing
- **Minimal Copying**: Reference-based operations

### Rendering Optimization

- **Dirty Checking**: Re-render only changed components
- **Layout Caching**: Reuse calculated layouts
- **Frame Rate Limiting**: Prevent excessive updates (1-second polling)

### CLI Optimization  

- **Parallel Queries**: Execute monitor/window queries concurrently
- **Response Caching**: Cache for sub-second repeat requests
- **Timeout Management**: Prevent hanging operations

## Error Handling

### Error Hierarchy

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("CLI communication failed")]
    Cli(#[from] CliError),
    
    #[error("Domain rule violation")] 
    Domain(#[from] DomainError),
    
    #[error("Terminal rendering failed")]
    Terminal(#[from] TerminalError),
}
```

### Recovery Strategy

- **Graceful Degradation**: Continue operation with partial data
- **Retry Logic**: Exponential backoff for transient failures
- **User Guidance**: Clear error messages with resolution steps
- **Fallback State**: Safe defaults when external systems fail

## Configuration

### Application Configuration

```rust
#[derive(Deserialize)]
pub struct Config {
    pub refresh_rate_ms: u64,    // Default: 1000
    pub glazewm_path: String,    // Default: "glazewm"
    pub timeout_ms: u64,         // Default: 5000
    pub quiet_mode: bool,        // Default: false
}
```

## Future Extensions

### Plugin Architecture (Planned)

```rust
pub trait Plugin {
    fn name(&self) -> &str;
    fn supports_window_manager(&self, wm_type: WindowManagerType) -> bool;
}

pub trait WindowManagerPlugin: Plugin {
    fn create_client(&self) -> Box<dyn WindowManagerClient>;
}
```

### Multi-Platform Support

The JSON-based architecture supports future expansion:

- **Linux**: i3, sway clients with same domain model
- **macOS**: yabai client with JSON schema mapping
- **Cross-Platform**: Identical core logic, different CLI clients

## Related Documentation

- **[← Back to README](../README.md)** - Project overview
- **[API Integration](API.md)** - glazewm JSON API details  
- **[Usage Guide](USAGE.md)** - CLI options and controls
- **[Building](BUILDING.md)** - Build and setup
- **[Contributing](CONTRIBUTE.md)** - Development workflow

## References

- [Domain-Driven Design](https://domainlanguage.com/ddd/) - Eric Evans
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html) - Robert Martin  
- [The Art of Unix Programming](http://www.catb.org/~esr/writings/taoup/) - Eric S. Raymond

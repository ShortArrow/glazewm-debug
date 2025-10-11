# Architecture

This article describes the software architecture of glazewm-debug, including design principles, component structure, and implementation patterns.

## Overview

glazewm-debug is designed following Domain-Driven Design (DDD) principles, Test-Driven Development (TDD) practices, and UNIX philosophy. The **CLI+JSON architecture** eliminates platform-specific complexity while maintaining modularity, testability, and clear separation of concerns.

**Architectural Innovation:**
The decision to use JSON as the universal interface creates a **platform-agnostic core** that can easily extend to other window managers and operating systems without code changes to the domain logic or UI layers.

## Design Principles

### UNIX Philosophy

The architecture adheres to classic UNIX principles:

- **Do One Thing Well**: Focus exclusively on glazewm state visualization
- **Composition over Monoliths**: Small, focused components that can be combined
- **Text-Based Interface**: All output is human-readable text
- **Tool Cooperation**: Designed to work with other UNIX tools via pipes and redirection

### Domain-Driven Design

The codebase is organized around business concepts rather than technical layers:

- **Ubiquitous Language**: Code uses terminology from the window management domain
- **Bounded Contexts**: Clear boundaries between different areas of functionality
- **Rich Domain Model**: Business logic encapsulated in domain entities
- **Anti-Corruption Layer**: Protection from external system changes

### Test-Driven Development

Code structure supports comprehensive testing:

- **Test-First Development**: Tests drive the design of interfaces
- **Dependency Injection**: All external dependencies are injected for testability
- **Pure Functions**: Emphasis on side-effect-free functions where possible
- **Behavior-Driven Tests**: Tests describe behavior in business terms

### JSON-First Architecture

The CLI+JSON approach provides unique architectural benefits:

- **Universal Interface**: JSON serves as a platform-agnostic data contract
- **Schema Stability**: Well-defined JSON schemas provide API stability
- **Testing Simplicity**: Easy to create test fixtures with JSON files
- **Extensibility**: New window managers only require JSON schema mapping
- **Debugging**: Human-readable JSON responses aid development and troubleshooting

### Interface-Driven Design

All external dependencies are abstracted through **trait interfaces** for maximum testability:

#### External System Interfaces

**Window Manager Client Interface:**

```rust
#[async_trait]
pub trait WindowManagerClient: Send + Sync {
    async fn query_monitors(&self) -> Result<Vec<Monitor>, ClientError>;
    async fn query_windows(&self) -> Result<Vec<Window>, ClientError>;
    async fn is_available(&self) -> bool;
}
```

**Terminal Interface:**

```rust
pub trait Terminal: Send + Sync {
    fn size(&self) -> Result<Size, TerminalError>;
    fn clear(&mut self) -> Result<(), TerminalError>;
    fn flush(&mut self) -> Result<(), TerminalError>;
    fn draw(&mut self, content: &str) -> Result<(), TerminalError>;
}
```

**Event Source Interface:**

```rust
#[async_trait] 
pub trait EventSource: Send + Sync {
    async fn next_event(&mut self) -> Result<Event, EventError>;
    fn has_events(&self) -> bool;
}

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyCode),
    Resize(Size),
    Timer,
    Quit,
}
```

**Configuration Interface:**

```rust
pub trait ConfigProvider: Send + Sync {
    fn get_refresh_rate(&self) -> u64;
    fn get_glazewm_path(&self) -> &str;
    fn is_quiet_mode(&self) -> bool;
    fn reload(&mut self) -> Result<(), ConfigError>;
}
```

## Component Architecture

### High-Level Structure

```mermaid
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

- **Command Execution**: Execute `glazewm query` commands via `tokio::process`
- **Response Handling**: Capture stdout/stderr from glazewm processes
- **Error Management**: Handle command failures, timeouts, and invalid responses
- **JSON Validation**: Ensure response format matches expected schema

#### JSON Parser Layer

- **Deserialization**: Convert JSON strings to Rust structs using `serde`
- **Schema Validation**: Ensure JSON matches expected glazewm API format
- **Error Recovery**: Handle malformed JSON and missing fields gracefully
- **Type Safety**: Provide strongly-typed data to domain layer

#### Domain Layer

- **Core Business Logic**: Window management concepts and rules (platform-agnostic)
- **Entity Definitions**: Monitor, Workspace, Window aggregates
- **Value Objects**: Immutable data structures (WindowId, Position, Size)
- **Domain Services**: Layout calculation, focus management (JSON-input based)

#### Application Layer

- **State Management**: Maintain current glazewm state from JSON updates
- **Update Loop**: Periodically refresh state via CLI queries
- **Event Handling**: Process user input and system events
- **Data Transformation**: Convert domain models to UI representations

#### TUI Layer

- **Rendering**: Display hierarchical window state using `ratatui`
- **Input Handling**: Process keyboard commands and navigation
- **Layout Management**: Calculate screen layouts for multi-monitor displays
- **Theme Support**: Apply visual styling to displayed information

## Module Structure

### Directory Layout

```text
src/
├── main.rs               # Application bootstrap & CLI argument parsing
├── cli/                  # glazewm CLI client (platform-agnostic)
│   ├── mod.rs           # CLI module exports
│   ├── client.rs        # Command execution via tokio::process
│   ├── types.rs         # glazewm JSON response types
│   └── parser.rs        # JSON deserialization & validation
├── domain/              # Core business logic (pure, platform-agnostic)
│   ├── mod.rs          # Domain module exports
│   ├── monitor.rs      # Monitor aggregate root
│   ├── workspace.rs    # Workspace entity
│   ├── window.rs       # Window entity
│   └── values.rs       # Value objects (Position, Size, WindowId, etc.)
├── app/                # Application state & coordination
│   ├── mod.rs         # Application exports
│   ├── state.rs       # Application state management
│   ├── update.rs      # State update loop
│   └── events.rs      # Event handling
├── tui/               # Terminal user interface (platform-agnostic)
│   ├── mod.rs        # TUI module exports
│   ├── ui.rs         # UI rendering with ratatui
│   ├── input.rs      # Keyboard input handling
│   └── layout.rs     # Screen layout calculation
└── config.rs         # Configuration management
```

**Architecture Benefits:**

- **Simplified Dependencies**: Each module has minimal, clear dependencies
- **Platform Agnostic**: Core logic works on any platform with JSON support
- **Easy Testing**: Each layer can be tested independently with JSON fixtures
- **Future Extension**: Adding new window managers only requires CLI client changes

### Module Dependencies

```text
main.rs
├── application (commands, queries, services)
│   └── domain (entities, values, services, events)
└── infrastructure
    ├── domain (for data mapping)
    └── application (for service implementation)
```

**Dependency Rules:**

- Domain layer has no dependencies on other layers
- Application layer depends only on domain
- Infrastructure layer can depend on domain and application
- No circular dependencies between modules

## Data Flow

### CLI Query Flow

```mermaid
Timer Trigger → CLI Client → tokio::process::Command → glazewm query → 
JSON Response → serde Parser → Domain Entities → TUI Render → Display
```

**Detailed Steps:**

1. **Timer Event**: Application timer triggers state refresh (1 second interval)
2. **Command Execution**: `tokio::process::Command::new("glazewm").args(["query", "monitors"])`
3. **JSON Capture**: Capture stdout containing JSON response
4. **Deserialization**: `serde_json::from_str::<MonitorResponse>(&json)`
5. **Domain Mapping**: Convert glazewm JSON to internal domain models
6. **UI Update**: `ratatui` renders updated state to terminal

### Error Flow

```mermaid
CLI Error → Command Failure → Error Recovery → Fallback State → UI Error Display
```

**Error Scenarios:**

- **Command Not Found**: glazewm not in PATH → Display setup instructions
- **Invalid JSON**: Malformed response → Log error, retry with exponential backoff
- **Timeout**: Command hangs → Kill process, display connection error
- **Permission Denied**: Access issues → Display permission guidance

### User Input Flow

```mermaid
Keyboard Input → crossterm Event → TUI Handler → Application State → 
UI Re-render → Display Update
```

## Domain Model

### Core Entities

#### Monitor Aggregate

```rust
pub struct Monitor {
    id: MonitorId,
    geometry: Rectangle,
    workspaces: Vec<Workspace>,
    focus_state: FocusState,
    display_properties: DisplayProperties,
}

impl Monitor {
    pub fn add_workspace(&mut self, workspace: Workspace) -> Result<(), DomainError>;
    pub fn remove_workspace(&mut self, id: WorkspaceId) -> Result<Workspace, DomainError>;
    pub fn focused_workspace(&self) -> Option<&Workspace>;
    pub fn total_window_count(&self) -> usize;
}
```

#### Workspace Entity

```rust
pub struct Workspace {
    id: WorkspaceId,
    name: WorkspaceName,
    windows: Vec<Window>,
    tiling_direction: TilingDirection,
    focus_state: FocusState,
    display_state: DisplayState,
}

impl Workspace {
    pub fn add_window(&mut self, window: Window) -> Result<(), DomainError>;
    pub fn remove_window(&mut self, id: WindowId) -> Result<Window, DomainError>;
    pub fn focused_window(&self) -> Option<&Window>;
    pub fn calculate_layout(&self) -> Vec<WindowLayout>;
}
```

#### Window Entity

```rust
pub struct Window {
    id: WindowId,
    title: WindowTitle,
    process_name: ProcessName,
    geometry: Rectangle,
    state: WindowState,
    focus_state: FocusState,
    tiling_properties: TilingProperties,
}

impl Window {
    pub fn resize(&mut self, new_size: Size) -> Result<(), DomainError>;
    pub fn change_state(&mut self, new_state: WindowState) -> Result<(), DomainError>;
    pub fn is_visible(&self) -> bool;
    pub fn display_name(&self) -> String;
}
```

### Value Objects

#### Geometry

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle {
    pub position: Position,
    pub size: Size,
}
```

#### Identifiers

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MonitorId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkspaceId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WindowId(String);
```

### Domain Services

#### Layout Service

```rust
pub struct LayoutService;

impl LayoutService {
    pub fn calculate_workspace_layout(
        workspace: &Workspace,
        container_size: Size,
    ) -> Vec<WindowLayout>;
    
    pub fn calculate_monitor_layout(
        monitors: &[Monitor],
        screen_size: Size,
    ) -> Vec<MonitorLayout>;
}
```

## Dependency Injection & Testability

### Interface-Driven Design Pattern

All external dependencies are abstracted through **trait interfaces** for maximum testability and following the **Dependency Inversion Principle**:

```rust
// High-level App depends on abstractions (traits), not concretions
pub struct App<WM, Term, Events, Cfg> 
where
    WM: WindowManagerClient,
    Term: Terminal,
    Events: EventSource,
    Cfg: ConfigProvider,
{
    window_manager: WM,
    terminal: Term,
    event_source: Events,
    config: Cfg,
    // ...
}
```

### Mock Implementations for Testing

#### MockWindowManagerClient

```rust
pub struct MockWindowManagerClient {
    responses: HashMap<String, String>, // JSON responses
    call_history: Vec<String>,          // Track calls
    failure_mode: bool,
    delay_ms: u64,
}

impl MockWindowManagerClient {
    pub fn with_json_responses() -> Self {
        let mut responses = HashMap::new();
        responses.insert(
            "query monitors".to_string(),
            include_str!("../test_fixtures/monitors.json").to_string()
        );
        responses.insert(
            "query windows".to_string(), 
            include_str!("../test_fixtures/windows.json").to_string()
        );
        
        Self {
            responses,
            call_history: Vec::new(),
            failure_mode: false,
            delay_ms: 0,
        }
    }
    
    pub fn simulate_failure(mut self) -> Self {
        self.failure_mode = true;
        self
    }
    
    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }
    
    pub fn call_history(&self) -> &[String] {
        &self.call_history
    }
}
```

#### MockTerminal for UI Testing

```rust
pub struct MockTerminal {
    buffer: String,
    size: Size,
    draw_history: Vec<String>,
    cursor_position: Position,
}

impl MockTerminal {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            buffer: String::new(),
            size: Size::new(width as u32, height as u32),
            draw_history: Vec::new(),
            cursor_position: Position::new(0, 0),
        }
    }
    
    // Test helpers
    pub fn last_drawn_content(&self) -> Option<&String> {
        self.draw_history.last()
    }
    
    pub fn contains_text(&self, text: &str) -> bool {
        self.buffer.contains(text)
    }
    
    pub fn draw_call_count(&self) -> usize {
        self.draw_history.len()
    }
}
```

#### MockEventSource for User Input Testing

```rust
pub struct MockEventSource {
    event_queue: VecDeque<Event>,
    auto_events: bool,
}

impl MockEventSource {
    pub fn with_events(events: Vec<Event>) -> Self {
        Self {
            event_queue: events.into(),
            auto_events: false,
        }
    }
    
    pub fn simulate_user_session() -> Self {
        Self::with_events(vec![
            Event::Key(KeyCode::Char('r')), // Refresh
            Event::Timer,                   // Auto refresh
            Event::Key(KeyCode::Char('q')), // Quit
        ])
    }
    
    pub fn simulate_navigation() -> Self {
        Self::with_events(vec![
            Event::Key(KeyCode::Up),        // Navigate up
            Event::Key(KeyCode::Down),      // Navigate down
            Event::Key(KeyCode::Enter),     // Select
            Event::Key(KeyCode::Escape),    // Back
        ])
    }
}
```

### Test Scenarios with Mocks

**Application State Management Test:**

```rust
#[tokio::test]
async fn should_update_state_when_glazewm_changes() {
    // Given
    let initial_monitor = MonitorTestBuilder::new()
        .with_workspace(WorkspaceTestBuilder::new().with_window_count(1).build())
        .build();
        
    let updated_monitor = MonitorTestBuilder::new() 
        .with_workspace(WorkspaceTestBuilder::new().with_window_count(3).build())
        .build();

    let mock_client = MockWindowManagerClient::new()
        .first_call_returns(vec![initial_monitor])
        .subsequent_calls_return(vec![updated_monitor]);
        
    let mock_terminal = MockTerminal::new(80, 24);
    let mock_events = MockEventSource::new();
    let mock_config = MockConfig::default();

    let mut app = App::new(mock_client, mock_terminal, mock_events, mock_config);

    // When
    app.update_state().await.unwrap();  // First update
    let initial_count = app.state().total_window_count();
    
    app.update_state().await.unwrap();  // Second update  
    let updated_count = app.state().total_window_count();

    // Then
    assert_eq!(initial_count, 1);
    assert_eq!(updated_count, 3);
    assert_eq!(mock_client.call_count(), 2);
}
```

**User Input Handling Test:**

```rust
#[tokio::test]
async fn should_handle_keyboard_navigation() {
    // Given
    let monitor = MonitorTestBuilder::new()
        .with_workspace(WorkspaceTestBuilder::new().with_name("WS1").build())
        .with_workspace(WorkspaceTestBuilder::new().with_name("WS2").focused().build())
        .build();

    let mock_client = MockWindowManagerClient::with_state(vec![monitor], vec![]);
    let mock_terminal = MockTerminal::new(80, 24);
    let mock_events = MockEventSource::with_events(vec![
        Event::Key(KeyCode::Up),   // Navigate to WS1
        Event::Key(KeyCode::Down), // Navigate back to WS2
        Event::Key(KeyCode::Char('q')), // Quit
    ]);
    let mock_config = MockConfig::default();

    let mut app = App::new(mock_client, mock_terminal, mock_events, mock_config);

    // When
    app.run_until_quit().await.unwrap();

    // Then
    let terminal_content = mock_terminal.last_drawn_content().unwrap();
    assert!(terminal_content.contains("WS2")); // Final state
    assert_eq!(mock_events.remaining_events(), 0); // All events processed
}
```

## CLI Integration Implementation

### glazewm CLI Client

#### Simple Command Execution

```rust
pub struct GlazewmClient {
    command_path: PathBuf,
    timeout: Duration,
}

impl GlazewmClient {
    pub async fn query_monitors(&self) -> Result<Vec<Monitor>, ClientError> {
        let json_response = self.execute_command(&["query", "monitors"]).await?;
        let glazewm_response: GlazewmMonitorResponse = serde_json::from_str(&json_response)?;
        Ok(glazewm_response.data.monitors.into_iter().map(Monitor::from).collect())
    }
    
    pub async fn query_windows(&self) -> Result<Vec<Window>, ClientError> {
        let json_response = self.execute_command(&["query", "windows"]).await?;
        let glazewm_response: GlazewmWindowResponse = serde_json::from_str(&json_response)?;
        Ok(glazewm_response.data.windows.into_iter().map(Window::from).collect())
    }
    
    async fn execute_command(&self, args: &[&str]) -> Result<String, ClientError> {
        let output = tokio::process::Command::new(&self.command_path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?
            .wait_with_output()
            .await?;
            
        if !output.status.success() {
            return Err(ClientError::CommandFailed {
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }
        
        Ok(String::from_utf8(output.stdout)?)
    }
}
```

#### JSON Response Types

```rust
#[derive(Debug, Deserialize)]
pub struct GlazewmMonitorResponse {
    pub data: MonitorData,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MonitorData {
    pub monitors: Vec<RawMonitor>,
}

#[derive(Debug, Deserialize)]
pub struct RawMonitor {
    pub id: String,
    pub width: i32,
    pub height: i32,
    #[serde(rename = "hasFocus")]
    pub has_focus: bool,
    pub children: Vec<RawWorkspace>,
}
```

#### Domain Conversion

```rust
impl From<RawMonitor> for Monitor {
    fn from(raw: RawMonitor) -> Self {
        Monitor::new(
            MonitorId::new(raw.id),
            Size::new(raw.width as u32, raw.height as u32),
            raw.has_focus,
            raw.children.into_iter().map(Workspace::from).collect(),
        )
    }
}
```

### TUI Implementation

#### Rendering Architecture

```rust
pub trait Renderer {
    fn render(&self, state: &ApplicationState) -> Result<(), RenderError>;
}

pub struct TuiRenderer {
    backend: Box<dyn Backend>,
    layout_calculator: LayoutCalculator,
    theme: Theme,
}

impl Renderer for TuiRenderer {
    fn render(&self, state: &ApplicationState) -> Result<(), RenderError> {
        let layout = self.layout_calculator.calculate(state);
        
        for monitor in &state.monitors {
            self.render_monitor(monitor, &layout)?;
        }
        
        Ok(())
    }
}
```

## Testing Architecture

### Test Structure

```text
tests/
├── unit/              # Fast, isolated tests
│   ├── domain/        # Domain logic tests
│   ├── application/   # Use case tests
│   └── infrastructure/ # Infrastructure tests
├── integration/       # Component interaction tests
│   ├── glazewm/       # glazewm integration tests
│   └── end_to_end/    # Full system tests
└── behavior/          # BDD-style feature tests
    └── features/      # Gherkin-style scenarios
```

### Test Doubles

#### Mocks and Stubs

```rust
pub struct MockWindowManagerClient {
    monitors: Vec<MonitorData>,
    windows: Vec<WindowData>,
    call_count: AtomicUsize,
}

#[async_trait]
impl WindowManagerClient for MockWindowManagerClient {
    async fn query_monitors(&self) -> Result<Vec<MonitorData>, ClientError> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        Ok(self.monitors.clone())
    }
}

// Test builders for complex scenarios
pub struct MonitorTestBuilder {
    id: String,
    workspaces: Vec<WorkspaceTestBuilder>,
    focused: bool,
}

impl MonitorTestBuilder {
    pub fn with_id(id: &str) -> Self;
    pub fn with_workspace(mut self, workspace: WorkspaceTestBuilder) -> Self;
    pub fn focused(mut self) -> Self;
    pub fn build(self) -> Monitor;
}
```

### Behavior-Driven Tests

```rust
#[cfg(test)]
mod monitor_behavior {
    use super::*;
    
    #[test]
    fn should_return_focused_workspace_when_workspace_has_focus() {
        // Given
        let monitor = MonitorTestBuilder::new()
            .with_workspace(
                WorkspaceTestBuilder::new()
                    .with_name("Workspace 1")
                    .focused()
            )
            .with_workspace(
                WorkspaceTestBuilder::new()
                    .with_name("Workspace 2")
            )
            .build();
        
        // When
        let focused = monitor.focused_workspace();
        
        // Then
        assert!(focused.is_some());
        assert_eq!(focused.unwrap().name(), "Workspace 1");
    }
}
```

## Error Handling

### Error Hierarchy

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),
    
    #[error("Infrastructure error: {0}")]
    Infrastructure(#[from] InfrastructureError),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
}

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Invalid window state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },
    
    #[error("Window {id} not found in workspace")]
    WindowNotFound { id: WindowId },
    
    #[error("Workspace capacity exceeded: maximum {max}, attempted {attempted}")]
    WorkspaceCapacityExceeded { max: usize, attempted: usize },
}

#[derive(Debug, thiserror::Error)]
pub enum InfrastructureError {
    #[error("glazewm connection failed: {0}")]
    GlazewmConnection(String),
    
    #[error("JSON parsing failed: {0}")]
    JsonParsing(#[from] serde_json::Error),
    
    #[error("Terminal rendering failed: {0}")]
    TerminalRendering(String),
}
```

### Error Recovery

```rust
pub struct ErrorRecoveryService {
    retry_policy: RetryPolicy,
    fallback_state: ApplicationState,
}

impl ErrorRecoveryService {
    pub async fn handle_glazewm_error(
        &self,
        error: GlazewmError,
    ) -> Result<ApplicationState, ApplicationError> {
        match error {
            GlazewmError::ConnectionTimeout => {
                self.retry_with_backoff().await
            }
            GlazewmError::ProcessNotFound => {
                Ok(self.fallback_state.clone())
            }
            GlazewmError::InvalidResponse => {
                Err(ApplicationError::Infrastructure(
                    InfrastructureError::GlazewmConnection(
                        "Invalid response from glazewm".to_string()
                    )
                ))
            }
        }
    }
}
```

## Performance Considerations

### Memory Management

- **Immutable Data Structures**: Prefer immutable data to avoid aliasing issues
- **Copy-on-Write**: Use `Arc` and `Cow` for shared data
- **Pool Allocation**: Reuse expensive objects where possible
- **Stack Allocation**: Prefer stack allocation for small, short-lived objects

### Rendering Performance

- **Dirty Checking**: Only re-render changed components
- **Virtual Rendering**: Calculate layout off-screen before display
- **Incremental Updates**: Update only changed screen regions
- **Frame Rate Limiting**: Prevent excessive refresh rates

### Data Flow Optimization

```rust
pub struct StateCache {
    current_state: Arc<ApplicationState>,
    previous_state: Option<Arc<ApplicationState>>,
    dirty_regions: Vec<ScreenRegion>,
}

impl StateCache {
    pub fn update(&mut self, new_state: ApplicationState) {
        if let Some(prev) = &self.previous_state {
            self.dirty_regions = self.calculate_diff(prev, &new_state);
        }
        
        self.previous_state = Some(self.current_state.clone());
        self.current_state = Arc::new(new_state);
    }
    
    pub fn needs_full_refresh(&self) -> bool {
        self.previous_state.is_none()
    }
}
```

## Configuration

### Application Configuration

```rust
#[derive(Debug, Deserialize)]
pub struct ApplicationConfig {
    pub refresh_rate_ms: u64,
    pub theme: ThemeConfig,
    pub layout: LayoutConfig,
    pub glazewm: GlazewmConfig,
}

#[derive(Debug, Deserialize)]
pub struct GlazewmConfig {
    pub command_path: Option<PathBuf>,
    pub timeout_ms: u64,
    pub retry_attempts: u32,
}

impl ApplicationConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError>;
    pub fn load_with_defaults() -> Self;
    pub fn validate(&self) -> Result<(), ValidationError>;
}
```

## Future Extensions

### Plugin Architecture

The architecture supports future plugin development:

```rust
pub trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self, context: &PluginContext) -> Result<(), PluginError>;
}

pub trait RenderPlugin: Plugin {
    fn render_monitor(&self, monitor: &Monitor, context: &RenderContext) -> Result<(), RenderError>;
    fn render_workspace(&self, workspace: &Workspace, context: &RenderContext) -> Result<(), RenderError>;
}
```

### Multi-Platform Support

While currently Windows-only, the architecture supports future platform expansion:

```rust
pub trait PlatformAdapter {
    type WindowManager: WindowManagerClient;
    
    fn detect_window_manager(&self) -> Result<Self::WindowManager, PlatformError>;
    fn create_terminal_backend(&self) -> Result<Box<dyn Backend>, PlatformError>;
}

pub struct WindowsPlatform;
impl PlatformAdapter for WindowsPlatform {
    type WindowManager = GlazewmClient;
    // Implementation...
}
```

## Related Documentation

- **[← Back to README](../README.md)** - Project overview and quick start
- **[Usage Guide](USAGE.md)** - CLI options and keyboard controls
- **[Display Format](DISPLAY.md)** - Output formats and visual indicators
- **[API Integration](API.md)** - glazewm JSON API details
- **[Building](BUILDING.md)** - Build instructions and setup
- **[Contributing](CONTRIBUTE.md)** - Development guidelines

## External References

- [Domain-Driven Design](https://domainlanguage.com/ddd/) - Eric Evans
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html) - Robert Martin  
- [The Art of Unix Programming](http://www.catb.org/~esr/writings/taoup/) - Eric S. Raymond
- [Test-Driven Development](https://www.oreilly.com/library/view/test-driven-development/0321146530/) - Kent Beck
- [Behavior-Driven Development](https://dannorth.net/introducing-bdd/) - Dan North

# glazewm API Integration

This document describes how glazewm-debug integrates with glazewm through its command-line JSON API, including response schemas, error handling, and data mapping.

## Integration Approach

### CLI+JSON Strategy

glazewm-debug uses a **CLI-first approach** for maximum simplicity and reliability:

```mermaid
glazewm-debug → tokio::process::Command → glazewm query → JSON Response → serde Parse
```

**Benefits of This Approach:**

- **Zero Dependencies**: No IPC libraries or native bindings
- **Version Independence**: Works with any glazewm version supporting JSON output
- **Process Isolation**: glazewm-debug cannot interfere with glazewm operation
- **Platform Agnostic**: JSON parsing identical across platforms
- **Easy Testing**: Simple to mock with JSON fixtures

## glazewm Commands

### Monitor State Query

**Command:**

```bash
glazewm query monitors
```

**Purpose:** Retrieve complete monitor hierarchy including workspaces and windows

**Response Time:** ~10-50ms typical

**Full Response Schema:**

```json
{
  "clientMessage": "query monitors",
  "data": {
    "monitors": [
      {
        "type": "monitor",
        "id": "41e706e9-9e2b-4882-aa78-a2b05fb4bd4d",
        "parentId": "1cb13c15-8bd1-4be0-8141-aa60d6179035",
        "children": [
          {
            "type": "workspace",
            "id": "ff6875d3-21a7-44c3-830f-ece20d73db94",
            "name": "1",
            "displayName": "1",
            "parentId": "41e706e9-9e2b-4882-aa78-a2b05fb4bd4d",
            "hasFocus": false,
            "isDisplayed": true,
            "width": 1880,
            "height": 952,
            "x": 20,
            "y": 60,
            "tilingDirection": "horizontal",
            "children": [
              {
                "type": "window",
                "id": "91be2ee1-50e8-40f6-a567-85cf87d59a6a",
                "parentId": "ff6875d3-21a7-44c3-830f-ece20d73db94",
                "hasFocus": true,  // This window currently has keyboard focus
                "tilingSize": 1.0,
                "width": 1880,
                "height": 952,
                "x": 20,
                "y": 60,
                "state": {"type": "tiling"},
                "displayState": "shown",
                "title": "Stack Overflow - Google Chrome",
                "className": "Chrome_WidgetWin_1",
                "processName": "chrome"
              }
            ]
          }
        ],
        "hasFocus": true,
        "width": 1920,
        "height": 1080,
        "x": 0,
        "y": 0,
        "dpi": 96,
        "scaleFactor": 1.0,
        "deviceName": "\\\\.\\DISPLAY1"
      }
    ]
  },
  "error": null,
  "success": true
}
```

### Window State Query

**Command:**

```bash
glazewm query windows
```

**Purpose:** Retrieve flat list of all windows with detailed properties

**Response Time:** ~5-30ms typical

**Focus State Behavior:**

- Only **one window system-wide** will have `"hasFocus": true`
- All other windows will have `"hasFocus": false`
- Focus follows Windows system focus (Alt+Tab, mouse clicks, etc.)
- Focus state is independent of workspace/monitor active states

**Simplified Response Schema:**

```json
{
  "clientMessage": "query windows",
  "data": {
    "windows": [
      {
        "type": "window",
        "id": "string",
        "parentId": "string",
        "hasFocus": boolean,     // true = currently focused window (only one per system)
        "tilingSize": number | null,
        "width": number,
        "height": number,
        "x": number,
        "y": number,
        "state": {
          "type": "tiling" | "floating" | "minimized"
        },
        "displayState": "shown" | "hiding" | "hidden",
        "title": "string",
        "className": "string",
        "processName": "string"
      }
    ]
  },
  "error": null,
  "success": boolean
}
```

## Focus State Hierarchy

glazewm provides a **multi-level focus system** that glazewm-debug visualizes:

### System-Wide Window Focus

- **Single Active Window**: Only one window has `"hasFocus": true` at any time
- **Global State**: This represents the window receiving keyboard input
- **Windows Focus Model**: Follows standard Windows focus behavior

### Workspace Active State  

- **Per-Monitor Active Workspace**: Each monitor has one active workspace
- **Workspace Visibility**: Active workspace is currently displayed
- **Independent of Window Focus**: A workspace can be active without containing the focused window

### Monitor Active State

- **Single Active Monitor**: Only one monitor has `"hasFocus": true`
- **Display Focus**: The monitor receiving new windows by default
- **User Attention**: Where new applications will open

### Focus Relationship Example

```json
{
  "monitors": [
    {
      "id": "monitor-1",
      "hasFocus": true,           // ← Active monitor
      "children": [
        {
          "id": "workspace-1", 
          "hasFocus": false,      // ← Not active workspace on this monitor
          "children": []
        },
        {
          "id": "workspace-2",
          "hasFocus": true,       // ← Active workspace on this monitor
          "children": [
            {
              "id": "window-1",
              "hasFocus": false,  // ← Not focused
              "title": "VS Code"
            },
            {
              "id": "window-2", 
              "hasFocus": true,   // ← Currently focused window (keyboard input)
              "title": "Chrome"
            }
          ]
        }
      ]
    },
    {
      "id": "monitor-2",
      "hasFocus": false,          // ← Inactive monitor
      "children": [
        {
          "id": "workspace-3",
          "hasFocus": true,       // ← Active workspace on monitor 2
          "children": [
            {
              "id": "window-3",
              "hasFocus": false,  // ← Not focused (focused window is on monitor-1)
              "title": "Discord"
            }
          ]
        }
      ]
    }
  ]
}
```

**Key Insights:**

- Window focus can be on a different monitor than the active monitor
- Active workspaces exist per-monitor, but window focus is system-wide
- glazewm tracks both logical (workspace) and physical (window) focus states

## Data Mapping

### Rust Type Definitions

#### Raw glazewm Types (CLI Layer)

```rust
#[derive(Debug, Deserialize)]
pub struct GlazewmResponse<T> {
    #[serde(rename = "clientMessage")]
    pub client_message: String,
    pub data: T,
    pub error: Option<String>,
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct MonitorData {
    pub monitors: Vec<RawMonitor>,
}

#[derive(Debug, Deserialize)]
pub struct RawMonitor {
    #[serde(rename = "type")]
    pub node_type: String,
    pub id: String,
    #[serde(rename = "parentId")]
    pub parent_id: String,
    pub children: Vec<RawWorkspace>,
    #[serde(rename = "hasFocus")]
    pub has_focus: bool,
    pub width: i32,
    pub height: i32,
    pub x: i32,
    pub y: i32,
    pub dpi: i32,
    #[serde(rename = "scaleFactor")]
    pub scale_factor: f64,
    #[serde(rename = "deviceName")]
    pub device_name: String,
}

#[derive(Debug, Deserialize)]
pub struct RawWorkspace {
    #[serde(rename = "type")]
    pub node_type: String,
    pub id: String,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "parentId")]
    pub parent_id: String,
    #[serde(rename = "hasFocus")]
    pub has_focus: bool,
    #[serde(rename = "isDisplayed")]
    pub is_displayed: bool,
    pub width: i32,
    pub height: i32,
    pub x: i32,
    pub y: i32,
    #[serde(rename = "tilingDirection")]
    pub tiling_direction: String,
    pub children: Vec<RawWindow>,
}

#[derive(Debug, Deserialize)]
pub struct RawWindow {
    #[serde(rename = "type")]
    pub node_type: String,
    pub id: String,
    #[serde(rename = "parentId")]
    pub parent_id: String,
    #[serde(rename = "hasFocus")]
    pub has_focus: bool,
    #[serde(rename = "tilingSize")]
    pub tiling_size: Option<f64>,
    pub width: i32,
    pub height: i32,
    pub x: i32,
    pub y: i32,
    pub state: WindowState,
    #[serde(rename = "displayState")]
    pub display_state: String,
    pub title: String,
    #[serde(rename = "className")]
    pub class_name: String,
    #[serde(rename = "processName")]
    pub process_name: String,
}

#[derive(Debug, Deserialize)]
pub struct WindowState {
    #[serde(rename = "type")]
    pub state_type: String,
}
```

#### Domain Types (Domain Layer)

```rust
#[derive(Debug, Clone)]
pub struct Monitor {
    pub id: MonitorId,
    pub geometry: Rectangle,
    pub workspaces: Vec<Workspace>,
    pub focus_state: FocusState,
    pub device_info: DeviceInfo,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub display_name: Option<String>,
    pub geometry: Rectangle,
    pub windows: Vec<Window>,
    pub tiling_direction: TilingDirection,
    pub focus_state: FocusState,
    pub display_state: DisplayState,
}

#[derive(Debug, Clone)]
pub struct Window {
    pub id: WindowId,
    pub title: String,
    pub process_name: String,
    pub class_name: String,
    pub geometry: Rectangle,
    pub tiling_size: Option<f64>,
    pub state: WindowState,
    pub display_state: DisplayState,
    pub focus_state: FocusState,
}
```

### Type Conversion

#### Monitor Conversion

```rust
impl From<RawMonitor> for Monitor {
    fn from(raw: RawMonitor) -> Self {
        Monitor {
            id: MonitorId::new(raw.id),
            geometry: Rectangle::new(
                Position::new(raw.x, raw.y),
                Size::new(raw.width as u32, raw.height as u32),
            ),
            workspaces: raw.children.into_iter().map(Workspace::from).collect(),
            focus_state: FocusState::from(raw.has_focus),
            device_info: DeviceInfo {
                device_name: raw.device_name,
                dpi: raw.dpi,
                scale_factor: raw.scale_factor,
            },
        }
    }
}
```

#### Window State Mapping

```rust
impl From<&str> for WindowState {
    fn from(state_str: &str) -> Self {
        match state_str {
            "tiling" => WindowState::Tiling,
            "floating" => WindowState::Floating,
            "minimized" => WindowState::Minimized,
            _ => WindowState::Unknown(state_str.to_string()),
        }
    }
}
```

## Error Handling

### CLI Execution Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("glazewm command not found in PATH")]
    CommandNotFound,
    
    #[error("glazewm command failed with exit code {code}: {stderr}")]
    CommandFailed { code: Option<i32>, stderr: String },
    
    #[error("Command timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    
    #[error("Failed to parse command output as UTF-8: {0}")]
    OutputEncoding(#[from] std::string::FromUtf8Error),
    
    #[error("No focused monitor found")]
    NoFocusedMonitor,
    
    #[error("No focused workspace found on active monitor")]
    NoFocusedWorkspace,
}
```

### JSON Parsing Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum JsonError {
    #[error("Invalid JSON syntax: {0}")]
    SyntaxError(#[from] serde_json::Error),
    
    #[error("glazewm returned error: {message}")]
    GlazewmError { message: String },
    
    #[error("Missing required field: {field}")]
    MissingField { field: String },
    
    #[error("Unexpected response format")]
    UnexpectedFormat,
}
```

### Recovery Strategies

#### Command Failures

```rust
impl GlazewmClient {
    async fn query_with_retry<T>(&self, args: &[&str]) -> Result<T, CliError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut attempts = 0;
        let max_retries = 3;
        
        loop {
            match self.execute_command(args).await {
                Ok(response) => return Ok(response),
                Err(CliError::Timeout { .. }) if attempts < max_retries => {
                    attempts += 1;
                    tokio::time::sleep(Duration::from_millis(100 * attempts)).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }
}
```

#### JSON Validation

```rust
impl GlazewmResponse<MonitorData> {
    pub fn validate(&self) -> Result<(), JsonError> {
        if !self.success {
            return Err(JsonError::GlazewmError {
                message: self.error.clone().unwrap_or_default(),
            });
        }
        
        if self.data.monitors.is_empty() {
            return Err(JsonError::UnexpectedFormat);
        }
        
        Ok(())
    }
}
```

## Performance Considerations

### CLI Execution Optimization

**Command Caching:**

```rust
pub struct CachedClient {
    client: GlazewmClient,
    monitor_cache: Option<(Instant, Vec<Monitor>)>,
    window_cache: Option<(Instant, Vec<Window>)>,
    cache_duration: Duration,
}

impl CachedClient {
    pub async fn query_monitors(&mut self) -> Result<Vec<Monitor>, CliError> {
        if let Some((last_update, ref monitors)) = &self.monitor_cache {
            if last_update.elapsed() < self.cache_duration {
                return Ok(monitors.clone());
            }
        }
        
        let monitors = self.client.query_monitors().await?;
        self.monitor_cache = Some((Instant::now(), monitors.clone()));
        Ok(monitors)
    }
}
```

**Parallel Queries:**

```rust
impl GlazewmClient {
    pub async fn query_all(&self) -> Result<(Vec<Monitor>, Vec<Window>), CliError> {
        let (monitor_result, window_result) = tokio::join!(
            self.query_monitors(),
            self.query_windows()
        );
        
        Ok((monitor_result?, window_result?))
    }
}
```

### JSON Processing Optimization

**Streaming Parse (Large Responses):**

```rust
use serde_json::Deserializer;

pub fn parse_large_response<T>(json: &str) -> Result<T, JsonError>
where
    T: for<'de> Deserialize<'de>,
{
    let mut de = Deserializer::from_str(json);
    let response = T::deserialize(&mut de)?;
    de.end()?;
    Ok(response)
}
```

## Testing with Real glazewm

### Integration Test Setup

```rust
#[tokio::test]
async fn test_real_glazewm_integration() {
    let client = GlazewmClient::new("glazewm".into());
    
    // Skip test if glazewm not available
    if !client.is_available().await {
        eprintln!("Skipping integration test: glazewm not available");
        return;
    }
    
    let monitors = client.query_monitors().await.unwrap();
    assert!(!monitors.is_empty());
    
    let windows = client.query_windows().await.unwrap();
    // May be empty if no windows open
}
```

### JSON Response Validation

**Schema Compliance Testing:**

```rust
#[test]
fn validate_real_glazewm_schemas() {
    // Capture real responses and validate against our types
    let monitor_json = include_str!("../fixtures/real_monitors.json");
    let window_json = include_str!("../fixtures/real_windows.json");
    
    let monitor_response: GlazewmResponse<MonitorData> = 
        serde_json::from_str(monitor_json).expect("Monitor schema validation failed");
    let window_response: GlazewmResponse<WindowData> = 
        serde_json::from_str(window_json).expect("Window schema validation failed");
        
    assert!(monitor_response.success);
    assert!(window_response.success);
}
```

## Mock Testing

### Test Fixtures

**Create Test Data:**

```bash
# Generate test fixtures from real glazewm
mkdir tests/fixtures
glazewm query monitors > tests/fixtures/monitors_typical.json
glazewm query windows > tests/fixtures/windows_typical.json

# Create edge case fixtures manually
echo '{"data": {"monitors": []}, "success": false, "error": "No monitors"}' > \
  tests/fixtures/monitors_error.json
```

**Use in Tests:**

```rust
#[test]
fn should_handle_empty_monitor_response() {
    let json = include_str!("../fixtures/monitors_empty.json");
    let result = parse_monitors_response(json);
    
    assert!(result.is_err());
    assert!(matches!(result, Err(JsonError::UnexpectedFormat)));
}
```

### Mock Client Implementation

```rust
pub struct MockGlazewmClient {
    monitor_responses: Vec<String>,
    window_responses: Vec<String>,
    call_count: Arc<AtomicUsize>,
}

impl MockGlazewmClient {
    pub fn new() -> Self {
        Self {
            monitor_responses: vec![
                include_str!("../fixtures/monitors_typical.json").to_string()
            ],
            window_responses: vec![
                include_str!("../fixtures/windows_typical.json").to_string()
            ],
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    pub fn with_error_response() -> Self {
        Self {
            monitor_responses: vec![
                r#"{"data": null, "success": false, "error": "Connection failed"}"#.to_string()
            ],
            window_responses: vec![],
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[async_trait]
impl WindowManagerClient for MockGlazewmClient {
    async fn query_monitors(&self) -> Result<Vec<Monitor>, CliError> {
        let call_index = self.call_count.fetch_add(1, Ordering::SeqCst);
        let response = &self.monitor_responses[call_index % self.monitor_responses.len()];
        parse_monitors_response(response)
    }
}
```

## Command Execution Implementation

### Core Client

```rust
use tokio::process::Command;
use std::process::Stdio;
use std::time::Duration;

pub struct GlazewmClient {
    command_path: PathBuf,
    timeout: Duration,
}

impl GlazewmClient {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            command_path: path.into(),
            timeout: Duration::from_secs(5),
        }
    }
    
    pub async fn is_available(&self) -> bool {
        match Command::new(&self.command_path)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
        {
            Ok(status) => status.success(),
            Err(_) => false,
        }
    }
    
    async fn execute_command(&self, args: &[&str]) -> Result<String, CliError> {
        let mut child = Command::new(&self.command_path)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| CliError::CommandNotFound)?;
        
        let output = tokio::time::timeout(self.timeout, child.wait_with_output())
            .await
            .map_err(|_| CliError::Timeout { 
                timeout_ms: self.timeout.as_millis() as u64 
            })?
            .map_err(|e| CliError::IoError(e))?;
        
        if !output.status.success() {
            return Err(CliError::CommandFailed {
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }
        
        String::from_utf8(output.stdout)
            .map_err(CliError::OutputEncoding)
    }
    
    /// Find the currently focused window across all windows
    pub async fn get_focused_window(&self) -> Result<Option<Window>, CliError> {
        let windows = self.query_windows().await?;
        Ok(windows.into_iter().find(|w| w.has_focus))
    }
    
    /// Get focus chain: focused monitor → focused workspace → focused window
    pub async fn get_focus_chain(&self) -> Result<FocusChain, CliError> {
        let monitors = self.query_monitors().await?;
        
        let focused_monitor = monitors.iter()
            .find(|m| m.has_focus)
            .ok_or(CliError::NoFocusedMonitor)?;
            
        let focused_workspace = focused_monitor.workspaces.iter()
            .find(|w| w.has_focus)
            .ok_or(CliError::NoFocusedWorkspace)?;
            
        let focused_window = focused_workspace.windows.iter()
            .find(|w| w.has_focus);
            
        Ok(FocusChain {
            monitor: focused_monitor.clone(),
            workspace: focused_workspace.clone(), 
            window: focused_window.cloned(),
        })
    }
}

/// Represents the complete focus chain in glazewm
#[derive(Debug, Clone)]
pub struct FocusChain {
    /// The currently active monitor (receives new windows)
    pub monitor: Monitor,
    /// The active workspace on the focused monitor
    pub workspace: Workspace, 
    /// The focused window (may be None if no windows have focus)
    pub window: Option<Window>,
}

impl FocusChain {
    /// Check if the focus chain is complete (window focus exists)
    pub fn is_complete(&self) -> bool {
        self.window.is_some()
    }
    
    /// Get a human-readable description of current focus
    pub fn describe(&self) -> String {
        match &self.window {
            Some(window) => format!(
                "Monitor {} → Workspace '{}' → Window '{}'",
                self.monitor.id, 
                self.workspace.name,
                window.title
            ),
            None => format!(
                "Monitor {} → Workspace '{}' → (No focused window)",
                self.monitor.id,
                self.workspace.name
            ),
        }
    }
}
```

## Future Extensions

### Alternative Command Support

The CLI approach easily supports alternative commands if glazewm adds them:

```rust
// Future: Real-time event streaming
pub async fn subscribe_events(&self) -> Result<EventStream, CliError> {
    let mut child = Command::new(&self.command_path)
        .args(["subscribe", "events"])
        .stdout(Stdio::piped())
        .spawn()?;
        
    // Parse JSON lines as they arrive
    let stdout = child.stdout.take().unwrap();
    Ok(EventStream::new(stdout))
}
```

### Other Window Managers

Adding support for other window managers only requires:

1. **Command Mapping:**

    ```rust
    match window_manager {
        WindowManager::Glazewm => vec!["query", "monitors"],
        WindowManager::I3 => vec!["-t", "get_tree"],
        WindowManager::Sway => vec!["-t", "get_tree"],
    }
    ```

2. **JSON Schema Mapping:**

    ```rust
    impl From<I3Node> for Monitor {
        fn from(node: I3Node) -> Self {
            // Convert i3's JSON format to our domain model
        }
    }
    ```

## glazewm Version Compatibility

### Supported Versions

- **glazewm 3.0.0+**: Full JSON API support
- **glazewm 2.x**: Limited support (may require different parsing)

### Compatibility Testing

```rust
#[tokio::test]
async fn test_glazewm_version_compatibility() {
    let client = GlazewmClient::new("glazewm");
    
    // Test version detection
    let version = client.get_version().await.unwrap();
    assert!(version.major >= 3, "glazewm 3.0.0+ required");
    
    // Test basic functionality
    let monitors = client.query_monitors().await.unwrap();
    assert!(!monitors.is_empty());
}
```

## Related Documentation

- **[← Back to README](../README.md)** - Project overview and quick start
- **[Usage Guide](USAGE.md)** - CLI options and keyboard controls
- **[Display Format](DISPLAY.md)** - Output interpretation guide
- **[Building](BUILDING.md)** - Build and setup instructions
- **[Architecture](ARCHITECTURE.md)** - Design principles and structure
- **[Contributing](CONTRIBUTE.md)** - Development workflow and guidelines

---

The CLI+JSON approach provides a robust, simple, and highly testable integration with glazewm. This design prioritizes reliability and maintainability over performance optimization, following UNIX philosophy principles.

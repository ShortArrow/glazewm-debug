# glazewm API Integration

This document describes how glazewm-debug integrates with glazewm through its command-line JSON API.

## Integration Strategy

### CLI+JSON Approach

glazewm-debug uses a **CLI-first approach** for maximum simplicity:

```
glazewm-debug → tokio::process::Command → glazewm query → JSON Response → serde Parse
```

**Strategic Benefits:**
- **Zero Dependencies**: No IPC libraries or native bindings
- **Version Independence**: Works with any glazewm version supporting JSON output
- **Process Isolation**: Complete separation from glazewm runtime
- **Platform Agnostic**: JSON parsing identical across platforms
- **Easy Testing**: Simple JSON fixture-based testing

## glazewm Commands

### Monitor State Query

```bash
glazewm query monitors
```

**Returns**: Complete monitor hierarchy including workspaces and windows
**Response Time**: ~10-50ms
**Format**: Nested JSON structure representing the full window management hierarchy

### Window State Query

```bash
glazewm query windows
```

**Returns**: Flat list of all windows with detailed properties  
**Response Time**: ~5-30ms
**Format**: Array of window objects with focus, state, and geometry information

## Focus State Hierarchy

glazewm provides a **three-level focus system**:

### System-Wide Window Focus
- Only **one window** has `"hasFocus": true` at any time
- Represents the window receiving keyboard input
- Independent of workspace/monitor active states

### Workspace Active State  
- **Per-monitor active workspace**: Each monitor has one active workspace
- Controls workspace visibility and window placement
- Multiple active workspaces can exist (one per monitor)

### Monitor Active State
- **Single active monitor**: Only one monitor has `"hasFocus": true`
- Determines where new windows open by default
- Affects window manager behavior and user focus

## JSON Schema Overview

### Monitor Response Structure

```json
{
  "data": {
    "monitors": [{
      "id": "string",
      "hasFocus": boolean,        // Active monitor
      "width": number,
      "height": number,
      "children": [{             // Workspaces
        "id": "string", 
        "name": "string",
        "hasFocus": boolean,     // Active workspace on this monitor
        "children": [{           // Windows
          "id": "string",
          "title": "string", 
          "hasFocus": boolean,   // Focused window (only 1 system-wide)
          "state": {"type": "tiling|floating|minimized"}
        }]
      }]
    }]
  }
}
```

### Window Response Structure

```json
{
  "data": {
    "windows": [{
      "id": "string",
      "title": "string",
      "processName": "string", 
      "hasFocus": boolean,       // Currently focused window
      "tilingSize": number|null, // Relative size in workspace
      "width": number,
      "height": number,
      "x": number,
      "y": number,
      "state": {"type": "tiling|floating|minimized"},
      "displayState": "shown|hiding|hidden"
    }]
  }
}
```

## Data Mapping Strategy

### Type Conversion Approach

**Raw glazewm JSON** → **Validated Rust Types** → **Domain Models**

```rust
// CLI layer: Raw JSON types (serde-derived)
#[derive(Deserialize)]
struct RawMonitor { /* glazewm fields */ }

// Domain layer: Rich business objects
struct Monitor { /* domain logic */ }

// Conversion: Validation + mapping
impl TryFrom<RawMonitor> for Monitor { /* ... */ }
```

### Error Handling Strategy

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("glazewm command failed")]
    CommandFailed,
    
    #[error("Invalid JSON response")]
    JsonParsing(#[from] serde_json::Error),
    
    #[error("Timeout after {ms}ms")]
    Timeout { ms: u64 },
}
```

## Implementation Architecture

### Client Interface

```rust
#[async_trait]
pub trait WindowManagerClient: Send + Sync {
    async fn query_monitors(&self) -> Result<Vec<Monitor>, ApiError>;
    async fn query_windows(&self) -> Result<Vec<Window>, ApiError>;
    async fn is_available(&self) -> bool;
}
```

### Real Implementation

```rust
pub struct GlazewmClient {
    command_path: PathBuf,
    timeout: Duration,
}

impl GlazewmClient {
    async fn execute_query(&self, args: &[&str]) -> Result<String, ApiError> {
        let output = tokio::process::Command::new(&self.command_path)
            .args(args)
            .output()
            .await?;
            
        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            Err(ApiError::CommandFailed)
        }
    }
}
```

### Mock Implementation

```rust
pub struct MockWindowManagerClient {
    json_responses: HashMap<String, String>,
    should_fail: bool,
}

impl MockWindowManagerClient {
    pub fn with_fixtures() -> Self {
        let mut responses = HashMap::new();
        responses.insert("monitors".to_string(), 
            include_str!("../fixtures/monitors.json").to_string());
        responses.insert("windows".to_string(),
            include_str!("../fixtures/windows.json").to_string());
            
        Self { json_responses: responses, should_fail: false }
    }
}
```

## Testing Strategy

### JSON Fixture Testing

**Test with Real Data:**
```bash
# Capture real responses for testing
glazewm query monitors > tests/fixtures/monitors_real.json
glazewm query windows > tests/fixtures/windows_real.json
```

**Use in Tests:**
```rust
#[test]
fn should_parse_real_glazewm_response() {
    let json = include_str!("../fixtures/monitors_real.json");
    let monitors = parse_monitors(json).unwrap();
    assert!(!monitors.is_empty());
}
```

### Error Scenario Testing

**Common Failure Cases:**
- Command not found (`glazewm` not in PATH)
- Invalid JSON response (malformed data)
- Timeout (glazewm hangs)
- Permission denied

### Performance Considerations

**CLI Execution Optimization:**
- Parallel queries when possible
- Response caching for repeated calls
- Timeout handling for reliability
- Error recovery with exponential backoff

## Future Extensions

### Alternative Window Managers

Adding support for other window managers requires only:

1. **Command Mapping**: Map different CLI commands to same interface
2. **JSON Schema Mapping**: Convert different JSON formats to common domain model
3. **Client Implementation**: New struct implementing `WindowManagerClient` trait

Example:
```rust
// i3 support would be:
struct I3Client { /* ... */ }

impl WindowManagerClient for I3Client {
    async fn query_monitors(&self) -> Result<Vec<Monitor>, ApiError> {
        let json = self.execute_command(&["-t", "get_tree"]).await?;
        parse_i3_tree_to_monitors(&json)
    }
}
```

## Related Documentation

- **[← Back to README](../README.md)** - Project overview
- **[Usage Guide](USAGE.md)** - CLI options and controls
- **[Display Format](DISPLAY.md)** - Output interpretation
- **[Architecture](ARCHITECTURE.md)** - Design principles
- **[Building](BUILDING.md)** - Build instructions
- **[Contributing](CONTRIBUTE.md)** - Development workflow

---

The CLI+JSON approach provides a robust foundation for window manager integration while maintaining simplicity and testability.
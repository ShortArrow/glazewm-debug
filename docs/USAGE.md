# Usage Guide

This guide covers detailed usage of glazewm-debug CLI options and keyboard controls.

## Command Line Interface

### Basic Execution

```bash
# Default execution (1-second refresh)
glazewm-debug

# Development execution
cargo run

# Release build
./target/release/glazewm-debug.exe
```

### Command Line Options

```text
glazewm-debug [OPTIONS]

OPTIONS:
    -r, --refresh-rate <MS>    Refresh interval in milliseconds [default: 1000]
    -q, --quiet               Minimal output mode
    -h, --help                Print help information
    -V, --version             Print version information
```

#### Option Details

**`--refresh-rate <MS>`**

- Controls glazewm query frequency
- Range: 100ms - 10000ms
- Lower values = more responsive, higher CPU usage

```bash
# Very responsive (250ms)
glazewm-debug --refresh-rate 250

# Conservative (5 seconds)  
glazewm-debug --refresh-rate 5000
```

**`--quiet`**

- Reduces output verbosity
- Hides status messages and debug info
- Useful for scripting

```bash
glazewm-debug --quiet
```

### Environment Variables

**`RUST_LOG`** - Control logging level:

```bash
# Debug logging
RUST_LOG=debug glazewm-debug

# Specific module logging
RUST_LOG=glazewm_debug::cli=trace glazewm-debug

# Error only
RUST_LOG=error glazewm-debug
```

**`NO_COLOR`** - Disable colored output:

```bash
NO_COLOR=1 glazewm-debug
```

## Keyboard Controls

### Navigation

| Key | Action | Description |
|-----|--------|-------------|
| `q` | Quit | Exit application |
| `Escape` | Quit | Alternative quit |
| `Ctrl+C` | Force Quit | Immediate termination |

### Refresh Controls

| Key | Action | Description |
|-----|--------|-------------|
| `r` | Force Refresh | Immediately query glazewm |
| `Space` | Toggle Pause | Pause/resume automatic updates |

### Display Controls

| Key | Action | Description |
|-----|--------|-------------|
| `c` | Compact Mode | Toggle detailed/compact view |
| `h` | Toggle Hidden | Show/hide hidden windows |
| `?` | Help | Display help overlay |

### Future Navigation

| Key | Action | Status |
|-----|--------|--------|
| `↑/↓` | Navigate Workspaces | Planned |
| `←/→` | Navigate Monitors | Planned |
| `Tab` | Focus Next | Planned |
| `Enter` | Show Details | Planned |

## Output Modes

### Default TUI Mode

Real-time terminal interface with:

- Hierarchical display (Monitor → Workspace → Window)
- Color-coded focus indicators
- Interactive keyboard controls
- Automatic updates every 1 second

### Quiet Mode (`--quiet`)

Reduced visual clutter:

- No status bar
- Minimal error messages
- No help prompts
- Reduced color usage

## Output Interpretation

### Basic Format

```text
┌─ Monitor 1 (1920x1080) [Active] ───────────────────────────┐
│ Workspace 2 [Active] ────────────────────────────────────── │
│ ┌─ vscode* (33.3%) ──┐ ┌─ chrome (66.7%) ──────────────────┐ │
│ │ main.rs - VS Code  │ │ Stack Overflow - Chrome            │ │
│ │ [T] 613x952        │ │ [T] 1267x952                       │ │
│ └────────────────────┘ └────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Element Breakdown:**

- `Monitor 1` - Monitor identifier
- `(1920x1080)` - Resolution
- `[Active]` - Focus state
- `vscode*` - Process name + focus indicator
- `(33.3%)` - Window size percentage
- `[T]` - Window state (Tiling/Floating/Minimized/Hidden)
- `613x952` - Absolute dimensions

### State Indicators

**Window States:**

- `[T]` - Tiling mode (normal)
- `[F]` - Floating mode  
- `[M]` - Minimized
- `[H]` - Hidden

**Focus Indicators:**

- `*` - Currently focused window
- `[Active]` - Active workspace/monitor
- **Bold** - Primary focus
- Dim - Inactive elements

## Troubleshooting

### Common Issues

- **"glazewm command not found"**

```bash
# Check PATH
where glazewm          # Windows
which glazewm          # Unix

# Verify installation
glazewm --version
```

- **"Permission denied"**

```bash
# Check glazewm service
sc query GlazeWM       # Windows

# Run as administrator if needed
```

- **"Invalid JSON response"**

```bash
# Test glazewm directly
glazewm query monitors
glazewm query windows

# Check for errors
glazewm query monitors 2>&1
```

- **High CPU usage**

```bash
# Reduce refresh frequency
glazewm-debug --refresh-rate 2000

# Use quiet mode
glazewm-debug --quiet
```

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug glazewm-debug

# Save debug output
RUST_LOG=debug glazewm-debug 2> debug.log
```

### Performance Tuning

**Optimal Refresh Rates:**

- **Development**: 500-1000ms
- **General Use**: 1000-2000ms  
- **Background Monitoring**: 5000ms+

**Memory Usage:**

- Normal: ~10-20MB RAM
- Large setups: ~50MB RAM
- Memory growth indicates bugs (report issues)

## Integration with Other Tools

### Future Features

**JSON Output:**

```bash
# Export state as JSON (planned)
glazewm-debug --output json > state.json

# CSV export for analysis (planned)
glazewm-debug --output csv > history.csv
```

**Pipe Integration:**

```bash
# Monitor specific windows (planned)
glazewm-debug --output json | jq '.windows[] | select(.title | contains("VS Code"))'

# Window count analysis (planned)  
glazewm-debug --output json | jq '.workspaces[] | {name: .name, count: (.windows | length)}'
```

### Configuration (Future)

```toml
# ~/.config/glazewm-debug.toml (planned)
[display]
refresh_rate_ms = 1000
compact_mode = false
show_hidden_windows = true

[cli]
glazewm_path = "glazewm"
timeout_ms = 5000

[keybindings]
quit = ["q", "Escape"]
refresh = ["r", "F5"]
help = ["?"]
```

## Related Documentation

- **[← Back to README](../README.md)** - Project overview
- **[Display Format](DISPLAY.md)** - Visual output interpretation
- **[API Integration](API.md)** - glazewm command details
- **[Building](BUILDING.md)** - Build and setup instructions
- **[Architecture](ARCHITECTURE.md)** - Design principles
- **[Contributing](CONTRIBUTE.md)** - Development guidelines

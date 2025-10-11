# Usage Guide

This guide covers detailed usage of glazewm-debug, including command-line options, keyboard controls, and output interpretation.

## Command Line Interface

### Basic Execution

```bash
# Default execution (1-second refresh rate)
glazewm-debug

# Using cargo during development
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

#### Detailed Option Descriptions

**`--refresh-rate <MS>` / `-r <MS>`**

- Controls how often glazewm-debug queries glazewm for state updates
- Default: 1000ms (1 second)
- Range: 100ms - 10000ms
- Lower values provide more responsive updates but use more CPU

```bash
# Very responsive (every 250ms)
glazewm-debug --refresh-rate 250

# Conservative (every 5 seconds)  
glazewm-debug --refresh-rate 5000
```

**`--quiet` / `-q`**

- Reduces output verbosity
- Hides status messages and debug information
- Useful for scripting or when running in background

```bash
# Minimal output
glazewm-debug --quiet
```

### Environment Variables

**`RUST_LOG`** - Control logging level

```bash
# Debug logging
RUST_LOG=debug glazewm-debug

# Specific module logging
RUST_LOG=glazewm_debug::cli=trace glazewm-debug

# Suppress most logging
RUST_LOG=error glazewm-debug
```

**`NO_COLOR`** - Disable colored output

```bash
# Disable colors for scripting
NO_COLOR=1 glazewm-debug
```

## Keyboard Controls

### Navigation

| Key | Action | Description |
|-----|--------|-------------|
| `q` | Quit | Exit the application |
| `Escape` | Quit | Alternative quit key |
| `Ctrl+C` | Force Quit | Immediate termination |

### Refresh Controls

| Key | Action | Description |
|-----|--------|-------------|
| `r` | Force Refresh | Immediately query glazewm for updated state |
| `Space` | Toggle Pause | Pause/resume automatic updates |

### Display Controls

| Key | Action | Description |
|-----|--------|-------------|
| `c` | Compact Mode | Toggle between detailed and compact view |
| `h` | Toggle Hidden | Show/hide hidden windows |
| `?` | Help | Display help overlay |

### Navigation (Future Features)

| Key | Action | Description |
|-----|--------|-------------|
| `↑/↓` | Navigate | Move between workspaces |
| `←/→` | Navigate | Move between monitors |
| `Tab` | Focus Next | Highlight next window |
| `Enter` | Details | Show detailed window information |

## Output Modes

### Default TUI Mode

The standard mode provides a real-time terminal interface showing the current glazewm state in a hierarchical format.

**Features:**

- Real-time updates
- Hierarchical display (Monitor → Workspace → Window)
- Color-coded focus indicators
- Interactive keyboard controls

### Quiet Mode (`--quiet`)

Reduces visual clutter for scripting or background operation.

**Changes in Quiet Mode:**

- No status bar
- Minimal error messages
- No help prompts
- Reduced color usage

## Output Interpretation

### Monitor Display

```text
┌─ Monitor 1 (1920x1080) [Active] ───────────────────────────┐
│ ...workspace content...                                     │
└─────────────────────────────────────────────────────────────┘
```

**Monitor Header Format:**

- `Monitor X` - Monitor identifier from glazewm
- `(1920x1080)` - Resolution in pixels
- `[Active]` - Currently focused monitor
- Box drawing characters frame the monitor content

### Workspace Display

```text
│ Workspace 2 [Active] ──────────────────────────────────────│
│ ...window content...                                        │
```

**Workspace Header Format:**

- `Workspace X` - Workspace name/number
- `[Active]` - Currently active workspace
- Window count may be shown in compact mode

### Window Display

#### Detailed View

```text
│ ┌─ vscode* (33.3%) ──┐ ┌─ chrome (66.7%) ──────────────────┐ │
│ │ main.rs - VS Code  │ │ Stack Overflow - Chrome            │ │
│ │ [T] 613x952        │ │ [T] 1267x952                       │ │
│ └────────────────────┘ └────────────────────────────────────┘ │
```

#### Compact View

```text
├─ WS 2 [Active] (3 windows)
│  ├─ wezterm* [T] CC-LED-MCP
│  ├─ chrome [T] Google Search
│  └─ vscode [T] main.rs
```

**Window Information:**

- `*` after name = Currently focused window
- `(33.3%)` = Percentage of workspace area (detailed view)
- `[T]` = Window state (see State Indicators below)
- Size information in pixels (detailed view)

### State Indicators

**Window States:**

- `[T]` - Tiling mode (normal tiled window)
- `[F]` - Floating mode (not tiled)
- `[M]` - Minimized (iconified)
- `[H]` - Hidden (not visible)

**Focus Indicators:**

- `*` - Currently focused window
- `[Active]` - Active workspace or monitor
- Bright/bold text - Focused elements
- Dim text - Inactive elements

### Multi-Monitor Layout

```text
┌─ Monitor 1 (1920x1080) ─────────┐ ┌─ Monitor 2 (2560x1440) [Active] ─┐
│ Workspace 1 [Active] ─────────── │ │ Workspace 4 [Active] ──────────── │
│ ┌─ chrome* ──────────────────────┐ │ │ ┌─ vscode* ─────┐ ┌─ terminal ───┐ │
│ │ Google Chrome                  │ │ │ │ main.rs       │ │ cargo build  │ │
│ │ [T]                           │ │ │ │ [T]           │ │ [T]          │ │
│ └───────────────────────────────┘ │ │ └───────────────┘ └──────────────┘ │
└──────────────────────────────────┘ └────────────────────────────────────┘
```

**Multi-Monitor Features:**

- Side-by-side monitor display
- Independent workspace tracking per monitor  
- Focus indication spans monitors
- Proportional sizing based on actual resolutions

## Troubleshooting

### Common Issues

- **"glazewm command not found"**

```bash
# Check if glazewm is in PATH
where glazewm
glazewm --version

# Add to PATH if needed (PowerShell)
$env:PATH += ";C:\Program Files\glazewm"
```

- **"Permission denied"**

```bash
# Run as administrator or check glazewm permissions
# Ensure glazewm service is running
sc query GlazeWM
```

- **"Invalid JSON response"**

```bash
# Test glazewm CLI directly
glazewm query monitors
glazewm query windows

# Check for error messages
glazewm query monitors 2>&1
```

- **High CPU usage**

```bash
# Increase refresh rate to reduce CPU load
glazewm-debug --refresh-rate 5000

# Use quiet mode
glazewm-debug --quiet --refresh-rate 2000
```

### Debug Mode

Enable debug logging for troubleshooting:

```bash
# Full debug output
RUST_LOG=debug glazewm-debug

# CLI-specific debugging
RUST_LOG=glazewm_debug::cli=trace glazewm-debug

# Save debug output to file
RUST_LOG=debug glazewm-debug 2> debug.log
```

### Performance Tuning

**Optimal Refresh Rates:**

- **Development/Debugging**: 500-1000ms
- **General Use**: 1000-2000ms  
- **Background Monitoring**: 5000ms+
- **High-Activity Periods**: 250-500ms

**Memory Usage:**

- Normal usage: ~10-20MB RAM
- With large window counts: ~50MB RAM
- Memory growth indicates a bug (please report)

## Integration with Other Tools

### Piping Output (Future Feature)

```bash
# Export current state as JSON
glazewm-debug --output json > state.json

# Monitor for specific windows
glazewm-debug --output json | jq '.windows[] | select(.title | contains("Chrome"))'

# CSV export for analysis
glazewm-debug --output csv > window_history.csv
```

### Scripting Usage

```bash
# Check if specific window is focused
glazewm-debug --quiet --output json | jq -r '.windows[] | select(.hasFocus) | .title'

# Count windows per workspace
glazewm-debug --output json | jq '.workspaces[] | {name: .name, count: (.windows | length)}'
```

### Configuration Files (Future Feature)

```toml
# ~/.config/glazewm-debug.toml
[display]
refresh_rate_ms = 1000
compact_mode = false
show_hidden_windows = true
theme = "default"

[cli]
glazewm_path = "glazewm"
timeout_ms = 5000

[keybindings]
quit = ["q", "Escape"]
refresh = ["r", "F5"]
help = ["?", "h"]
```

## Related Documentation

- **[← Back to README](../README.md)** - Project overview and quick start
- **[Display Format](DISPLAY.md)** - Visual output interpretation
- **[API Integration](API.md)** - glazewm command details
- **[Building](BUILDING.md)** - Build and setup instructions
- **[Architecture](ARCHITECTURE.md)** - Design principles
- **[Contributing](CONTRIBUTE.md)** - Development guidelines

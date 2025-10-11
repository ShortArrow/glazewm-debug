# Display Format Guide

This guide explains how glazewm-debug presents window manager state information, including layout formats, visual indicators, and display modes.

## Overview

glazewm-debug follows a **hierarchical display model** that mirrors the actual structure of glazewm's window management:

```text
Monitor
├── Workspace
│   ├── Window
│   ├── Window
│   └── ...
├── Workspace
└── ...
```

All display output uses **Unicode box-drawing characters** for clear visual hierarchy and works in any terminal that supports UTF-8.

## Display Modes

### Detailed View (Default)

The detailed view provides maximum information with graphical window layouts:

```text
┌─ Monitor 1 (1920x1080) [Active] ───────────────────────────┐
│ Workspace 2 [Active] ────────────────────────────────────── │
│ ┌─ vscode* (50%) ─────┐ ┌─ chrome (30%) ─┐ ┌─ discord (20%)┐ │
│ │ main.rs - VS Code   │ │ GitHub - Chrome │ │ @user - Discord│ │
│ │ [T] 940x952         │ │ [T] 564x952     │ │ [T] 376x952   │ │
│ └─────────────────────┘ └─────────────────┘ └───────────────┘ │
│                                                             │
│ Workspace 1 ──────────────────────────────────────────────  │
│ ┌─ notepad (100%) ──────────────────────────────────────────┐ │
│ │ README.md - Notepad                                       │ │
│ │ [T] (Hidden)                                             │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

**Features:**

- Visual window layout representation
- Exact pixel dimensions
- Proportional sizing indicators
- Full window titles

### Compact View

Activated with `c` key, shows more information in less space:

```text
Monitor 1 (1920x1080) [Active]
├─ WS 2 [Active] (3 windows)
│  ├─ vscode* [T] main.rs - Visual Studio Code
│  ├─ chrome [T] GitHub - Google Chrome
│  └─ discord [T] @user - Discord
└─ WS 1 (1 window)
   └─ notepad [T] README.md - Notepad (Hidden)
```

**Features:**

- Tree-style hierarchy
- Window count summaries
- Truncated titles for space efficiency
- Quick overview of large setups

## Element Descriptions

### Monitor Headers

```text
┌─ Monitor 1 (1920x1080) [Active] ───────────────────────────┐
```

**Format Components:**

- `Monitor 1` - Monitor identifier from glazewm
- `(1920x1080)` - Screen resolution in pixels
- `[Active]` - Currently focused monitor (only one at a time)
- Box frame - Visual boundary for monitor content

**Additional Monitor Information:**

- **DPI** - Displayed when different from 96 DPI
- **Position** - Relative position in multi-monitor setup
- **Scale Factor** - Windows scaling percentage

### Workspace Headers

```text
│ Workspace 2 [Active] ────────────────────────────────────── │
```

**Format Components:**

- `Workspace X` - Workspace name or number from glazewm
- `[Active]` - Currently active workspace on this monitor
- Window count - `(3 windows)` in compact mode

**Workspace States:**

- `[Active]` - Currently displayed workspace
- `[Focused]` - Has focused windows (in multi-workspace scenarios)
- `(Empty)` - No windows in workspace
- `(Hidden)` - Not currently displayed

### Window Representations

#### Detailed Window Boxes

```text
│ ┌─ wezterm* (33.3%) ─┐ │
│ │ CC-LED-MCP Version │ │
│ │ [T] 613x952        │ │
│ └───────────────────┘ │
```

**Box Header:**

- `wezterm*` - Process name + focus indicator
- `(33.3%)` - Percentage of workspace width/height

**Box Content:**

- Line 1: Window title (truncated if needed)
- Line 2: State indicator + dimensions

#### Compact Window Lines

```text
│  ├─ wezterm* [T] CC-LED-MCP Version
```

**Format:**

- Tree connector (`├─`, `└─`)
- Process name + focus indicator
- State indicator in brackets
- Window title

## State Indicators

### Window States

| Indicator | State | Description |
|-----------|-------|-------------|
| `[T]` | Tiling | Normal tiled window (most common) |
| `[F]` | Floating | Window floating above tiled layout |
| `[M]` | Minimized | Window minimized to taskbar |
| `[H]` | Hidden | Window exists but not visible |

### Focus Indicators

glazewm-debug displays a **three-level focus hierarchy**:

| Level | Indicator | Meaning | Description |
|-------|-----------|---------|-------------|
| Window | `*` | Focused Window | **Currently receiving keyboard input** (only 1 system-wide) |
| Workspace | `[Active]` | Active Workspace | Currently displayed workspace (1 per monitor) |
| Monitor | `[Active]` | Active Monitor | Monitor receiving new windows (1 system-wide) |

**Visual Emphasis:**

- **Bold** - Primary focus (most important)
- Bright - Secondary focus
- Normal - Active but not focused
- Dim - Inactive elements

### Focus State Examples

**Same Monitor Focus:**

```text
┌─ Monitor 1 (1920x1080) [Active] ───────────────────────────┐
│ Workspace 2 [Active] ────────────────────────────────────── │
│ ┌─ vscode* (50%) ─────┐ ┌─ chrome (50%) ─────────────────┐ │
│ │ main.rs - VS Code   │ │ GitHub - Chrome                │ │
│ │ [T] 940x952         │ │ [T] 940x952                    │ │
│ └─────────────────────┘ └────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

- *Focus chain: Monitor 1 → Workspace 2 → VS Code*

**Cross-Monitor Focus:**

```text  
┌─ Monitor 1 (1920x1080) [Active] ─┐ ┌─ Monitor 2 (2560x1440) ────────┐
│ Workspace 1 [Active] ─────────── │ │ Workspace 3 [Active] ──────────── │
│ ┌─ chrome (100%) ──────────────┐ │ │ ┌─ discord* (100%) ──────────────┐ │
│ │ YouTube - Chrome             │ │ │ │ @user - Discord                │ │
│ │ [T]                         │ │ │ │ [T] ← Focused window          │ │
│ └─────────────────────────────┘ │ │ └─────────────────────────────────┘ │
└─────────────────────────────────┘ └──────────────────────────────────────┘
```

- *Focus chain: Monitor 1 (active) → Workspace 3 (monitor 2) → Discord*

### Display States

| State | Indicator | Description |
|-------|-----------|-------------|
| Shown | Normal text | Window visible and normal |
| Hidden | `(Hidden)` | Window not currently displayed |
| Hiding | `(Hiding)` | Window transitioning to hidden |

## Multi-Monitor Display

### Side-by-Side Layout

For multiple monitors, glazewm-debug displays them horizontally:

```text
┌─ Monitor 1 (1920x1080) ─────────┐ ┌─ Monitor 2 (2560x1440) [Active] ─┐
│ Workspace 1 [Active] ─────────── │ │ Workspace 4 [Active] ──────────── │
│ ┌─ chrome* ──────────────────────┐ │ │ ┌─ vscode* ─────┐ ┌─ terminal ───┐ │
│ │ Google Chrome                  │ │ │ │ main.rs       │ │ cargo build  │ │
│ │ [T]                           │ │ │ │ [T]           │ │ [T]          │ │
│ └───────────────────────────────┘ │ │ └───────────────┘ └──────────────┘ │
└──────────────────────────────────┘ └────────────────────────────────────┘
```

### Vertical Stacking (Large Monitor Count)

With 3+ monitors, display switches to vertical stacking:

```text
┌─ Monitor 1 (1920x1080) ─────────────────────────────────────┐
│ Workspace 1 [Active] ─────────────────────────────────────── │
│ ┌─ chrome* ──────────────────────────────────────────────────┐ │
│ │ Google Chrome                                              │ │
└─────────────────────────────────────────────────────────────┘

┌─ Monitor 2 (2560x1440) [Active] ───────────────────────────┐
│ Workspace 4 [Active] ────────────────────────────────────── │
│ ┌─ vscode* ─────┐ ┌─ terminal ──────────────────────────────┐ │
│ │ main.rs       │ │ cargo build                            │ │
└─────────────────────────────────────────────────────────────┘
```

## Color Coding

### Default Theme

| Element | Color | Purpose |
|---------|-------|---------|
| Monitor Headers | Blue | Clear monitor boundaries |
| Active Workspace | Green | Current workspace identification |
| Focused Window | Yellow/Bright | Immediate attention |
| Window Borders | White/Gray | Structure definition |
| Hidden Elements | Dim Gray | Reduced prominence |
| Error States | Red | Problem identification |

### No-Color Mode

When `NO_COLOR=1` is set, all information is conveyed through:

- Text styling (bold, dim)
- Unicode symbols (`*`, `[Active]`)
- Box drawing characters
- Indentation levels

## Window Layout Representation

### Tiling Direction Indicators

glazewm-debug shows the actual spatial relationship of windows:

**Horizontal Tiling:**

```text
│ ┌─ Window A ─┐ ┌─ Window B ─┐ ┌─ Window C ─┐ │
│ │            │ │            │ │            │ │
│ └────────────┘ └────────────┘ └────────────┘ │
```

**Vertical Tiling:**

```text
│ ┌─ Window A ──────────────────────────────────┐ │
│ │                                             │ │
│ ├─ Window B ──────────────────────────────────┤ │
│ │                                             │ │
│ └─ Window C ──────────────────────────────────┘ │
```

**Mixed Layout:**

```text
│ ┌─ Window A ─┐ ┌─ Window B ──────────────────┐ │
│ │            │ │                             │ │
│ │            │ ├─ Window C ──────────────────┤ │
│ │            │ │                             │ │
│ └────────────┘ └─ Window D ──────────────────┘ │
```

### Size Proportions

**Percentage Display:**

- Shows relative size within workspace
- Calculated from glazewm's `tilingSize` property
- Updates in real-time as windows are resized

**Absolute Dimensions:**

- Pixel-perfect size information
- Useful for debugging layout issues
- Includes window borders and decorations

## Special Cases

### Empty Workspaces

```text
│ Workspace 3 ────────────────────────────────────────────── │
│ (Empty)                                                     │
```

### Single Window Workspace

```text
│ Workspace 5 ────────────────────────────────────────────── │
│ ┌─ firefox* (100%) ──────────────────────────────────────┐ │
│ │ Mozilla Firefox                                         │ │
│ │ [T] 1880x952                                           │ │
│ └─────────────────────────────────────────────────────────┘ │
```

### Floating Windows

Floating windows are shown with special indicators:

```text
│ ┌─ calculator [F] ─┐                                       │
│ │ Calculator       │  ← Floating above tiled layout       │
│ │ 320x240          │                                       │
│ └──────────────────┘                                       │
│ ┌─ editor* (100%) ─────────────────────────────────────────┐ │
│ │ Main Editor Window                                       │ │
│ │ [T] 1560x952                                            │ │
│ └─────────────────────────────────────────────────────────┘ │
```

### Minimized Windows

```text
│ Workspace 2 [Active] ────────────────────────────────────── │
│ ┌─ code* (60%) ──────────────┐ ┌─ browser (40%) ──────────┐ │
│ │ VS Code                     │ │ Chrome                   │ │
│ │ [T] 1128x952               │ │ [T] 752x952              │ │
│ └─────────────────────────────┘ └─────────────────────────┘ │
│                                                             │
│ Minimized: notepad [M], calculator [M]                     │
```

## Terminal Compatibility

### Required Features

- **UTF-8 Support** - For box drawing characters
- **Color Support** - 16+ colors recommended
- **Unicode Width** - Proper character width calculation
- **Cursor Control** - For real-time updates

### Tested Terminals

**Windows:**

- ✅ Windows Terminal (Recommended)
- ✅ PowerShell 7
- ✅ ConEmu
- ⚠️ Command Prompt (limited color support)

**Cross-Platform:**

- ✅ Alacritty
- ✅ iTerm2 (macOS)
- ✅ GNOME Terminal (Linux)
- ✅ Konsole (Linux)

### Fallback Display

For terminals with limited Unicode support:

```text
Monitor 1 (1920x1080) [Active]
+-- Workspace 2 [Active]
|   +-- vscode* [T] main.rs - VS Code
|   +-- chrome [T] GitHub - Chrome
|   +-- discord [T] @user - Discord
+-- Workspace 1
    +-- notepad [T] README.md (Hidden)
```

## Accessibility

### Screen Reader Support

- Semantic structure with clear hierarchy
- Text-based indicators for all visual information
- Consistent formatting for predictable navigation

### High Contrast Mode

Activated with `--high-contrast` (future feature):

- Increased contrast ratios
- Bold text for focus indicators
- Simplified color scheme

### Font Size Independence

- Uses character-based layout
- Scales with terminal font size
- No pixel-perfect positioning requirements

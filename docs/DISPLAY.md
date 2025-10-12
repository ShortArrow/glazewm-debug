# Display Format Guide

This guide explains how glazewm-debug presents window manager state information.

## Overview

glazewm-debug follows a **hierarchical display model** mirroring glazewm's structure:

```text
Monitor
├── Workspace
│   ├── Window
│   ├── Window
│   └── ...
├── Workspace
└── ...
```

All output uses **Unicode box-drawing characters** for clear visual hierarchy.

## Display Modes

### Detailed View (Default)

Provides maximum information with graphical layouts:

```text
┌─ Monitor 1 (1920x1080) [Active] ───────────────────────────┐
│ ┌─ Workspace 2 [Active] ────────────────────────────────┐ │
│ │ ┌─ vscode* (50%) ─────┐ ┌─ chrome (30%) ─┐ ┌─ discord│ │
│ │ │ main.rs - VS Code   │ │ GitHub - Chrome │ │ @user - │ │
│ │ │ [T] 940x952         │ │ [T] 564x952     │ │ [T] 376x│ │
│ │ └─────────────────────┘ └─────────────────┘ └─────────┘ │
│ └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Compact View (Press `c`)

Tree-style hierarchy for space efficiency:

```text
Monitor 1 (1920x1080) [Active]
├─ WS 2 [Active] (3 windows)
│  ├─ vscode* [T] main.rs - Visual Studio Code
│  ├─ chrome [T] GitHub - Google Chrome
│  └─ discord [T] @user - Discord
└─ WS 1 (1 window)
   └─ notepad [T] README.md - Notepad (Hidden)
```

## Focus State Indicators

glazewm-debug displays a **three-level focus hierarchy**:

| Level | Indicator | Meaning | Description |
|-------|-----------|---------|-------------|
| Window | `*` | Focused Window | Currently receiving keyboard input (1 system-wide) |
| Workspace | `[Active]` | Active Workspace | Currently displayed (1 per monitor) |
| Monitor | `[Active]` | Active Monitor | Receives new windows (1 system-wide) |

### Focus Examples

**Same Monitor Focus:**

```text
┌─ Monitor 1 [Active] ────────────────────────────────────────┐
│ Workspace 2 [Active] ────────────────────────────────────── │
│ ┌─ vscode* (50%) ─────┐ ┌─ chrome (50%) ─────────────────┐ │
│ │ main.rs - VS Code   │ │ GitHub - Chrome                │ │
│ └─────────────────────┘ └────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

- *Focus chain: Monitor 1 → Workspace 2 → VS Code*

**Cross-Monitor Focus:**

```text  
┌─ Monitor 1 [Active] ─────────┐ ┌─ Monitor 2 ─────────────────┐
│ Workspace 1 [Active] ─────── │ │ Workspace 3 [Active] ────── │
│ ┌─ chrome (100%) ──────────┐ │ │ ┌─ discord* (100%) ────────┐ │
│ │ YouTube - Chrome         │ │ │ │ @user - Discord          │ │
│ └─────────────────────────┘ │ │ └─────────────────────────┘ │
└─────────────────────────────┘ └──────────────────────────────┘
```

- *Focus chain: Monitor 1 (active) → Workspace 3 (monitor 2) → Discord*

## Window State Indicators

| Indicator | State | Description |
|-----------|-------|-------------|
| `[T]` | Tiling | Normal tiled window |
| `[F]` | Floating | Window floating above layout |
| `[M]` | Minimized | Minimized to taskbar |
| `[H]` | Hidden | Not visible |

## Multi-Monitor Display

### Side-by-Side Layout

Multiple monitors displayed horizontally:

```text
┌─ Monitor 1 (1920x1080) ─────────┐ ┌─ Monitor 2 (2560x1440) [Active] ─┐
│ Workspace 1 [Active] ─────────── │ │ Workspace 4 [Active] ──────────── │
│ ┌─ chrome* ──────────────────────┐ │ │ ┌─ vscode* ─────┐ ┌─ powershell ─┐ │
│ │ Netflix - Chrome               │ │ │ │ main.rs       │ │ PowerShell   │ │
│ │ [T]                           │ │ │ │ [T]           │ │ [T]          │ │
│ └───────────────────────────────┘ │ │ └───────────────┘ └──────────────┘ │
└─────────────────────────────────┘ └────────────────────────────────────┘
```

## Special Cases

### Empty Workspaces

```text
│ Workspace 3 ────────────────────────────────────────────── │
│ (Empty)                                                     │
```

### Floating Windows

```text
│ ┌─ calculator [F] ─┐                                       │
│ │ Calculator       │  ← Floating above tiled layout       │
└─────────────────────┘
│ ┌─ notepad* (100%) ─────────────────────────────────────────┐ │
│ │ document.txt - Notepad                                   │ │
└─────────────────────────────────────────────────────────────┘
```

### Minimized Windows

```text
│ ┌─ vscode* (60%) ──────────────┐ ┌─ chrome (40%) ──────────┐ │
│ │ main.rs - VS Code           │ │ YouTube - Chrome         │ │
│ └─────────────────────────────┘ └─────────────────────────┘ │
│ Minimized: notepad [M], calculator [M], discord [M]        │
```

## Terminal Compatibility

### Required Features

- **UTF-8 Support**: For box drawing characters
- **Color Support**: 16+ colors recommended  
- **Unicode Width**: Proper character width calculation

### Tested Terminals

**Windows:**

- ✅ Windows Terminal (Recommended)
- ✅ PowerShell 7
- ⚠️ Command Prompt (limited color)

**Cross-Platform:**

- ✅ Alacritty, iTerm2, GNOME Terminal, Konsole

### Fallback Display

For limited Unicode support:

```text
Monitor 1 (1920x1080) [Active]
+-- Workspace 2 [Active]
|   +-- vscode* [T] main.rs - VS Code
|   +-- chrome [T] GitHub - Chrome
|   +-- discord [T] @user - Discord
+-- Workspace 1
    +-- notepad [T] README.md (Hidden)
```

## Color Coding

### Default Theme

- **Monitor Headers**: Blue
- **Active Elements**: Green
- **Focused Window**: Yellow/Bright
- **Hidden Elements**: Dim Gray
- **Error States**: Red

### No-Color Mode (`NO_COLOR=1`)

All information conveyed through:

- Text styling (bold, dim)
- Unicode symbols (`*`, `[Active]`)
- Box drawing characters
- Indentation levels

## Layout Representation

### Tiling Direction

**Horizontal Tiling:**

```text
│ ┌─ Window A ─┐ ┌─ Window B ─┐ ┌─ Window C ─┐ │
│ └────────────┘ └────────────┘ └────────────┘ │
```

**Vertical Tiling:**

```text
│ ┌─ Window A ──────────────────────────────────┐ │
│ ├─ Window B ──────────────────────────────────┤ │
│ └─ Window C ──────────────────────────────────┘ │
```

### Size Information

**Percentage Display**: Relative size within workspace (from `tilingSize`)
**Absolute Dimensions**: Pixel-perfect size information  
**Real-time Updates**: Changes as windows are resized

## Related Documentation

- **[← Back to README](../README.md)** - Project overview
- **[Usage Guide](USAGE.md)** - CLI options and keyboard controls
- **[API Integration](API.md)** - glazewm command details
- **[Building](BUILDING.md)** - Build and setup instructions
- **[Architecture](ARCHITECTURE.md)** - Design principles
- **[Contributing](CONTRIBUTE.md)** - Development guidelines

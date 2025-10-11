// Domain layer module
// Contains pure business logic with no external dependencies

pub mod values;
pub mod window;
pub mod workspace;
pub mod monitor;
pub mod errors;

// Re-export public types
pub use values::{Position, Size, Rectangle, MonitorId, WorkspaceId, WindowId};
pub use window::Window;
pub use workspace::Workspace;
pub use monitor::Monitor;
pub use errors::DomainError;

/// Focus state of windows, workspaces, and monitors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusState {
    /// Currently has focus (receiving input)
    Focused,
    /// Does not have focus
    Unfocused,
}

impl FocusState {
    pub fn is_focused(self) -> bool {
        matches!(self, FocusState::Focused)
    }
}

/// Window state in the tiling system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    /// Window is part of tiling layout
    Tiling,
    /// Window is floating above tiling layout
    Floating,
    /// Window is minimized to taskbar
    Minimized,
}

/// Display state of windows and workspaces
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayState {
    /// Currently visible on screen
    Shown,
    /// Not visible but exists
    Hidden,
    /// Transitioning to hidden state
    Hiding,
}

impl DisplayState {
    pub fn is_visible(self) -> bool {
        matches!(self, DisplayState::Shown)
    }
}

/// Tiling direction for workspace layout
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilingDirection {
    /// Windows arranged side-by-side
    Horizontal,
    /// Windows arranged vertically stacked
    Vertical,
}

impl Default for TilingDirection {
    fn default() -> Self {
        TilingDirection::Horizontal
    }
}
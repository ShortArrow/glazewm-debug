// Domain-specific error types
// Represents business rule violations and invalid state transitions

use crate::domain::{MonitorId, WindowId, WorkspaceId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Window {id} not found in workspace")]
    WindowNotFound { id: WindowId },

    #[error("Workspace {id} not found in monitor")]
    WorkspaceNotFound { id: WorkspaceId },

    #[error("Duplicate window ID {id} in workspace")]
    DuplicateWindowId { id: WindowId },

    #[error("Duplicate workspace ID {id} in monitor")]
    DuplicateWorkspaceId { id: WorkspaceId },

    #[error("Invalid window state transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        from: crate::domain::WindowState,
        to: crate::domain::WindowState,
    },

    #[error("Invalid geometry: width and height must be positive")]
    InvalidGeometry { width: u32, height: u32 },

    #[error("Invalid DPI value: {dpi} (must be > 0)")]
    InvalidDpi { dpi: i32 },

    #[error("Invalid scale factor: {scale} (must be > 0.0)")]
    InvalidScaleFactor { scale: f64 },

    #[error("Workspace capacity exceeded: maximum {max}, attempted {attempted}")]
    WorkspaceCapacityExceeded { max: usize, attempted: usize },

    #[error("Multiple active workspaces detected on monitor {monitor_id}")]
    MultipleActiveWorkspaces { monitor_id: MonitorId },

    #[error("No active workspace found on monitor {monitor_id}")]
    NoActiveWorkspace { monitor_id: MonitorId },
}

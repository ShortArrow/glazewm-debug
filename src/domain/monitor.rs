// Monitor aggregate root implementation
// Represents a physical monitor containing workspaces

use crate::domain::values::Rectangle;
use crate::domain::{DomainError, FocusState, MonitorId, Window, Workspace, WorkspaceId};

/// Device information for a monitor
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceInfo {
    pub dpi: i32,
    pub scale_factor: f64,
    pub device_name: String,
}

impl DeviceInfo {
    pub fn new(dpi: i32, scale_factor: f64, device_name: String) -> Self {
        Self {
            dpi,
            scale_factor,
            device_name,
        }
    }
}

/// Monitor aggregate root containing workspaces and windows
#[derive(Debug, Clone, PartialEq)]
pub struct Monitor {
    id: MonitorId,
    geometry: Rectangle,
    workspaces: Vec<Workspace>,
    focus_state: FocusState,
    device_info: DeviceInfo,
}

impl Monitor {
    /// Create a new monitor with basic properties
    pub fn new(
        id: MonitorId,
        geometry: Rectangle,
        workspaces: Vec<Workspace>,
        focus_state: FocusState,
        dpi: i32,
        scale_factor: f64,
    ) -> Self {
        let device_info = DeviceInfo::new(dpi, scale_factor, format!("Monitor {}", id));

        Self {
            id,
            geometry,
            workspaces,
            focus_state,
            device_info,
        }
    }

    /// Create a monitor with validation
    pub fn try_new(
        id: MonitorId,
        geometry: Rectangle,
        workspaces: Vec<Workspace>,
        focus_state: FocusState,
        dpi: i32,
        scale_factor: f64,
    ) -> Result<Self, DomainError> {
        // Validate geometry
        if geometry.size.width == 0 || geometry.size.height == 0 {
            return Err(DomainError::InvalidGeometry {
                width: geometry.size.width,
                height: geometry.size.height,
            });
        }

        // Validate DPI
        if dpi <= 0 {
            return Err(DomainError::InvalidDpi { dpi });
        }

        // Validate scale factor
        if scale_factor <= 0.0 {
            return Err(DomainError::InvalidScaleFactor {
                scale: scale_factor,
            });
        }

        Ok(Self::new(
            id,
            geometry,
            workspaces,
            focus_state,
            dpi,
            scale_factor,
        ))
    }

    // Getters
    pub fn id(&self) -> &MonitorId {
        &self.id
    }

    pub fn geometry(&self) -> &Rectangle {
        &self.geometry
    }

    pub fn workspaces(&self) -> &[Workspace] {
        &self.workspaces
    }

    pub fn focus_state(&self) -> &FocusState {
        &self.focus_state
    }

    pub fn dpi(&self) -> i32 {
        self.device_info.dpi
    }

    pub fn scale_factor(&self) -> f64 {
        self.device_info.scale_factor
    }

    pub fn device_name(&self) -> &str {
        &self.device_info.device_name
    }

    // Computed properties
    pub fn is_focused(&self) -> bool {
        self.focus_state.is_focused()
    }

    pub fn workspace_count(&self) -> usize {
        self.workspaces.len()
    }

    pub fn total_window_count(&self) -> usize {
        self.workspaces.iter().map(|ws| ws.window_count()).sum()
    }

    /// Get all active workspaces on this monitor
    pub fn active_workspaces(&self) -> Vec<&Workspace> {
        self.workspaces
            .iter()
            .filter(|ws| ws.is_focused())
            .collect()
    }

    /// Find the focused window across all workspaces on this monitor
    pub fn focused_window(&self) -> Option<&Window> {
        self.workspaces
            .iter()
            .find_map(|workspace| workspace.focused_window())
    }

    // Workspace management
    pub fn add_workspace(&mut self, workspace: Workspace) -> Result<(), DomainError> {
        // Check for duplicate workspace ID
        if self.workspaces.iter().any(|ws| ws.id() == workspace.id()) {
            return Err(DomainError::DuplicateWorkspaceId {
                id: workspace.id().clone(),
            });
        }

        // If the new workspace is active, deactivate others
        if workspace.is_focused() {
            self.deactivate_all_workspaces();
        }

        self.workspaces.push(workspace);
        Ok(())
    }

    pub fn remove_workspace(
        &mut self,
        workspace_id: &WorkspaceId,
    ) -> Result<Workspace, DomainError> {
        let position = self
            .workspaces
            .iter()
            .position(|ws| ws.id() == workspace_id)
            .ok_or_else(|| DomainError::WorkspaceNotFound {
                id: workspace_id.clone(),
            })?;

        Ok(self.workspaces.remove(position))
    }

    /// Deactivate all workspaces (used when adding new active workspace)
    fn deactivate_all_workspaces(&mut self) {
        for workspace in &mut self.workspaces {
            workspace.set_focus_state(FocusState::Unfocused);
        }
    }

    pub fn set_focus_state(&mut self, focus_state: FocusState) {
        self.focus_state = focus_state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{values::Rectangle, DisplayState, TilingDirection, WindowId, WindowState};

    fn create_test_workspace(id: &str, name: &str, focused: bool) -> Workspace {
        Workspace::new(
            WorkspaceId::new(id.to_string()),
            name.to_string(),
            Vec::new(),
            TilingDirection::Horizontal,
            if focused {
                FocusState::Focused
            } else {
                FocusState::Unfocused
            },
            DisplayState::Shown,
        )
    }

    fn create_test_window(id: &str, title: &str, process: &str) -> Window {
        Window::new(
            WindowId::new(id.to_string()),
            title.to_string(),
            process.to_string(),
            Rectangle::from_coords(0, 0, 800, 600),
            WindowState::Tiling,
            FocusState::Unfocused,
            DisplayState::Shown,
        )
    }

    fn create_focused_window(id: &str, title: &str, process: &str, focused: bool) -> Window {
        Window::new(
            WindowId::new(id.to_string()),
            title.to_string(),
            process.to_string(),
            Rectangle::from_coords(0, 0, 800, 600),
            WindowState::Tiling,
            if focused {
                FocusState::Focused
            } else {
                FocusState::Unfocused
            },
            DisplayState::Shown,
        )
    }

    #[test]
    fn should_create_monitor_with_basic_properties() {
        let monitor = Monitor::new(
            MonitorId::new("monitor-1".to_string()),
            Rectangle::from_coords(0, 0, 1920, 1080),
            Vec::new(),
            FocusState::Focused,
            96,
            1.0,
        );

        assert!(monitor.is_focused());
        assert_eq!(monitor.workspace_count(), 0);
        assert_eq!(monitor.dpi(), 96);
        assert_eq!(monitor.scale_factor(), 1.0);
    }

    #[test]
    fn should_validate_geometry() {
        let result = Monitor::try_new(
            MonitorId::new("invalid".to_string()),
            Rectangle::from_coords(0, 0, 0, 0), // Invalid size
            Vec::new(),
            FocusState::Focused,
            96,
            1.0,
        );

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DomainError::InvalidGeometry { .. }
        ));
    }

    #[test]
    fn should_add_workspace() {
        let mut monitor = Monitor::new(
            MonitorId::new("monitor-1".to_string()),
            Rectangle::from_coords(0, 0, 1920, 1080),
            Vec::new(),
            FocusState::Focused,
            96,
            1.0,
        );

        let workspace = create_test_workspace("ws-1", "Development", false);
        let workspace_id = workspace.id().clone();

        assert!(monitor.add_workspace(workspace).is_ok());
        assert_eq!(monitor.workspace_count(), 1);
        assert_eq!(monitor.workspaces()[0].id(), &workspace_id);
    }

    #[test]
    fn should_enforce_single_active_workspace() {
        let workspace1 = create_test_workspace("ws-1", "First", true); // Active
        let workspace2 = create_test_workspace("ws-2", "Second", false);

        let mut monitor = Monitor::new(
            MonitorId::new("monitor-1".to_string()),
            Rectangle::from_coords(0, 0, 1920, 1080),
            vec![workspace1, workspace2],
            FocusState::Focused,
            96,
            1.0,
        );

        let new_active_workspace = create_test_workspace("ws-3", "Third", true); // Also active

        assert!(monitor.add_workspace(new_active_workspace).is_ok());

        // Only one workspace should be active
        let active_workspaces = monitor.active_workspaces();
        assert_eq!(active_workspaces.len(), 1);
        assert_eq!(active_workspaces[0].name(), "Third");
    }

    #[test]
    fn should_find_focused_window_across_workspaces() {
        // Create workspace with focused window
        let focused_window = create_focused_window("focused", "VS Code", "Code", true);
        let unfocused_window = create_focused_window("unfocused", "Chrome", "chrome", false);

        let workspace = Workspace::new(
            WorkspaceId::new("ws-1".to_string()),
            "Development".to_string(),
            vec![unfocused_window, focused_window],
            TilingDirection::Horizontal,
            FocusState::Focused,
            DisplayState::Shown,
        );

        let monitor = Monitor::new(
            MonitorId::new("monitor-1".to_string()),
            Rectangle::from_coords(0, 0, 1920, 1080),
            vec![workspace],
            FocusState::Focused,
            96,
            1.0,
        );

        let focused = monitor.focused_window();
        assert!(focused.is_some());
        assert_eq!(focused.unwrap().title(), "VS Code");
    }

    #[test]
    fn should_calculate_total_window_count() {
        let ws1_windows = vec![
            create_test_window("w1", "App1", "app1"),
            create_test_window("w2", "App2", "app2"),
            create_test_window("w3", "App3", "app3"),
        ];
        let workspace1 = Workspace::new(
            WorkspaceId::new("ws-1".to_string()),
            "Workspace 1".to_string(),
            ws1_windows,
            TilingDirection::Horizontal,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        let ws2_windows = vec![
            create_test_window("w4", "App4", "app4"),
            create_test_window("w5", "App5", "app5"),
        ];
        let workspace2 = Workspace::new(
            WorkspaceId::new("ws-2".to_string()),
            "Workspace 2".to_string(),
            ws2_windows,
            TilingDirection::Vertical,
            FocusState::Focused,
            DisplayState::Shown,
        );

        let monitor = Monitor::new(
            MonitorId::new("monitor-1".to_string()),
            Rectangle::from_coords(0, 0, 1920, 1080),
            vec![workspace1, workspace2],
            FocusState::Focused,
            96,
            1.0,
        );

        assert_eq!(monitor.total_window_count(), 5); // 3 + 2
    }
}

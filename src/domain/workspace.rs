// Workspace entity implementation
// Represents a logical workspace containing windows

use crate::domain::values::{Position, Size};
use crate::domain::{
    DisplayState, DomainError, FocusState, TilingDirection, Window, WindowId, WorkspaceId,
};

/// Layout information for a window within a workspace
#[derive(Debug, Clone, PartialEq)]
pub struct WindowLayout {
    pub window_id: WindowId,
    pub position: Position,
    pub size: Size,
}

impl WindowLayout {
    pub fn new(window_id: WindowId, position: Position, size: Size) -> Self {
        Self {
            window_id,
            position,
            size,
        }
    }
}

/// Workspace entity containing windows and layout information
#[derive(Debug, Clone, PartialEq)]
pub struct Workspace {
    id: WorkspaceId,
    name: String,
    windows: Vec<Window>,
    tiling_direction: TilingDirection,
    focus_state: FocusState,
    display_state: DisplayState,
}

impl Workspace {
    /// Create a new workspace
    pub fn new(
        id: WorkspaceId,
        name: String,
        windows: Vec<Window>,
        tiling_direction: TilingDirection,
        focus_state: FocusState,
        display_state: DisplayState,
    ) -> Self {
        Self {
            id,
            name,
            windows,
            tiling_direction,
            focus_state,
            display_state,
        }
    }

    // Getters
    pub fn id(&self) -> &WorkspaceId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn windows(&self) -> &[Window] {
        &self.windows
    }

    pub fn tiling_direction(&self) -> &TilingDirection {
        &self.tiling_direction
    }

    pub fn focus_state(&self) -> &FocusState {
        &self.focus_state
    }

    pub fn display_state(&self) -> &DisplayState {
        &self.display_state
    }

    // Computed properties
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    pub fn is_focused(&self) -> bool {
        self.focus_state.is_focused()
    }

    pub fn is_visible(&self) -> bool {
        self.display_state.is_visible()
    }

    /// Find the focused window in this workspace
    pub fn focused_window(&self) -> Option<&Window> {
        self.windows.iter().find(|window| window.is_focused())
    }

    // Window management
    pub fn add_window(&mut self, window: Window) -> Result<(), DomainError> {
        // Check for duplicate window ID
        if self.windows.iter().any(|w| w.id() == window.id()) {
            return Err(DomainError::DuplicateWindowId {
                id: window.id().clone(),
            });
        }

        self.windows.push(window);
        Ok(())
    }

    pub fn remove_window(&mut self, window_id: &WindowId) -> Result<Window, DomainError> {
        let position = self
            .windows
            .iter()
            .position(|w| w.id() == window_id)
            .ok_or_else(|| DomainError::WindowNotFound {
                id: window_id.clone(),
            })?;

        Ok(self.windows.remove(position))
    }

    /// Calculate layout for all windows in this workspace
    pub fn calculate_layout(&self, container_size: Size) -> Vec<WindowLayout> {
        if self.windows.is_empty() {
            return Vec::new();
        }

        match self.tiling_direction {
            TilingDirection::Horizontal => self.calculate_horizontal_layout(container_size),
            TilingDirection::Vertical => self.calculate_vertical_layout(container_size),
        }
    }

    fn calculate_horizontal_layout(&self, container_size: Size) -> Vec<WindowLayout> {
        let window_count = self.windows.len() as u32;
        let window_width = container_size.width / window_count;

        self.windows
            .iter()
            .enumerate()
            .map(|(index, window)| {
                let x = (index as u32) * window_width;
                let position = Position::new(x as i32, 0);
                let size = Size::new(window_width, container_size.height);

                WindowLayout::new(window.id().clone(), position, size)
            })
            .collect()
    }

    fn calculate_vertical_layout(&self, container_size: Size) -> Vec<WindowLayout> {
        let window_count = self.windows.len() as u32;
        let window_height = container_size.height / window_count;

        self.windows
            .iter()
            .enumerate()
            .map(|(index, window)| {
                let y = (index as u32) * window_height;
                let position = Position::new(0, y as i32);
                let size = Size::new(container_size.width, window_height);

                WindowLayout::new(window.id().clone(), position, size)
            })
            .collect()
    }

    // State mutations
    pub fn set_focus_state(&mut self, focus_state: FocusState) {
        self.focus_state = focus_state;
    }

    pub fn set_tiling_direction(&mut self, direction: TilingDirection) {
        self.tiling_direction = direction;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{values::Rectangle, WindowState};

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
    fn should_create_empty_workspace() {
        let workspace = Workspace::new(
            WorkspaceId::new("test".to_string()),
            "Test Workspace".to_string(),
            Vec::new(),
            TilingDirection::Horizontal,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        assert!(workspace.is_empty());
        assert_eq!(workspace.window_count(), 0);
        assert!(!workspace.is_focused());
    }

    #[test]
    fn should_add_window() {
        let mut workspace = Workspace::new(
            WorkspaceId::new("test".to_string()),
            "Test".to_string(),
            Vec::new(),
            TilingDirection::Horizontal,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        let window = create_test_window("w1", "VS Code", "Code");
        let window_id = window.id().clone();

        assert!(workspace.add_window(window).is_ok());
        assert_eq!(workspace.window_count(), 1);
        assert_eq!(workspace.windows()[0].id(), &window_id);
    }

    #[test]
    fn should_prevent_duplicate_window_ids() {
        let mut workspace = Workspace::new(
            WorkspaceId::new("test".to_string()),
            "Test".to_string(),
            Vec::new(),
            TilingDirection::Horizontal,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        let window1 = create_test_window("same-id", "Window 1", "app1");
        let window2 = create_test_window("same-id", "Window 2", "app2");

        assert!(workspace.add_window(window1).is_ok());
        assert!(workspace.add_window(window2).is_err());
        assert_eq!(workspace.window_count(), 1);
    }

    #[test]
    fn should_remove_window() {
        let window1 = create_test_window("w1", "Keep", "app1");
        let window2 = create_test_window("w2", "Remove", "app2");
        let window2_id = window2.id().clone();

        let mut workspace = Workspace::new(
            WorkspaceId::new("test".to_string()),
            "Test".to_string(),
            vec![window1, window2],
            TilingDirection::Horizontal,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        let removed = workspace.remove_window(&window2_id);
        assert!(removed.is_ok());
        assert_eq!(workspace.window_count(), 1);
        assert_eq!(workspace.windows()[0].title(), "Keep");
    }

    #[test]
    fn should_find_focused_window() {
        let window1 = create_focused_window("w1", "Chrome", "chrome", false);
        let window2 = create_focused_window("w2", "VS Code", "Code", true);

        let workspace = Workspace::new(
            WorkspaceId::new("test".to_string()),
            "Test".to_string(),
            vec![window1, window2],
            TilingDirection::Horizontal,
            FocusState::Focused,
            DisplayState::Shown,
        );

        let focused = workspace.focused_window();
        assert!(focused.is_some());
        assert_eq!(focused.unwrap().title(), "VS Code");
    }

    #[test]
    fn should_calculate_horizontal_layout() {
        let window1 = create_test_window("w1", "Window 1", "app1");
        let window2 = create_test_window("w2", "Window 2", "app2");

        let workspace = Workspace::new(
            WorkspaceId::new("test".to_string()),
            "Test".to_string(),
            vec![window1, window2],
            TilingDirection::Horizontal,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        let layout = workspace.calculate_layout(Size::new(1920, 1080));

        assert_eq!(layout.len(), 2);

        // Windows should be side by side
        assert_eq!(layout[0].position.x, 0);
        assert_eq!(layout[1].position.x, 960); // Half width

        // Same height
        assert_eq!(layout[0].size.height, 1080);
        assert_eq!(layout[1].size.height, 1080);

        // Equal width
        assert_eq!(layout[0].size.width, 960);
        assert_eq!(layout[1].size.width, 960);
    }

    #[test]
    fn should_calculate_vertical_layout() {
        let window1 = create_test_window("w1", "Window 1", "app1");
        let window2 = create_test_window("w2", "Window 2", "app2");

        let workspace = Workspace::new(
            WorkspaceId::new("test".to_string()),
            "Test".to_string(),
            vec![window1, window2],
            TilingDirection::Vertical,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        let layout = workspace.calculate_layout(Size::new(1920, 1080));

        assert_eq!(layout.len(), 2);

        // Windows should be stacked
        assert_eq!(layout[0].position.y, 0);
        assert_eq!(layout[1].position.y, 540); // Half height

        // Same width
        assert_eq!(layout[0].size.width, 1920);
        assert_eq!(layout[1].size.width, 1920);

        // Equal height
        assert_eq!(layout[0].size.height, 540);
        assert_eq!(layout[1].size.height, 540);
    }
}

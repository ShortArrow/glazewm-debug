// Window entity implementation
// Represents an individual application window in the window management domain

use crate::domain::values::Rectangle;
use crate::domain::{DisplayState, DomainError, FocusState, WindowId, WindowState};

/// Window entity representing an individual application window
#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    id: WindowId,
    title: String,
    process_name: String,
    geometry: Rectangle,
    state: WindowState,
    focus_state: FocusState,
    display_state: DisplayState,
}

impl Window {
    /// Create a new window with specified properties
    pub fn new(
        id: WindowId,
        title: String,
        process_name: String,
        geometry: Rectangle,
        state: WindowState,
        focus_state: FocusState,
        display_state: DisplayState,
    ) -> Self {
        Self {
            id,
            title,
            process_name,
            geometry,
            state,
            focus_state,
            display_state,
        }
    }

    // Getters
    pub fn id(&self) -> &WindowId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn process_name(&self) -> &str {
        &self.process_name
    }

    pub fn geometry(&self) -> &Rectangle {
        &self.geometry
    }

    pub fn state(&self) -> &WindowState {
        &self.state
    }

    pub fn focus_state(&self) -> &FocusState {
        &self.focus_state
    }

    pub fn display_state(&self) -> &DisplayState {
        &self.display_state
    }

    // Behavior methods
    pub fn is_focused(&self) -> bool {
        self.focus_state.is_focused()
    }

    pub fn is_visible(&self) -> bool {
        self.display_state.is_visible()
    }

    /// Generate display name for UI (process: title)
    pub fn display_name(&self) -> String {
        format!("{}: {}", self.process_name, self.title)
    }

    /// Generate truncated display name for compact UI
    pub fn display_name_truncated(&self, max_len: usize) -> String {
        let full_name = self.display_name();
        if full_name.len() <= max_len {
            full_name
        } else {
            let truncated = &full_name[..max_len.saturating_sub(3)];
            format!("{}...", truncated)
        }
    }

    /// Get state indicator for display ([T], [F], [M], [H])
    pub fn state_indicator(&self) -> &'static str {
        match (&self.state, &self.display_state) {
            (_, DisplayState::Hidden) => "[H]",
            (WindowState::Minimized, _) => "[M]",
            (WindowState::Floating, _) => "[F]",
            (WindowState::Fullscreen, _) => "[F]", // Fullscreen treated as floating for display
            (WindowState::Tiling, _) => "[T]",
        }
    }

    // State mutations
    pub fn change_state(&mut self, new_state: WindowState) -> Result<(), DomainError> {
        // For now, allow all state transitions
        // Future: Add business rules for valid transitions
        self.state = new_state;
        Ok(())
    }

    pub fn set_focus_state(&mut self, focus_state: FocusState) {
        self.focus_state = focus_state;
    }

    pub fn set_geometry(&mut self, geometry: Rectangle) {
        self.geometry = geometry;
    }

    pub fn set_display_state(&mut self, display_state: DisplayState) {
        self.display_state = display_state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_window() -> Window {
        Window::new(
            WindowId::new("test-window".to_string()),
            "Test Window".to_string(),
            "test".to_string(),
            Rectangle::from_coords(0, 0, 800, 600),
            WindowState::Tiling,
            FocusState::Unfocused,
            DisplayState::Shown,
        )
    }

    #[test]
    fn should_create_window_correctly() {
        let window = create_test_window();

        assert_eq!(window.title(), "Test Window");
        assert_eq!(window.process_name(), "test");
        assert!(!window.is_focused());
        assert!(window.is_visible());
    }

    #[test]
    fn should_generate_display_name() {
        let window = create_test_window();
        assert_eq!(window.display_name(), "test: Test Window");
    }

    #[test]
    fn should_truncate_long_display_names() {
        let mut window = create_test_window();
        window.title = "This is a very long window title that should be truncated".to_string();

        let truncated = window.display_name_truncated(20);
        assert!(truncated.len() <= 20);
        assert!(truncated.contains("..."));
    }

    #[test]
    fn should_handle_state_changes() {
        let mut window = create_test_window();

        assert!(window.change_state(WindowState::Floating).is_ok());
        assert_eq!(window.state(), &WindowState::Floating);

        assert!(window.change_state(WindowState::Minimized).is_ok());
        assert_eq!(window.state(), &WindowState::Minimized);
    }

    #[test]
    fn should_update_focus_state() {
        let mut window = create_test_window();

        assert!(!window.is_focused());

        window.set_focus_state(FocusState::Focused);
        assert!(window.is_focused());
        assert_eq!(window.focus_state(), &FocusState::Focused);
    }

    #[test]
    fn should_generate_state_indicators() {
        let mut window = create_test_window();

        // Test tiling state
        window.state = WindowState::Tiling;
        assert_eq!(window.state_indicator(), "[T]");

        // Test floating state
        window.state = WindowState::Floating;
        assert_eq!(window.state_indicator(), "[F]");

        // Test minimized state
        window.state = WindowState::Minimized;
        assert_eq!(window.state_indicator(), "[M]");

        // Test fullscreen state (should show as floating)
        window.state = WindowState::Fullscreen;
        assert_eq!(window.state_indicator(), "[F]");

        // Test hidden state (overrides window state)
        window.state = WindowState::Tiling;
        window.display_state = DisplayState::Hidden;
        assert_eq!(window.state_indicator(), "[H]");
    }
}

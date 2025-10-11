use glazewm_debug::domain::values::{Position, Rectangle, Size};
use glazewm_debug::domain::{DisplayState, FocusState, Window, WindowId, WindowState};

mod window_creation {
    use super::*;

    #[test]
    fn should_create_window_with_valid_properties() {
        // Given
        let window_id = WindowId::new("test-window-1".to_string());
        let title = "main.rs - Visual Studio Code".to_string();
        let process_name = "Code".to_string();
        let geometry = Rectangle::new(Position::new(100, 200), Size::new(800, 600));

        // When
        let window = Window::new(
            window_id.clone(),
            title.clone(),
            process_name.clone(),
            geometry,
            WindowState::Tiling,
            FocusState::Focused,
            DisplayState::Shown,
        );

        // Then
        assert_eq!(window.id(), &window_id);
        assert_eq!(window.title(), &title);
        assert_eq!(window.process_name(), &process_name);
        assert_eq!(window.geometry(), &geometry);
        assert_eq!(window.state(), &WindowState::Tiling);
        assert_eq!(window.focus_state(), &FocusState::Focused);
        assert_eq!(window.display_state(), &DisplayState::Shown);
    }

    #[test]
    fn should_create_window_with_floating_state() {
        // Given
        let window_id = WindowId::new("floating-window".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(400, 300));

        // When
        let window = Window::new(
            window_id,
            "Calculator".to_string(),
            "calc".to_string(),
            geometry,
            WindowState::Floating,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        // Then
        assert_eq!(window.state(), &WindowState::Floating);
        assert!(!window.is_focused());
        assert!(window.is_visible());
    }

    #[test]
    fn should_create_minimized_window() {
        // Given
        let window_id = WindowId::new("minimized-window".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(0, 0));

        // When
        let window = Window::new(
            window_id,
            "Hidden App".to_string(),
            "app".to_string(),
            geometry,
            WindowState::Minimized,
            FocusState::Unfocused,
            DisplayState::Hidden,
        );

        // Then
        assert_eq!(window.state(), &WindowState::Minimized);
        assert_eq!(window.display_state(), &DisplayState::Hidden);
        assert!(!window.is_visible());
    }
}

mod window_behavior {
    use super::*;

    #[test]
    fn should_determine_visibility_correctly() {
        // Given
        let window_id = WindowId::new("test-window".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(800, 600));

        // When - Shown window
        let shown_window = Window::new(
            window_id.clone(),
            "Visible Window".to_string(),
            "app".to_string(),
            geometry,
            WindowState::Tiling,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        // When - Hidden window
        let hidden_window = Window::new(
            window_id.clone(),
            "Hidden Window".to_string(),
            "app".to_string(),
            geometry,
            WindowState::Tiling,
            FocusState::Unfocused,
            DisplayState::Hidden,
        );

        // Then
        assert!(shown_window.is_visible());
        assert!(!hidden_window.is_visible());
    }

    #[test]
    fn should_determine_focus_correctly() {
        // Given
        let window_id = WindowId::new("test-window".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(800, 600));

        // When - Focused window
        let focused_window = Window::new(
            window_id.clone(),
            "Focused Window".to_string(),
            "app".to_string(),
            geometry,
            WindowState::Tiling,
            FocusState::Focused,
            DisplayState::Shown,
        );

        // When - Unfocused window
        let unfocused_window = Window::new(
            window_id.clone(),
            "Unfocused Window".to_string(),
            "app".to_string(),
            geometry,
            WindowState::Tiling,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        // Then
        assert!(focused_window.is_focused());
        assert!(!unfocused_window.is_focused());
    }

    #[test]
    fn should_generate_display_name_correctly() {
        // Given
        let window_id = WindowId::new("test-window".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(800, 600));

        let window = Window::new(
            window_id,
            "README.md - Visual Studio Code".to_string(),
            "Code".to_string(),
            geometry,
            WindowState::Tiling,
            FocusState::Focused,
            DisplayState::Shown,
        );

        // When
        let display_name = window.display_name();

        // Then
        assert_eq!(display_name, "Code: README.md - Visual Studio Code");
    }

    #[test]
    fn should_handle_long_titles_in_display_name() {
        // Given
        let window_id = WindowId::new("test-window".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(800, 600));

        let long_title = "This is a very long window title that should be truncated for display purposes in the TUI interface";
        let window = Window::new(
            window_id,
            long_title.to_string(),
            "chrome".to_string(),
            geometry,
            WindowState::Tiling,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        // When
        let display_name = window.display_name_truncated(30);

        // Then
        assert!(display_name.len() <= 30);
        assert!(display_name.contains("chrome"));
        assert!(display_name.contains("...") || display_name.len() < long_title.len());
    }
}

mod window_state_transitions {
    use super::*;

    #[test]
    fn should_allow_valid_state_transitions() {
        // Given
        let window_id = WindowId::new("test-window".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(800, 600));

        let mut window = Window::new(
            window_id,
            "Test Window".to_string(),
            "test".to_string(),
            geometry,
            WindowState::Tiling,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        // When - Valid transitions
        assert!(window.change_state(WindowState::Floating).is_ok());
        assert_eq!(window.state(), &WindowState::Floating);

        assert!(window.change_state(WindowState::Minimized).is_ok());
        assert_eq!(window.state(), &WindowState::Minimized);

        assert!(window.change_state(WindowState::Tiling).is_ok());
        assert_eq!(window.state(), &WindowState::Tiling);
    }

    #[test]
    fn should_update_focus_state() {
        // Given
        let window_id = WindowId::new("test-window".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(800, 600));

        let mut window = Window::new(
            window_id,
            "Test Window".to_string(),
            "test".to_string(),
            geometry,
            WindowState::Tiling,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        // When
        window.set_focus_state(FocusState::Focused);

        // Then
        assert!(window.is_focused());
        assert_eq!(window.focus_state(), &FocusState::Focused);
    }

    #[test]
    fn should_update_geometry() {
        // Given
        let window_id = WindowId::new("test-window".to_string());
        let initial_geometry = Rectangle::new(Position::new(0, 0), Size::new(800, 600));

        let mut window = Window::new(
            window_id,
            "Test Window".to_string(),
            "test".to_string(),
            initial_geometry,
            WindowState::Tiling,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        // When
        let new_geometry = Rectangle::new(Position::new(100, 100), Size::new(1000, 800));
        window.set_geometry(new_geometry);

        // Then
        assert_eq!(window.geometry(), &new_geometry);
        assert_ne!(window.geometry(), &initial_geometry);
    }
}

mod window_validation {
    use super::*;

    #[test]
    fn should_validate_window_properties() {
        // Given
        let window_id = WindowId::new("test-window".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(800, 600));

        let window = Window::new(
            window_id,
            "Test Window".to_string(),
            "test".to_string(),
            geometry,
            WindowState::Tiling,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        // When/Then - Basic validation
        assert!(!window.title().is_empty());
        assert!(!window.process_name().is_empty());
        assert!(window.geometry().size.width > 0);
        assert!(window.geometry().size.height > 0);
    }

    #[test]
    fn should_handle_edge_case_titles() {
        let window_id = WindowId::new("test".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(100, 100));

        // Empty title
        let empty_title_window = Window::new(
            window_id.clone(),
            "".to_string(),
            "test".to_string(),
            geometry,
            WindowState::Tiling,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        // Unicode title
        let unicode_window = Window::new(
            window_id.clone(),
            "🚀 Rust Project - VS Code".to_string(),
            "Code".to_string(),
            geometry,
            WindowState::Tiling,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        // Then
        assert_eq!(empty_title_window.display_name(), "test: ");
        assert!(unicode_window.display_name().contains("🚀"));
    }
}

use glazewm_debug::domain::values::{Position, Rectangle, Size};
use glazewm_debug::domain::{DisplayState, DomainError, WindowState};
use glazewm_debug::domain::{
    FocusState, TilingDirection, Window, WindowId, Workspace, WorkspaceId,
};

mod workspace_creation {
    use super::*;

    #[test]
    fn should_create_empty_workspace() {
        // Given
        let workspace_id = WorkspaceId::new("workspace-1".to_string());
        let name = "Development".to_string();

        // When
        let workspace = Workspace::new(
            workspace_id.clone(),
            name.clone(),
            Vec::new(),
            TilingDirection::Horizontal,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        // Then
        assert_eq!(workspace.id(), &workspace_id);
        assert_eq!(workspace.name(), &name);
        assert_eq!(workspace.windows().len(), 0);
        assert!(workspace.is_empty());
        assert_eq!(workspace.tiling_direction(), &TilingDirection::Horizontal);
    }

    #[test]
    fn should_create_workspace_with_windows() {
        // Given
        let workspace_id = WorkspaceId::new("workspace-2".to_string());
        let window1 = create_test_window("window-1", "VS Code", "Code");
        let window2 = create_test_window("window-2", "Chrome", "chrome");
        let windows = vec![window1, window2];

        // When
        let workspace = Workspace::new(
            workspace_id,
            "Coding".to_string(),
            windows,
            TilingDirection::Vertical,
            FocusState::Focused,
            DisplayState::Shown,
        );

        // Then
        assert_eq!(workspace.windows().len(), 2);
        assert!(!workspace.is_empty());
        assert_eq!(workspace.tiling_direction(), &TilingDirection::Vertical);
        assert!(workspace.is_focused());
    }
}

mod workspace_window_management {
    use super::*;

    #[test]
    fn should_add_window_successfully() {
        // Given
        let workspace_id = WorkspaceId::new("workspace-1".to_string());
        let mut workspace = Workspace::new(
            workspace_id,
            "Test".to_string(),
            Vec::new(),
            TilingDirection::Horizontal,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        let window = create_test_window("new-window", "Notepad", "notepad");

        // When
        let result = workspace.add_window(window.clone());

        // Then
        assert!(result.is_ok());
        assert_eq!(workspace.windows().len(), 1);
        assert_eq!(workspace.windows()[0].id(), window.id());
    }

    #[test]
    fn should_remove_window_successfully() {
        // Given
        let workspace_id = WorkspaceId::new("workspace-1".to_string());
        let window1 = create_test_window("window-1", "VS Code", "Code");
        let window2 = create_test_window("window-2", "Chrome", "chrome");
        let window1_id = window1.id().clone();

        let mut workspace = Workspace::new(
            workspace_id,
            "Test".to_string(),
            vec![window1, window2],
            TilingDirection::Horizontal,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        // When
        let result = workspace.remove_window(&window1_id);

        // Then
        assert!(result.is_ok());
        assert_eq!(workspace.windows().len(), 1);
        assert_ne!(workspace.windows()[0].id(), &window1_id);
    }

    #[test]
    fn should_return_error_when_removing_nonexistent_window() {
        // Given
        let workspace_id = WorkspaceId::new("workspace-1".to_string());
        let window = create_test_window("window-1", "VS Code", "Code");
        let mut workspace = Workspace::new(
            workspace_id,
            "Test".to_string(),
            vec![window],
            TilingDirection::Horizontal,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        let nonexistent_id = WindowId::new("nonexistent".to_string());

        // When
        let result = workspace.remove_window(&nonexistent_id);

        // Then
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DomainError::WindowNotFound { .. }
        ));
        assert_eq!(workspace.windows().len(), 1); // No change
    }

    #[test]
    fn should_prevent_adding_duplicate_window_ids() {
        // Given
        let workspace_id = WorkspaceId::new("workspace-1".to_string());
        let window1 = create_test_window("same-id", "Window 1", "app1");
        let window2 = create_test_window("same-id", "Window 2", "app2");

        let mut workspace = Workspace::new(
            workspace_id,
            "Test".to_string(),
            vec![window1],
            TilingDirection::Horizontal,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        // When
        let result = workspace.add_window(window2);

        // Then
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DomainError::DuplicateWindowId { .. }
        ));
        assert_eq!(workspace.windows().len(), 1);
    }
}

mod workspace_focus_management {
    use super::*;

    #[test]
    fn should_find_focused_window() {
        // Given
        let workspace_id = WorkspaceId::new("workspace-1".to_string());
        let unfocused_window = create_focused_window("window-1", "Chrome", "chrome", false);
        let focused_window = create_focused_window("window-2", "VS Code", "Code", true);

        let workspace = Workspace::new(
            workspace_id,
            "Test".to_string(),
            vec![unfocused_window, focused_window],
            TilingDirection::Horizontal,
            FocusState::Focused,
            DisplayState::Shown,
        );

        // When
        let focused = workspace.focused_window();

        // Then
        assert!(focused.is_some());
        assert_eq!(focused.unwrap().title(), "VS Code");
        assert!(focused.unwrap().is_focused());
    }

    #[test]
    fn should_return_none_when_no_focused_window() {
        // Given
        let workspace_id = WorkspaceId::new("workspace-1".to_string());
        let window1 = create_focused_window("window-1", "Chrome", "chrome", false);
        let window2 = create_focused_window("window-2", "Firefox", "firefox", false);

        let workspace = Workspace::new(
            workspace_id,
            "Test".to_string(),
            vec![window1, window2],
            TilingDirection::Horizontal,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        // When
        let focused = workspace.focused_window();

        // Then
        assert!(focused.is_none());
    }

    #[test]
    fn should_count_windows_correctly() {
        // Given
        let workspace_id = WorkspaceId::new("workspace-1".to_string());
        let windows = vec![
            create_test_window("w1", "App1", "app1"),
            create_test_window("w2", "App2", "app2"),
            create_test_window("w3", "App3", "app3"),
        ];

        let workspace = Workspace::new(
            workspace_id,
            "Test".to_string(),
            windows,
            TilingDirection::Horizontal,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        // When/Then
        assert_eq!(workspace.window_count(), 3);
        assert!(!workspace.is_empty());
    }
}

mod workspace_layout_calculation {
    use super::*;

    #[test]
    fn should_calculate_horizontal_layout() {
        // Given
        let workspace_id = WorkspaceId::new("workspace-1".to_string());
        let windows = vec![
            create_test_window("w1", "VS Code", "Code"),
            create_test_window("w2", "Chrome", "chrome"),
        ];

        let workspace = Workspace::new(
            workspace_id,
            "Test".to_string(),
            windows,
            TilingDirection::Horizontal,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        let container_size = Size::new(1920, 1080);

        // When
        let layout = workspace.calculate_layout(container_size);

        // Then
        assert_eq!(layout.len(), 2);

        // Horizontal layout - windows should be side by side
        let total_width = layout.iter().map(|l| l.size.width).sum::<u32>();
        assert_eq!(total_width, container_size.width);

        // All windows should have same height
        assert!(layout
            .iter()
            .all(|l| l.size.height == container_size.height));
    }

    #[test]
    fn should_calculate_vertical_layout() {
        // Given
        let workspace_id = WorkspaceId::new("workspace-1".to_string());
        let windows = vec![
            create_test_window("w1", "Terminal", "powershell"),
            create_test_window("w2", "Editor", "notepad"),
        ];

        let workspace = Workspace::new(
            workspace_id,
            "Test".to_string(),
            windows,
            TilingDirection::Vertical,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        let container_size = Size::new(1920, 1080);

        // When
        let layout = workspace.calculate_layout(container_size);

        // Then
        assert_eq!(layout.len(), 2);

        // Vertical layout - windows should be stacked
        let total_height = layout.iter().map(|l| l.size.height).sum::<u32>();
        assert_eq!(total_height, container_size.height);

        // All windows should have same width
        assert!(layout.iter().all(|l| l.size.width == container_size.width));
    }
}

// Helper functions for test data creation

fn create_test_window(id: &str, title: &str, process: &str) -> Window {
    Window::new(
        WindowId::new(id.to_string()),
        title.to_string(),
        process.to_string(),
        Rectangle::new(Position::new(0, 0), Size::new(800, 600)),
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
        Rectangle::new(Position::new(0, 0), Size::new(800, 600)),
        WindowState::Tiling,
        if focused {
            FocusState::Focused
        } else {
            FocusState::Unfocused
        },
        DisplayState::Shown,
    )
}

use glazewm_debug::domain::values::{Position, Rectangle, Size};
use glazewm_debug::domain::{DisplayState, DomainError, FocusState, TilingDirection, WindowState};
use glazewm_debug::domain::{Monitor, MonitorId, Window, WindowId, Workspace, WorkspaceId};

mod monitor_creation {
    use super::*;

    #[test]
    fn should_create_monitor_with_basic_properties() {
        // Given
        let monitor_id = MonitorId::new("monitor-1".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(1920, 1080));

        // When
        let monitor = Monitor::new(
            monitor_id.clone(),
            geometry,
            Vec::new(),
            FocusState::Focused,
            96,  // DPI
            1.0, // Scale factor
        );

        // Then
        assert_eq!(monitor.id(), &monitor_id);
        assert_eq!(monitor.geometry(), &geometry);
        assert_eq!(monitor.workspaces().len(), 0);
        assert!(monitor.is_focused());
        assert_eq!(monitor.dpi(), 96);
        assert_eq!(monitor.scale_factor(), 1.0);
    }

    #[test]
    fn should_create_monitor_with_workspaces() {
        // Given
        let monitor_id = MonitorId::new("monitor-1".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(2560, 1440));

        let workspace1 = create_test_workspace("ws-1", "Development", true);
        let workspace2 = create_test_workspace("ws-2", "Communication", false);
        let workspaces = vec![workspace1, workspace2];

        // When
        let monitor = Monitor::new(
            monitor_id,
            geometry,
            workspaces,
            FocusState::Focused,
            144, // High DPI
            1.5, // 150% scaling
        );

        // Then
        assert_eq!(monitor.workspaces().len(), 2);
        assert_eq!(monitor.dpi(), 144);
        assert_eq!(monitor.scale_factor(), 1.5);

        // Should have one active workspace
        assert_eq!(monitor.active_workspaces().len(), 1);
        assert_eq!(monitor.active_workspaces()[0].name(), "Development");
    }
}

mod monitor_workspace_management {
    use super::*;

    #[test]
    fn should_add_workspace_successfully() {
        // Given
        let monitor_id = MonitorId::new("monitor-1".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(1920, 1080));
        let mut monitor = Monitor::new(
            monitor_id,
            geometry,
            Vec::new(),
            FocusState::Focused,
            96,
            1.0,
        );

        let workspace = create_test_workspace("new-ws", "New Workspace", false);

        // When
        let result = monitor.add_workspace(workspace.clone());

        // Then
        assert!(result.is_ok());
        assert_eq!(monitor.workspaces().len(), 1);
        assert_eq!(monitor.workspaces()[0].id(), workspace.id());
    }

    #[test]
    fn should_remove_workspace_successfully() {
        // Given
        let monitor_id = MonitorId::new("monitor-1".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(1920, 1080));

        let workspace1 = create_test_workspace("ws-1", "Keep", false);
        let workspace2 = create_test_workspace("ws-2", "Remove", false);
        let workspace2_id = workspace2.id().clone();

        let mut monitor = Monitor::new(
            monitor_id,
            geometry,
            vec![workspace1, workspace2],
            FocusState::Focused,
            96,
            1.0,
        );

        // When
        let result = monitor.remove_workspace(&workspace2_id);

        // Then
        assert!(result.is_ok());
        assert_eq!(monitor.workspaces().len(), 1);
        assert_eq!(monitor.workspaces()[0].name(), "Keep");
    }

    #[test]
    fn should_enforce_single_active_workspace() {
        // Given
        let monitor_id = MonitorId::new("monitor-1".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(1920, 1080));

        let workspace1 = create_test_workspace("ws-1", "First", true);
        let workspace2 = create_test_workspace("ws-2", "Second", false);

        let mut monitor = Monitor::new(
            monitor_id,
            geometry,
            vec![workspace1, workspace2],
            FocusState::Focused,
            96,
            1.0,
        );

        let new_active_workspace = create_test_workspace("ws-3", "Third", true);

        // When - Adding another active workspace should deactivate others
        let result = monitor.add_workspace(new_active_workspace);

        // Then
        assert!(result.is_ok());

        // Only one workspace should be active
        let active_workspaces = monitor.active_workspaces();
        assert_eq!(active_workspaces.len(), 1);
        assert_eq!(active_workspaces[0].name(), "Third");
    }
}

mod monitor_focus_management {
    use super::*;

    #[test]
    fn should_find_focused_window_across_workspaces() {
        // Given
        let monitor_id = MonitorId::new("monitor-1".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(1920, 1080));

        // Workspace 1 with unfocused windows
        let ws1_windows = vec![
            create_focused_window("w1", "Chrome", "chrome", false),
            create_focused_window("w2", "Firefox", "firefox", false),
        ];
        let workspace1 = Workspace::new(
            WorkspaceId::new("ws-1".to_string()),
            "Browsers".to_string(),
            ws1_windows,
            TilingDirection::Horizontal,
            FocusState::Unfocused,
            DisplayState::Shown,
        );

        // Workspace 2 with focused window
        let ws2_windows = vec![
            create_focused_window("w3", "VS Code", "Code", true), // ← Focused
            create_focused_window("w4", "Notepad", "notepad", false),
        ];
        let workspace2 = Workspace::new(
            WorkspaceId::new("ws-2".to_string()),
            "Development".to_string(),
            ws2_windows,
            TilingDirection::Vertical,
            FocusState::Focused,
            DisplayState::Shown,
        );

        let monitor = Monitor::new(
            monitor_id,
            geometry,
            vec![workspace1, workspace2],
            FocusState::Focused,
            96,
            1.0,
        );

        // When
        let focused_window = monitor.focused_window();

        // Then
        assert!(focused_window.is_some());
        assert_eq!(focused_window.unwrap().title(), "VS Code");
        assert_eq!(focused_window.unwrap().process_name(), "Code");
    }

    #[test]
    fn should_return_none_when_no_focused_window() {
        // Given
        let monitor_id = MonitorId::new("monitor-1".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(1920, 1080));

        let workspace = create_workspace_with_unfocused_windows();
        let monitor = Monitor::new(
            monitor_id,
            geometry,
            vec![workspace],
            FocusState::Focused,
            96,
            1.0,
        );

        // When
        let focused_window = monitor.focused_window();

        // Then
        assert!(focused_window.is_none());
    }

    #[test]
    fn should_calculate_total_window_count() {
        // Given
        let monitor_id = MonitorId::new("monitor-1".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(1920, 1080));

        let workspace1 = create_workspace_with_window_count(3);
        let workspace2 = create_workspace_with_window_count(2);
        let workspace3 = create_workspace_with_window_count(0); // Empty

        let monitor = Monitor::new(
            monitor_id,
            geometry,
            vec![workspace1, workspace2, workspace3],
            FocusState::Focused,
            96,
            1.0,
        );

        // When
        let total_windows = monitor.total_window_count();

        // Then
        assert_eq!(total_windows, 5); // 3 + 2 + 0
    }
}

mod monitor_validation {
    use super::*;

    #[test]
    fn should_validate_monitor_geometry() {
        // Given - Invalid geometry (zero size)
        let monitor_id = MonitorId::new("invalid-monitor".to_string());
        let invalid_geometry = Rectangle::new(Position::new(0, 0), Size::new(0, 0));

        // When
        let result = Monitor::try_new(
            monitor_id,
            invalid_geometry,
            Vec::new(),
            FocusState::Focused,
            96,
            1.0,
        );

        // Then
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DomainError::InvalidGeometry { .. }
        ));
    }

    #[test]
    fn should_validate_dpi_values() {
        // Given
        let monitor_id = MonitorId::new("monitor-1".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(1920, 1080));

        // When - Invalid DPI (too low)
        let result = Monitor::try_new(
            monitor_id,
            geometry,
            Vec::new(),
            FocusState::Focused,
            0, // Invalid DPI
            1.0,
        );

        // Then
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DomainError::InvalidDpi { .. }
        ));
    }

    #[test]
    fn should_validate_scale_factor() {
        // Given
        let monitor_id = MonitorId::new("monitor-1".to_string());
        let geometry = Rectangle::new(Position::new(0, 0), Size::new(1920, 1080));

        // When - Invalid scale factor (negative)
        let result = Monitor::try_new(
            monitor_id,
            geometry,
            Vec::new(),
            FocusState::Focused,
            96,
            -1.0, // Invalid scale factor
        );

        // Then
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DomainError::InvalidScaleFactor { .. }
        ));
    }
}

// Helper functions

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

fn create_workspace_with_unfocused_windows() -> Workspace {
    let windows = vec![
        create_focused_window("w1", "Chrome", "chrome", false),
        create_focused_window("w2", "Firefox", "firefox", false),
    ];

    Workspace::new(
        WorkspaceId::new("ws-unfocused".to_string()),
        "No Focus".to_string(),
        windows,
        TilingDirection::Horizontal,
        FocusState::Unfocused,
        DisplayState::Shown,
    )
}

fn create_workspace_with_window_count(count: usize) -> Workspace {
    let windows = (0..count)
        .map(|i| create_test_window(&format!("window-{}", i), &format!("App {}", i), "app"))
        .collect();

    Workspace::new(
        WorkspaceId::new(format!("ws-{}", count)),
        format!("Workspace with {} windows", count),
        windows,
        TilingDirection::Horizontal,
        FocusState::Unfocused,
        DisplayState::Shown,
    )
}

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

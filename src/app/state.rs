// Application state management
// Central state for the entire application

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::Monitor;
use crate::tui::DisplayMode;

/// Central application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current monitors and their workspaces/windows
    monitors: Arc<RwLock<Vec<Monitor>>>,
    /// Whether the application should continue running
    running: Arc<RwLock<bool>>,
    /// Last update timestamp for debugging
    last_update: Arc<RwLock<Option<std::time::Instant>>>,
    /// Current display mode for the TUI
    display_mode: Arc<RwLock<DisplayMode>>,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> Self {
        Self {
            monitors: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(true)),
            last_update: Arc::new(RwLock::new(None)),
            display_mode: Arc::new(RwLock::new(DisplayMode::Detailed)),
        }
    }

    /// Update monitors from CLI data
    pub async fn update_monitors(&self, monitors: Vec<Monitor>) {
        let mut current_monitors = self.monitors.write().await;
        *current_monitors = monitors;

        let mut last_update = self.last_update.write().await;
        *last_update = Some(std::time::Instant::now());
    }

    /// Get current monitors (read-only)
    pub async fn get_monitors(&self) -> Vec<Monitor> {
        self.monitors.read().await.clone()
    }

    /// Check if the application should continue running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Signal the application to stop
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Get time since last update
    pub async fn time_since_last_update(&self) -> Option<std::time::Duration> {
        let last_update = self.last_update.read().await;
        last_update.map(|instant| instant.elapsed())
    }

    /// Get monitor count
    pub async fn monitor_count(&self) -> usize {
        self.monitors.read().await.len()
    }

    /// Get total window count across all monitors
    pub async fn total_window_count(&self) -> usize {
        let monitors = self.monitors.read().await;
        monitors.iter().map(|m| m.total_window_count()).sum()
    }

    /// Get the focused monitor, if any
    pub async fn focused_monitor(&self) -> Option<Monitor> {
        let monitors = self.monitors.read().await;
        monitors.iter().find(|m| m.is_focused()).cloned()
    }

    /// Get current display mode
    pub async fn get_display_mode(&self) -> DisplayMode {
        *self.display_mode.read().await
    }

    /// Set display mode
    pub async fn set_display_mode(&self, mode: DisplayMode) {
        let mut current_mode = self.display_mode.write().await;
        *current_mode = mode;
    }

    /// Toggle display mode between Detailed and Compact
    pub async fn toggle_display_mode(&self) {
        let mut current_mode = self.display_mode.write().await;
        *current_mode = match *current_mode {
            DisplayMode::Detailed => DisplayMode::Compact,
            DisplayMode::Compact => DisplayMode::Detailed,
        };
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        values::{MonitorId, Position, Rectangle, Size},
        FocusState,
    };

    #[tokio::test]
    async fn should_create_empty_state() {
        let state = AppState::new();

        assert!(state.is_running().await);
        assert_eq!(state.monitor_count().await, 0);
        assert_eq!(state.total_window_count().await, 0);
        assert!(state.time_since_last_update().await.is_none());
    }

    #[tokio::test]
    async fn should_update_monitors() {
        let state = AppState::new();

        let monitor = Monitor::new(
            MonitorId::new("test".to_string()),
            Rectangle::new(Position::new(0, 0), Size::new(1920, 1080)),
            Vec::new(),
            FocusState::Focused,
            96,
            1.0,
        );

        state.update_monitors(vec![monitor.clone()]).await;

        assert_eq!(state.monitor_count().await, 1);
        assert!(state.time_since_last_update().await.is_some());

        let monitors = state.get_monitors().await;
        assert_eq!(monitors.len(), 1);
        assert_eq!(monitors[0].id(), monitor.id());
    }

    #[tokio::test]
    async fn should_stop_application() {
        let state = AppState::new();

        assert!(state.is_running().await);

        state.stop().await;

        assert!(!state.is_running().await);
    }

    #[tokio::test]
    async fn should_find_focused_monitor() {
        let state = AppState::new();

        let focused_monitor = Monitor::new(
            MonitorId::new("focused".to_string()),
            Rectangle::new(Position::new(0, 0), Size::new(1920, 1080)),
            Vec::new(),
            FocusState::Focused,
            96,
            1.0,
        );

        let unfocused_monitor = Monitor::new(
            MonitorId::new("unfocused".to_string()),
            Rectangle::new(Position::new(1920, 0), Size::new(1920, 1080)),
            Vec::new(),
            FocusState::Unfocused,
            96,
            1.0,
        );

        state
            .update_monitors(vec![unfocused_monitor, focused_monitor.clone()])
            .await;

        let result = state.focused_monitor().await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().id(), focused_monitor.id());
    }

    #[tokio::test]
    async fn should_toggle_display_mode() {
        let state = AppState::new();

        // Initial mode should be Detailed
        assert_eq!(state.get_display_mode().await, DisplayMode::Detailed);

        // Toggle to Compact
        state.toggle_display_mode().await;
        assert_eq!(state.get_display_mode().await, DisplayMode::Compact);

        // Toggle back to Detailed
        state.toggle_display_mode().await;
        assert_eq!(state.get_display_mode().await, DisplayMode::Detailed);
    }

    #[tokio::test]
    async fn should_set_display_mode_directly() {
        let state = AppState::new();

        // Set to Compact
        state.set_display_mode(DisplayMode::Compact).await;
        assert_eq!(state.get_display_mode().await, DisplayMode::Compact);

        // Set to Detailed
        state.set_display_mode(DisplayMode::Detailed).await;
        assert_eq!(state.get_display_mode().await, DisplayMode::Detailed);
    }
}

// JSON parser for glazewm CLI responses
// Converts JSON responses from glazewm into domain models

use serde::Deserialize;
use serde_json::Value;

use crate::cli::errors::CliError;
use crate::domain::{
    values::{Position, Rectangle, Size},
    DisplayState, FocusState, Monitor, MonitorId, TilingDirection, Window, WindowId, WindowState,
    Workspace, WorkspaceId,
};

/// Raw JSON structures from glazewm CLI

#[derive(Debug, Deserialize)]
struct MonitorResponse {
    monitors: Vec<RawMonitor>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawMonitor {
    #[serde(rename = "type")]
    monitor_type: String,
    id: String,
    #[serde(default)]
    parent_id: Option<String>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    scale_factor: f64,
    dpi: i32,
    #[serde(rename = "hasFocus")]
    has_focus: bool,
    children: Vec<RawMonitorChild>,
    #[serde(default, rename = "childFocusOrder")]
    child_focus_order: Vec<String>,
    #[serde(default)]
    handle: Option<i64>,
    #[serde(default)]
    device_name: Option<String>,
    #[serde(default)]
    device_path: Option<String>,
    #[serde(default)]
    hardware_id: Option<String>,
    #[serde(default)]
    working_rect: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
enum RawMonitorChild {
    #[serde(rename = "workspace")]
    Workspace {
        id: String,
        name: String,
        #[serde(default)]
        display_name: Option<String>,
        #[serde(default)]
        parent_id: Option<String>,
        #[serde(rename = "hasFocus")]
        has_focus: bool,
        #[serde(rename = "isDisplayed")]
        is_displayed: bool,
        #[serde(rename = "tilingDirection")]
        tiling_direction: String,
        children: Vec<RawWorkspaceChild>,
        #[serde(default, rename = "childFocusOrder")]
        child_focus_order: Vec<String>,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default)]
        width: u32,
        #[serde(default)]
        height: u32,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
enum RawWorkspaceChild {
    #[serde(rename = "window")]
    Window {
        id: String,
        #[serde(default)]
        parent_id: Option<String>,
        #[serde(rename = "hasFocus")]
        has_focus: bool,
        #[serde(default)]
        tiling_size: Option<f64>,
        width: u32,
        height: u32,
        x: i32,
        y: i32,
        state: RawWindowState,
        #[serde(default)]
        prev_state: Option<serde_json::Value>,
        #[serde(rename = "displayState")]
        display_state: String,
        #[serde(default)]
        border_delta: Option<serde_json::Value>,
        #[serde(default)]
        floating_placement: Option<serde_json::Value>,
        #[serde(default)]
        handle: Option<i64>,
        title: String,
        #[serde(default)]
        class_name: Option<String>,
        #[serde(rename = "processName")]
        process_name: String,
        #[serde(default)]
        active_drag: Option<serde_json::Value>,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawWindowState {
    #[serde(rename = "type")]
    state_type: String,
}

/// Parser for converting glazewm JSON responses to domain models
pub struct GlazewmParser;

impl GlazewmParser {
    /// Parse monitors response from glazewm
    pub fn parse_monitors(json: &Value) -> Result<Vec<Monitor>, CliError> {
        let response: MonitorResponse = serde_json::from_value(json["data"].clone())?;

        let mut monitors = Vec::new();

        for raw_monitor in response.monitors {
            let monitor = Self::convert_monitor(raw_monitor)?;
            monitors.push(monitor);
        }

        Ok(monitors)
    }

    /// Parse windows response from glazewm and group by workspace
    /// Note: glazewm monitors response already includes windows in workspace children,
    /// so this method is primarily for separate window querying if needed
    pub fn parse_windows(_json: &Value, monitors: &[Monitor]) -> Result<Vec<Monitor>, CliError> {
        // In glazewm, windows are already included in the monitor/workspace hierarchy
        // So we just return the existing monitors for now
        Ok(monitors.to_vec())
    }

    fn convert_monitor(raw: RawMonitor) -> Result<Monitor, CliError> {
        let geometry = Rectangle::new(
            Position::new(raw.x, raw.y),
            Size::new(raw.width, raw.height),
        );

        let focus_state = if raw.has_focus {
            FocusState::Focused
        } else {
            FocusState::Unfocused
        };

        let mut workspaces = Vec::new();
        for child in raw.children {
            if let RawMonitorChild::Workspace {
                id,
                name,
                display_name: _,
                parent_id: _,
                has_focus,
                is_displayed,
                tiling_direction,
                children,
                child_focus_order: _,
                x: _,
                y: _,
                width: _,
                height: _,
            } = child
            {
                let workspace = Self::convert_workspace_from_raw(
                    id,
                    name,
                    has_focus,
                    is_displayed,
                    tiling_direction,
                    children,
                )?;
                workspaces.push(workspace);
            }
        }

        Monitor::try_new(
            MonitorId::new(raw.id),
            geometry,
            workspaces,
            focus_state,
            raw.dpi,
            raw.scale_factor,
        )
        .map_err(|e| CliError::JsonParseError {
            message: format!("Failed to create monitor: {}", e),
        })
    }

    fn convert_workspace_from_raw(
        id: String,
        name: String,
        has_focus: bool,
        is_displayed: bool,
        tiling_direction: String,
        children: Vec<RawWorkspaceChild>,
    ) -> Result<Workspace, CliError> {
        let focus_state = if has_focus {
            FocusState::Focused
        } else {
            FocusState::Unfocused
        };

        let display_state = if is_displayed {
            DisplayState::Shown
        } else {
            DisplayState::Hidden
        };

        let workspace_tiling_direction = match tiling_direction.as_str() {
            "horizontal" => TilingDirection::Horizontal,
            "vertical" => TilingDirection::Vertical,
            _ => TilingDirection::Horizontal, // Default fallback
        };

        // Convert windows from children
        let mut windows = Vec::new();
        for child in children {
            if let RawWorkspaceChild::Window {
                id: window_id,
                parent_id: _,
                has_focus: window_has_focus,
                tiling_size: _,
                width,
                height,
                x,
                y,
                state,
                prev_state: _,
                display_state: window_display_state,
                border_delta: _,
                floating_placement: _,
                handle: _,
                title,
                class_name: _,
                process_name,
                active_drag: _,
            } = child
            {
                let window = Self::convert_window_from_raw(
                    window_id,
                    window_has_focus,
                    width,
                    height,
                    x,
                    y,
                    state,
                    window_display_state,
                    title,
                    process_name,
                )?;
                windows.push(window);
            }
        }

        Ok(Workspace::new(
            WorkspaceId::new(id),
            name,
            windows,
            workspace_tiling_direction,
            focus_state,
            display_state,
        ))
    }

    fn convert_window_from_raw(
        id: String,
        has_focus: bool,
        width: u32,
        height: u32,
        x: i32,
        y: i32,
        state: RawWindowState,
        display_state: String,
        title: String,
        process_name: String,
    ) -> Result<Window, CliError> {
        let geometry = Rectangle::new(Position::new(x, y), Size::new(width, height));

        let window_state = match state.state_type.as_str() {
            "tiling" => WindowState::Tiling,
            "floating" => WindowState::Floating,
            "fullscreen" => WindowState::Fullscreen,
            "minimized" => WindowState::Minimized,
            _ => WindowState::Tiling, // Default fallback
        };

        let focus_state = if has_focus {
            FocusState::Focused
        } else {
            FocusState::Unfocused
        };

        let window_display_state = match display_state.as_str() {
            "shown" => DisplayState::Shown,
            "hidden" => DisplayState::Hidden,
            _ => DisplayState::Shown, // Default fallback
        };

        Ok(Window::new(
            WindowId::new(id),
            title,
            process_name,
            geometry,
            window_state,
            focus_state,
            window_display_state,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_monitor_json() {
        let json = serde_json::json!({
            "data": {
                "monitors": [{
                    "type": "monitor",
                    "id": "monitor-1",
                    "x": 0,
                    "y": 0,
                    "width": 1920,
                    "height": 1080,
                    "scaleFactor": 1.0,
                    "dpi": 96,
                    "hasFocus": true,
                    "children": [{
                        "type": "workspace",
                        "id": "ws-1",
                        "name": "Workspace 1",
                        "hasFocus": true,
                        "isDisplayed": true,
                        "tilingDirection": "horizontal",
                        "children": [],
                        "childFocusOrder": []
                    }],
                    "childFocusOrder": ["ws-1"]
                }]
            }
        });

        let monitors = GlazewmParser::parse_monitors(&json).unwrap();

        assert_eq!(monitors.len(), 1);
        assert!(monitors[0].is_focused());
        assert_eq!(monitors[0].workspace_count(), 1);
    }

    #[test]
    fn should_parse_workspace_with_windows() {
        let json = serde_json::json!({
            "data": {
                "monitors": [{
                    "type": "monitor",
                    "id": "monitor-1",
                    "x": 0,
                    "y": 0,
                    "width": 1920,
                    "height": 1080,
                    "scaleFactor": 1.0,
                    "dpi": 96,
                    "hasFocus": true,
                    "children": [{
                        "type": "workspace",
                        "id": "ws-1",
                        "name": "Workspace 1",
                        "hasFocus": true,
                        "isDisplayed": true,
                        "tilingDirection": "horizontal",
                        "children": [{
                            "type": "window",
                            "id": "window-1",
                            "hasFocus": true,
                            "width": 800,
                            "height": 600,
                            "x": 100,
                            "y": 100,
                            "state": { "type": "tiling" },
                            "displayState": "shown",
                            "title": "VS Code",
                            "processName": "Code"
                        }]
                    }]
                }]
            }
        });

        let monitors = GlazewmParser::parse_monitors(&json).unwrap();

        assert_eq!(monitors.len(), 1);
        assert_eq!(monitors[0].workspace_count(), 1);
        assert_eq!(monitors[0].total_window_count(), 1);
    }

    #[test]
    fn should_handle_invalid_json() {
        let invalid_json = serde_json::json!({
            "invalid": "structure"
        });

        let result = GlazewmParser::parse_monitors(&invalid_json);
        assert!(result.is_err());
    }
}

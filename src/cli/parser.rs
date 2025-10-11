// JSON parser for glazewm CLI responses
// Converts JSON responses from glazewm into domain models

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

use crate::cli::errors::CliError;
use crate::domain::{
    values::{Position, Rectangle, Size},
    DisplayState, FocusState, Monitor, MonitorId, TilingDirection, Window, WindowId, WindowState,
    Workspace, WorkspaceId,
};

/// Raw JSON structures from glazewm CLI
#[derive(Debug, Deserialize)]
struct GlazewmResponse {
    success: bool,
    data: Value,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MonitorResponse {
    monitors: Vec<RawMonitor>,
}

#[derive(Debug, Deserialize)]
struct WindowResponse {
    windows: Vec<RawWindow>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawMonitor {
    id: String,
    name: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    scale_factor: f64,
    dpi: i32,
    has_focus: bool,
    workspaces: Vec<RawWorkspace>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawWorkspace {
    id: String,
    name: String,
    has_focus: bool,
    display_state: String,
    tiling_direction: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawWindow {
    #[serde(rename = "type")]
    window_type: String,
    id: String,
    parent_id: String,
    has_focus: bool,
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    state: RawWindowState,
    display_state: String,
    title: String,
    class_name: String,
    process_name: String,
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
    pub fn parse_windows(
        json: &Value,
        monitors: &[Monitor],
    ) -> Result<Vec<Monitor>, CliError> {
        let response: WindowResponse = serde_json::from_value(json["data"].clone())?;
        
        // Group windows by their parent workspace ID
        let mut windows_by_workspace: HashMap<String, Vec<Window>> = HashMap::new();
        
        for raw_window in response.windows {
            if raw_window.window_type == "window" {
                let window = Self::convert_window(raw_window)?;
                windows_by_workspace
                    .entry(window.id().to_string()) // Using window ID as workspace key for now
                    .or_insert_with(Vec::new)
                    .push(window);
            }
        }

        // For now, return the monitors as-is since we need more workspace parsing logic
        // TODO: Implement proper workspace-window association
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

        let workspaces = raw
            .workspaces
            .into_iter()
            .map(Self::convert_workspace)
            .collect::<Result<Vec<_>, _>>()?;

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

    fn convert_workspace(raw: RawWorkspace) -> Result<Workspace, CliError> {
        let focus_state = if raw.has_focus {
            FocusState::Focused
        } else {
            FocusState::Unfocused
        };

        let display_state = match raw.display_state.as_str() {
            "shown" => DisplayState::Shown,
            "hidden" => DisplayState::Hidden,
            _ => DisplayState::Shown, // Default fallback
        };

        let tiling_direction = match raw.tiling_direction.as_str() {
            "horizontal" => TilingDirection::Horizontal,
            "vertical" => TilingDirection::Vertical,
            _ => TilingDirection::Horizontal, // Default fallback
        };

        Ok(Workspace::new(
            WorkspaceId::new(raw.id),
            raw.name,
            Vec::new(), // Windows will be added separately
            tiling_direction,
            focus_state,
            display_state,
        ))
    }

    fn convert_window(raw: RawWindow) -> Result<Window, CliError> {
        let geometry = Rectangle::new(
            Position::new(raw.x, raw.y),
            Size::new(raw.width, raw.height),
        );

        let state = match raw.state.state_type.as_str() {
            "tiling" => WindowState::Tiling,
            "floating" => WindowState::Floating,
            "fullscreen" => WindowState::Fullscreen,
            "minimized" => WindowState::Minimized,
            _ => WindowState::Tiling, // Default fallback
        };

        let focus_state = if raw.has_focus {
            FocusState::Focused
        } else {
            FocusState::Unfocused
        };

        let display_state = match raw.display_state.as_str() {
            "shown" => DisplayState::Shown,
            "hidden" => DisplayState::Hidden,
            _ => DisplayState::Shown, // Default fallback
        };

        Ok(Window::new(
            WindowId::new(raw.id),
            raw.title,
            raw.process_name,
            geometry,
            state,
            focus_state,
            display_state,
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
                    "id": "monitor-1",
                    "name": "Primary Monitor",
                    "x": 0,
                    "y": 0,
                    "width": 1920,
                    "height": 1080,
                    "scaleFactor": 1.0,
                    "dpi": 96,
                    "hasFocus": true,
                    "workspaces": [{
                        "id": "ws-1",
                        "name": "Workspace 1",
                        "hasFocus": true,
                        "displayState": "shown",
                        "tilingDirection": "horizontal"
                    }]
                }]
            }
        });

        let monitors = GlazewmParser::parse_monitors(&json).unwrap();
        
        assert_eq!(monitors.len(), 1);
        assert!(monitors[0].is_focused());
        assert_eq!(monitors[0].workspace_count(), 1);
    }

    #[test]
    fn should_parse_window_json() {
        let json = serde_json::json!({
            "data": {
                "windows": [{
                    "type": "window",
                    "id": "window-1",
                    "parentId": "ws-1",
                    "hasFocus": true,
                    "width": 800,
                    "height": 600,
                    "x": 100,
                    "y": 100,
                    "state": { "type": "tiling" },
                    "displayState": "shown",
                    "title": "VS Code",
                    "className": "Code",
                    "processName": "Code"
                }]
            }
        });

        let monitors = vec![];
        let result = GlazewmParser::parse_windows(&json, &monitors);
        
        assert!(result.is_ok());
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
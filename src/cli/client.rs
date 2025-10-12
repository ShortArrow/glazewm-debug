// Glazewm CLI client implementation
// Handles execution of glazewm commands and response processing

use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

use crate::cli::errors::CliError;

/// Trait for communicating with glazewm CLI
#[async_trait]
pub trait GlazewmClient {
    /// Query all monitors from glazewm
    async fn query_monitors(&self) -> Result<Value, CliError>;

    /// Query all windows from glazewm
    async fn query_windows(&self) -> Result<Value, CliError>;
}

/// Real implementation of GlazewmClient that executes actual commands
pub struct RealGlazewmClient {
    glazewm_path: PathBuf,
    command_timeout: Duration,
}

impl RealGlazewmClient {
    /// Create a new glazewm client
    pub fn new(glazewm_path: PathBuf, command_timeout: Duration) -> Self {
        Self {
            glazewm_path,
            command_timeout,
        }
    }

    /// Execute a glazewm query command
    async fn execute_query(&self, query_type: &str) -> Result<Value, CliError> {
        let command_str = format!("{} query {}", self.glazewm_path.display(), query_type);

        // Log the command we're trying to execute
        tracing::debug!("Executing command: {}", command_str);
        tracing::debug!("Glazewm path: {:?}", self.glazewm_path);
        tracing::debug!("Command timeout: {:?}", self.command_timeout);

        // Create command
        let mut cmd = Command::new(&self.glazewm_path);
        cmd.args(&["query", query_type]);

        // Execute with timeout
        let output = timeout(self.command_timeout, cmd.output())
            .await
            .map_err(|_| {
                tracing::error!("Command timed out: {}", command_str);
                CliError::CommandTimeout {
                    command: command_str.clone(),
                    timeout: self.command_timeout,
                }
            })?
            .map_err(|e| {
                tracing::error!("Command execution failed: {} - Error: {}", command_str, e);
                CliError::CommandExecutionFailed {
                    command: command_str.clone(),
                }
            })?;

        // Check exit status
        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(-1);
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(CliError::CommandFailed {
                command: command_str,
                code: exit_code,
                stderr,
            });
        }

        // Parse JSON response
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json_value: Value = serde_json::from_str(&stdout)?;

        // Validate basic response structure
        self.validate_response(&json_value)?;

        Ok(json_value)
    }

    /// Validate that the response has the expected structure
    fn validate_response(&self, response: &Value) -> Result<(), CliError> {
        // Check for success field
        if !response
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            return Err(CliError::InvalidJsonSchema {
                field: "success".to_string(),
            });
        }

        // Check for data field
        if response.get("data").is_none() {
            return Err(CliError::InvalidJsonSchema {
                field: "data".to_string(),
            });
        }

        Ok(())
    }
}

#[async_trait]
impl GlazewmClient for RealGlazewmClient {
    async fn query_monitors(&self) -> Result<Value, CliError> {
        self.execute_query("monitors").await
    }

    async fn query_windows(&self) -> Result<Value, CliError> {
        self.execute_query("windows").await
    }
}

/// Demo client that provides sample data without requiring glazewm
pub struct DemoGlazewmClient {
    window_count: std::sync::atomic::AtomicUsize,
}

impl DemoGlazewmClient {
    pub fn new() -> Self {
        Self {
            window_count: std::sync::atomic::AtomicUsize::new(1),
        }
    }

    fn get_demo_data(&self) -> Value {
        let window_count = self
            .window_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let focused_window = window_count % 4; // Rotate focus every 4 updates

        serde_json::json!({
            "success": true,
            "data": {
                "monitors": [
                    {
                        "type": "monitor",
                        "id": "demo-monitor-1",
                        "x": 0,
                        "y": 0,
                        "width": 1920,
                        "height": 1080,
                        "scaleFactor": 1.0,
                        "dpi": 96,
                        "hasFocus": true,
                        "children": [
                            {
                                "type": "workspace",
                                "id": "demo-workspace-1",
                                "name": "Development",
                                "hasFocus": true,
                                "isDisplayed": true,
                                "children": [
                                    {
                                        "type": "window",
                                        "id": "demo-window-1",
                                        "title": "Visual Studio Code - glazewm-debug",
                                        "processName": "Code",
                                        "className": "Chrome_WidgetWin_1",
                                        "x": 0,
                                        "y": 0,
                                        "width": 960,
                                        "height": 1040,
                                        "state": {"type": "tiling"},
                                        "hasFocus": focused_window == 0,
                                        "isDisplayed": true,
                                        "displayState": "shown"
                                    },
                                    {
                                        "type": "window",
                                        "id": "demo-window-2",
                                        "title": "Firefox - Documentation",
                                        "processName": "firefox",
                                        "className": "MozillaWindowClass",
                                        "x": 960,
                                        "y": 0,
                                        "width": 960,
                                        "height": 1040,
                                        "state": {"type": "tiling"},
                                        "hasFocus": focused_window == 1,
                                        "isDisplayed": true,
                                        "displayState": "shown"
                                    }
                                ]
                            },
                            {
                                "type": "workspace",
                                "id": "demo-workspace-2",
                                "name": "Testing",
                                "hasFocus": false,
                                "isDisplayed": false,
                                "children": [
                                    {
                                        "type": "window",
                                        "id": "demo-window-3",
                                        "title": "Terminal - cargo test",
                                        "processName": "wezterm-gui",
                                        "className": "WEZTERM_GUI_CLASS",
                                        "x": 0,
                                        "y": 0,
                                        "width": 1920,
                                        "height": 1040,
                                        "state": {"type": "tiling"},
                                        "hasFocus": false,
                                        "isDisplayed": false,
                                        "displayState": "hidden"
                                    }
                                ]
                            }
                        ]
                    },
                    {
                        "type": "monitor",
                        "id": "demo-monitor-2",
                        "x": 1920,
                        "y": 0,
                        "width": 2560,
                        "height": 1440,
                        "scaleFactor": 1.25,
                        "dpi": 120,
                        "hasFocus": false,
                        "children": [
                            {
                                "type": "workspace",
                                "id": "demo-workspace-3",
                                "name": "Communication",
                                "hasFocus": true,
                                "isDisplayed": true,
                                "children": [
                                    {
                                        "type": "window",
                                        "id": "demo-window-4",
                                        "title": "Discord - #general",
                                        "processName": "Discord",
                                        "className": "Chrome_WidgetWin_1",
                                        "x": 0,
                                        "y": 0,
                                        "width": 1280,
                                        "height": 1400,
                                        "state": {"type": "tiling"},
                                        "hasFocus": focused_window == 2,
                                        "isDisplayed": true,
                                        "displayState": "shown"
                                    },
                                    {
                                        "type": "window",
                                        "id": "demo-window-5",
                                        "title": "Spotify - Currently Playing",
                                        "processName": "Spotify",
                                        "className": "Chrome_WidgetWin_1",
                                        "x": 1280,
                                        "y": 0,
                                        "width": 1280,
                                        "height": 1400,
                                        "state": {"type": "floating"},
                                        "hasFocus": focused_window == 3,
                                        "isDisplayed": true,
                                        "displayState": "shown"
                                    }
                                ]
                            }
                        ]
                    }
                ]
            },
            "error": null
        })
    }
}

#[async_trait]
impl GlazewmClient for DemoGlazewmClient {
    async fn query_monitors(&self) -> Result<Value, CliError> {
        // Simulate some processing time
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(self.get_demo_data())
    }

    async fn query_windows(&self) -> Result<Value, CliError> {
        // For demo, windows are included in monitor query
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(serde_json::json!({
            "success": true,
            "data": { "windows": [] },
            "error": null
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn should_create_client_with_parameters() {
        let client = RealGlazewmClient::new(PathBuf::from("glazewm"), Duration::from_secs(5));

        assert_eq!(client.glazewm_path, PathBuf::from("glazewm"));
        assert_eq!(client.command_timeout, Duration::from_secs(5));
    }

    #[test]
    fn should_validate_successful_response() {
        let client = RealGlazewmClient::new(PathBuf::from("glazewm"), Duration::from_secs(5));

        let valid_response = serde_json::json!({
            "success": true,
            "data": {},
            "error": null
        });

        assert!(client.validate_response(&valid_response).is_ok());
    }

    #[test]
    fn should_reject_invalid_response() {
        let client = RealGlazewmClient::new(PathBuf::from("glazewm"), Duration::from_secs(5));

        let invalid_response = serde_json::json!({
            "success": false,
            "error": "Some error"
        });

        assert!(client.validate_response(&invalid_response).is_err());
    }
}

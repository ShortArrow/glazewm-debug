// Update loop for periodic CLI polling
// Manages the 1-second interval updates from glazewm

use std::path::PathBuf;
use std::time::Duration;
use tokio::time::{interval, timeout};
use tracing::{debug, error};

use crate::app::AppState;
use crate::cli::{CliError, GlazewmClient, GlazewmParser, RealGlazewmClient};

/// Error types for the update loop
#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("CLI error: {0}")]
    CliError(#[from] CliError),

    #[error("Update loop stopped")]
    Stopped,
}

/// Configuration for the update loop
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    /// How often to poll glazewm
    pub refresh_interval: Duration,
    /// Timeout for individual CLI commands
    pub command_timeout: Duration,
    /// Path to glazewm executable
    pub glazewm_path: PathBuf,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            refresh_interval: Duration::from_secs(1),
            command_timeout: Duration::from_secs(5),
            glazewm_path: PathBuf::from("glazewm"),
        }
    }
}

/// The main update loop that polls glazewm periodically
pub struct UpdateLoop {
    client: Box<dyn GlazewmClient + Send + Sync>,
    config: UpdateConfig,
    state: AppState,
}

impl UpdateLoop {
    /// Create a new update loop with a real client
    pub fn new(config: UpdateConfig, state: AppState) -> Self {
        let client = RealGlazewmClient::new(config.glazewm_path.clone(), config.command_timeout);

        Self {
            client: Box::new(client),
            config,
            state,
        }
    }

    /// Create update loop with a custom client (for testing)
    pub fn with_client(
        client: Box<dyn GlazewmClient + Send + Sync>,
        config: UpdateConfig,
        state: AppState,
    ) -> Self {
        Self {
            client,
            config,
            state,
        }
    }

    /// Start the update loop
    /// This will run until the application state is set to stop
    pub async fn run(&self) -> Result<(), UpdateError> {
        debug!(
            "Starting update loop with {:?} interval",
            self.config.refresh_interval
        );

        let mut interval_timer = interval(self.config.refresh_interval);

        while self.state.is_running().await {
            interval_timer.tick().await;

            match self.update_once().await {
                Ok(()) => {
                    debug!("Successfully updated state");
                }
                Err(UpdateError::CliError(cli_err)) => {
                    error!("CLI error during update: {}", cli_err);
                    // Continue running even on CLI errors
                    // User might have glazewm closed temporarily
                }
                Err(UpdateError::Stopped) => {
                    debug!("Update loop stopped by application state");
                    break;
                }
            }
        }

        debug!("Update loop finished");
        Ok(())
    }

    /// Perform a single update cycle
    async fn update_once(&self) -> Result<(), UpdateError> {
        if !self.state.is_running().await {
            return Err(UpdateError::Stopped);
        }

        // Query monitors first
        let monitors_json = timeout(self.config.command_timeout, self.client.query_monitors())
            .await
            .map_err(|_| CliError::CommandTimeout {
                command: "query monitors".to_string(),
                timeout: self.config.command_timeout,
            })??;

        // Parse monitors
        let monitors = GlazewmParser::parse_monitors(&monitors_json)?;

        // Note: Windows are already included in the monitor/workspace hierarchy from glazewm
        // No separate window parsing is needed

        // Update application state
        self.state.update_monitors(monitors).await;

        Ok(())
    }

    /// Get current application state
    pub fn state(&self) -> &AppState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::Value;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// Mock client for testing
    struct MockGlazewmClient {
        call_count: Arc<AtomicUsize>,
        should_fail: bool,
    }

    impl MockGlazewmClient {
        fn new(should_fail: bool) -> Self {
            Self {
                call_count: Arc::new(AtomicUsize::new(0)),
                should_fail,
            }
        }

        fn call_count(&self) -> usize {
            self.call_count.load(Ordering::Relaxed)
        }
    }

    #[async_trait]
    impl GlazewmClient for MockGlazewmClient {
        async fn query_monitors(&self) -> Result<Value, CliError> {
            self.call_count.fetch_add(1, Ordering::Relaxed);

            if self.should_fail {
                return Err(CliError::CommandExecutionFailed {
                    command: "query monitors".to_string(),
                });
            }

            Ok(serde_json::json!({
                "success": true,
                "data": {
                    "monitors": [{
                        "type": "monitor",
                        "id": "mock-monitor",
                        "x": 0,
                        "y": 0,
                        "width": 1920,
                        "height": 1080,
                        "scaleFactor": 1.0,
                        "dpi": 96,
                        "hasFocus": true,
                        "children": [],
                        "childFocusOrder": []
                    }]
                },
                "error": null
            }))
        }

        async fn query_windows(&self) -> Result<Value, CliError> {
            Ok(serde_json::json!({
                "success": true,
                "data": {
                    "windows": []
                },
                "error": null
            }))
        }
    }

    #[tokio::test]
    async fn should_create_update_loop() {
        let config = UpdateConfig::default();
        let state = AppState::new();
        let _update_loop = UpdateLoop::new(config, state);
    }

    #[tokio::test]
    async fn should_perform_single_update() {
        let config = UpdateConfig::default();
        let state = AppState::new();
        let client = MockGlazewmClient::new(false);
        let update_loop = UpdateLoop::with_client(Box::new(client), config, state.clone());

        let result = update_loop.update_once().await;
        assert!(result.is_ok());

        // Should have updated state with one monitor
        assert_eq!(state.monitor_count().await, 1);
    }

    #[tokio::test]
    async fn should_handle_cli_errors() {
        let config = UpdateConfig::default();
        let state = AppState::new();
        let client = MockGlazewmClient::new(true); // Will fail
        let update_loop = UpdateLoop::with_client(Box::new(client), config, state.clone());

        let result = update_loop.update_once().await;
        assert!(result.is_err());

        // State should remain unchanged
        assert_eq!(state.monitor_count().await, 0);
    }

    #[tokio::test]
    async fn should_stop_when_application_stops() {
        let config = UpdateConfig::default();
        let state = AppState::new();
        let client = MockGlazewmClient::new(false);
        let update_loop = UpdateLoop::with_client(Box::new(client), config, state.clone());

        // Stop the application
        state.stop().await;

        let result = update_loop.update_once().await;
        assert!(matches!(result.unwrap_err(), UpdateError::Stopped));
    }
}

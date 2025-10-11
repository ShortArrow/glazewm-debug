// Glazewm CLI client implementation
// Handles execution of glazewm commands and response processing

use std::path::PathBuf;
use std::time::Duration;
use async_trait::async_trait;
use serde_json::Value;
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
        
        // Create command
        let mut cmd = Command::new(&self.glazewm_path);
        cmd.args(&["query", query_type]);

        // Execute with timeout
        let output = timeout(self.command_timeout, cmd.output())
            .await
            .map_err(|_| CliError::CommandTimeout {
                command: command_str.clone(),
                timeout: self.command_timeout,
            })?
            .map_err(|_| CliError::CommandExecutionFailed {
                command: command_str.clone(),
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
        if !response.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn should_create_client_with_parameters() {
        let client = RealGlazewmClient::new(
            PathBuf::from("glazewm"),
            Duration::from_secs(5),
        );

        assert_eq!(client.glazewm_path, PathBuf::from("glazewm"));
        assert_eq!(client.command_timeout, Duration::from_secs(5));
    }

    #[test]
    fn should_validate_successful_response() {
        let client = RealGlazewmClient::new(
            PathBuf::from("glazewm"),
            Duration::from_secs(5),
        );

        let valid_response = serde_json::json!({
            "success": true,
            "data": {},
            "error": null
        });

        assert!(client.validate_response(&valid_response).is_ok());
    }

    #[test]
    fn should_reject_invalid_response() {
        let client = RealGlazewmClient::new(
            PathBuf::from("glazewm"),
            Duration::from_secs(5),
        );

        let invalid_response = serde_json::json!({
            "success": false,
            "error": "Some error"
        });

        assert!(client.validate_response(&invalid_response).is_err());
    }
}
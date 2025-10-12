// CLI error types
// Defines errors that can occur during CLI communication with glazewm

use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during CLI operations
#[derive(Error, Debug, Clone)]
pub enum CliError {
    #[error("Failed to execute command: {command}")]
    CommandExecutionFailed { command: String },

    #[error("Command timed out after {timeout:?}: {command}")]
    CommandTimeout { command: String, timeout: Duration },

    #[error("Command returned non-zero exit code {code}: {command}")]
    CommandFailed {
        command: String,
        code: i32,
        stderr: String,
    },

    #[error("Failed to parse JSON response: {message}")]
    JsonParseError { message: String },

    #[error("Invalid JSON schema: missing field '{field}'")]
    InvalidJsonSchema { field: String },

    #[error("glazewm executable not found at path: {path}")]
    GlazewmNotFound { path: String },

    #[error("IO error: {message}")]
    IoError { message: String },
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        CliError::IoError {
            message: error.to_string(),
        }
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        CliError::JsonParseError {
            message: error.to_string(),
        }
    }
}

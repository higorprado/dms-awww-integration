//! Comprehensive error types for dms-awww
//!
//! This module defines all error types used throughout the application,
//! using thiserror for clean error messages and source tracking.

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for dms-awww
#[derive(Debug, Error)]
pub enum DmsAwwwError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// IO errors from file operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing errors
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    /// Config file parsing errors
    #[error("Config file error: {0}")]
    ConfigFile(#[from] config::ConfigError),

    /// Command not found in PATH
    #[error("Command not found: {0}")]
    CommandNotFound(String),

    /// Command execution failed
    #[error("Command '{0}' failed with exit code {1}")]
    CommandFailed(String, i32),

    /// Command execution failed with signal
    #[error("Command '{0}' terminated by signal: {1}")]
    CommandTerminated(String, String),

    /// Invalid wallpaper path
    #[error("Invalid wallpaper path: {0}")]
    InvalidWallpaperPath(String),

    /// Niri IPC error
    #[error("Niri IPC error: {0}")]
    NiriIpc(String),

    /// File watcher error
    #[error("File watcher error: {0}")]
    Watcher(String),

    /// Path expansion error
    #[error("Failed to expand path '{path}': {reason}")]
    PathExpansion { path: String, reason: String },

    /// Missing required configuration value
    #[error("Missing required configuration: {0}")]
    MissingConfig(String),

    /// Invalid configuration value
    #[error("Invalid configuration value for '{key}': {reason}")]
    InvalidConfig { key: String, reason: String },

    /// DMS session file not found
    #[error("DMS session file not found: {0}")]
    SessionFileNotFound(PathBuf),

    /// DMS settings file not found
    #[error("DMS settings file not found: {0}")]
    SettingsFileNotFound(PathBuf),

    /// No monitors detected
    #[error("No monitors detected")]
    NoMonitorsDetected,

    /// Per-monitor wallpaper parsing error
    #[error("Failed to parse per-monitor wallpapers: {0}")]
    PerMonitorParsingError(String),

    /// Notification error
    #[error("Notification error: {0}")]
    NotificationError(#[from] notify::Error),

    /// Timeout waiting for event
    #[error("Timeout waiting for event")]
    Timeout,

    /// Multiple errors occurred
    #[error("Multiple errors occurred")]
    MultipleErrors(Vec<String>),
}

/// Result type alias for dms-awww
pub type Result<T> = std::result::Result<T, DmsAwwwError>;

impl DmsAwwwError {
    /// Check if this error is recoverable (should retry)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            DmsAwwwError::Io(_)
                | DmsAwwwError::CommandFailed(_, _)
                | DmsAwwwError::Timeout
                | DmsAwwwError::Watcher(_)
        )
    }

    /// Check if this error is critical (should exit)
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            DmsAwwwError::Config(_)
                | DmsAwwwError::CommandNotFound(_)
                | DmsAwwwError::SessionFileNotFound(_)
                | DmsAwwwError::NoMonitorsDetected
        )
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            DmsAwwwError::Config(msg) => format!("Configuration error: {msg}"),
            DmsAwwwError::Io(err) => format!("File system error: {err}"),
            DmsAwwwError::Json(err) => format!("Failed to parse JSON: {err}"),
            DmsAwwwError::ConfigFile(err) => format!("Failed to load config file: {err}"),
            DmsAwwwError::CommandNotFound(cmd) => {
                format!("Required command not found: {cmd}\nPlease install it and try again.")
            }
            DmsAwwwError::CommandFailed(cmd, code) => {
                format!("Command '{cmd}' failed with exit code {code}")
            }
            DmsAwwwError::CommandTerminated(cmd, signal) => {
                format!("Command '{cmd}' was terminated by signal: {signal}")
            }
            DmsAwwwError::InvalidWallpaperPath(path) => {
                format!("Invalid wallpaper path: {path}\nFile does not exist or is not accessible.")
            }
            DmsAwwwError::NiriIpc(msg) => format!("Niri IPC error: {msg}"),
            DmsAwwwError::Watcher(msg) => format!("File watcher error: {msg}"),
            DmsAwwwError::PathExpansion { path, .. } => {
                format!("Could not expand path: {path}")
            }
            DmsAwwwError::MissingConfig(key) => {
                format!("Missing required configuration: {key}\nPlease check your config file.")
            }
            DmsAwwwError::InvalidConfig { key, reason } => {
                format!("Invalid configuration for '{key}': {reason}")
            }
            DmsAwwwError::SessionFileNotFound(path) => {
                format!(
                    "DMS session file not found: {}\nMake sure DMS is running.",
                    path.display()
                )
            }
            DmsAwwwError::SettingsFileNotFound(path) => {
                format!("DMS settings file not found: {}", path.display())
            }
            DmsAwwwError::NoMonitorsDetected => {
                "No monitors detected. Make sure your compositor is running.".to_string()
            }
            DmsAwwwError::PerMonitorParsingError(msg) => {
                format!("Failed to parse per-monitor wallpapers: {msg}")
            }
            DmsAwwwError::NotificationError(err) => {
                format!("File system notification error: {err}")
            }
            DmsAwwwError::Timeout => "Operation timed out".to_string(),
            DmsAwwwError::MultipleErrors(errors) => {
                format!("Multiple errors occurred:\n{}", errors.join("\n"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recoverable() {
        let io_err = DmsAwwwError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        ));
        assert!(io_err.is_recoverable());
        assert!(!io_err.is_critical());

        let config_err = DmsAwwwError::Config("test".to_string());
        assert!(!config_err.is_recoverable());
        assert!(config_err.is_critical());
    }

    #[test]
    fn test_user_message() {
        let err = DmsAwwwError::CommandNotFound("awww".to_string());
        let msg = err.user_message();
        assert!(msg.contains("awww"));
        assert!(msg.contains("install"));
    }
}

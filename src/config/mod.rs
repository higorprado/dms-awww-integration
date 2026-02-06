//! Configuration management for dms-awww
//!
//! This module handles loading, validating, and managing configuration
//! from files and environment variables.

use crate::error::{DmsAwwwError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::env;
use shellexpand;

/// Default log level
const DEFAULT_LOG_LEVEL: &str = "info";

/// Default log file location
const DEFAULT_LOG_FILE: &str = "/tmp/dms_awww.log";

/// Default DMS session file path
const DEFAULT_SESSION_FILE: &str = "~/.local/state/DankMaterialShell/session.json";

/// Default DMS settings file path
const DEFAULT_SETTINGS_FILE: &str = "~/.config/DankMaterialShell/settings.json";

/// Default DMS cache directory
const DEFAULT_CACHE_DIR: &str = "~/.cache/DankMaterialShell";

/// Default matugen scheme type
const DEFAULT_MATUGEN_SCHEME: &str = "scheme-tonal-spot";

/// Default quickshell directory
const DEFAULT_SHELL_DIR: &str = "/usr/share/quickshell/dms";

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General settings
    #[serde(default)]
    pub general: GeneralConfig,

    /// DMS-specific settings
    #[serde(default)]
    pub dms: DmsConfig,

    /// Niri-specific settings
    #[serde(default)]
    pub niri: NiriConfig,

    /// Awww-specific settings
    #[serde(default)]
    pub awww: AwwwConfig,

    /// Matugen-specific settings
    #[serde(default)]
    pub matugen: MatugenConfig,
}

/// General configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Log file path
    #[serde(default = "default_log_file")]
    pub log_file: String,

    /// Automatically detect monitors via Niri
    #[serde(default = "default_auto_detect_monitors")]
    pub auto_detect_monitors: bool,

    /// Debounce delay in milliseconds for file changes
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
}

/// DMS configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmsConfig {
    /// Path to DMS session.json file
    #[serde(default = "default_session_file")]
    pub session_file: String,

    /// Path to DMS settings.json file
    #[serde(default = "default_settings_file")]
    pub settings_file: String,

    /// Path to DMS cache directory
    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,
}

/// Niri configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NiriConfig {
    /// Explicitly defined outputs (overrides auto-detection)
    #[serde(default)]
    pub outputs: Vec<String>,
}

/// Awww configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwwwConfig {
    /// Enable awww integration
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Additional arguments to pass to awww
    #[serde(default)]
    pub extra_args: Vec<String>,
}

/// Matugen configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatugenConfig {
    /// Enable matugen integration
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Default scheme type
    #[serde(default = "default_matugen_scheme")]
    pub default_scheme: String,

    /// Quickshell directory
    #[serde(default = "default_shell_dir")]
    pub shell_dir: String,
}

fn default_log_level() -> String {
    DEFAULT_LOG_LEVEL.to_string()
}

fn default_log_file() -> String {
    DEFAULT_LOG_FILE.to_string()
}

fn default_auto_detect_monitors() -> bool {
    true
}

fn default_debounce_ms() -> u64 {
    100
}

fn default_session_file() -> String {
    DEFAULT_SESSION_FILE.to_string()
}

fn default_settings_file() -> String {
    DEFAULT_SETTINGS_FILE.to_string()
}

fn default_cache_dir() -> String {
    DEFAULT_CACHE_DIR.to_string()
}

fn default_enabled() -> bool {
    true
}

fn default_matugen_scheme() -> String {
    DEFAULT_MATUGEN_SCHEME.to_string()
}

fn default_shell_dir() -> String {
    DEFAULT_SHELL_DIR.to_string()
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            log_file: default_log_file(),
            auto_detect_monitors: default_auto_detect_monitors(),
            debounce_ms: default_debounce_ms(),
        }
    }
}

impl Default for DmsConfig {
    fn default() -> Self {
        Self {
            session_file: default_session_file(),
            settings_file: default_settings_file(),
            cache_dir: default_cache_dir(),
        }
    }
}

impl Default for NiriConfig {
    fn default() -> Self {
        Self {
            outputs: Vec::new(),
        }
    }
}

impl Default for AwwwConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            extra_args: Vec::new(),
        }
    }
}

impl Default for MatugenConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            default_scheme: default_matugen_scheme(),
            shell_dir: default_shell_dir(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            dms: DmsConfig::default(),
            niri: NiriConfig::default(),
            awww: AwwwConfig::default(),
            matugen: MatugenConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from file with environment variable overrides
    pub fn load() -> Result<Self> {
        let mut settings = config::Config::builder();

        // Check for config files in order of priority
        let config_paths = vec![
            Self::xdg_config_home("dms-awww/config.toml"),
            Self::xdg_config_home("dms-awww/config.yaml"),
            Self::xdg_config_home("dms-awww/config.yml"),
            PathBuf::from("/etc/dms-awww/config.toml"),
        ];

        let mut found_config = false;
        for path in config_paths {
            if path.exists() {
                tracing::debug!("Loading configuration from: {}", path.display());
                let source = match path.extension().and_then(|e| e.to_str()) {
                    Some("toml") => {
                        config::File::from(path.as_path())
                            .format(config::FileFormat::Toml)
                    }
                    Some("yaml" | "yml") => {
                        config::File::from(path.as_path())
                            .format(config::FileFormat::Yaml)
                    }
                    _ => continue,
                };

                settings = settings.add_source(source);
                found_config = true;
                break;
            }
        }

        if !found_config {
            tracing::debug!("No configuration file found, using defaults");
        }

        // Build base configuration
        let mut config: Config = settings
            .build()?
            .try_deserialize()
            .unwrap_or_else(|_| Config::default());

        // Apply environment variable overrides
        config.apply_env_overrides();

        // Expand paths
        config.expand_paths();

        Ok(config)
    }

    /// Get XDG config home path
    fn xdg_config_home(path: &str) -> PathBuf {
        let xdg = env::var("XDG_CONFIG_HOME")
            .unwrap_or_else(|_| format!("{}/.config", env::var("HOME").unwrap()));
        PathBuf::from(xdg).join(path)
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) {
        // Helper to get env var with prefix
        let get_env = |suffix: &str| -> Option<String> {
            env::var(format!("DMS_AWWW_{}", suffix)).ok()
        };

        // General overrides
        if let Some(level) = get_env("LOG_LEVEL") {
            self.general.log_level = level;
        }
        if let Some(file) = get_env("LOG_FILE") {
            self.general.log_file = file;
        }
        if let Some(val) = get_env("AUTO_DETECT_MONITORS") {
            self.general.auto_detect_monitors = val.parse().unwrap_or(self.general.auto_detect_monitors);
        }

        // DMS overrides
        if let Some(path) = get_env("SESSION_FILE") {
            self.dms.session_file = path;
        }
        if let Some(path) = get_env("SETTINGS_FILE") {
            self.dms.settings_file = path;
        }
        if let Some(path) = get_env("CACHE_DIR") {
            self.dms.cache_dir = path;
        }

        // Niri overrides
        if let Some(outputs) = get_env("NIRI_OUTPUTS") {
            self.niri.outputs = outputs
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // Awww overrides
        if let Some(val) = get_env("AWWW_ENABLED") {
            self.awww.enabled = val.parse().unwrap_or(self.awww.enabled);
        }

        // Matugen overrides
        if let Some(val) = get_env("MATUGEN_ENABLED") {
            self.matugen.enabled = val.parse().unwrap_or(self.matugen.enabled);
        }
        if let Some(scheme) = get_env("MATUGEN_SCHEME") {
            self.matugen.default_scheme = scheme;
        }
        if let Some(dir) = get_env("SHELL_DIR") {
            self.matugen.shell_dir = dir;
        }
    }

    /// Expand ~ and environment variables in paths
    fn expand_paths(&mut self) {
        self.dms.session_file = Self::expand_path(&self.dms.session_file);
        self.dms.settings_file = Self::expand_path(&self.dms.settings_file);
        self.dms.cache_dir = Self::expand_path(&self.dms.cache_dir);
        self.general.log_file = Self::expand_path(&self.general.log_file);
        self.matugen.shell_dir = Self::expand_path(&self.matugen.shell_dir);
    }

    /// Expand a single path (handles ~ and $VAR)
    pub fn expand_path(path: &str) -> String {
        shellexpand::full(path)
            .map(|s| s.into_owned())
            .unwrap_or_else(|_| path.to_string())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Check if session file parent directory exists
        let session_path = PathBuf::from(&self.dms.session_file);
        if let Some(parent) = session_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                tracing::warn!("Session file parent directory does not exist: {}", parent.display());
            }
        }

        // Check if settings file parent directory exists
        let settings_path = PathBuf::from(&self.dms.settings_file);
        if let Some(parent) = settings_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                tracing::warn!("Settings file parent directory does not exist: {}", parent.display());
            }
        }

        // Check if cache dir exists or can be created
        let cache_path = PathBuf::from(&self.dms.cache_dir);
        if let Some(parent) = cache_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                tracing::warn!("Cache parent directory does not exist: {}", parent.display());
            }
        }

        // Validate log file directory
        let log_path = PathBuf::from(&self.general.log_file);
        if let Some(parent) = log_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                tracing::warn!("Log file parent directory does not exist: {}", parent.display());
            }
        }

        // Validate log level
        match self.general.log_level.to_lowercase().as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => {
                return Err(DmsAwwwError::InvalidConfig {
                    key: "log_level".to_string(),
                    reason: format!("invalid log level: {}", self.general.log_level),
                });
            }
        }

        Ok(())
    }

    /// Get the expanded session file path
    pub fn session_file_path(&self) -> PathBuf {
        PathBuf::from(&self.dms.session_file)
    }

    /// Get the expanded settings file path
    pub fn settings_file_path(&self) -> PathBuf {
        PathBuf::from(&self.dms.settings_file)
    }

    /// Get the expanded cache directory path
    pub fn cache_dir_path(&self) -> PathBuf {
        PathBuf::from(&self.dms.cache_dir)
    }

    /// Get the expanded log file path
    pub fn log_file_path(&self) -> PathBuf {
        PathBuf::from(&self.general.log_file)
    }

    /// Check if awww is enabled
    pub fn awww_enabled(&self) -> bool {
        self.awww.enabled
    }

    /// Check if matugen is enabled
    pub fn matugen_enabled(&self) -> bool {
        self.matugen.enabled
    }

    /// Get effective monitor outputs (explicit or auto-detected)
    pub fn get_monitor_outputs(&self) -> Vec<String> {
        if !self.niri.outputs.is_empty() {
            return self.niri.outputs.clone();
        }
        Vec::new() // Will trigger auto-detection
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.log_level, "info");
        assert_eq!(config.general.auto_detect_monitors, true);
        assert!(config.awww.enabled);
        assert!(config.matugen.enabled);
    }

    #[test]
    fn test_path_expansion() {
        let home = env::var("HOME").unwrap();
        let expanded = Config::expand_path("~/.config/test");
        assert!(expanded.starts_with(&home));
        assert!(expanded.contains("/.config/test"));
    }

    #[test]
    fn test_path_expansion_env_var() {
        env::set_var("DMS_TEST_VAR", "/tmp/test");
        let expanded = Config::expand_path("$DMS_TEST_VAR/file.txt");
        assert_eq!(expanded, "/tmp/test/file.txt");
        env::remove_var("DMS_TEST_VAR");
    }

    #[test]
    fn test_validate_log_level() {
        let config = Config::default();
        config.validate().unwrap();

        let mut invalid_config = Config::default();
        invalid_config.general.log_level = "invalid".to_string();
        assert!(invalid_config.validate().is_err());
    }
}

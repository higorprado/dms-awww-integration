//! DMS JSON parsing and session management
//!
//! This module handles parsing of DMS session.json and settings.json files,
//! extracting wallpaper information and theme settings.

use crate::error::{DmsAwwwError, Result};
use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Parsed DMS session.json structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionJson {
    /// Current wallpaper path (single wallpaper mode)
    #[serde(rename = "wallpaperPath")]
    pub wallpaper_path: Option<String>,

    /// Whether per-monitor wallpapers are enabled
    #[serde(rename = "perMonitorWallpaper")]
    pub per_monitor_wallpaper: Option<bool>,

    /// Per-monitor wallpaper mappings
    #[serde(rename = "monitorWallpapers", default)]
    pub monitor_wallpapers: HashMap<String, String>,

    /// Whether light mode is enabled
    #[serde(rename = "isLightMode")]
    pub is_light_mode: Option<bool>,
}

/// Parsed DMS settings.json structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsJson {
    /// Matugen scheme type
    #[serde(rename = "matugenScheme")]
    pub matugen_scheme: Option<String>,

    /// Other settings are stored as raw values
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// Represents a wallpaper change event
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WallpaperChange {
    /// The wallpaper path(s) to apply
    pub wallpapers: Vec<Wallpaper>,

    /// Whether light mode is enabled
    pub is_light_mode: bool,
}

/// Represents a single wallpaper assignment
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wallpaper {
    /// Path to the wallpaper image
    pub path: String,

    /// Target monitor (None for all monitors in single mode)
    pub monitor: Option<String>,
}

impl Wallpaper {
    /// Create a new wallpaper for all monitors
    pub fn new(path: String) -> Self {
        Self {
            path,
            monitor: None,
        }
    }

    /// Create a new wallpaper for a specific monitor
    pub fn for_monitor(path: String, monitor: String) -> Self {
        Self {
            path,
            monitor: Some(monitor),
        }
    }

    /// Check if the wallpaper path is valid (not a color)
    pub fn is_valid_image(&self) -> bool {
        !self.path.starts_with('#') && !self.path.starts_with("/#")
    }

    /// Check if the wallpaper file exists
    pub fn exists(&self) -> bool {
        Path::new(&self.path).exists()
    }
}

/// DMS session manager
pub struct DmsSession {
    config: Config,
    last_wallpapers: Vec<String>,
}

impl DmsSession {
    /// Create a new DMS session manager
    pub fn new(config: Config) -> Self {
        Self {
            config,
            last_wallpapers: Vec::new(),
        }
    }

    /// Read and parse the session.json file
    pub fn read_session(&self) -> Result<SessionJson> {
        let session_path = self.config.session_file_path();

        if !session_path.exists() {
            return Err(DmsAwwwError::SessionFileNotFound(session_path));
        }

        let content = fs::read_to_string(&session_path)
            .map_err(|e| DmsAwwwError::Io(e))?;

        let session: SessionJson = serde_json::from_str(&content)?;

        Ok(session)
    }

    /// Read and parse the settings.json file
    pub fn read_settings(&self) -> Result<SettingsJson> {
        let settings_path = self.config.settings_file_path();

        if !settings_path.exists() {
            return Err(DmsAwwwError::SettingsFileNotFound(settings_path));
        }

        let content = fs::read_to_string(&settings_path)
            .map_err(|e| DmsAwwwError::Io(e))?;

        let settings: SettingsJson = serde_json::from_str(&content)?;

        Ok(settings)
    }

    /// Get the current wallpaper state
    pub fn get_current_state(&self) -> Result<WallpaperChange> {
        let session = self.read_session()?;
        let settings = self.read_settings().ok();

        let is_light_mode = session.is_light_mode.unwrap_or(false);
        let _matugen_scheme = settings
            .and_then(|s| s.matugen_scheme)
            .unwrap_or_else(|| self.config.matugen.default_scheme.clone());

        let mut wallpapers = Vec::new();

        if session.per_monitor_wallpaper.unwrap_or(false) {
            // Per-monitor mode: extract each monitor's wallpaper
            for (monitor, path) in &session.monitor_wallpapers {
                if !path.is_empty() {
                    wallpapers.push(Wallpaper::for_monitor(path.clone(), monitor.clone()));
                }
            }

            // If no per-monitor wallpapers found, fall back to main wallpaper
            if wallpapers.is_empty() {
                if let Some(path) = session.wallpaper_path {
                    wallpapers.push(Wallpaper::new(path));
                }
            }
        } else {
            // Single wallpaper mode
            if let Some(path) = session.wallpaper_path {
                wallpapers.push(Wallpaper::new(path));
            }
        }

        // Filter out invalid wallpaper paths (colors, empty strings)
        let wallpapers: Vec<Wallpaper> = wallpapers
            .into_iter()
            .filter(|w| w.is_valid_image() && !w.path.is_empty())
            .collect();

        if wallpapers.is_empty() {
            return Err(DmsAwwwError::InvalidWallpaperPath(
                "No valid wallpapers found in session".to_string(),
            ));
        }

        Ok(WallpaperChange {
            wallpapers,
            is_light_mode,
        })
    }

    /// Check if wallpaper has changed since last check
    pub fn has_changed(&mut self) -> Result<bool> {
        let state = self.get_current_state()?;
        let current_paths: Vec<String> = state
            .wallpapers
            .iter()
            .map(|w| w.path.clone())
            .collect();

        let has_changed = current_paths != self.last_wallpapers;

        if has_changed {
            self.last_wallpapers = current_paths;
        }

        Ok(has_changed)
    }

    /// Get the matugen scheme from settings
    pub fn get_matugen_scheme(&self) -> String {
        match self.read_settings() {
            Ok(settings) => {
                settings.matugen_scheme.unwrap_or_else(|| {
                    self.config.matugen.default_scheme.clone()
                })
            }
            Err(_) => self.config.matugen.default_scheme.clone(),
        }
    }

    /// Get the current light/dark mode
    pub fn get_theme_mode(&self) -> Result<String> {
        let session = self.read_session()?;
        Ok(if session.is_light_mode.unwrap_or(false) {
            "light".to_string()
        } else {
            "dark".to_string()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    fn create_test_session(dir: &Path, content: &str) -> PathBuf {
        let session_path = dir.join("session.json");
        let mut file = fs::File::create(&session_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        session_path
    }

    #[test]
    fn test_parse_session_single_wallpaper() {
        let json = r#"{
            "wallpaperPath": "/path/to/wallpaper.jpg",
            "perMonitorWallpaper": false,
            "isLightMode": false
        }"#;

        let session: SessionJson = serde_json::from_str(json).unwrap();
        assert_eq!(session.wallpaper_path, Some("/path/to/wallpaper.jpg".to_string()));
        assert_eq!(session.per_monitor_wallpaper, Some(false));
        assert_eq!(session.is_light_mode, Some(false));
    }

    #[test]
    fn test_parse_session_per_monitor() {
        let json = r#"{
            "wallpaperPath": "/path/to/wallpaper.jpg",
            "perMonitorWallpaper": true,
            "monitorWallpapers": {
                "HDMI-A-1": "/path/to/wp1.jpg",
                "DP-1": "/path/to/wp2.jpg"
            },
            "isLightMode": true
        }"#;

        let session: SessionJson = serde_json::from_str(json).unwrap();
        assert!(session.per_monitor_wallpaper.unwrap());
        assert_eq!(session.monitor_wallpapers.len(), 2);
        assert_eq!(
            session.monitor_wallpapers.get("HDMI-A-1"),
            Some(&"/path/to/wp1.jpg".to_string())
        );
    }

    #[test]
    fn test_parse_settings() {
        let json = r#"{
            "matugenScheme": "scheme-tonal-spot",
            "otherSetting": "value"
        }"#;

        let settings: SettingsJson = serde_json::from_str(json).unwrap();
        assert_eq!(settings.matugen_scheme, Some("scheme-tonal-spot".to_string()));
    }

    #[test]
    fn test_wallpaper_is_valid_image() {
        let wp = Wallpaper::new("/path/to/image.jpg".to_string());
        assert!(wp.is_valid_image());

        let color_wp = Wallpaper::new("#ff0000".to_string());
        assert!(!color_wp.is_valid_image());

        let color_wp2 = Wallpaper::new("/#ff0000".to_string());
        assert!(!color_wp2.is_valid_image());
    }
}

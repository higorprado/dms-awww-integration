//! Common test utilities for dms-awww integration tests

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test fixture builder for creating test DMS session files
pub struct SessionFixture {
    wallpaper_path: Option<String>,
    per_monitor_wallpaper: Option<bool>,
    monitor_wallpapers: Vec<(String, String)>,
    is_light_mode: Option<bool>,
}

impl Default for SessionFixture {
    fn default() -> Self {
        Self {
            wallpaper_path: Some("/tmp/wallpaper.jpg".to_string()),
            per_monitor_wallpaper: Some(false),
            monitor_wallpapers: Vec::new(),
            is_light_mode: Some(false),
        }
    }
}

impl SessionFixture {
    /// Create a new session fixture builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the wallpaper path
    pub fn wallpaper_path(mut self, path: impl Into<String>) -> Self {
        self.wallpaper_path = Some(path.into());
        self
    }

    /// Enable per-monitor wallpaper mode
    pub fn per_monitor(mut self, enabled: bool) -> Self {
        self.per_monitor_wallpaper = Some(enabled);
        self
    }

    /// Add a monitor wallpaper
    pub fn monitor_wallpaper(mut self, monitor: impl Into<String>, path: impl Into<String>) -> Self {
        self.monitor_wallpapers.push((monitor.into(), path.into()));
        self
    }

    /// Set light mode
    pub fn light_mode(mut self, enabled: bool) -> Self {
        self.is_light_mode = Some(enabled);
        self
    }

    /// Build the session JSON string
    pub fn build_json(&self) -> String {
        let mut json = String::from("{");

        if let Some(wp) = &self.wallpaper_path {
            json.push_str(&format!("\"wallpaperPath\":\"{}\"", wp));
        }

        if let Some(pmw) = self.per_monitor_wallpaper {
            if !json.is_empty() && json != "{" {
                json.push_str(",");
            }
            json.push_str(&format!("\"perMonitorWallpaper\":{}", if pmw { "true" } else { "false" }));
        }

        if !self.monitor_wallpapers.is_empty() {
            if !json.is_empty() && json != "{" {
                json.push_str(",");
            }
            json.push_str("\"monitorWallpapers\":{");
            let wallpapers: Vec<String> = self.monitor_wallpapers
                .iter()
                .map(|(m, p)| format!("\"{}\":\"{}\"", m, p))
                .collect();
            json.push_str(&wallpapers.join(","));
            json.push_str("}");
        }

        if let Some(lm) = self.is_light_mode {
            if !json.is_empty() && json != "{" {
                json.push_str(",");
            }
            json.push_str(&format!("\"isLightMode\":{}", if lm { "true" } else { "false" }));
        }

        json.push_str("}");
        json
    }

    /// Write the session to a file in the given directory
    pub fn write_to(&self, dir: &Path) -> PathBuf {
        let session_path = dir.join("session.json");
        let mut file = File::create(&session_path).unwrap();
        file.write_all(self.build_json().as_bytes()).unwrap();
        session_path
    }

    /// Create a temporary directory with this session file
    #[allow(dead_code)]
    pub fn create_temp_dir(&self) -> TempDir {
        let dir = TempDir::new().unwrap();
        self.write_to(dir.path());
        dir
    }
}

/// Test fixture builder for creating test DMS settings files
pub struct SettingsFixture {
    matugen_scheme: Option<String>,
    other_fields: Vec<(String, String)>,
}

impl Default for SettingsFixture {
    fn default() -> Self {
        Self {
            matugen_scheme: Some("scheme-tonal-spot".to_string()),
            other_fields: Vec::new(),
        }
    }
}

impl SettingsFixture {
    /// Create a new settings fixture builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the matugen scheme
    pub fn matugen_scheme(mut self, scheme: impl Into<String>) -> Self {
        self.matugen_scheme = Some(scheme.into());
        self
    }

    /// Add an extra field
    #[allow(dead_code)]
    pub fn field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.other_fields.push((key.into(), value.into()));
        self
    }

    /// Build the settings JSON string
    pub fn build_json(&self) -> String {
        let mut json = String::from("{");

        if let Some(ms) = &self.matugen_scheme {
            json.push_str(&format!("\"matugenScheme\":\"{}\"", ms));
        }

        for (key, value) in &self.other_fields {
            if !json.is_empty() && json != "{" {
                json.push_str(",");
            }
            json.push_str(&format!("\"{}\":\"{}\"", key, value));
        }

        json.push_str("}");
        json
    }

    /// Write the settings to a file in the given directory
    pub fn write_to(&self, dir: &Path) -> PathBuf {
        let settings_path = dir.join("settings.json");
        let mut file = File::create(&settings_path).unwrap();
        file.write_all(self.build_json().as_bytes()).unwrap();
        settings_path
    }
}

/// Test fixture for creating config files
pub struct ConfigFixture {
    format: ConfigFormat,
    content: String,
}

pub enum ConfigFormat {
    Toml,
    Yaml,
}

impl ConfigFixture {
    /// Create a new TOML config fixture
    pub fn toml() -> Self {
        Self {
            format: ConfigFormat::Toml,
            content: String::new(),
        }
    }

    /// Create a new YAML config fixture
    pub fn yaml() -> Self {
        Self {
            format: ConfigFormat::Yaml,
            content: String::new(),
        }
    }

    /// Set log level
    pub fn log_level(mut self, level: &str) -> Self {
        match self.format {
            ConfigFormat::Toml => {
                self.content.push_str(&format!("[general]\nlog_level = \"{}\"\n", level));
            }
            ConfigFormat::Yaml => {
                self.content.push_str(&format!("general:\n  log_level: {}\n", level));
            }
        }
        self
    }

    /// Set awww enabled
    pub fn awww_enabled(mut self, enabled: bool) -> Self {
        match self.format {
            ConfigFormat::Toml => {
                self.content.push_str(&format!("[awww]\nenabled = {}\n", if enabled { "true" } else { "false" }));
            }
            ConfigFormat::Yaml => {
                self.content.push_str(&format!("awww:\n  enabled: {}\n", if enabled { "true" } else { "false" }));
            }
        }
        self
    }

    /// Set matugen enabled
    pub fn matugen_enabled(mut self, enabled: bool) -> Self {
        match self.format {
            ConfigFormat::Toml => {
                self.content.push_str(&format!("[matugen]\nenabled = {}\n", if enabled { "true" } else { "false" }));
            }
            ConfigFormat::Yaml => {
                self.content.push_str(&format!("matugen:\n  enabled: {}\n", if enabled { "true" } else { "false" }));
            }
        }
        self
    }

    /// Add custom content
    pub fn custom(mut self, content: &str) -> Self {
        self.content.push_str(content);
        self
    }

    /// Write the config to a file
    pub fn write_to(&self, path: &Path) {
        let mut file = File::create(path).unwrap();
        file.write_all(self.content.as_bytes()).unwrap();
    }

    /// Create a config directory with this file
    pub fn create_temp_config(&self, filename: &str) -> TempDir {
        let dir = TempDir::new().unwrap();
        let config_dir = dir.path().join("dms-awww");
        fs::create_dir_all(&config_dir).unwrap();
        self.write_to(&config_dir.join(filename));
        dir
    }
}

/// Create a temporary test image file (minimal PNG)
pub fn create_test_image(path: &Path) {
    // Minimal 1x1 PNG file
    let png_data = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, // IHDR chunk length
        0x49, 0x48, 0x44, 0x52, // IHDR
        0x00, 0x00, 0x00, 0x01, // width: 1
        0x00, 0x00, 0x00, 0x01, // height: 1
        0x08, 0x02, 0x00, 0x00, 0x00, // bit depth: 8, color type: 2 (RGB)
        0x90, 0x77, 0x53, 0xDE, // CRC
        0x00, 0x00, 0x00, 0x0C, // IDAT chunk length
        0x49, 0x44, 0x41, 0x54, // IDAT
        0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, // compressed data
        0x18, 0xDD, 0x8D, 0xB4, // CRC
        0x00, 0x00, 0x00, 0x00, // IEND chunk length
        0x49, 0x45, 0x4E, 0x44, // IEND
        0xAE, 0x42, 0x60, 0x82, // CRC
    ];
    fs::write(path, png_data).unwrap();
}

/// Create a mock niri msg outputs response
pub fn mock_niri_outputs() -> String {
    r#"[{
        "name": "HDMI-A-1",
        "enabled": true,
        "make": "Dell",
        "model": "U2720Q",
        "resolution": {"width": 3840, "height": 2160},
        "position": {"x": 0, "y": 0},
        "refresh_rate": 60.0,
        "physicalSize": {"width": 597, "height": 336}
    }, {
        "name": "DP-1",
        "enabled": true,
        "make": "LG",
        "model": "27GN950",
        "resolution": {"width": 3840, "height": 2160},
        "position": {"x": 3840, "y": 0},
        "refresh_rate": 144.0,
        "physicalSize": {"width": 597, "height": 336}
    }]"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_fixture_basic() {
        let json = SessionFixture::new().build_json();
        assert!(json.contains("wallpaperPath"));
        assert!(json.contains("/tmp/wallpaper.jpg"));
    }

    #[test]
    fn test_session_fixture_per_monitor() {
        let json = SessionFixture::new()
            .per_monitor(true)
            .monitor_wallpaper("HDMI-A-1", "/tmp/wp1.jpg")
            .monitor_wallpaper("DP-1", "/tmp/wp2.jpg")
            .build_json();

        assert!(json.contains("monitorWallpapers"));
        assert!(json.contains("HDMI-A-1"));
        assert!(json.contains("DP-1"));
    }

    #[test]
    fn test_config_fixture_toml() {
        let fixture = ConfigFixture::toml()
            .log_level("debug")
            .awww_enabled(false);

        assert!(fixture.content.contains("[general]"));
        assert!(fixture.content.contains("log_level"));
        assert!(fixture.content.contains("[awww]"));
    }

    #[test]
    fn test_config_fixture_yaml() {
        let fixture = ConfigFixture::yaml()
            .log_level("debug")
            .matugen_enabled(true);

        assert!(fixture.content.contains("general:"));
        assert!(fixture.content.contains("log_level"));
        assert!(fixture.content.contains("matugen:"));
    }

    #[test]
    fn test_create_test_image() {
        let dir = TempDir::new().unwrap();
        let img_path = dir.path().join("test.png");
        create_test_image(&img_path);
        assert!(img_path.exists());
        assert!(img_path.metadata().unwrap().len() > 0);
    }
}

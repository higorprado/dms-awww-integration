//! DMS integration tests

mod common;

use std::fs;
use std::path::Path;

use common::SessionFixture;
use common::SettingsFixture;
use dms_awww::config::Config;
use dms_awww::dms::{DmsSession, SessionJson, SettingsJson, Wallpaper, WallpaperChange};

/// Create a test config with paths pointing to a temp directory
fn test_config_with_dir(dir: &Path) -> Config {
    let mut config = Config::default();
    config.dms.session_file = dir.join("session.json").to_str().unwrap().to_string();
    config.dms.settings_file = dir.join("settings.json").to_str().unwrap().to_string();
    config.dms.cache_dir = dir.join("cache").to_str().unwrap().to_string();
    config
}

#[test]
fn test_session_json_parse_single_wallpaper() {
    let json = r#"{
        "wallpaperPath": "/path/to/wallpaper.jpg",
        "perMonitorWallpaper": false,
        "isLightMode": false
    }"#;

    let session: SessionJson = serde_json::from_str(json).unwrap();

    assert_eq!(session.wallpaper_path, Some("/path/to/wallpaper.jpg".to_string()));
    assert_eq!(session.per_monitor_wallpaper, Some(false));
    assert_eq!(session.is_light_mode, Some(false));
    assert!(session.monitor_wallpapers.is_empty());
}

#[test]
fn test_session_json_parse_per_monitor_wallpapers() {
    let json = r#"{
        "wallpaperPath": "/path/to/wallpaper.jpg",
        "perMonitorWallpaper": true,
        "monitorWallpapers": {
            "HDMI-A-1": "/path/to/wp1.jpg",
            "DP-1": "/path/to/wp2.jpg",
            "eDP-1": "/path/to/wp3.jpg"
        },
        "isLightMode": true
    }"#;

    let session: SessionJson = serde_json::from_str(json).unwrap();

    assert!(session.per_monitor_wallpaper.unwrap());
    assert_eq!(session.monitor_wallpapers.len(), 3);
    assert_eq!(session.monitor_wallpapers.get("HDMI-A-1"), Some(&"/path/to/wp1.jpg".to_string()));
    assert_eq!(session.monitor_wallpapers.get("DP-1"), Some(&"/path/to/wp2.jpg".to_string()));
    assert_eq!(session.monitor_wallpapers.get("eDP-1"), Some(&"/path/to/wp3.jpg".to_string()));
    assert_eq!(session.is_light_mode, Some(true));
}

#[test]
fn test_session_json_parse_with_is_light_mode() {
    let json_light = r#"{
        "wallpaperPath": "/path/to/wallpaper.jpg",
        "isLightMode": true
    }"#;

    let session_light: SessionJson = serde_json::from_str(json_light).unwrap();
    assert_eq!(session_light.is_light_mode, Some(true));

    let json_dark = r#"{
        "wallpaperPath": "/path/to/wallpaper.jpg",
        "isLightMode": false
    }"#;

    let session_dark: SessionJson = serde_json::from_str(json_dark).unwrap();
    assert_eq!(session_dark.is_light_mode, Some(false));
}

#[test]
fn test_settings_json_parse_matugen_scheme() {
    let json = r#"{
        "matugenScheme": "scheme-expressive"
    }"#;

    let settings: SettingsJson = serde_json::from_str(json).unwrap();

    assert_eq!(settings.matugen_scheme, Some("scheme-expressive".to_string()));
}

#[test]
fn test_settings_json_parse_with_other_fields() {
    let json = r#"{
        "matugenScheme": "scheme-tonal-spot",
        "otherField": "someValue",
        "anotherField": 42
    }"#;

    let settings: SettingsJson = serde_json::from_str(json).unwrap();

    assert_eq!(settings.matugen_scheme, Some("scheme-tonal-spot".to_string()));
    assert_eq!(settings.other.len(), 2);
    assert!(settings.other.contains_key("otherField"));
    assert!(settings.other.contains_key("anotherField"));
}

#[test]
fn test_wallpaper_validation_filters_colors() {
    let wp1 = Wallpaper::new("#ff0000".to_string());
    assert!(!wp1.is_valid_image(), "Color starting with # should be invalid");

    let wp2 = Wallpaper::new("/#ff0000".to_string());
    assert!(!wp2.is_valid_image(), "Color starting with /# should be invalid");

    let wp3 = Wallpaper::new("/path/to/image.jpg".to_string());
    assert!(wp3.is_valid_image(), "Normal path should be valid");
}

#[test]
fn test_wallpaper_exists() {
    let temp_dir = tempfile::TempDir::new().unwrap();

    // Create an existing file
    let existing_file = temp_dir.path().join("exists.jpg");
    fs::write(&existing_file, b"fake image").unwrap();

    let wp_exists = Wallpaper::new(existing_file.to_str().unwrap().to_string());
    assert!(wp_exists.exists());

    let wp_not_exists = Wallpaper::new("/nonexistent/path.jpg".to_string());
    assert!(!wp_not_exists.exists());
}

#[test]
fn test_wallpaper_for_monitor() {
    let wp = Wallpaper::for_monitor("/path/to/wp.jpg".to_string(), "HDMI-A-1".to_string());

    assert_eq!(wp.path, "/path/to/wp.jpg");
    assert_eq!(wp.monitor, Some("HDMI-A-1".to_string()));
}

#[test]
fn test_dms_session_read_session() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    SessionFixture::new()
        .wallpaper_path("/tmp/test.jpg")
        .write_to(temp_dir.path());

    let session = DmsSession::new(config);
    let result = session.read_session();

    assert!(result.is_ok());
    let session_json = result.unwrap();
    assert_eq!(session_json.wallpaper_path, Some("/tmp/test.jpg".to_string()));
}

#[test]
fn test_dms_session_read_session_not_found() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    // Don't create session.json
    let session = DmsSession::new(config);
    let result = session.read_session();

    assert!(result.is_err());
}

#[test]
fn test_dms_session_read_settings() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    SettingsFixture::new()
        .matugen_scheme("scheme-expressive")
        .write_to(temp_dir.path());

    let session = DmsSession::new(config);
    let result = session.read_settings();

    assert!(result.is_ok());
    let settings_json = result.unwrap();
    assert_eq!(settings_json.matugen_scheme, Some("scheme-expressive".to_string()));
}

#[test]
fn test_dms_session_get_current_state_single_wallpaper() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    // Create both session and settings files
    SessionFixture::new()
        .wallpaper_path("/tmp/test.jpg")
        .light_mode(false)
        .write_to(temp_dir.path());

    SettingsFixture::new()
        .matugen_scheme("scheme-tonal-spot")
        .write_to(temp_dir.path());

    let session = DmsSession::new(config);
    let state = session.get_current_state();

    assert!(state.is_ok());
    let state = state.unwrap();
    assert_eq!(state.wallpapers.len(), 1);
    assert_eq!(state.wallpapers[0].path, "/tmp/test.jpg");
    assert_eq!(state.is_light_mode, false);
}

#[test]
fn test_dms_session_get_current_state_per_monitor() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    SessionFixture::new()
        .per_monitor(true)
        .monitor_wallpaper("HDMI-A-1", "/tmp/wp1.jpg")
        .monitor_wallpaper("DP-1", "/tmp/wp2.jpg")
        .light_mode(true)
        .write_to(temp_dir.path());

    SettingsFixture::new().write_to(temp_dir.path());

    let session = DmsSession::new(config);
    let state = session.get_current_state();

    assert!(state.is_ok());
    let state = state.unwrap();
    assert_eq!(state.wallpapers.len(), 2);
    assert_eq!(state.is_light_mode, true);

    // Check that we have both monitors with correct wallpapers
    let hdmi_wp = state.wallpapers.iter()
        .find(|w| w.monitor.as_deref() == Some("HDMI-A-1"))
        .expect("HDMI-A-1 wallpaper not found");
    assert_eq!(hdmi_wp.path, "/tmp/wp1.jpg");

    let dp_wp = state.wallpapers.iter()
        .find(|w| w.monitor.as_deref() == Some("DP-1"))
        .expect("DP-1 wallpaper not found");
    assert_eq!(dp_wp.path, "/tmp/wp2.jpg");
}

#[test]
fn test_dms_session_get_current_state_filters_colors() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    SessionFixture::new()
        .wallpaper_path("#ff0000")
        .write_to(temp_dir.path());

    SettingsFixture::new().write_to(temp_dir.path());

    let session = DmsSession::new(config);
    let state = session.get_current_state();

    assert!(state.is_err());
}

#[test]
fn test_dms_session_get_current_state_empty_wallpapers() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    // Create session with empty wallpaper path
    let json = r#"{
        "wallpaperPath": "",
        "perMonitorWallpaper": false
    }"#;
    fs::write(temp_dir.path().join("session.json"), json).unwrap();
    SettingsFixture::new().write_to(temp_dir.path());

    let session = DmsSession::new(config);
    let state = session.get_current_state();

    assert!(state.is_err());
}

#[test]
fn test_dms_session_has_changed() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    SessionFixture::new()
        .wallpaper_path("/tmp/test.jpg")
        .write_to(temp_dir.path());

    SettingsFixture::new().write_to(temp_dir.path());

    let mut session = DmsSession::new(config);

    // First check should return true (initial state)
    let has_changed = session.has_changed();
    assert!(has_changed.is_ok());
    assert!(has_changed.unwrap());

    // Second check with same file should return false
    let has_changed = session.has_changed();
    assert!(has_changed.is_ok());
    assert!(!has_changed.unwrap());

    // Modify the file
    SessionFixture::new()
        .wallpaper_path("/tmp/test2.jpg")
        .write_to(temp_dir.path());

    let has_changed = session.has_changed();
    assert!(has_changed.is_ok());
    assert!(has_changed.unwrap());
}

#[test]
fn test_dms_session_get_matugen_scheme() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    SettingsFixture::new()
        .matugen_scheme("scheme-vibrant")
        .write_to(temp_dir.path());

    let session = DmsSession::new(config);
    let scheme = session.get_matugen_scheme();

    assert_eq!(scheme, "scheme-vibrant");
}

#[test]
fn test_dms_session_get_matugen_scheme_fallback_to_default() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    // Don't create settings.json - should use default from config
    let session = DmsSession::new(config);
    let scheme = session.get_matugen_scheme();

    assert_eq!(scheme, "scheme-tonal-spot"); // Default value
}

#[test]
fn test_dms_session_get_theme_mode() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    // Test light mode
    SessionFixture::new()
        .light_mode(true)
        .write_to(temp_dir.path());

    let session = DmsSession::new(config.clone());
    assert_eq!(session.get_theme_mode().unwrap(), "light");

    // Test dark mode
    SessionFixture::new()
        .light_mode(false)
        .write_to(temp_dir.path());

    let session = DmsSession::new(config);
    assert_eq!(session.get_theme_mode().unwrap(), "dark");
}

#[test]
fn test_per_monitor_fallback_to_single() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    // Per-monitor enabled but no monitor wallpapers
    let json = r#"{
        "wallpaperPath": "/tmp/fallback.jpg",
        "perMonitorWallpaper": true,
        "monitorWallpapers": {},
        "isLightMode": false
    }"#;
    fs::write(temp_dir.path().join("session.json"), json).unwrap();
    SettingsFixture::new().write_to(temp_dir.path());

    let session = DmsSession::new(config);
    let state = session.get_current_state();

    assert!(state.is_ok());
    let state = state.unwrap();
    assert_eq!(state.wallpapers.len(), 1);
    assert_eq!(state.wallpapers[0].path, "/tmp/fallback.jpg");
}

#[test]
fn test_session_json_missing_optional_fields() {
    let json = r#"{}"#;

    let session: SessionJson = serde_json::from_str(json).unwrap();

    assert_eq!(session.wallpaper_path, None);
    assert_eq!(session.per_monitor_wallpaper, None);
    assert_eq!(session.is_light_mode, None);
    assert!(session.monitor_wallpapers.is_empty());
}

#[test]
fn test_session_json_empty_strings_filtered() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    // Per-monitor with empty paths
    let json = r#"{
        "wallpaperPath": "",
        "perMonitorWallpaper": true,
        "monitorWallpapers": {
            "HDMI-A-1": "/tmp/wp1.jpg",
            "DP-1": "",
            "eDP-1": "/tmp/wp3.jpg"
        }
    }"#;
    fs::write(temp_dir.path().join("session.json"), json).unwrap();
    SettingsFixture::new().write_to(temp_dir.path());

    let session = DmsSession::new(config);
    let state = session.get_current_state();

    assert!(state.is_ok());
    let state = state.unwrap();
    // Should only have 2 wallpapers (empty one filtered out)
    assert_eq!(state.wallpapers.len(), 2);
}

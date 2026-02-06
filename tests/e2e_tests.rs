//! End-to-end integration tests

mod common;

use std::fs;
use std::path::Path;
use std::time::Duration;

use common::{create_test_image, SessionFixture, SettingsFixture};
use dms_awww::config::Config;
use dms_awww::dms::DmsSession;
use dms_awww::executor::Executor;
use tokio::time::sleep;

/// Create a test config with paths pointing to a temp directory
fn test_config_with_dir(dir: &Path) -> Config {
    let mut config = Config::default();
    config.dms.session_file = dir.join("session.json").to_str().unwrap().to_string();
    config.dms.settings_file = dir.join("settings.json").to_str().unwrap().to_string();
    config.dms.cache_dir = dir.join("cache").to_str().unwrap().to_string();
    config.awww.enabled = false; // Disable actual commands in tests
    config.matugen.enabled = false;
    config
}

#[tokio::test]
async fn test_complete_workflow_single_wallpaper() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    // Create test image
    let test_image = temp_dir.path().join("wallpaper.jpg");
    create_test_image(&test_image);

    // Create session and settings files
    SessionFixture::new()
        .wallpaper_path(test_image.to_str().unwrap())
        .light_mode(false)
        .write_to(temp_dir.path());

    SettingsFixture::new()
        .matugen_scheme("scheme-tonal-spot")
        .write_to(temp_dir.path());

    // Create session manager
    let session = DmsSession::new(config.clone());

    // Get current state
    let state = session.get_current_state();
    assert!(state.is_ok());
    let state = state.unwrap();

    assert_eq!(state.wallpapers.len(), 1);
    assert_eq!(state.wallpapers[0].path, test_image.to_str().unwrap());
    assert!(!state.is_light_mode);

    // Create executor
    let executor = Executor::new(config, vec!["ALL".to_string()]);

    // Check dependencies (should pass since we disabled awww/matugen)
    assert!(executor.check_dependencies().is_ok());
}

#[tokio::test]
async fn test_complete_workflow_per_monitor() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    // Create test images
    let test_image1 = temp_dir.path().join("wallpaper1.jpg");
    let test_image2 = temp_dir.path().join("wallpaper2.jpg");
    create_test_image(&test_image1);
    create_test_image(&test_image2);

    // Create session and settings files
    SessionFixture::new()
        .per_monitor(true)
        .monitor_wallpaper("HDMI-A-1", test_image1.to_str().unwrap())
        .monitor_wallpaper("DP-1", test_image2.to_str().unwrap())
        .light_mode(true)
        .write_to(temp_dir.path());

    SettingsFixture::new()
        .matugen_scheme("scheme-expressive")
        .write_to(temp_dir.path());

    // Create session manager
    let session = DmsSession::new(config.clone());

    // Get current state
    let state = session.get_current_state();
    assert!(state.is_ok());
    let state = state.unwrap();

    assert_eq!(state.wallpapers.len(), 2);
    assert!(state.is_light_mode);

    // Check for both monitors (order may vary)
    let hdmi_found = state.wallpapers.iter().any(|w| w.monitor.as_deref() == Some("HDMI-A-1"));
    let dp_found = state.wallpapers.iter().any(|w| w.monitor.as_deref() == Some("DP-1"));
    assert!(hdmi_found, "HDMI-A-1 wallpaper not found");
    assert!(dp_found, "DP-1 wallpaper not found");

    // Create executor with monitors
    let executor = Executor::new(config, vec!["HDMI-A-1".to_string(), "DP-1".to_string()]);
    assert!(executor.check_dependencies().is_ok());
}

#[tokio::test]
async fn test_workflow_wallpaper_change_detection() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    // Create initial wallpaper
    let test_image1 = temp_dir.path().join("wallpaper1.jpg");
    create_test_image(&test_image1);

    SessionFixture::new()
        .wallpaper_path(test_image1.to_str().unwrap())
        .write_to(temp_dir.path());

    SettingsFixture::new().write_to(temp_dir.path());

    let mut session = DmsSession::new(config);

    // First check - should detect initial state
    let has_changed = session.has_changed();
    assert!(has_changed.is_ok());
    assert!(has_changed.unwrap(), "Initial check should detect as changed");

    // Second check - no change
    let has_changed = session.has_changed();
    assert!(has_changed.is_ok());
    assert!(!has_changed.unwrap(), "Second check should show no change");

    // Change the wallpaper
    let test_image2 = temp_dir.path().join("wallpaper2.jpg");
    create_test_image(&test_image2);

    SessionFixture::new()
        .wallpaper_path(test_image2.to_str().unwrap())
        .write_to(temp_dir.path());

    // Third check - should detect change
    sleep(Duration::from_millis(10)).await; // Small delay to ensure file timestamp changes
    let has_changed = session.has_changed();
    assert!(has_changed.is_ok());
    assert!(has_changed.unwrap(), "Third check should detect new wallpaper");
}

#[tokio::test]
async fn test_workflow_theme_mode_detection() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    // Test dark mode
    SessionFixture::new()
        .wallpaper_path("/tmp/wp.jpg")
        .light_mode(false)
        .write_to(temp_dir.path());

    SettingsFixture::new().write_to(temp_dir.path());

    let session = DmsSession::new(config.clone());
    assert_eq!(session.get_theme_mode().unwrap(), "dark");

    // Test light mode
    SessionFixture::new()
        .wallpaper_path("/tmp/wp.jpg")
        .light_mode(true)
        .write_to(temp_dir.path());

    let session = DmsSession::new(config);
    assert_eq!(session.get_theme_mode().unwrap(), "light");
}

#[tokio::test]
async fn test_workflow_matugen_scheme_override() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    SessionFixture::new()
        .wallpaper_path("/tmp/wp.jpg")
        .write_to(temp_dir.path());

    SettingsFixture::new()
        .matugen_scheme("scheme-vibrant")
        .write_to(temp_dir.path());

    let session = DmsSession::new(config);
    let scheme = session.get_matugen_scheme();

    assert_eq!(scheme, "scheme-vibrant");
}

#[tokio::test]
async fn test_workflow_error_recovery_missing_session() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    // Don't create session file
    SettingsFixture::new().write_to(temp_dir.path());

    let session = DmsSession::new(config);

    // Should return an error
    let result = session.get_current_state();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_workflow_error_recovery_invalid_wallpaper() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    // Use a color value instead of path
    SessionFixture::new()
        .wallpaper_path("#ff0000")
        .write_to(temp_dir.path());

    SettingsFixture::new().write_to(temp_dir.path());

    let session = DmsSession::new(config);
    let result = session.get_current_state();

    // Should return an error (no valid wallpapers)
    assert!(result.is_err());
}

#[tokio::test]
async fn test_workflow_with_multiple_empty_wallpapers() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    // Create one valid image
    let test_image = temp_dir.path().join("valid.jpg");
    create_test_image(&test_image);

    // Per-monitor setup with some empty values and a color value
    let image_path = test_image.to_str().unwrap();
    // Build JSON manually to avoid format string escaping issues
    let json = format!(
        "{{\"wallpaperPath\":\"\",\"perMonitorWallpaper\":true,\"monitorWallpapers\":{{\"HDMI-A-1\":\"#ff0000\",\"DP-1\":\"\",\"eDP-1\":\"{}\"}}}}",
        image_path
    );
    fs::write(temp_dir.path().join("session.json"), json).unwrap();
    SettingsFixture::new().write_to(temp_dir.path());

    let session = DmsSession::new(config);
    let state = session.get_current_state();

    // Should succeed with only the valid wallpaper (color values filtered out)
    assert!(state.is_ok());
    let state = state.unwrap();
    assert_eq!(state.wallpapers.len(), 1);
    assert_eq!(state.wallpapers[0].monitor, Some("eDP-1".to_string()));
    assert_eq!(state.wallpapers[0].path, test_image.to_str().unwrap());
}

#[tokio::test]
async fn test_complete_workflow_light_to_dark_transition() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    let test_image = temp_dir.path().join("wallpaper.jpg");
    create_test_image(&test_image);

    // Start with light mode
    SessionFixture::new()
        .wallpaper_path(test_image.to_str().unwrap())
        .light_mode(true)
        .write_to(temp_dir.path());

    SettingsFixture::new().write_to(temp_dir.path());

    let session = DmsSession::new(config.clone());
    assert_eq!(session.get_theme_mode().unwrap(), "light");

    // Transition to dark mode
    SessionFixture::new()
        .wallpaper_path(test_image.to_str().unwrap())
        .light_mode(false)
        .write_to(temp_dir.path());

    let session = DmsSession::new(config);
    assert_eq!(session.get_theme_mode().unwrap(), "dark");
}

#[tokio::test]
async fn test_config_validation_with_paths() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let mut config = test_config_with_dir(temp_dir.path());

    // The parent directories should exist (temp_dir)
    assert!(config.validate().is_ok());

    // Test with non-existent parent
    config.dms.session_file = "/nonexistent/path/session.json".to_string();
    let result = config.validate();
    // Validation should warn but not error (just logs)
    assert!(result.is_ok());
}

#[test]
fn test_error_messages_are_user_friendly() {
    use dms_awww::error::DmsAwwwError;

    let tests = vec![
        (
            DmsAwwwError::CommandNotFound("awww".to_string()),
            "install"
        ),
        (
            DmsAwwwError::InvalidWallpaperPath("/nonexistent.jpg".to_string()),
            "wallpaper"
        ),
        (
            DmsAwwwError::SessionFileNotFound("/tmp/session.json".into()),
            "session"
        ),
        (
            DmsAwwwError::NoMonitorsDetected,
            "monitor"
        ),
    ];

    for (error, keyword) in tests {
        let msg = error.user_message().to_lowercase();
        assert!(
            msg.contains(keyword),
            "Error message '{}' should contain '{}'",
            msg,
            keyword
        );
    }
}

#[tokio::test]
async fn test_executor_with_explicit_monitors() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    let test_image = temp_dir.path().join("wallpaper.jpg");
    create_test_image(&test_image);

    SessionFixture::new()
        .wallpaper_path(test_image.to_str().unwrap())
        .write_to(temp_dir.path());

    SettingsFixture::new().write_to(temp_dir.path());

    let monitors = vec![
        "HDMI-A-1".to_string(),
        "DP-1".to_string(),
        "eDP-1".to_string(),
    ];
    let executor = Executor::new(config, monitors);

    // Executor created successfully
    assert!(executor.check_dependencies().is_ok());
}

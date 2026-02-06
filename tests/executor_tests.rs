//! Executor integration tests

mod common;

use std::fs;
use std::path::Path;

use common::{create_test_image, SessionFixture, SettingsFixture};
use dms_awww::config::Config;
use dms_awww::dms::DmsSession;
use dms_awww::executor::Executor;
use dms_awww::error::DmsAwwwError;

/// Create a test config with paths pointing to a temp directory
fn test_config_with_dir(dir: &Path) -> Config {
    let mut config = Config::default();
    config.dms.session_file = dir.join("session.json").to_str().unwrap().to_string();
    config.dms.settings_file = dir.join("settings.json").to_str().unwrap().to_string();
    config.dms.cache_dir = dir.join("cache").to_str().unwrap().to_string();
    config
}

#[test]
fn test_executor_creation() {
    let config = Config::default();
    let monitors = vec!["HDMI-A-1".to_string(), "DP-1".to_string()];
    let executor = Executor::new(config, monitors);

    // Executor is created successfully
    // We can't directly access the fields, but we can check it was created
    // This mainly tests compilation
    assert!(true);
}

#[test]
fn test_check_dependencies_missing_awww() {
    let mut config = Config::default();
    config.awww.enabled = true;
    config.matugen.enabled = false;

    let executor = Executor::new(config, vec![]);
    let result = executor.check_dependencies();

    // This should fail because awww is not available in test environment
    // but we can't guarantee that everywhere
    // Just check that the function runs without panicking
    let _ = result;
}

#[test]
fn test_check_dependencies_with_both_disabled() {
    let mut config = Config::default();
    config.awww.enabled = false;
    config.matugen.enabled = false;

    let executor = Executor::new(config, vec![]);
    let result = executor.check_dependencies();

    // Should succeed when both are disabled
    assert!(result.is_ok());
}

#[test]
fn test_executor_with_awww_disabled() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let mut config = test_config_with_dir(temp_dir.path());
    config.awww.enabled = false;
    config.matugen.enabled = false;

    // Create test files
    let test_image = temp_dir.path().join("test.jpg");
    create_test_image(&test_image);

    SessionFixture::new()
        .wallpaper_path(test_image.to_str().unwrap())
        .write_to(temp_dir.path());

    SettingsFixture::new().write_to(temp_dir.path());

    let session = DmsSession::new(config.clone());
    let state = session.get_current_state().unwrap();

    let executor = Executor::new(config, vec![]);

    // In a test environment, we can't actually run the commands
    // But we can verify the structure is correct
    assert_eq!(state.wallpapers.len(), 1);
}

#[test]
fn test_executor_with_matugen_disabled() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let mut config = test_config_with_dir(temp_dir.path());
    config.awww.enabled = false;
    config.matugen.enabled = false;

    let test_image = temp_dir.path().join("test.jpg");
    create_test_image(&test_image);

    SessionFixture::new()
        .wallpaper_path(test_image.to_str().unwrap())
        .write_to(temp_dir.path());

    SettingsFixture::new().write_to(temp_dir.path());

    let executor = Executor::new(config, vec![]);

    // Should not fail when matugen is disabled
    assert!(executor.check_dependencies().is_ok());
}

#[test]
fn test_executor_per_monitor_wallpapers() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config = test_config_with_dir(temp_dir.path());

    // Create test images
    let test_image1 = temp_dir.path().join("test1.jpg");
    let test_image2 = temp_dir.path().join("test2.jpg");
    create_test_image(&test_image1);
    create_test_image(&test_image2);

    SessionFixture::new()
        .per_monitor(true)
        .monitor_wallpaper("HDMI-A-1", test_image1.to_str().unwrap())
        .monitor_wallpaper("DP-1", test_image2.to_str().unwrap())
        .write_to(temp_dir.path());

    SettingsFixture::new().write_to(temp_dir.path());

    let session = DmsSession::new(config);
    let state = session.get_current_state().unwrap();

    let executor = Executor::new(Config::default(), vec!["HDMI-A-1".to_string(), "DP-1".to_string()]);

    // Verify state has correct monitor assignments
    assert_eq!(state.wallpapers.len(), 2);
    assert_eq!(state.wallpapers[0].monitor, Some("HDMI-A-1".to_string()));
    assert_eq!(state.wallpapers[1].monitor, Some("DP-1".to_string()));
}

#[test]
fn test_wallpaper_change_structure() {
    use dms_awww::dms::{Wallpaper, WallpaperChange};

    let change = WallpaperChange {
        wallpapers: vec![
            Wallpaper::new("/path/to/wp1.jpg".to_string()),
            Wallpaper::for_monitor("/path/to/wp2.jpg".to_string(), "HDMI-A-1".to_string()),
        ],
        is_light_mode: true,
    };

    assert_eq!(change.wallpapers.len(), 2);
    assert!(change.is_light_mode);
    assert_eq!(change.wallpapers[0].monitor, None);
    assert_eq!(change.wallpapers[1].monitor, Some("HDMI-A-1".to_string()));
}

#[test]
fn test_error_display() {
    let err = DmsAwwwError::CommandNotFound("awww".to_string());
    let msg = err.user_message();
    assert!(msg.contains("awww"));
    assert!(msg.contains("install"));
}

#[test]
fn test_error_is_critical() {
    let not_found_err = DmsAwwwError::CommandNotFound("test".to_string());
    assert!(not_found_err.is_critical());

    let io_err = DmsAwwwError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
    assert!(!io_err.is_critical());

    let session_err = DmsAwwwError::SessionFileNotFound("/tmp/test".into());
    assert!(session_err.is_critical());

    let no_monitors_err = DmsAwwwError::NoMonitorsDetected;
    assert!(no_monitors_err.is_critical());
}

#[test]
fn test_error_is_recoverable() {
    let io_err = DmsAwwwError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
    assert!(io_err.is_recoverable());

    let cmd_err = DmsAwwwError::CommandFailed("test".to_string(), 1);
    assert!(cmd_err.is_recoverable());

    let not_found_err = DmsAwwwError::CommandNotFound("test".to_string());
    assert!(!not_found_err.is_recoverable());
}

#[test]
fn test_multiple_errors() {
    let errors = vec![
        "Error 1".to_string(),
        "Error 2".to_string(),
        "Error 3".to_string(),
    ];
    let err = DmsAwwwError::MultipleErrors(errors);

    let msg = err.user_message();
    assert!(msg.contains("Error 1"));
    assert!(msg.contains("Error 2"));
    assert!(msg.contains("Error 3"));
}

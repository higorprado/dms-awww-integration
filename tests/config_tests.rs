//! Configuration integration tests

mod common;

use std::env;
use std::fs;
use std::path::PathBuf;
use serial_test::serial;

// Import from the main crate
use dms_awww::config::Config;

/// Helper to clear all DMS_AWWW environment variables
fn clear_env_vars() {
    let vars_to_clear = [
        "DMS_AWWW_LOG_LEVEL",
        "DMS_AWWW_LOG_FILE",
        "DMS_AWWW_AUTO_DETECT_MONITORS",
        "DMS_AWWW_SESSION_FILE",
        "DMS_AWWW_SETTINGS_FILE",
        "DMS_AWWW_CACHE_DIR",
        "DMS_AWWW_NIRI_OUTPUTS",
        "DMS_AWWW_AWWW_ENABLED",
        "DMS_AWWW_MATUGEN_ENABLED",
        "DMS_AWWW_MATUGEN_SCHEME",
        "DMS_AWWW_SHELL_DIR",
        "XDG_CONFIG_HOME",
    ];
    for var in vars_to_clear {
        env::remove_var(var);
    }
}

#[serial]
#[test]
fn test_config_load_default_values() {
    clear_env_vars();
    let config = Config::default();

    // Should return defaults
    assert_eq!(config.general.log_level, "info");
    assert_eq!(config.general.auto_detect_monitors, true);
    assert!(config.awww.enabled);
    assert!(config.matugen.enabled);
}

#[serial]
#[test]
fn test_config_load_from_toml() {
    clear_env_vars();
    let temp_dir = tempfile::TempDir::new().unwrap();

    // Create config directory and file
    let config_dir = temp_dir.path().join("dms-awww");
    fs::create_dir_all(&config_dir).unwrap();
    let config_file = config_dir.join("config.toml");

    let toml_content = r#"
[general]
log_level = "debug"
log_file = "/tmp/test.log"
auto_detect_monitors = false
debounce_ms = 200

[dms]
session_file = "/tmp/session.json"
settings_file = "/tmp/settings.json"
cache_dir = "/tmp/cache"

[niri]
outputs = ["HDMI-A-1", "DP-1"]

[awww]
enabled = false
extra_args = ["--fast"]

[matugen]
enabled = false
default_scheme = "scheme-expressive"
shell_dir = "/tmp/shell"
"#;
    fs::write(&config_file, toml_content).unwrap();

    // Set XDG_CONFIG_HOME to temp dir
    env::set_var("XDG_CONFIG_HOME", temp_dir.path());

    let config = Config::load().unwrap();

    assert_eq!(config.general.log_level, "debug");
    assert_eq!(config.general.log_file, "/tmp/test.log");
    assert_eq!(config.general.auto_detect_monitors, false);
    assert_eq!(config.general.debounce_ms, 200);
    assert_eq!(config.dms.session_file, "/tmp/session.json");
    assert!(!config.awww.enabled);
    assert!(!config.matugen.enabled);
    assert_eq!(config.niri.outputs.len(), 2);

    clear_env_vars();
}

#[serial]
#[test]
fn test_config_load_from_yaml() {
    clear_env_vars();
    let temp_dir = tempfile::TempDir::new().unwrap();

    // Create config directory and file
    let config_dir = temp_dir.path().join("dms-awww");
    fs::create_dir_all(&config_dir).unwrap();
    let config_file = config_dir.join("config.yaml");

    let yaml_content = r#"
general:
  log_level: trace
  log_file: /var/log/dms-awww.log
  auto_detect_monitors: true

awww:
  enabled: true

matugen:
  enabled: false
"#;
    fs::write(&config_file, yaml_content).unwrap();

    env::set_var("XDG_CONFIG_HOME", temp_dir.path());

    let config = Config::load().unwrap();

    assert_eq!(config.general.log_level, "trace");
    assert_eq!(config.general.log_file, "/var/log/dms-awww.log");
    assert!(config.awww.enabled);
    assert!(!config.matugen.enabled);

    clear_env_vars();
}

#[serial]
#[test]
fn test_env_var_override_log_level() {
    clear_env_vars();
    env::set_var("DMS_AWWW_LOG_LEVEL", "debug");

    let config = Config::load().unwrap();
    assert_eq!(config.general.log_level, "debug");

    clear_env_vars();
}

#[serial]
#[test]
fn test_env_var_override_boolean() {
    clear_env_vars();
    env::set_var("DMS_AWWW_AWWW_ENABLED", "false");
    env::set_var("DMS_AWWW_MATUGEN_ENABLED", "false");

    let config = Config::load().unwrap();
    assert!(!config.awww.enabled);
    assert!(!config.matugen.enabled);

    clear_env_vars();
}

#[serial]
#[test]
fn test_env_var_override_outputs() {
    clear_env_vars();
    env::set_var("DMS_AWWW_NIRI_OUTPUTS", "HDMI-A-1,DP-1,eDP-1");

    let config = Config::load().unwrap();
    assert_eq!(config.niri.outputs.len(), 3);
    assert_eq!(config.niri.outputs[0], "HDMI-A-1");
    assert_eq!(config.niri.outputs[1], "DP-1");
    assert_eq!(config.niri.outputs[2], "eDP-1");

    clear_env_vars();
}

#[serial]
#[test]
fn test_env_var_override_paths() {
    clear_env_vars();
    env::set_var("DMS_AWWW_SESSION_FILE", "/custom/session.json");
    env::set_var("DMS_AWWW_SETTINGS_FILE", "/custom/settings.json");
    env::set_var("DMS_AWWW_CACHE_DIR", "/custom/cache");

    let config = Config::load().unwrap();
    // After expansion, paths should be set
    assert!(config.dms.session_file.contains("session.json"));
    assert!(config.dms.settings_file.contains("settings.json"));
    assert!(config.dms.cache_dir.contains("cache"));

    clear_env_vars();
}

#[serial]
#[test]
fn test_path_expansion_tilde() {
    clear_env_vars();
    let home = env::var("HOME").unwrap();

    env::set_var("DMS_AWWW_SESSION_FILE", "~/test/session.json");
    let config = Config::load().unwrap();

    // After expansion, should not contain ~
    assert!(!config.dms.session_file.contains('~'));
    assert!(config.dms.session_file.starts_with(&home));

    clear_env_vars();
}

#[serial]
#[test]
fn test_path_expansion_env_var() {
    clear_env_vars();
    env::set_var("TEST_PATH_VAR", "/tmp/test");

    env::set_var("DMS_AWWW_SESSION_FILE", "$TEST_PATH_VAR/session.json");
    let config = Config::load().unwrap();

    assert_eq!(config.dms.session_file, "/tmp/test/session.json");

    clear_env_vars();
    env::remove_var("TEST_PATH_VAR");
}

#[serial]
#[test]
fn test_validation_with_invalid_log_level() {
    clear_env_vars();
    env::set_var("DMS_AWWW_LOG_LEVEL", "invalid");

    let config = Config::load().unwrap();
    let result = config.validate();

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("log_level"));
    }

    clear_env_vars();
}

#[serial]
#[test]
fn test_validation_with_valid_log_levels() {
    clear_env_vars();

    for level in ["trace", "debug", "info", "warn", "error"] {
        clear_env_vars();
        env::set_var("DMS_AWWW_LOG_LEVEL", level);
        let config = Config::load().unwrap();
        assert!(config.validate().is_ok(), "Log level {} should be valid", level);
    }

    clear_env_vars();
}

#[serial]
#[test]
fn test_config_file_priority() {
    clear_env_vars();
    let temp_dir = tempfile::TempDir::new().unwrap();

    let config_dir = temp_dir.path().join("dms-awww");
    fs::create_dir_all(&config_dir).unwrap();

    // Create both TOML and YAML files - TOML should take priority
    let toml_file = config_dir.join("config.toml");
    let yaml_file = config_dir.join("config.yaml");

    fs::write(&toml_file, "[general]\nlog_level = \"debug\"").unwrap();
    fs::write(&yaml_file, "general:\n  log_level: trace").unwrap();

    env::set_var("XDG_CONFIG_HOME", temp_dir.path());

    let config = Config::load().unwrap();
    assert_eq!(config.general.log_level, "debug", "TOML should have priority");

    clear_env_vars();
}

#[serial]
#[test]
fn test_get_monitor_outputs_explicit() {
    clear_env_vars();
    env::set_var("DMS_AWWW_NIRI_OUTPUTS", "HDMI-A-1,DP-1");

    let config = Config::load().unwrap();
    let outputs = config.get_monitor_outputs();

    assert_eq!(outputs.len(), 2);

    clear_env_vars();
}

#[serial]
#[test]
fn test_get_monitor_outputs_empty_returns_empty_vec() {
    clear_env_vars();

    let config = Config::load().unwrap();
    let outputs = config.get_monitor_outputs();

    // Empty outputs should return empty vec, triggering auto-detection
    assert!(outputs.is_empty());
}

#[serial]
#[test]
fn test_awww_enabled() {
    clear_env_vars();
    let config = Config::load().unwrap();
    assert!(config.awww_enabled());

    clear_env_vars();
    env::set_var("DMS_AWWW_AWWW_ENABLED", "false");
    let config = Config::load().unwrap();
    assert!(!config.awww_enabled());

    clear_env_vars();
}

#[serial]
#[test]
fn test_matugen_enabled() {
    clear_env_vars();
    let config = Config::load().unwrap();
    assert!(config.matugen_enabled());

    clear_env_vars();
    env::set_var("DMS_AWWW_MATUGEN_ENABLED", "false");
    let config = Config::load().unwrap();
    assert!(!config.matugen_enabled());

    clear_env_vars();
}

#[serial]
#[test]
fn test_expand_path_static() {
    clear_env_vars();

    // Test tilde expansion
    let home = env::var("HOME").unwrap();
    let expanded = Config::expand_path("~/.config/test");
    assert!(expanded.starts_with(&home));

    // Test env var expansion
    env::set_var("TEST_VAR", "/tmp/test");
    let expanded = Config::expand_path("$TEST_VAR/file.txt");
    assert_eq!(expanded, "/tmp/test/file.txt");

    clear_env_vars();
}

#[serial]
#[test]
fn test_config_session_file_path() {
    clear_env_vars();
    let config = Config::default();

    let path = config.session_file_path();
    assert!(path.to_string_lossy().contains("session.json"));
}

#[serial]
#[test]
fn test_config_settings_file_path() {
    clear_env_vars();
    let config = Config::default();

    let path = config.settings_file_path();
    assert!(path.to_string_lossy().contains("settings.json"));
}

#[serial]
#[test]
fn test_config_cache_dir_path() {
    clear_env_vars();
    let config = Config::default();

    let path = config.cache_dir_path();
    assert!(path.to_string_lossy().contains("DankMaterialShell"));
}

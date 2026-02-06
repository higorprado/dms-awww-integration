// DMS parsing benchmarks using criterion

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use dms_awww::config::Config;
use dms_awww::dms::{DmsSession, SessionJson, SettingsJson};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn bench_parse_session_json(c: &mut Criterion) {
    let json_single = r#"{
        "wallpaperPath": "/path/to/wallpaper.jpg",
        "perMonitorWallpaper": false,
        "isLightMode": false
    }"#;

    let json_per_monitor = r#"{
        "wallpaperPath": "/path/to/wallpaper.jpg",
        "perMonitorWallpaper": true,
        "monitorWallpapers": {
            "HDMI-A-1": "/path/to/wp1.jpg",
            "DP-1": "/path/to/wp2.jpg",
            "eDP-1": "/path/to/wp3.jpg"
        },
        "isLightMode": true
    }"#;

    let json_large = r#"{
        "wallpaperPath": "/path/to/wallpaper.jpg",
        "perMonitorWallpaper": true,
        "monitorWallpapers": {
            "HDMI-A-1": "/path/to/wp1.jpg",
            "DP-1": "/path/to/wp2.jpg",
            "eDP-1": "/path/to/wp3.jpg",
            "HDMI-A-2": "/path/to/wp4.jpg",
            "DP-2": "/path/to/wp5.jpg",
            "eDP-2": "/path/to/wp6.jpg"
        },
        "isLightMode": true
    }"#;

    let mut group = c.benchmark_group("parse_session_json");

    group.bench_function("single_wallpaper", |b| {
        b.iter(|| {
            let session: SessionJson = serde_json::from_str(black_box(json_single)).unwrap();
            black_box(session)
        })
    });

    group.bench_function("per_monitor_3", |b| {
        b.iter(|| {
            let session: SessionJson = serde_json::from_str(black_box(json_per_monitor)).unwrap();
            black_box(session)
        })
    });

    group.bench_function("per_monitor_6", |b| {
        b.iter(|| {
            let session: SessionJson = serde_json::from_str(black_box(json_large)).unwrap();
            black_box(session)
        })
    });

    group.finish();
}

fn bench_parse_settings_json(c: &mut Criterion) {
    let json_simple = r#"{
        "matugenScheme": "scheme-tonal-spot"
    }"#;

    let json_with_fields = r#"{
        "matugenScheme": "scheme-expressive",
        "otherSetting": "value",
        "anotherSetting": 42
    }"#;

    let mut group = c.benchmark_group("parse_settings_json");

    group.bench_function("simple", |b| {
        b.iter(|| {
            let settings: SettingsJson = serde_json::from_str(black_box(json_simple)).unwrap();
            black_box(settings)
        })
    });

    group.bench_function("with_fields", |b| {
        b.iter(|| {
            let settings: SettingsJson = serde_json::from_str(black_box(json_with_fields)).unwrap();
            black_box(settings)
        })
    });

    group.finish();
}

fn bench_session_get_current_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_get_current_state");

    // Setup for single wallpaper
    let temp_dir_single = TempDir::new().unwrap();
    let mut config_single = Config::default();
    config_single.dms.session_file = temp_dir_single.path().join("session.json").to_str().unwrap().to_string();
    config_single.dms.settings_file = temp_dir_single.path().join("settings.json").to_str().unwrap().to_string();

    let json_single = r#"{
        "wallpaperPath": "/tmp/wallpaper.jpg",
        "perMonitorWallpaper": false,
        "isLightMode": false
    }"#;
    fs::write(temp_dir_single.path().join("session.json"), json_single).unwrap();
    fs::write(temp_dir_single.path().join("settings.json"), r#"{"matugenScheme":"scheme-tonal-spot"}"#).unwrap();

    group.bench_function("single_wallpaper", |b| {
        let session = DmsSession::new(config_single.clone());
        b.iter(|| {
            let state = session.get_current_state();
            black_box(state)
        })
    });

    // Setup for per-monitor
    let temp_dir_multi = TempDir::new().unwrap();
    let mut config_multi = Config::default();
    config_multi.dms.session_file = temp_dir_multi.path().join("session.json").to_str().unwrap().to_string();
    config_multi.dms.settings_file = temp_dir_multi.path().join("settings.json").to_str().unwrap().to_string();

    let json_multi = r#"{
        "wallpaperPath": "/tmp/wallpaper.jpg",
        "perMonitorWallpaper": true,
        "monitorWallpapers": {
            "HDMI-A-1": "/tmp/wp1.jpg",
            "DP-1": "/tmp/wp2.jpg",
            "eDP-1": "/tmp/wp3.jpg"
        },
        "isLightMode": true
    }"#;
    fs::write(temp_dir_multi.path().join("session.json"), json_multi).unwrap();
    fs::write(temp_dir_multi.path().join("settings.json"), r#"{"matugenScheme":"scheme-tonal-spot"}"#).unwrap();

    group.bench_function("per_monitor_3", |b| {
        let session = DmsSession::new(config_multi.clone());
        b.iter(|| {
            let state = session.get_current_state();
            black_box(state)
        })
    });

    group.finish();
}

fn bench_session_has_changed(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let mut config = Config::default();
    config.dms.session_file = temp_dir.path().join("session.json").to_str().unwrap().to_string();
    config.dms.settings_file = temp_dir.path().join("settings.json").to_str().unwrap().to_string();

    let json = r#"{
        "wallpaperPath": "/tmp/wallpaper.jpg",
        "perMonitorWallpaper": false,
        "isLightMode": false
    }"#;
    fs::write(temp_dir.path().join("session.json"), json).unwrap();
    fs::write(temp_dir.path().join("settings.json"), r#"{"matugenScheme":"scheme-tonal-spot"}"#).unwrap();

    c.bench_function("session_has_changed_no_change", |b| {
        b.iter(|| {
            let mut session = DmsSession::new(config.clone());
            // First call
            let _ = session.has_changed();
            // Second call (no change)
            let changed = session.has_changed();
            black_box(changed)
        })
    });
}

fn bench_session_read_file(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let mut config = Config::default();
    config.dms.session_file = temp_dir.path().join("session.json").to_str().unwrap().to_string();

    let json = r#"{
        "wallpaperPath": "/tmp/wallpaper.jpg",
        "perMonitorWallpaper": false,
        "isLightMode": false
    }"#;
    fs::write(temp_dir.path().join("session.json"), json).unwrap();

    c.bench_function("session_read_session", |b| {
        let session = DmsSession::new(config.clone());
        b.iter(|| {
            let result = session.read_session();
            black_box(result)
        })
    });
}

fn bench_wallpaper_validation(c: &mut Criterion) {
    use dms_awww::dms::Wallpaper;

    c.bench_function("wallpaper_is_valid_image", |b| {
        let valid = Wallpaper::new("/path/to/image.jpg".to_string());
        b.iter(|| {
            let is_valid = valid.is_valid_image();
            black_box(is_valid)
        })
    });
}

fn bench_session_helpers(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let mut config = Config::default();
    config.dms.session_file = temp_dir.path().join("session.json").to_str().unwrap().to_string();
    config.dms.settings_file = temp_dir.path().join("settings.json").to_str().unwrap().to_string();

    let json_session = r#"{
        "wallpaperPath": "/tmp/wallpaper.jpg",
        "isLightMode": true
    }"#;
    let json_settings = r#"{
        "matugenScheme": "scheme-expressive"
    }"#;
    fs::write(temp_dir.path().join("session.json"), json_session).unwrap();
    fs::write(temp_dir.path().join("settings.json"), json_settings).unwrap();

    let session = DmsSession::new(config);

    let mut group = c.benchmark_group("session_helpers");

    group.bench_function("get_matugen_scheme", |b| {
        b.iter(|| {
            let scheme = session.get_matugen_scheme();
            black_box(scheme)
        })
    });

    group.bench_function("get_theme_mode", |b| {
        b.iter(|| {
            let mode = session.get_theme_mode();
            black_box(mode)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_session_json,
    bench_parse_settings_json,
    bench_session_get_current_state,
    bench_session_has_changed,
    bench_session_read_file,
    bench_wallpaper_validation,
    bench_session_helpers
);

criterion_main!(benches);

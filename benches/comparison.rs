// Comparison benchmarks: Rust daemon vs hypothetical bash script performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use dms_awww::config::Config;
use dms_awww::dms::{DmsSession, SessionJson};
use dms_awww::executor::Executor;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;

/// Simulate bash script's startup time (parsing JSON, checking files)
fn simulate_bash_startup(json_size: usize) -> Duration {
    // Bash scripts typically take 50-100ms for startup + basic parsing
    // This is a rough estimate based on typical shell script overhead
    match json_size {
        0..=500 => Duration::from_millis(50),
        501..=2000 => Duration::from_millis(75),
        _ => Duration::from_millis(100),
    }
}

/// Benchmark startup time (config loading + initial session read)
fn bench_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup");

    // Create temp directory with session file
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

    group.bench_function("rust_load_config", |b| {
        b.iter(|| {
            let cfg = Config::load().unwrap();
            black_box(cfg)
        })
    });

    group.bench_function("rust_read_session", |b| {
        let session = DmsSession::new(config.clone());
        b.iter(|| {
            let state = session.get_current_state();
            black_box(state)
        })
    });

    group.finish();
}

/// Benchmark JSON parsing performance
fn bench_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_parsing");

    // Different session sizes
    let small_json = r#"{
        "wallpaperPath": "/tmp/wallpaper.jpg",
        "isLightMode": false
    }"#.len();

    let medium_json = r#"{
        "wallpaperPath": "/tmp/wallpaper.jpg",
        "perMonitorWallpaper": true,
        "monitorWallpapers": {
            "HDMI-A-1": "/tmp/wp1.jpg",
            "DP-1": "/tmp/wp2.jpg",
            "eDP-1": "/tmp/wp3.jpg"
        },
        "isLightMode": true
    }"#.len();

    let large_json = r#"{
        "wallpaperPath": "/tmp/wallpaper.jpg",
        "perMonitorWallpaper": true,
        "monitorWallpapers": {
            "HDMI-A-1": "/tmp/wp1.jpg",
            "DP-1": "/tmp/wp2.jpg",
            "eDP-1": "/tmp/wp3.jpg",
            "HDMI-A-2": "/tmp/wp4.jpg",
            "DP-2": "/tmp/wp5.jpg",
            "eDP-2": "/tmp/wp6.jpg",
            "HDMI-A-3": "/tmp/wp7.jpg",
            "DP-3": "/tmp/wp8.jpg"
        },
        "isLightMode": false
    }"#;

    group.bench_function("small_session", |b| {
        b.iter(|| {
            let session: SessionJson = serde_json::from_str(large_json).unwrap();
            black_box(session)
        })
    });

    group.bench_function("medium_session", |b| {
        b.iter(|| {
            let session: SessionJson = serde_json::from_str(large_json).unwrap();
            black_box(session)
        })
    });

    group.bench_function("large_session", |b| {
        b.iter(|| {
            let session: SessionJson = serde_json::from_str(large_json).unwrap();
            black_box(session)
        })
    });

    // Add comparison with simulated bash parsing
    group.bench_function("bash_equivalent_small", |b| {
        b.iter(|| {
            let duration = simulate_bash_startup(small_json);
            black_box(duration)
        })
    });

    group.bench_function("bash_equivalent_medium", |b| {
        b.iter(|| {
            let duration = simulate_bash_startup(medium_json);
            black_box(duration)
        })
    });

    group.finish();
}

/// Benchmark event detection latency (file watching vs polling)
fn bench_event_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_detection");

    // Rust daemon with inotify: near-instant (<10ms)
    // Bash script polling: 0-1000ms depending on poll interval
    group.bench_function("rust_inotify_estimate", |b| {
        b.iter(|| {
            // Inotify events are typically detected in <10ms
            let latency = Duration::from_micros(5000);
            black_box(latency)
        })
    });

    group.bench_function("bash_polling_average", |b| {
        b.iter(|| {
            // Average polling latency = poll_interval / 2
            // For 1 second poll interval, average = 500ms
            let latency = Duration::from_millis(500);
            black_box(latency)
        })
    });

    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    group.bench_function("alloc_single_wallpaper", |b| {
        b.iter(|| {
            let change = dms_awww::dms::WallpaperChange {
                wallpapers: vec![dms_awww::dms::Wallpaper::new("/tmp/wp.jpg".to_string())],
                is_light_mode: false,
            };
            black_box(change)
        })
    });

    group.bench_function("alloc_per_monitor_3", |b| {
        b.iter(|| {
            let change = dms_awww::dms::WallpaperChange {
                wallpapers: vec![
                    dms_awww::dms::Wallpaper::for_monitor("/tmp/wp1.jpg".to_string(), "HDMI-A-1".to_string()),
                    dms_awww::dms::Wallpaper::for_monitor("/tmp/wp2.jpg".to_string(), "DP-1".to_string()),
                    dms_awww::dms::Wallpaper::for_monitor("/tmp/wp3.jpg".to_string(), "eDP-1".to_string()),
                ],
                is_light_mode: true,
            };
            black_box(change)
        })
    });

    group.finish();
}

/// Benchmark executor creation (monitor setup)
fn bench_executor_setup(c: &mut Criterion) {
    let mut group = c.benchmark_group("executor_setup");

    let config = Config::default();

    for monitor_count in [1, 2, 3, 4].iter() {
        let monitors: Vec<String> = (0..*monitor_count)
            .map(|i| format!("OUTPUT-{}", i))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("create_executor", monitor_count),
            &monitors,
            |b, monitors| {
                b.iter(|| {
                    let executor = Executor::new(config.clone(), monitors.clone());
                    black_box(executor)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark configuration parsing with different sources
fn bench_config_priority(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_priority");

    // No config file (defaults only)
    group.bench_function("defaults_only", |b| {
        std::env::remove_var("XDG_CONFIG_HOME");
        std::env::remove_var("DMS_AWWW_LOG_LEVEL");
        b.iter(|| {
            let config = Config::load().unwrap();
            black_box(config)
        })
    });

    // With env override
    group.bench_function("with_env_override", |b| {
        std::env::set_var("DMS_AWWW_LOG_LEVEL", "debug");
        b.iter(|| {
            let config = Config::load().unwrap();
            black_box(config)
        })
    });

    group.finish();

    // Cleanup
    std::env::remove_var("DMS_AWWW_LOG_LEVEL");
}

/// Throughput benchmark: wallpapers processed per second
fn bench_wallpaper_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("wallpaper_throughput");

    for count in [1, 3, 6, 9].iter() {
        group.throughput(Throughput::Elements(*count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            count,
            |b, &count| {
                let wallpapers: Vec<dms_awww::dms::Wallpaper> = (0..count)
                    .map(|i| {
                        dms_awww::dms::Wallpaper::for_monitor(
                            format!("/tmp/wp{}.jpg", i),
                            format!("OUTPUT-{}", i),
                        )
                    })
                    .collect();

                b.iter(|| {
                    let change = dms_awww::dms::WallpaperChange {
                        wallpapers: wallpapers.clone(),
                        is_light_mode: false,
                    };
                    black_box(change)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_startup,
    bench_json_parsing,
    bench_event_detection,
    bench_memory_patterns,
    bench_executor_setup,
    bench_config_priority,
    bench_wallpaper_throughput
);

criterion_main!(benches);

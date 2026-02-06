// Config loading benchmarks using criterion

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use dms_awww::config::Config;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn bench_config_load_default(c: &mut Criterion) {
    // Clear environment to ensure clean state
    env::remove_var("XDG_CONFIG_HOME");
    let vars_to_clear = [
        "DMS_AWWW_LOG_LEVEL", "DMS_AWWW_LOG_FILE", "DMS_AWWW_AUTO_DETECT_MONITORS",
        "DMS_AWWW_SESSION_FILE", "DMS_AWWW_SETTINGS_FILE", "DMS_AWWW_CACHE_DIR",
        "DMS_AWWW_NIRI_OUTPUTS", "DMS_AWWW_AWWW_ENABLED", "DMS_AWWW_MATUGEN_ENABLED",
        "DMS_AWWW_MATUGEN_SCHEME", "DMS_AWWW_SHELL_DIR",
    ];
    for var in vars_to_clear {
        env::remove_var(var);
    }

    c.bench_function("config_load_default", |b| {
        b.iter(|| {
            let config = Config::load().unwrap();
            black_box(config)
        })
    });
}

fn bench_config_path_expansion(c: &mut Criterion) {
    let home = env::var("HOME").unwrap();
    let test_cases = vec![
        ("~/.config/test", "tilde expansion"),
        ("/tmp/test", "absolute path"),
        ("$HOME/test", "env var expansion"),
    ];

    let mut group = c.benchmark_group("path_expansion");

    for (path, desc) in test_cases {
        group.bench_function(BenchmarkId::from_parameter(desc), |b| {
            b.iter(|| {
                let expanded = Config::expand_path(black_box(path));
                black_box(expanded)
            })
        });
    }

    group.finish();
}

fn bench_config_env_vars(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_env_parsing");

    // Benchmark with various numbers of env vars set
    for count in [0, 3, 6, 9].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, _| {
            b.iter_batched(
                || {
                    // Setup: clear all env vars
                    for var in [
                        "DMS_AWWW_LOG_LEVEL", "DMS_AWWW_AWWW_ENABLED", "DMS_AWWW_MATUGEN_ENABLED",
                        "DMS_AWWW_SESSION_FILE", "DMS_AWWW_NIRI_OUTPUTS", "DMS_AWWW_CACHE_DIR",
                    ] {
                        env::remove_var(var);
                    }
                    // Set the specified number of vars
                    match count {
                        3 => {
                            env::set_var("DMS_AWWW_LOG_LEVEL", "debug");
                            env::set_var("DMS_AWWW_AWWW_ENABLED", "false");
                            env::set_var("DMS_AWWW_MATUGEN_ENABLED", "false");
                        }
                        6 => {
                            env::set_var("DMS_AWWW_LOG_LEVEL", "debug");
                            env::set_var("DMS_AWWW_AWWW_ENABLED", "false");
                            env::set_var("DMS_AWWW_MATUGEN_ENABLED", "false");
                            env::set_var("DMS_AWWW_SESSION_FILE", "/tmp/test.json");
                            env::set_var("DMS_AWWW_NIRI_OUTPUTS", "HDMI-A-1,DP-1");
                            env::set_var("DMS_AWWW_CACHE_DIR", "/tmp/cache");
                        }
                        9 => {
                            env::set_var("DMS_AWWW_LOG_LEVEL", "debug");
                            env::set_var("DMS_AWWW_LOG_FILE", "/tmp/test.log");
                            env::set_var("DMS_AWWW_AUTO_DETECT_MONITORS", "false");
                            env::set_var("DMS_AWWW_AWWW_ENABLED", "false");
                            env::set_var("DMS_AWWW_MATUGEN_ENABLED", "false");
                            env::set_var("DMS_AWWW_SESSION_FILE", "/tmp/test.json");
                            env::set_var("DMS_AWWW_NIRI_OUTPUTS", "HDMI-A-1,DP-1");
                            env::set_var("DMS_AWWW_CACHE_DIR", "/tmp/cache");
                            env::set_var("DMS_AWWW_MATUGEN_SCHEME", "scheme-expressive");
                        }
                        _ => {}
                    }
                },
                |_| {
                    let config = Config::load().unwrap();
                    black_box(config)
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();

    // Cleanup
    for var in [
        "DMS_AWWW_LOG_LEVEL", "DMS_AWWW_LOG_FILE", "DMS_AWWW_AUTO_DETECT_MONITORS",
        "DMS_AWWW_AWWW_ENABLED", "DMS_AWWW_MATUGEN_ENABLED", "DMS_AWWW_SESSION_FILE",
        "DMS_AWWW_NIRI_OUTPUTS", "DMS_AWWW_CACHE_DIR", "DMS_AWWW_MATUGEN_SCHEME",
    ] {
        env::remove_var(var);
    }
}

fn bench_config_load_from_file(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_from_file");

    for size in ["small", "medium", "large"].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let temp_dir = TempDir::new().unwrap();
            let config_dir = temp_dir.path().join("dms-awww");
            fs::create_dir_all(&config_dir).unwrap();

            let content = match *size {
                "small" => {
                    r#"
[general]
log_level = "debug"
"#
                }
                "medium" => {
                    r#"
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
"#
                }
                "large" => {
                    r#"
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
outputs = ["HDMI-A-1", "DP-1", "eDP-1", "HDMI-A-2", "DP-2"]

[awww]
enabled = false
extra_args = ["--fast", "--verbose", "--no-cache"]

[matugen]
enabled = false
default_scheme = "scheme-expressive"
shell_dir = "/tmp/shell"
"#
                }
                _ => "",
            };

            let config_file = config_dir.join("config.toml");
            fs::write(&config_file, content).unwrap();
            env::set_var("XDG_CONFIG_HOME", temp_dir.path());

            b.iter(|| {
                let config = Config::load().unwrap();
                black_box(config)
            });

            env::remove_var("XDG_CONFIG_HOME");
        });
    }

    group.finish();
}

fn bench_config_validation(c: &mut Criterion) {
    c.bench_function("config_validate", |b| {
        let config = Config::default();
        b.iter(|| {
            let result = config.validate();
            black_box(result)
        })
    });
}

fn bench_config_helpers(c: &mut Criterion) {
    let config = Config::default();

    let mut group = c.benchmark_group("config_helpers");

    group.bench_function("session_file_path", |b| {
        b.iter(|| {
            let path = config.session_file_path();
            black_box(path)
        })
    });

    group.bench_function("settings_file_path", |b| {
        b.iter(|| {
            let path = config.settings_file_path();
            black_box(path)
        })
    });

    group.bench_function("awww_enabled", |b| {
        b.iter(|| {
            let enabled = config.awww_enabled();
            black_box(enabled)
        })
    });

    group.bench_function("get_monitor_outputs", |b| {
        b.iter(|| {
            let outputs = config.get_monitor_outputs();
            black_box(outputs)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_config_load_default,
    bench_config_path_expansion,
    bench_config_env_vars,
    bench_config_load_from_file,
    bench_config_validation,
    bench_config_helpers
);

criterion_main!(benches);

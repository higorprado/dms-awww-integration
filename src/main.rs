//! dms-awww: Efficient wallpaper management for DMS using awww
//!
//! This daemon monitors DMS session.json for wallpaper changes and applies
//! them via awww (efficient, no VRAM) while keeping themes in sync via matugen.

mod config;
mod dms;
mod error;
mod executor;
mod niri;
mod watcher;

use crate::config::Config;
use crate::dms::DmsSession;
use crate::error::{DmsAwwwError, Result};
use crate::executor::Executor;
use crate::watcher::DebouncedWatcher;
use clap::Parser;
use std::time::Duration;
use tracing::{error, info, warn, Level};
use tracing_subscriber::{
    fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

/// dms-awww: Efficient wallpaper management for DMS using awww
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,

    /// Log level (overrides config)
    #[arg(short, long)]
    log_level: Option<String>,

    /// Run once and exit (for testing)
    #[arg(short, long)]
    once: bool,

    /// Verbose output (shortcut for --log-level debug)
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose {
        Level::DEBUG
    } else if let Some(level) = &args.log_level {
        parse_log_level(level)
    } else {
        Level::INFO
    };

    init_logging(log_level);

    info!("dms-awww v{} starting", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            eprintln!("Error: {}", e.user_message());
            std::process::exit(1);
        }
    };

    // Override log level from command line
    let final_log_level = if args.verbose {
        Level::DEBUG
    } else if let Some(level) = &args.log_level {
        parse_log_level(level)
    } else {
        parse_log_level(&config.general.log_level)
    };

    // Reinitialize logging with correct level if changed
    if final_log_level != log_level {
        init_logging(final_log_level);
    }

    // Validate configuration
    if let Err(e) = config.validate() {
        error!("Configuration validation failed: {}", e);
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    }

    // Run the main application
    let result = run(config, args.once).await;

    if let Err(e) = result {
        error!("Fatal error: {}", e);
        eprintln!("Error: {}", e.user_message());
        std::process::exit(1);
    }

    info!("dms-awww shutting down");
}

/// Initialize logging with console output
fn init_logging(level: Level) {
    let filter = EnvFilter::builder()
        .with_default_directive(level.into())
        .from_env_lossy();

    // Console layer
    let console_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_filter(filter);

    tracing_subscriber::registry().with(console_layer).init();
}

/// Parse log level string
fn parse_log_level(s: &str) -> Level {
    match s.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    }
}

/// Main application logic
async fn run(config: Config, once: bool) -> Result<()> {
    // Create DMS session manager
    let session = DmsSession::new(config.clone());

    // Get monitor outputs
    let monitors = if config.general.auto_detect_monitors {
        info!("Auto-detecting monitors...");
        niri::get_monitor_outputs(config.get_monitor_outputs(), true).await?
    } else {
        config.get_monitor_outputs()
    };

    if monitors.is_empty() {
        warn!("No monitors configured, using 'ALL' as fallback");
    } else {
        info!("Using monitors: {:?}", monitors);
    }

    // Create executor
    let executor = Executor::new(config.clone(), monitors);

    // Check dependencies
    executor.check_dependencies()?;

    // Initial wallpaper check/apply
    info!("Performing initial wallpaper check...");
    if let Err(e) = apply_wallpaper_once(&session, &executor).await {
        warn!("Initial wallpaper check failed: {}", e);
        // Don't fail on initial check - file might not exist yet
    }

    if once {
        info!("Running once, exiting...");
        return Ok(());
    }

    // Start file watcher
    info!("Starting file watcher for: {}", config.dms.session_file);
    let mut watcher = DebouncedWatcher::new(
        config.session_file_path(),
        config.general.debounce_ms,
    )?;

    info!("Entering event loop (Ctrl+C to exit)");

    // Main event loop
    loop {
        match watcher.next().await {
            Ok(Some(event)) => {
                info!("File event: {:?}", event);

                // Small delay to let the file write complete
                tokio::time::sleep(Duration::from_millis(50)).await;

                if let Err(e) = apply_wallpaper_once(&session, &executor).await {
                    error!("Failed to apply wallpaper: {}", e);

                    if e.is_critical() {
                        return Err(e);
                    }
                }
            }
            Ok(None) => {
                warn!("File watcher channel closed");
                return Err(DmsAwwwError::Watcher("Channel closed".to_string()));
            }
            Err(e) => {
                error!("Watcher error: {}", e);

                let error: DmsAwwwError = e;
                if error.is_critical() {
                    return Err(error);
                }

                // Retry after delay for non-critical errors
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

/// Apply wallpaper once (check for change and apply)
async fn apply_wallpaper_once(session: &DmsSession, executor: &Executor) -> Result<()> {
    // Get current wallpaper state
    let state = session.get_current_state()?;

    info!(
        "Wallpaper state: {} wallpaper(s), light mode: {}",
        state.wallpapers.len(), state.is_light_mode
    );

    // Apply the wallpaper
    executor.apply_wallpaper(&state).await?;

    Ok(())
}

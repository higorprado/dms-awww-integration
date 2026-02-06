//! Command execution for wallpaper and theme updates
//!
//! This module handles parallel execution of awww and matugen commands
//! with proper error handling and logging.

use crate::config::Config;
use crate::dms::WallpaperChange;
use crate::error::{DmsAwwwError, Result};
use tokio::process::Command;
use tokio::task::JoinSet;
use which::which;

/// Executor for applying wallpapers and themes
pub struct Executor {
    config: Config,
    monitors: Vec<String>,
}

impl Executor {
    /// Create a new executor
    pub fn new(config: Config, monitors: Vec<String>) -> Self {
        Self { config, monitors }
    }

    /// Check if all required commands are available
    pub fn check_dependencies(&self) -> Result<()> {
        if self.config.awww_enabled() {
            which("awww").map_err(|_| DmsAwwwError::CommandNotFound("awww".to_string()))?;
            tracing::debug!("awww found: {}", which("awww").unwrap().display());
        }

        if self.config.matugen_enabled() {
            which("dms").map_err(|_| DmsAwwwError::CommandNotFound("dms".to_string()))?;
            tracing::debug!("dms found: {}", which("dms").unwrap().display());
        }

        Ok(())
    }

    /// Apply wallpaper changes with both awww and matugen in parallel
    pub async fn apply_wallpaper(&self, change: &WallpaperChange) -> Result<()> {
        let mut results: Vec<Result<()>> = Vec::new();

        // Run awww and matugen in parallel
        let (awww_result, matugen_result) = tokio::join!(
            self.apply_awww(change),
            self.apply_matugen(change)
        );

        if let Err(e) = awww_result {
            tracing::error!("awww failed: {}", e);
            results.push(Err(e));
        }

        if let Err(e) = matugen_result {
            tracing::error!("matugen failed: {}", e);
            results.push(Err(e));
        }

        // Report results
        if results.is_empty() {
            tracing::info!("Wallpaper applied successfully");
            Ok(())
        } else if results.len() == 1 {
            results.into_iter().next().unwrap()
        } else {
            let error_messages: Vec<String> = results
                .into_iter()
                .filter_map(|e| e.err().map(|err| err.to_string()))
                .collect();
            Err(DmsAwwwError::MultipleErrors(error_messages))
        }
    }

    /// Apply wallpaper via awww for all monitors
    async fn apply_awww(&self, change: &WallpaperChange) -> Result<()> {
        if !self.config.awww_enabled() {
            tracing::debug!("awww is disabled, skipping");
            return Ok(());
        }

        tracing::info!("Applying wallpaper via awww");

        let mut tasks = JoinSet::new();

        // Create a task for each monitor-wallpaper combination
        for wallpaper in &change.wallpapers {
            let monitors_to_apply = if wallpaper.monitor.is_some() {
                // Per-monitor wallpaper
                vec![wallpaper.monitor.clone().unwrap()]
            } else if !self.monitors.is_empty() {
                // Apply to all detected monitors
                self.monitors.clone()
            } else {
                // No monitors, use "ALL" as fallback
                vec!["ALL".to_string()]
            };

            for monitor in monitors_to_apply {
                let path = wallpaper.path.clone();
                let extra_args = self.config.awww.extra_args.clone();

                tasks.spawn(async move {
                    Self::apply_awww_for_monitor(&path, &monitor, &extra_args).await
                });
            }
        }

        // Wait for all tasks to complete
        let mut errors = Vec::new();
        while let Some(result) = tasks.join_next().await {
            if let Err(e) = result {
                errors.push(format!("Task panic: {}", e));
            } else if let Err(e) = result.unwrap() {
                errors.push(e.to_string());
            }
        }

        if !errors.is_empty() {
            return Err(DmsAwwwError::MultipleErrors(errors));
        }

        Ok(())
    }

    /// Apply wallpaper for a single monitor
    async fn apply_awww_for_monitor(
        path: &str,
        monitor: &str,
        extra_args: &[String],
    ) -> Result<()> {
        tracing::debug!("Applying wallpaper {} to monitor {}", path, monitor);

        // Verify the file exists
        if !std::path::Path::new(path).exists() {
            return Err(DmsAwwwError::InvalidWallpaperPath(path.to_string()));
        }

        let mut cmd = Command::new("awww");

        // Add extra args if configured
        for arg in extra_args {
            cmd.arg(arg);
        }

        cmd.args(["img", "-o", monitor, path]);

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let code = output.status.code().unwrap_or(-1);
            tracing::error!("awww failed for {}: {}", monitor, stderr);
            return Err(DmsAwwwError::CommandFailed("awww".to_string(), code));
        }

        tracing::debug!("awww succeeded for monitor {}", monitor);
        Ok(())
    }

    /// Apply theme via DMS matugen
    async fn apply_matugen(&self, change: &WallpaperChange) -> Result<()> {
        if !self.config.matugen_enabled() {
            tracing::debug!("matugen is disabled, skipping");
            return Ok(());
        }

        // Use the first wallpaper for theme generation
        let wallpaper = change.wallpapers.first()
            .ok_or_else(|| DmsAwwwError::InvalidWallpaperPath("No wallpapers".to_string()))?;

        // Verify the file exists
        if !std::path::Path::new(&wallpaper.path).exists() {
            return Err(DmsAwwwError::InvalidWallpaperPath(wallpaper.path.clone()));
        }

        tracing::info!("Triggering DMS matugen for theme update");

        let mode = if change.is_light_mode { "light" } else { "dark" };
        let matugen_type = self.config.matugen.default_scheme.clone();

        let cache_dir = &self.config.dms.cache_dir;
        let shell_dir = &self.config.matugen.shell_dir;
        let config_dir = std::env::var("HOME")
            .map(|h| format!("{}/.config", h))
            .unwrap_or_else(|_| "~/.config".to_string());

        let output = Command::new("dms")
            .args([
                "matugen", "queue",
                "--state-dir", cache_dir,
                "--shell-dir", shell_dir,
                "--config-dir", &config_dir,
                "--kind", "image",
                "--value", &wallpaper.path,
                "--mode", mode,
                "--matugen-type", &matugen_type,
                "--wait",
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let code = output.status.code().unwrap_or(-1);
            tracing::error!("dms matugen failed: {}", stderr);
            return Err(DmsAwwwError::CommandFailed("dms matugen".to_string(), code));
        }

        tracing::info!("DMS matugen completed successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let config = Config::default();
        let monitors = vec!["HDMI-A-1".to_string()];
        let executor = Executor::new(config, monitors);
        assert_eq!(executor.monitors.len(), 1);
    }
}

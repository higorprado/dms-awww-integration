//! Niri IPC integration for monitor detection
//!
//! This module provides integration with Niri's IPC system to detect
//! available monitors and outputs.

use crate::error::{DmsAwwwError, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tokio::process::Command as AsyncCommand;

/// Niri output information from `niri msg outputs`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NiriOutput {
    /// Output name (e.g., "HDMI-A-1", "eDP-1")
    pub name: String,

    /// Whether this output is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Output make/model
    #[serde(default)]
    pub make: String,

    #[serde(default)]
    pub model: String,

    /// Output resolution
    #[serde(default)]
    pub resolution: Option<OutputResolution>,

    /// Output position
    #[serde(default)]
    pub position: Option<OutputPosition>,

    /// Refresh rate
    #[serde(default)]
    pub refresh_rate: Option<f32>,

    /// Physical size
    #[serde(default, rename = "physicalSize")]
    pub physical_size: Option<PhysicalSize>,

    /// Current workspace
    #[serde(default, rename = "currentWorkspace")]
    pub current_workspace: Option<String>,
}

/// Output resolution in pixels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputResolution {
    pub width: u32,
    pub height: u32,
}

/// Output position in layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputPosition {
    pub x: i32,
    pub y: i32,
}

/// Physical size of the output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalSize {
    pub width: u32,
    pub height: u32,
}

/// Niri IPC client
pub struct NiriClient;

impl NiriClient {
    /// Detect all enabled outputs using `niri msg outputs`
    /// First tries JSON output (newer niri versions), falls back to text parsing
    pub async fn detect_outputs() -> Result<Vec<String>> {
        // First try newer niri (JSON by default)
        let json_output = AsyncCommand::new("niri")
            .args(["msg", "outputs"])
            .output()
            .await;

        if let Ok(output) = json_output {
            // Try parsing as JSON first
            if let Ok(outputs) = parse_niri_outputs(&output.stdout) {
                tracing::debug!("Detected Niri outputs (JSON): {:?}", outputs);
                return Ok(outputs);
            }
            // Fall back to text parsing
            if let Ok(outputs) = parse_niri_outputs_text(&output.stdout) {
                tracing::debug!("Detected Niri outputs (text): {:?}", outputs);
                return Ok(outputs);
            }
        }

        // Try older niri with -j flag
        let output = AsyncCommand::new("niri")
            .args(["msg", "outputs", "-j"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DmsAwwwError::NiriIpc(format!(
                "niri msg outputs failed: {}",
                stderr
            )));
        }

        let outputs: Vec<NiriOutput> = serde_json::from_slice(&output.stdout)?;

        let enabled_outputs: Vec<String> = outputs
            .into_iter()
            .filter(|o| o.enabled)
            .map(|o| o.name)
            .collect();

        if enabled_outputs.is_empty() {
            return Err(DmsAwwwError::NoMonitorsDetected);
        }

        tracing::debug!("Detected Niri outputs (legacy -j): {:?}", enabled_outputs);

        Ok(enabled_outputs)
    }

    /// Detect all outputs synchronously (for fallback scenarios)
    pub fn detect_outputs_sync() -> Result<Vec<String>> {
        let output = Command::new("niri")
            .args(["msg", "outputs"])
            .output()?;

        // Try parsing as JSON first
        if let Ok(outputs) = parse_niri_outputs(&output.stdout) {
            return Ok(outputs);
        }
        // Fall back to text parsing
        if let Ok(outputs) = parse_niri_outputs_text(&output.stdout) {
            return Ok(outputs);
        }

        // Try older niri with -j flag
        let output = Command::new("niri")
            .args(["msg", "outputs", "-j"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DmsAwwwError::NiriIpc(format!(
                "niri msg outputs failed: {}",
                stderr
            )));
        }

        let outputs: Vec<NiriOutput> = serde_json::from_slice(&output.stdout)?;

        let enabled_outputs: Vec<String> = outputs
            .into_iter()
            .filter(|o| o.enabled)
            .map(|o| o.name)
            .collect();

        if enabled_outputs.is_empty() {
            return Err(DmsAwwwError::NoMonitorsDetected);
        }

        Ok(enabled_outputs)
    }

    /// Check if Niri is running
    pub async fn is_running() -> bool {
        AsyncCommand::new("niri")
            .args(["msg", "outputs"])
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Check if Niri is running synchronously
    pub fn is_running_sync() -> bool {
        Command::new("niri")
            .args(["msg", "outputs"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get full output information
    pub async fn get_outputs() -> Result<Vec<NiriOutput>> {
        let output = AsyncCommand::new("niri")
            .args(["msg", "outputs"])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DmsAwwwError::NiriIpc(format!(
                "niri msg outputs failed: {}",
                stderr
            )));
        }

        let outputs = serde_json::from_slice(&output.stdout)?;
        Ok(outputs)
    }
}

/// Parse niri outputs from JSON format (newer niri)
fn parse_niri_outputs(stdout: &[u8]) -> Result<Vec<String>> {
    let outputs: Vec<NiriOutput> = serde_json::from_slice(stdout)?;

    let enabled_outputs: Vec<String> = outputs
        .into_iter()
        .filter(|o| o.enabled)
        .map(|o| o.name)
        .collect();

    if enabled_outputs.is_empty() {
        return Err(DmsAwwwError::NoMonitorsDetected);
    }

    Ok(enabled_outputs)
}

/// Parse niri outputs from text format (fallback)
/// Only returns enabled monitors (those with a current mode, not "Disabled")
fn parse_niri_outputs_text(stdout: &[u8]) -> Result<Vec<String>> {
    let text = String::from_utf8_lossy(stdout);
    let mut outputs = Vec::new();
    let mut current_name: Option<String> = None;
    let mut is_disabled = false;

    for line in text.lines() {
        // Look for lines like: Output "Samsung..." (HDMI-A-1)
        if let Some(rest) = line.strip_prefix("Output ") {
            // Save the previous output if it was enabled
            if let Some(ref name) = current_name {
                if !is_disabled {
                    outputs.push(name.clone());
                }
            }

            // Find the output name in parentheses
            if let Some(start) = rest.find('(') {
                if let Some(end) = rest[start..].find(')') {
                    current_name = Some(rest[start + 1..start + end].to_string());
                    is_disabled = false;
                }
            }
        } else if line.trim() == "Disabled" {
            is_disabled = true;
        }
    }

    // Don't forget the last output
    if let Some(ref name) = current_name {
        if !is_disabled {
            outputs.push(name.clone());
        }
    }

    if outputs.is_empty() {
        return Err(DmsAwwwError::NoMonitorsDetected);
    }

    Ok(outputs)
}

/// Helper to get outputs with fallback behavior
pub async fn get_monitor_outputs(
    explicit_outputs: Vec<String>,
    auto_detect: bool,
) -> Result<Vec<String>> {
    // If explicit outputs are provided, use them
    if !explicit_outputs.is_empty() {
        tracing::debug!("Using explicit monitor outputs: {:?}", explicit_outputs);
        return Ok(explicit_outputs);
    }

    // If auto-detect is enabled, try Niri
    if auto_detect {
        match NiriClient::detect_outputs().await {
            Ok(outputs) => {
                tracing::debug!("Auto-detected monitor outputs: {:?}", outputs);
                return Ok(outputs);
            }
            Err(e) => {
                tracing::warn!("Failed to auto-detect monitors via Niri: {}", e);
                // Return a default fallback
                tracing::info!("Falling back to default output: ALL");
                return Ok(vec!["ALL".to_string()]);
            }
        }
    }

    // No outputs available
    Err(DmsAwwwError::NoMonitorsDetected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_niri_output() {
        let json = r#"[{
            "name": "HDMI-A-1",
            "enabled": true,
            "make": "Dell",
            "model": "U2720Q",
            "resolution": {"width": 3840, "height": 2160},
            "position": {"x": 0, "y": 0},
            "refresh_rate": 60.0,
            "physicalSize": {"width": 597, "height": 336}
        }]"#;

        let outputs: Vec<NiriOutput> = serde_json::from_str(json).unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].name, "HDMI-A-1");
        assert!(outputs[0].enabled);
        assert_eq!(outputs[0].make, "Dell");
    }
}

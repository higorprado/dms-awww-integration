//! Niri integration tests

use dms_awww::niri::{NiriOutput, NiriClient};
use dms_awww::error::DmsAwwwError;

#[test]
fn test_parse_niri_output_single() {
    let json = r#"[
        {
            "name": "HDMI-A-1",
            "enabled": true,
            "make": "Dell",
            "model": "U2720Q",
            "resolution": {"width": 3840, "height": 2160},
            "position": {"x": 0, "y": 0},
            "refresh_rate": 60.0,
            "physicalSize": {"width": 597, "height": 336},
            "currentWorkspace": "1"
        }
    ]"#;

    let outputs: Vec<NiriOutput> = serde_json::from_str(json).unwrap();

    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].name, "HDMI-A-1");
    assert!(outputs[0].enabled);
    assert_eq!(outputs[0].make, "Dell");
    assert_eq!(outputs[0].model, "U2720Q");
    assert_eq!(outputs[0].resolution.as_ref().unwrap().width, 3840);
    assert_eq!(outputs[0].resolution.as_ref().unwrap().height, 2160);
    assert_eq!(outputs[0].position.as_ref().unwrap().x, 0);
    assert_eq!(outputs[0].position.as_ref().unwrap().y, 0);
    assert_eq!(outputs[0].refresh_rate, Some(60.0));
    assert_eq!(outputs[0].physical_size.as_ref().unwrap().width, 597);
    assert_eq!(outputs[0].current_workspace, Some("1".to_string()));
}

#[test]
fn test_parse_niri_output_multiple() {
    let json = r#"[
        {
            "name": "HDMI-A-1",
            "enabled": true,
            "make": "Dell",
            "model": "U2720Q"
        },
        {
            "name": "DP-1",
            "enabled": true,
            "make": "LG",
            "model": "27GN950"
        },
        {
            "name": "eDP-1",
            "enabled": false,
            "make": "Lenovo",
            "model": "Thinkpad"
        }
    ]"#;

    let outputs: Vec<NiriOutput> = serde_json::from_str(json).unwrap();

    assert_eq!(outputs.len(), 3);
    assert_eq!(outputs[0].name, "HDMI-A-1");
    assert_eq!(outputs[1].name, "DP-1");
    assert_eq!(outputs[2].name, "eDP-1");
    assert!(outputs[0].enabled);
    assert!(outputs[1].enabled);
    assert!(!outputs[2].enabled);
}

#[test]
fn test_filter_enabled_outputs() {
    let json = r#"[
        {
            "name": "HDMI-A-1",
            "enabled": true
        },
        {
            "name": "DP-1",
            "enabled": false
        },
        {
            "name": "eDP-1",
            "enabled": true
        }
    ]"#;

    let outputs: Vec<NiriOutput> = serde_json::from_str(json).unwrap();
    let enabled: Vec<String> = outputs
        .into_iter()
        .filter(|o| o.enabled)
        .map(|o| o.name)
        .collect();

    assert_eq!(enabled.len(), 2);
    assert!(enabled.contains(&"HDMI-A-1".to_string()));
    assert!(enabled.contains(&"eDP-1".to_string()));
    assert!(!enabled.contains(&"DP-1".to_string()));
}

#[test]
fn test_parse_niri_output_minimal() {
    let json = r#"[
        {
            "name": "HDMI-A-1",
            "enabled": true
        }
    ]"#;

    let outputs: Vec<NiriOutput> = serde_json::from_str(json).unwrap();

    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].name, "HDMI-A-1");
    assert!(outputs[0].enabled);
    assert!(outputs[0].make.is_empty());
    assert!(outputs[0].model.is_empty());
    assert!(outputs[0].resolution.is_none());
    assert!(outputs[0].position.is_none());
    assert!(outputs[0].refresh_rate.is_none());
    assert!(outputs[0].physical_size.is_none());
    assert!(outputs[0].current_workspace.is_none());
}

#[test]
fn test_parse_niri_output_invalid_json() {
    let json = r#"invalid json"#;

    let result: Result<Vec<NiriOutput>, _> = serde_json::from_str(json);

    assert!(result.is_err());
}

#[test]
fn test_parse_niri_output_empty_array() {
    let json = r#"[]"#;

    let outputs: Vec<NiriOutput> = serde_json::from_str(json).unwrap();

    assert!(outputs.is_empty());
}

#[test]
fn test_niri_output_with_all_fields() {
    let json = r#"[
        {
            "name": "DP-1",
            "enabled": true,
            "make": "LG",
            "model": "27GN950",
            "resolution": {"width": 3840, "height": 2160},
            "position": {"x": 3840, "y": 0},
            "refresh_rate": 144.0,
            "physicalSize": {"width": 597, "height": 336},
            "currentWorkspace": "2"
        }
    ]"#;

    let outputs: Vec<NiriOutput> = serde_json::from_str(json).unwrap();

    assert_eq!(outputs.len(), 1);
    let output = &outputs[0];

    assert_eq!(output.name, "DP-1");
    assert!(output.enabled);
    assert_eq!(output.make, "LG");
    assert_eq!(output.model, "27GN950");

    let resolution = output.resolution.as_ref().unwrap();
    assert_eq!(resolution.width, 3840);
    assert_eq!(resolution.height, 2160);

    let position = output.position.as_ref().unwrap();
    assert_eq!(position.x, 3840);
    assert_eq!(position.y, 0);

    assert_eq!(output.refresh_rate, Some(144.0));

    let physical_size = output.physical_size.as_ref().unwrap();
    assert_eq!(physical_size.width, 597);
    assert_eq!(physical_size.height, 336);

    assert_eq!(output.current_workspace, Some("2".to_string()));
}

#[test]
fn test_niri_client_is_running_sync() {
    // In test environment, niri is probably not running
    let is_running = NiriClient::is_running_sync();

    // Just check the function runs without panicking
    // Result depends on whether niri is actually running
    let _ = is_running;
}

#[test]
fn test_niri_error_display() {
    let err = DmsAwwwError::NiriIpc("Test error message".to_string());
    let msg = err.user_message();

    assert!(msg.contains("Niri IPC error"));
    assert!(msg.contains("Test error message"));
}

#[test]
fn test_niri_error_is_critical() {
    let no_monitors = DmsAwwwError::NoMonitorsDetected;
    assert!(no_monitors.is_critical());

    let niri_err = DmsAwwwError::NiriIpc("Connection failed".to_string());
    // Niri errors are recoverable (might try again later)
    assert!(!niri_err.is_critical());
}

#[test]
fn test_niri_error_is_recoverable() {
    let niri_err = DmsAwwwError::NiriIpc("Connection failed".to_string());
    // Niri IPC errors are not in the recoverable list, so return false
    assert!(!niri_err.is_recoverable());

    let no_monitors = DmsAwwwError::NoMonitorsDetected;
    // No monitors is critical, not recoverable
    assert!(!no_monitors.is_recoverable());
}

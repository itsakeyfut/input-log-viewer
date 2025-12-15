//! JSON parser for input log files (.ilj format).
//!
//! This module provides functionality to parse JSON-formatted input log files
//! into the internal `InputLog` structure.

// Allow dead code for Phase 1 - these types will be used in later phases
#![allow(dead_code)]

use serde::Deserialize;
use thiserror::Error;

use super::log::{ButtonState, InputEvent, InputKind, InputLog, InputMapping, LogMetadata};

/// Errors that can occur during input log parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    /// JSON syntax error
    #[error("Invalid JSON syntax: {0}")]
    JsonSyntax(#[from] serde_json::Error),

    /// Missing required field in the JSON structure
    #[error("Missing required field: {field}")]
    MissingField { field: &'static str },

    /// Invalid hex color format
    #[error("Invalid color format '{value}': expected hex color like #RRGGBB")]
    InvalidColor { value: String },

    /// Invalid enum value
    #[error("Invalid {field} value '{value}': expected one of {expected}")]
    InvalidEnumValue {
        field: &'static str,
        value: String,
        expected: &'static str,
    },

    /// Unsupported format version
    #[error("Unsupported format version {version}: expected version 1")]
    UnsupportedVersion { version: u32 },
}

// ============================================================================
// Intermediate JSON structures for deserialization
// ============================================================================

/// Top-level JSON structure for .ilj files.
#[derive(Debug, Deserialize)]
struct JsonInputLog {
    version: u32,
    metadata: JsonMetadata,
    #[serde(default)]
    mappings: Vec<JsonMapping>,
    events: Vec<JsonEvent>,
}

/// Metadata section in JSON format.
#[derive(Debug, Deserialize)]
struct JsonMetadata {
    target_fps: u32,
    frame_count: u64,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    source: Option<String>,
}

/// Input mapping in JSON format.
#[derive(Debug, Deserialize)]
struct JsonMapping {
    id: u32,
    name: String,
    #[serde(default)]
    color: Option<String>,
}

/// Input event in JSON format.
#[derive(Debug, Deserialize)]
struct JsonEvent {
    frame: u64,
    id: u32,
    kind: String,
    #[serde(default)]
    state: Option<String>,
    value: [f32; 2],
}

// ============================================================================
// Parser implementation
// ============================================================================

/// Parse a JSON string into an `InputLog`.
///
/// # Arguments
/// * `content` - The JSON string to parse
///
/// # Returns
/// * `Ok(InputLog)` - Successfully parsed input log
/// * `Err(ParseError)` - Parsing failed with a descriptive error
///
/// # Example
/// ```ignore
/// let json = r#"{"version": 1, "metadata": {...}, "mappings": [...], "events": [...]}"#;
/// let log = parse_json(json)?;
/// ```
pub fn parse_json(content: &str) -> Result<InputLog, ParseError> {
    // Parse JSON into intermediate structure
    let json_log: JsonInputLog = serde_json::from_str(content)?;

    // Validate version
    if json_log.version != 1 {
        return Err(ParseError::UnsupportedVersion {
            version: json_log.version,
        });
    }

    // Convert metadata
    let metadata = LogMetadata {
        version: json_log.version,
        target_fps: json_log.metadata.target_fps,
        frame_count: json_log.metadata.frame_count,
        created_at: json_log.metadata.created_at,
        source: json_log.metadata.source,
    };

    // Convert mappings
    let mappings = json_log
        .mappings
        .into_iter()
        .map(convert_mapping)
        .collect::<Result<Vec<_>, _>>()?;

    // Convert events
    let events = json_log
        .events
        .into_iter()
        .map(convert_event)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(InputLog {
        metadata,
        mappings,
        events,
    })
}

/// Convert a JSON mapping to an `InputMapping`.
fn convert_mapping(json: JsonMapping) -> Result<InputMapping, ParseError> {
    let color = match json.color {
        Some(hex) => Some(parse_hex_color(&hex)?),
        None => None,
    };

    Ok(InputMapping {
        id: json.id,
        name: json.name,
        color,
    })
}

/// Convert a JSON event to an `InputEvent`.
fn convert_event(json: JsonEvent) -> Result<InputEvent, ParseError> {
    let kind = parse_input_kind(&json.kind)?;

    // For Button inputs, state is required; for axis inputs, default to Released
    let state = match kind {
        InputKind::Button => match &json.state {
            Some(s) => parse_button_state(s)?,
            None => {
                return Err(ParseError::MissingField {
                    field: "state (required for Button kind)",
                });
            }
        },
        InputKind::Axis1D | InputKind::Axis2D => match &json.state {
            Some(s) => parse_button_state(s)?,
            None => ButtonState::Released,
        },
    };

    Ok(InputEvent {
        frame: json.frame,
        id: json.id,
        kind,
        state,
        value: json.value,
    })
}

/// Parse a hex color string (e.g., "#FF5555") into RGB bytes.
fn parse_hex_color(hex: &str) -> Result<[u8; 3], ParseError> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Err(ParseError::InvalidColor {
            value: format!("#{}", hex),
        });
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ParseError::InvalidColor {
        value: format!("#{}", hex),
    })?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ParseError::InvalidColor {
        value: format!("#{}", hex),
    })?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ParseError::InvalidColor {
        value: format!("#{}", hex),
    })?;

    Ok([r, g, b])
}

/// Parse an input kind string into an `InputKind` enum.
fn parse_input_kind(s: &str) -> Result<InputKind, ParseError> {
    match s {
        "Button" => Ok(InputKind::Button),
        "Axis1D" => Ok(InputKind::Axis1D),
        "Axis2D" => Ok(InputKind::Axis2D),
        _ => Err(ParseError::InvalidEnumValue {
            field: "kind",
            value: s.to_string(),
            expected: "Button, Axis1D, Axis2D",
        }),
    }
}

/// Parse a button state string into a `ButtonState` enum.
fn parse_button_state(s: &str) -> Result<ButtonState, ParseError> {
    match s {
        "Released" => Ok(ButtonState::Released),
        "Pressed" => Ok(ButtonState::Pressed),
        "Held" => Ok(ButtonState::Held),
        _ => Err(ParseError::InvalidEnumValue {
            field: "state",
            value: s.to_string(),
            expected: "Released, Pressed, Held",
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_valid() {
        assert_eq!(parse_hex_color("#FF5555").unwrap(), [255, 85, 85]);
        assert_eq!(parse_hex_color("#4CAF50").unwrap(), [76, 175, 80]);
        assert_eq!(parse_hex_color("#000000").unwrap(), [0, 0, 0]);
        assert_eq!(parse_hex_color("#FFFFFF").unwrap(), [255, 255, 255]);
        // Without leading #
        assert_eq!(parse_hex_color("2196F3").unwrap(), [33, 150, 243]);
    }

    #[test]
    fn test_parse_hex_color_invalid() {
        assert!(parse_hex_color("#FFF").is_err()); // Too short
        assert!(parse_hex_color("#FFFFFFF").is_err()); // Too long
        assert!(parse_hex_color("#GGGGGG").is_err()); // Invalid hex
    }

    #[test]
    fn test_parse_input_kind() {
        assert_eq!(parse_input_kind("Button").unwrap(), InputKind::Button);
        assert_eq!(parse_input_kind("Axis1D").unwrap(), InputKind::Axis1D);
        assert_eq!(parse_input_kind("Axis2D").unwrap(), InputKind::Axis2D);
        assert!(parse_input_kind("Invalid").is_err());
    }

    #[test]
    fn test_parse_button_state() {
        assert_eq!(
            parse_button_state("Released").unwrap(),
            ButtonState::Released
        );
        assert_eq!(parse_button_state("Pressed").unwrap(), ButtonState::Pressed);
        assert_eq!(parse_button_state("Held").unwrap(), ButtonState::Held);
        assert!(parse_button_state("Invalid").is_err());
    }

    #[test]
    fn test_parse_json_minimal() {
        let json = r#"{
            "version": 1,
            "metadata": {
                "target_fps": 60,
                "frame_count": 100
            },
            "mappings": [],
            "events": []
        }"#;

        let log = parse_json(json).unwrap();
        assert_eq!(log.metadata.version, 1);
        assert_eq!(log.metadata.target_fps, 60);
        assert_eq!(log.metadata.frame_count, 100);
        assert!(log.mappings.is_empty());
        assert!(log.events.is_empty());
    }

    #[test]
    fn test_parse_json_with_mappings() {
        let json = r##"{
            "version": 1,
            "metadata": { "target_fps": 60, "frame_count": 10 },
            "mappings": [
                { "id": 0, "name": "A Button", "color": "#FF0000" },
                { "id": 1, "name": "B Button" }
            ],
            "events": []
        }"##;

        let log = parse_json(json).unwrap();
        assert_eq!(log.mappings.len(), 2);
        assert_eq!(log.mappings[0].id, 0);
        assert_eq!(log.mappings[0].name, "A Button");
        assert_eq!(log.mappings[0].color, Some([255, 0, 0]));
        assert_eq!(log.mappings[1].color, None);
    }

    #[test]
    fn test_parse_json_with_events() {
        let json = r#"{
            "version": 1,
            "metadata": { "target_fps": 60, "frame_count": 10 },
            "mappings": [],
            "events": [
                { "frame": 0, "id": 0, "kind": "Button", "state": "Pressed", "value": [1.0, 0.0] },
                { "frame": 1, "id": 10, "kind": "Axis1D", "value": [0.5, 0.0] },
                { "frame": 2, "id": 20, "kind": "Axis2D", "value": [0.5, 0.5] }
            ]
        }"#;

        let log = parse_json(json).unwrap();
        assert_eq!(log.events.len(), 3);

        assert_eq!(log.events[0].frame, 0);
        assert_eq!(log.events[0].kind, InputKind::Button);
        assert_eq!(log.events[0].state, ButtonState::Pressed);

        assert_eq!(log.events[1].kind, InputKind::Axis1D);
        assert_eq!(log.events[1].value[0], 0.5);

        assert_eq!(log.events[2].kind, InputKind::Axis2D);
    }

    #[test]
    fn test_parse_json_invalid_syntax() {
        let json = "{ invalid json }";
        let result = parse_json(json);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::JsonSyntax(_)));
    }

    #[test]
    fn test_parse_json_unsupported_version() {
        let json = r#"{
            "version": 99,
            "metadata": { "target_fps": 60, "frame_count": 10 },
            "events": []
        }"#;

        let result = parse_json(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::UnsupportedVersion { version: 99 }
        ));
    }

    #[test]
    fn test_parse_json_missing_button_state() {
        let json = r#"{
            "version": 1,
            "metadata": { "target_fps": 60, "frame_count": 10 },
            "events": [
                { "frame": 0, "id": 0, "kind": "Button", "value": [1.0, 0.0] }
            ]
        }"#;

        let result = parse_json(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::MissingField { .. }
        ));
    }

    #[test]
    fn test_parse_json_invalid_kind() {
        let json = r#"{
            "version": 1,
            "metadata": { "target_fps": 60, "frame_count": 10 },
            "events": [
                { "frame": 0, "id": 0, "kind": "InvalidKind", "value": [1.0, 0.0] }
            ]
        }"#;

        let result = parse_json(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::InvalidEnumValue { .. }
        ));
    }

    #[test]
    fn test_parse_json_invalid_color() {
        let json = r#"{
            "version": 1,
            "metadata": { "target_fps": 60, "frame_count": 10 },
            "mappings": [
                { "id": 0, "name": "Test", "color": "invalid" }
            ],
            "events": []
        }"#;

        let result = parse_json(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::InvalidColor { .. }
        ));
    }

    #[test]
    fn test_parse_sample_ilj() {
        // Test parsing the actual sample.ilj file
        let content = include_str!("../../assets/sample.ilj");
        let log = parse_json(content).expect("Failed to parse sample.ilj");

        // Verify metadata
        assert_eq!(log.metadata.version, 1);
        assert_eq!(log.metadata.target_fps, 60);
        assert_eq!(log.metadata.frame_count, 120);
        assert_eq!(
            log.metadata.created_at,
            Some("2025-01-15T10:30:00Z".to_string())
        );
        assert_eq!(log.metadata.source, Some("SampleGame v1.0".to_string()));

        // Verify mappings
        assert_eq!(log.mappings.len(), 7);
        assert_eq!(log.mappings[0].id, 0);
        assert_eq!(log.mappings[0].name, "A Button");
        assert_eq!(log.mappings[0].color, Some([76, 175, 80])); // #4CAF50

        // Verify events
        assert!(!log.events.is_empty());
        // First event is an Axis1D event
        assert_eq!(log.events[0].frame, 0);
        assert_eq!(log.events[0].id, 10);
        assert_eq!(log.events[0].kind, InputKind::Axis1D);

        // Check a button event (frame 5, A button pressed)
        let button_event = log.events.iter().find(|e| e.frame == 5 && e.id == 0);
        assert!(button_event.is_some());
        let button_event = button_event.unwrap();
        assert_eq!(button_event.kind, InputKind::Button);
        assert_eq!(button_event.state, ButtonState::Pressed);
    }
}

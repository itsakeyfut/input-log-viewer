//! Parser for input log files (.ilj and .ilb formats).
//!
//! This module provides functionality to parse both JSON-formatted (.ilj) and
//! binary-formatted (.ilb) input log files into the internal `InputLog` structure.

// Allow dead code for Phase 1 - these types will be used in later phases
#![allow(dead_code)]

use bytemuck::{Pod, Zeroable};
use serde::Deserialize;
use thiserror::Error;

use super::log::{ButtonState, InputEvent, InputKind, InputLog, InputMapping, LogMetadata};

/// Expected magic number for binary files: "ILOG"
const BINARY_MAGIC: [u8; 4] = *b"ILOG";

/// Currently supported binary format version
const BINARY_VERSION: u32 = 1;

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

    /// Invalid magic number in binary file
    #[error("Invalid magic number: expected 'ILOG', found '{found}'")]
    InvalidMagic { found: String },

    /// Binary file too small to contain header
    #[error("Binary file too small: expected at least {expected} bytes, found {found}")]
    FileTooSmall { expected: usize, found: usize },

    /// Binary file has invalid event data
    #[error("Invalid binary event at index {index}: {reason}")]
    InvalidBinaryEvent { index: usize, reason: String },

    /// Event count mismatch between header and actual data
    #[error(
        "Event count mismatch: header declares {header_count} events, but file contains {actual_count}"
    )]
    EventCountMismatch {
        header_count: u64,
        actual_count: usize,
    },
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
// Binary format structures
// ============================================================================

/// Binary file header (32 bytes).
///
/// The header is stored at the beginning of every .ilb file and contains
/// metadata about the log file.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct BinaryHeader {
    /// Magic number identifying the file format: "ILOG"
    pub magic: [u8; 4],
    /// Format version number (currently 1)
    pub version: u32,
    /// Reserved flags (currently unused, must be 0)
    pub flags: u32,
    /// Target frames per second of the original game
    pub target_fps: u32,
    /// Total number of frames in the log
    pub frame_count: u64,
    /// Total number of events in the file
    pub event_count: u64,
}

impl BinaryHeader {
    /// Size of the header in bytes.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Validate the header and return Ok if valid.
    pub fn validate(&self) -> Result<(), ParseError> {
        // Check magic number
        if self.magic != BINARY_MAGIC {
            let found = String::from_utf8_lossy(&self.magic).to_string();
            return Err(ParseError::InvalidMagic { found });
        }

        // Check version
        if self.version != BINARY_VERSION {
            return Err(ParseError::UnsupportedVersion {
                version: self.version,
            });
        }

        Ok(())
    }
}

/// Binary event structure (24 bytes).
///
/// Each event in a binary file is represented by this fixed-size structure.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct BinaryEvent {
    /// Frame number when this event occurred
    pub frame: u64,
    /// Input identifier
    pub id: u32,
    /// Input kind (0=Button, 1=Axis1D, 2=Axis2D)
    pub kind: u8,
    /// Button state (0=Released, 1=Pressed, 2=Held)
    pub state: u8,
    /// Padding for alignment
    pub _padding: [u8; 2],
    /// Input values (1D uses index 0, 2D uses both)
    pub value: [f32; 2],
}

impl BinaryEvent {
    /// Size of a single event in bytes.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Convert the binary event to an `InputEvent`.
    pub fn to_input_event(self, index: usize) -> Result<InputEvent, ParseError> {
        let kind = match self.kind {
            0 => InputKind::Button,
            1 => InputKind::Axis1D,
            2 => InputKind::Axis2D,
            _ => {
                return Err(ParseError::InvalidBinaryEvent {
                    index,
                    reason: format!("invalid kind value: {}", self.kind),
                });
            }
        };

        let state = match self.state {
            0 => ButtonState::Released,
            1 => ButtonState::Pressed,
            2 => ButtonState::Held,
            _ => {
                return Err(ParseError::InvalidBinaryEvent {
                    index,
                    reason: format!("invalid state value: {}", self.state),
                });
            }
        };

        Ok(InputEvent {
            frame: self.frame,
            id: self.id,
            kind,
            state,
            value: self.value,
        })
    }
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

    // Check length and ensure ASCII to prevent panic on UTF-8 multi-byte slicing
    if hex.len() != 6 || !hex.is_ascii() {
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

// ============================================================================
// Binary parser implementation
// ============================================================================

/// Parse a binary byte slice into an `InputLog`.
///
/// # Arguments
/// * `data` - The raw bytes of the .ilb file
///
/// # Returns
/// * `Ok(InputLog)` - Successfully parsed input log
/// * `Err(ParseError)` - Parsing failed with a descriptive error
///
/// # Binary Format
/// The binary format consists of:
/// - A 32-byte header (BinaryHeader)
/// - Followed by N events (BinaryEvent), each 24 bytes
///
/// # Example
/// ```ignore
/// let data = std::fs::read("log.ilb")?;
/// let log = parse_binary(&data)?;
/// ```
pub fn parse_binary(data: &[u8]) -> Result<InputLog, ParseError> {
    // Check minimum size for header
    if data.len() < BinaryHeader::SIZE {
        return Err(ParseError::FileTooSmall {
            expected: BinaryHeader::SIZE,
            found: data.len(),
        });
    }

    // Parse and validate header
    let header: &BinaryHeader = bytemuck::from_bytes(&data[..BinaryHeader::SIZE]);
    header.validate()?;

    // Calculate expected data size
    let event_data_start = BinaryHeader::SIZE;
    let event_data = &data[event_data_start..];
    let actual_event_count = event_data.len() / BinaryEvent::SIZE;

    // Verify event count matches header
    if actual_event_count != header.event_count as usize {
        return Err(ParseError::EventCountMismatch {
            header_count: header.event_count,
            actual_count: actual_event_count,
        });
    }

    // Parse events
    let binary_events: &[BinaryEvent] =
        bytemuck::cast_slice(&event_data[..actual_event_count * BinaryEvent::SIZE]);

    let events = binary_events
        .iter()
        .enumerate()
        .map(|(i, e)| (*e).to_input_event(i))
        .collect::<Result<Vec<_>, _>>()?;

    // Build metadata (binary format doesn't include created_at or source)
    let metadata = LogMetadata {
        version: header.version,
        target_fps: header.target_fps,
        frame_count: header.frame_count,
        created_at: None,
        source: None,
    };

    // Binary format doesn't include mappings - create default mappings from events
    let mappings = generate_default_mappings(&events);

    Ok(InputLog {
        metadata,
        mappings,
        events,
    })
}

/// Generate default mappings from events.
///
/// Since binary format doesn't include mapping information, this function
/// creates default mappings based on the unique input IDs found in events.
fn generate_default_mappings(events: &[InputEvent]) -> Vec<InputMapping> {
    use std::collections::BTreeSet;

    // Collect unique input IDs (BTreeSet keeps them sorted)
    let unique_ids: BTreeSet<u32> = events.iter().map(|e| e.id).collect();

    unique_ids
        .into_iter()
        .map(|id| InputMapping {
            id,
            name: format!("Input {}", id),
            color: None,
        })
        .collect()
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
        assert!(parse_hex_color("你好").is_err()); // Non-ASCII (6 bytes but multi-byte UTF-8)
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

    // ========================================================================
    // Binary parser tests
    // ========================================================================

    /// Helper to create a valid binary header
    fn create_test_header(frame_count: u64, event_count: u64) -> BinaryHeader {
        BinaryHeader {
            magic: *b"ILOG",
            version: 1,
            flags: 0,
            target_fps: 60,
            frame_count,
            event_count,
        }
    }

    /// Helper to create a test binary event
    fn create_test_binary_event(frame: u64, id: u32, kind: u8, state: u8) -> BinaryEvent {
        BinaryEvent {
            frame,
            id,
            kind,
            state,
            _padding: [0, 0],
            value: [1.0, 0.0],
        }
    }

    #[test]
    fn test_binary_header_size() {
        // Header should be exactly 32 bytes as per spec
        assert_eq!(BinaryHeader::SIZE, 32);
    }

    #[test]
    fn test_binary_event_size() {
        // Event should be exactly 24 bytes as per spec
        assert_eq!(BinaryEvent::SIZE, 24);
    }

    #[test]
    fn test_binary_header_validate_valid() {
        let header = create_test_header(100, 5);
        assert!(header.validate().is_ok());
    }

    #[test]
    fn test_binary_header_validate_invalid_magic() {
        let mut header = create_test_header(100, 5);
        header.magic = *b"FAKE";
        let result = header.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::InvalidMagic { .. }
        ));
    }

    #[test]
    fn test_binary_header_validate_invalid_version() {
        let mut header = create_test_header(100, 5);
        header.version = 99;
        let result = header.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::UnsupportedVersion { version: 99 }
        ));
    }

    #[test]
    fn test_binary_event_to_input_event() {
        let event = create_test_binary_event(10, 5, 0, 1); // Button, Pressed
        let input_event = event.to_input_event(0).unwrap();

        assert_eq!(input_event.frame, 10);
        assert_eq!(input_event.id, 5);
        assert_eq!(input_event.kind, InputKind::Button);
        assert_eq!(input_event.state, ButtonState::Pressed);
    }

    #[test]
    fn test_binary_event_invalid_kind() {
        let event = create_test_binary_event(0, 0, 99, 0); // Invalid kind
        let result = event.to_input_event(0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::InvalidBinaryEvent { index: 0, .. }
        ));
    }

    #[test]
    fn test_binary_event_invalid_state() {
        let event = create_test_binary_event(0, 0, 0, 99); // Invalid state
        let result = event.to_input_event(0);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::InvalidBinaryEvent { index: 0, .. }
        ));
    }

    #[test]
    fn test_parse_binary_file_too_small() {
        let data = vec![0u8; 10]; // Too small for header
        let result = parse_binary(&data);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::FileTooSmall {
                expected: 32,
                found: 10
            }
        ));
    }

    #[test]
    fn test_parse_binary_empty_events() {
        let header = create_test_header(100, 0);
        let data: Vec<u8> = bytemuck::bytes_of(&header).to_vec();

        let log = parse_binary(&data).unwrap();
        assert_eq!(log.metadata.version, 1);
        assert_eq!(log.metadata.target_fps, 60);
        assert_eq!(log.metadata.frame_count, 100);
        assert!(log.events.is_empty());
        assert!(log.mappings.is_empty());
    }

    #[test]
    fn test_parse_binary_with_events() {
        let header = create_test_header(100, 2);
        let events = [
            create_test_binary_event(0, 0, 0, 1), // Button, Pressed
            create_test_binary_event(5, 1, 1, 0), // Axis1D, Released
        ];

        let mut data: Vec<u8> = bytemuck::bytes_of(&header).to_vec();
        data.extend_from_slice(bytemuck::cast_slice(&events));

        let log = parse_binary(&data).unwrap();
        assert_eq!(log.events.len(), 2);

        assert_eq!(log.events[0].frame, 0);
        assert_eq!(log.events[0].id, 0);
        assert_eq!(log.events[0].kind, InputKind::Button);
        assert_eq!(log.events[0].state, ButtonState::Pressed);

        assert_eq!(log.events[1].frame, 5);
        assert_eq!(log.events[1].id, 1);
        assert_eq!(log.events[1].kind, InputKind::Axis1D);
    }

    #[test]
    fn test_parse_binary_event_count_mismatch() {
        let header = create_test_header(100, 5); // Claims 5 events
        let events = [create_test_binary_event(0, 0, 0, 0)]; // Only 1 event

        let mut data: Vec<u8> = bytemuck::bytes_of(&header).to_vec();
        data.extend_from_slice(bytemuck::cast_slice(&events));

        let result = parse_binary(&data);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::EventCountMismatch {
                header_count: 5,
                actual_count: 1
            }
        ));
    }

    #[test]
    fn test_parse_binary_generates_default_mappings() {
        let header = create_test_header(100, 3);
        let events = [
            create_test_binary_event(0, 5, 0, 1),
            create_test_binary_event(1, 10, 0, 1),
            create_test_binary_event(2, 5, 0, 0), // Duplicate ID
        ];

        let mut data: Vec<u8> = bytemuck::bytes_of(&header).to_vec();
        data.extend_from_slice(bytemuck::cast_slice(&events));

        let log = parse_binary(&data).unwrap();

        // Should have 2 unique mappings (IDs 5 and 10)
        assert_eq!(log.mappings.len(), 2);

        // Mappings should be sorted by ID
        assert_eq!(log.mappings[0].id, 5);
        assert_eq!(log.mappings[0].name, "Input 5");
        assert_eq!(log.mappings[1].id, 10);
        assert_eq!(log.mappings[1].name, "Input 10");
    }

    #[test]
    fn test_parse_sample_ilb() {
        // Test parsing the actual sample.ilb file
        let data = include_bytes!("../../assets/sample.ilb");
        let log = parse_binary(data).expect("Failed to parse sample.ilb");

        // Verify metadata
        assert_eq!(log.metadata.version, 1);
        assert_eq!(log.metadata.target_fps, 60);
        assert_eq!(log.metadata.frame_count, 120);

        // Binary format doesn't include created_at or source
        assert!(log.metadata.created_at.is_none());
        assert!(log.metadata.source.is_none());

        // Verify events were parsed
        assert!(!log.events.is_empty());

        // Verify mappings were generated
        assert!(!log.mappings.is_empty());
    }
}

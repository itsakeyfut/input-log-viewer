//! Input log data structures.
//!
//! This module defines the core data structures for representing input logs,
//! including events, mappings, and metadata.

// Allow dead code for Phase 1 - these types will be used in later phases
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Input type classification.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputKind {
    /// Button input (on/off state)
    Button = 0,
    /// Single-axis input (e.g., trigger)
    Axis1D = 1,
    /// Dual-axis input (e.g., analog stick)
    Axis2D = 2,
}

/// Button state for button-type inputs.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ButtonState {
    /// Button released or not pressed
    Released = 0,
    /// Button press started this frame
    Pressed = 1,
    /// Button held down (continuing from previous frame)
    Held = 2,
}

/// A single input event at a specific frame.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputEvent {
    /// Frame number when this event occurred
    pub frame: u64,
    /// Input identifier
    pub id: u32,
    /// Type of input
    pub kind: InputKind,
    /// Button state (only valid for Button kind)
    pub state: ButtonState,
    /// Input value (1D uses index 0, 2D uses both)
    pub value: [f32; 2],
}

/// Mapping from input ID to display information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputMapping {
    /// Input identifier
    pub id: u32,
    /// Display name for this input
    pub name: String,
    /// Optional RGB color for visualization
    #[serde(default)]
    pub color: Option<[u8; 3]>,
}

/// Metadata about the input log.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogMetadata {
    /// Format version
    pub version: u32,
    /// Target frames per second of the original game
    pub target_fps: u32,
    /// Total number of frames in the log
    pub frame_count: u64,
    /// Timestamp when the log was created
    #[serde(default)]
    pub created_at: Option<String>,
    /// Source application that generated the log
    #[serde(default)]
    pub source: Option<String>,
}

/// Complete input log containing metadata, mappings, and events.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InputLog {
    /// Log metadata
    pub metadata: LogMetadata,
    /// Input ID to name/color mappings
    pub mappings: Vec<InputMapping>,
    /// All input events sorted by frame
    pub events: Vec<InputEvent>,
}

/// A bookmark marking an important frame.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bookmark {
    /// Frame number of the bookmark
    pub frame: u64,
    /// Optional label for the bookmark
    #[serde(default)]
    pub label: Option<String>,
}

impl Default for LogMetadata {
    fn default() -> Self {
        Self {
            version: 1,
            target_fps: 60,
            frame_count: 0,
            created_at: None,
            source: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_kind_values() {
        assert_eq!(InputKind::Button as u8, 0);
        assert_eq!(InputKind::Axis1D as u8, 1);
        assert_eq!(InputKind::Axis2D as u8, 2);
    }

    #[test]
    fn test_button_state_values() {
        assert_eq!(ButtonState::Released as u8, 0);
        assert_eq!(ButtonState::Pressed as u8, 1);
        assert_eq!(ButtonState::Held as u8, 2);
    }

    #[test]
    fn test_default_metadata() {
        let metadata = LogMetadata::default();
        assert_eq!(metadata.version, 1);
        assert_eq!(metadata.target_fps, 60);
        assert_eq!(metadata.frame_count, 0);
    }

    #[test]
    fn test_default_input_log() {
        let log = InputLog::default();
        assert!(log.mappings.is_empty());
        assert!(log.events.is_empty());
    }
}

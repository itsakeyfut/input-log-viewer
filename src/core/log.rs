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

impl InputLog {
    /// Get the display name for an input ID.
    ///
    /// Returns the mapped name if available, otherwise falls back to "Input #N" format.
    pub fn get_input_name(&self, id: u32) -> String {
        self.mappings
            .iter()
            .find(|m| m.id == id)
            .map(|m| m.name.clone())
            .unwrap_or_else(|| format!("Input #{}", id))
    }

    /// Get the color for an input ID, if one is mapped.
    pub fn get_input_color(&self, id: u32) -> Option<[u8; 3]> {
        self.mappings
            .iter()
            .find(|m| m.id == id)
            .and_then(|m| m.color)
    }

    /// Get effective mappings that include entries for all unique input IDs.
    ///
    /// This returns the existing mappings plus auto-generated mappings for any
    /// input IDs that appear in events but don't have explicit mappings.
    /// The fallback format is "Input #N" with no color.
    pub fn get_effective_mappings(&self) -> Vec<InputMapping> {
        use std::collections::BTreeSet;

        // Collect all unique input IDs from events
        let event_ids: BTreeSet<u32> = self.events.iter().map(|e| e.id).collect();

        // Collect IDs that already have mappings
        let mapped_ids: BTreeSet<u32> = self.mappings.iter().map(|m| m.id).collect();

        // Start with existing mappings
        let mut result = self.mappings.clone();

        // Add fallback mappings for unmapped IDs
        for id in event_ids {
            if !mapped_ids.contains(&id) {
                result.push(InputMapping {
                    id,
                    name: format!("Input #{}", id),
                    color: None,
                });
            }
        }

        // Sort by ID for consistent ordering
        result.sort_by_key(|m| m.id);
        result
    }
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

    #[test]
    fn test_get_input_name_with_mapping() {
        let log = InputLog {
            mappings: vec![
                InputMapping {
                    id: 0,
                    name: "A Button".to_string(),
                    color: Some([255, 0, 0]),
                },
                InputMapping {
                    id: 1,
                    name: "B Button".to_string(),
                    color: None,
                },
            ],
            ..Default::default()
        };

        assert_eq!(log.get_input_name(0), "A Button");
        assert_eq!(log.get_input_name(1), "B Button");
    }

    #[test]
    fn test_get_input_name_fallback() {
        let log = InputLog {
            mappings: vec![InputMapping {
                id: 0,
                name: "A Button".to_string(),
                color: None,
            }],
            ..Default::default()
        };

        // ID 5 has no mapping, should fall back to "Input #5"
        assert_eq!(log.get_input_name(5), "Input #5");
        assert_eq!(log.get_input_name(99), "Input #99");
    }

    #[test]
    fn test_get_input_color_with_mapping() {
        let log = InputLog {
            mappings: vec![
                InputMapping {
                    id: 0,
                    name: "A Button".to_string(),
                    color: Some([255, 0, 0]),
                },
                InputMapping {
                    id: 1,
                    name: "B Button".to_string(),
                    color: None,
                },
            ],
            ..Default::default()
        };

        assert_eq!(log.get_input_color(0), Some([255, 0, 0]));
        assert_eq!(log.get_input_color(1), None);
        assert_eq!(log.get_input_color(99), None);
    }

    #[test]
    fn test_get_effective_mappings_no_unmapped() {
        let log = InputLog {
            mappings: vec![
                InputMapping {
                    id: 0,
                    name: "A Button".to_string(),
                    color: Some([255, 0, 0]),
                },
                InputMapping {
                    id: 1,
                    name: "B Button".to_string(),
                    color: None,
                },
            ],
            events: vec![
                InputEvent {
                    frame: 0,
                    id: 0,
                    kind: InputKind::Button,
                    state: ButtonState::Pressed,
                    value: [1.0, 0.0],
                },
                InputEvent {
                    frame: 1,
                    id: 1,
                    kind: InputKind::Button,
                    state: ButtonState::Pressed,
                    value: [1.0, 0.0],
                },
            ],
            ..Default::default()
        };

        let effective = log.get_effective_mappings();
        assert_eq!(effective.len(), 2);
        assert_eq!(effective[0].id, 0);
        assert_eq!(effective[0].name, "A Button");
        assert_eq!(effective[1].id, 1);
        assert_eq!(effective[1].name, "B Button");
    }

    #[test]
    fn test_get_effective_mappings_with_unmapped() {
        let log = InputLog {
            mappings: vec![InputMapping {
                id: 0,
                name: "A Button".to_string(),
                color: Some([255, 0, 0]),
            }],
            events: vec![
                InputEvent {
                    frame: 0,
                    id: 0,
                    kind: InputKind::Button,
                    state: ButtonState::Pressed,
                    value: [1.0, 0.0],
                },
                InputEvent {
                    frame: 1,
                    id: 5,
                    kind: InputKind::Button,
                    state: ButtonState::Pressed,
                    value: [1.0, 0.0],
                },
                InputEvent {
                    frame: 2,
                    id: 10,
                    kind: InputKind::Axis1D,
                    state: ButtonState::Released,
                    value: [0.5, 0.0],
                },
            ],
            ..Default::default()
        };

        let effective = log.get_effective_mappings();
        assert_eq!(effective.len(), 3);

        // Should be sorted by ID
        assert_eq!(effective[0].id, 0);
        assert_eq!(effective[0].name, "A Button");
        assert_eq!(effective[0].color, Some([255, 0, 0]));

        assert_eq!(effective[1].id, 5);
        assert_eq!(effective[1].name, "Input #5");
        assert_eq!(effective[1].color, None);

        assert_eq!(effective[2].id, 10);
        assert_eq!(effective[2].name, "Input #10");
        assert_eq!(effective[2].color, None);
    }

    #[test]
    fn test_get_effective_mappings_empty() {
        let log = InputLog::default();
        let effective = log.get_effective_mappings();
        assert!(effective.is_empty());
    }
}

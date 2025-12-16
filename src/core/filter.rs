//! Filter state for input visibility.
//!
//! This module defines the filter state used to control which inputs
//! are displayed on the timeline.

use std::collections::HashSet;

use super::log::{InputKind, InputLog, InputMapping};

/// Filter state for controlling input visibility on the timeline.
#[derive(Debug, Clone)]
pub struct FilterState {
    /// Set of visible input IDs. Empty means all inputs are visible.
    pub visible_ids: HashSet<u32>,
    /// Whether to show button-type inputs
    pub show_button: bool,
    /// Whether to show 1D axis inputs
    pub show_axis1d: bool,
    /// Whether to show 2D axis inputs
    pub show_axis2d: bool,
    /// Whether the filter has been initialized with input IDs from a log
    initialized: bool,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            visible_ids: HashSet::new(),
            show_button: true,
            show_axis1d: true,
            show_axis2d: true,
            initialized: false,
        }
    }
}

impl FilterState {
    /// Create a new filter state with default settings (all visible).
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize the filter with all input IDs from the given log.
    /// This should be called when a new log file is loaded.
    pub fn initialize_from_log(&mut self, log: &InputLog) {
        self.visible_ids.clear();
        for mapping in &log.mappings {
            self.visible_ids.insert(mapping.id);
        }
        self.initialized = true;
    }

    /// Reset the filter to show all inputs from the log.
    #[allow(dead_code)] // Will be used for filter reset functionality
    pub fn reset(&mut self, log: &InputLog) {
        self.initialize_from_log(log);
        self.show_button = true;
        self.show_axis1d = true;
        self.show_axis2d = true;
    }

    /// Check if an input ID is visible based on the current filter settings.
    pub fn is_visible(&self, id: u32, kind: InputKind) -> bool {
        // Check type filter first
        let type_visible = match kind {
            InputKind::Button => self.show_button,
            InputKind::Axis1D => self.show_axis1d,
            InputKind::Axis2D => self.show_axis2d,
        };

        if !type_visible {
            return false;
        }

        // If not initialized or empty, show all
        if !self.initialized || self.visible_ids.is_empty() {
            return true;
        }

        // Check specific ID visibility
        self.visible_ids.contains(&id)
    }

    /// Toggle visibility of a specific input ID.
    #[allow(dead_code)] // Alternative to set_id_visible
    pub fn toggle_id(&mut self, id: u32) {
        if self.visible_ids.contains(&id) {
            self.visible_ids.remove(&id);
        } else {
            self.visible_ids.insert(id);
        }
    }

    /// Set visibility of a specific input ID.
    pub fn set_id_visible(&mut self, id: u32, visible: bool) {
        if visible {
            self.visible_ids.insert(id);
        } else {
            self.visible_ids.remove(&id);
        }
    }

    /// Select all input IDs from the given mappings.
    pub fn select_all(&mut self, mappings: &[InputMapping]) {
        for mapping in mappings {
            self.visible_ids.insert(mapping.id);
        }
    }

    /// Deselect all input IDs (clear the visible set).
    pub fn deselect_all(&mut self) {
        self.visible_ids.clear();
    }

    /// Check if all IDs from the given mappings are selected.
    #[allow(dead_code)] // For future UI state display
    pub fn all_selected(&self, mappings: &[InputMapping]) -> bool {
        mappings.iter().all(|m| self.visible_ids.contains(&m.id))
    }

    /// Check if no IDs are selected.
    #[allow(dead_code)] // For future UI state display
    pub fn none_selected(&self) -> bool {
        self.visible_ids.is_empty()
    }

    /// Get the visible mappings from a log based on current filter.
    /// Returns mappings in their original order, filtered by visibility.
    #[allow(dead_code)] // Alternative way to get filtered mappings
    pub fn get_visible_mappings<'a>(&self, log: &'a InputLog) -> Vec<&'a InputMapping> {
        log.mappings
            .iter()
            .filter(|m| {
                // Determine the input kind for this mapping by looking at events
                let kind = log
                    .events
                    .iter()
                    .find(|e| e.id == m.id)
                    .map(|e| e.kind)
                    .unwrap_or(InputKind::Button);
                self.is_visible(m.id, kind)
            })
            .collect()
    }

    /// Check if the filter has been initialized.
    #[allow(dead_code)] // For checking filter state
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Count how many input types are currently enabled.
    pub fn enabled_type_count(&self) -> u32 {
        let mut count = 0;
        if self.show_button {
            count += 1;
        }
        if self.show_axis1d {
            count += 1;
        }
        if self.show_axis2d {
            count += 1;
        }
        count
    }

    /// Check if a specific input type can be disabled.
    /// Returns false if this is the last enabled type (to prevent empty view).
    pub fn can_disable_type(&self, kind: InputKind) -> bool {
        let is_currently_enabled = match kind {
            InputKind::Button => self.show_button,
            InputKind::Axis1D => self.show_axis1d,
            InputKind::Axis2D => self.show_axis2d,
        };

        // Can always enable, but can only disable if at least one other type remains
        !is_currently_enabled || self.enabled_type_count() > 1
    }

    /// Set the visibility of a specific input type.
    /// Returns true if the change was applied, false if it would result in no types enabled.
    #[allow(dead_code)] // Utility method for programmatic type filtering
    pub fn set_type_visible(&mut self, kind: InputKind, visible: bool) -> bool {
        // Always allow enabling
        if visible {
            match kind {
                InputKind::Button => self.show_button = true,
                InputKind::Axis1D => self.show_axis1d = true,
                InputKind::Axis2D => self.show_axis2d = true,
            }
            return true;
        }

        // Check if we can disable this type
        if !self.can_disable_type(kind) {
            return false;
        }

        match kind {
            InputKind::Button => self.show_button = false,
            InputKind::Axis1D => self.show_axis1d = false,
            InputKind::Axis2D => self.show_axis2d = false,
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::log::{InputEvent, LogMetadata};

    fn create_test_log() -> InputLog {
        InputLog {
            metadata: LogMetadata::default(),
            mappings: vec![
                InputMapping {
                    id: 0,
                    name: "A Button".to_string(),
                    color: None,
                },
                InputMapping {
                    id: 1,
                    name: "B Button".to_string(),
                    color: None,
                },
                InputMapping {
                    id: 10,
                    name: "Left Stick X".to_string(),
                    color: None,
                },
            ],
            events: vec![
                InputEvent {
                    frame: 0,
                    id: 0,
                    kind: InputKind::Button,
                    state: crate::core::log::ButtonState::Pressed,
                    value: [1.0, 0.0],
                },
                InputEvent {
                    frame: 0,
                    id: 10,
                    kind: InputKind::Axis1D,
                    state: crate::core::log::ButtonState::Released,
                    value: [0.5, 0.0],
                },
            ],
        }
    }

    #[test]
    fn test_default_filter_state() {
        let filter = FilterState::default();
        assert!(filter.show_button);
        assert!(filter.show_axis1d);
        assert!(filter.show_axis2d);
        assert!(filter.visible_ids.is_empty());
        assert!(!filter.is_initialized());
    }

    #[test]
    fn test_initialize_from_log() {
        let log = create_test_log();
        let mut filter = FilterState::new();
        filter.initialize_from_log(&log);

        assert!(filter.is_initialized());
        assert!(filter.visible_ids.contains(&0));
        assert!(filter.visible_ids.contains(&1));
        assert!(filter.visible_ids.contains(&10));
    }

    #[test]
    fn test_toggle_id() {
        let log = create_test_log();
        let mut filter = FilterState::new();
        filter.initialize_from_log(&log);

        // Initially visible
        assert!(filter.visible_ids.contains(&0));

        // Toggle off
        filter.toggle_id(0);
        assert!(!filter.visible_ids.contains(&0));

        // Toggle on
        filter.toggle_id(0);
        assert!(filter.visible_ids.contains(&0));
    }

    #[test]
    fn test_select_deselect_all() {
        let log = create_test_log();
        let mut filter = FilterState::new();
        filter.initialize_from_log(&log);

        // Deselect all
        filter.deselect_all();
        assert!(filter.none_selected());

        // Select all
        filter.select_all(&log.mappings);
        assert!(filter.all_selected(&log.mappings));
    }

    #[test]
    fn test_is_visible() {
        let log = create_test_log();
        let mut filter = FilterState::new();
        filter.initialize_from_log(&log);

        // All visible by default
        assert!(filter.is_visible(0, InputKind::Button));
        assert!(filter.is_visible(10, InputKind::Axis1D));

        // Remove one
        filter.set_id_visible(0, false);
        assert!(!filter.is_visible(0, InputKind::Button));
        assert!(filter.is_visible(10, InputKind::Axis1D));

        // Disable type filter
        filter.show_axis1d = false;
        assert!(!filter.is_visible(10, InputKind::Axis1D));
    }

    #[test]
    fn test_type_filter() {
        let log = create_test_log();
        let mut filter = FilterState::new();
        filter.initialize_from_log(&log);

        filter.show_button = false;
        assert!(!filter.is_visible(0, InputKind::Button));
        assert!(filter.is_visible(10, InputKind::Axis1D));

        filter.show_button = true;
        filter.show_axis1d = false;
        assert!(filter.is_visible(0, InputKind::Button));
        assert!(!filter.is_visible(10, InputKind::Axis1D));
    }

    #[test]
    fn test_enabled_type_count() {
        let mut filter = FilterState::new();

        // All enabled by default
        assert_eq!(filter.enabled_type_count(), 3);

        // Disable one
        filter.show_button = false;
        assert_eq!(filter.enabled_type_count(), 2);

        // Disable another
        filter.show_axis1d = false;
        assert_eq!(filter.enabled_type_count(), 1);

        // Disable all (not recommended, but testing the count)
        filter.show_axis2d = false;
        assert_eq!(filter.enabled_type_count(), 0);
    }

    #[test]
    fn test_can_disable_type() {
        let mut filter = FilterState::new();

        // All enabled - can disable any
        assert!(filter.can_disable_type(InputKind::Button));
        assert!(filter.can_disable_type(InputKind::Axis1D));
        assert!(filter.can_disable_type(InputKind::Axis2D));

        // Disable two types - cannot disable the last one
        filter.show_button = false;
        filter.show_axis1d = false;
        assert!(!filter.can_disable_type(InputKind::Axis2D));

        // Can still enable (can_disable returns true for disabled types)
        assert!(filter.can_disable_type(InputKind::Button)); // Already disabled
        assert!(filter.can_disable_type(InputKind::Axis1D)); // Already disabled
    }

    #[test]
    fn test_set_type_visible() {
        let mut filter = FilterState::new();

        // Disable Button type
        assert!(filter.set_type_visible(InputKind::Button, false));
        assert!(!filter.show_button);

        // Disable Axis1D type
        assert!(filter.set_type_visible(InputKind::Axis1D, false));
        assert!(!filter.show_axis1d);

        // Cannot disable last type (Axis2D)
        assert!(!filter.set_type_visible(InputKind::Axis2D, false));
        assert!(filter.show_axis2d); // Still enabled

        // Can enable any type
        assert!(filter.set_type_visible(InputKind::Button, true));
        assert!(filter.show_button);
    }
}

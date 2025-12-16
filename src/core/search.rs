//! Search functionality for finding frames with specific input events.
//!
//! This module provides the ability to search through input logs
//! to find frames that match specific criteria.

use super::log::{ButtonState, InputEvent, InputKind, InputLog};

/// Search query criteria for finding matching frames.
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    /// Filter by input ID (None matches any input)
    pub input_id: Option<u32>,
    /// Filter by input kind (None matches any kind)
    pub kind: Option<InputKind>,
    /// Filter by button state (None matches any state, only applies to Button kind)
    pub button_state: Option<ButtonState>,
}

impl SearchQuery {
    /// Create a new empty search query that matches all events.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a query to search for a specific input ID.
    pub fn with_input_id(input_id: u32) -> Self {
        Self {
            input_id: Some(input_id),
            ..Default::default()
        }
    }

    /// Set the input ID filter.
    #[allow(dead_code)]
    pub fn input_id(mut self, id: u32) -> Self {
        self.input_id = Some(id);
        self
    }

    /// Set the input kind filter.
    #[allow(dead_code)]
    pub fn kind(mut self, kind: InputKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Set the button state filter (only applies to Button kind).
    pub fn button_state(mut self, state: ButtonState) -> Self {
        self.button_state = Some(state);
        self
    }

    /// Check if the query has any criteria set.
    pub fn is_empty(&self) -> bool {
        self.input_id.is_none() && self.kind.is_none() && self.button_state.is_none()
    }

    /// Check if an event matches this query.
    pub fn matches(&self, event: &InputEvent) -> bool {
        // Check input ID filter
        if let Some(id) = self.input_id
            && event.id != id
        {
            return false;
        }

        // Check input kind filter
        if let Some(kind) = self.kind
            && event.kind != kind
        {
            return false;
        }

        // Check button state filter (only for Button kind events)
        if let Some(state) = self.button_state {
            // Only apply state filter to Button type events
            if event.kind == InputKind::Button && event.state != state {
                return false;
            }
        }

        true
    }
}

/// Search results containing matching frame numbers.
#[derive(Debug, Clone, Default)]
pub struct SearchResult {
    /// List of frame numbers that match the query (sorted ascending)
    pub matches: Vec<u64>,
    /// Current position in the matches list (None if no results or not navigated)
    pub current_index: Option<usize>,
}

impl SearchResult {
    /// Create a new empty search result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create search results from a list of matching frames.
    pub fn from_matches(matches: Vec<u64>) -> Self {
        let current_index = if matches.is_empty() { None } else { Some(0) };
        Self {
            matches,
            current_index,
        }
    }

    /// Get the total number of matches.
    pub fn count(&self) -> usize {
        self.matches.len()
    }

    /// Check if there are any matches.
    pub fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }

    /// Get the current match frame number.
    pub fn current_frame(&self) -> Option<u64> {
        self.current_index
            .and_then(|i| self.matches.get(i).copied())
    }

    /// Get the current 1-based position for display (e.g., "3 of 10").
    pub fn current_position(&self) -> Option<usize> {
        self.current_index.map(|i| i + 1)
    }

    /// Move to the next match and return the frame number.
    pub fn next(&mut self) -> Option<u64> {
        if self.matches.is_empty() {
            return None;
        }

        let new_index = match self.current_index {
            Some(i) => (i + 1) % self.matches.len(),
            None => 0,
        };
        self.current_index = Some(new_index);
        self.matches.get(new_index).copied()
    }

    /// Move to the previous match and return the frame number.
    pub fn prev(&mut self) -> Option<u64> {
        if self.matches.is_empty() {
            return None;
        }

        let new_index = match self.current_index {
            Some(0) => self.matches.len() - 1,
            Some(i) => i - 1,
            None => self.matches.len() - 1,
        };
        self.current_index = Some(new_index);
        self.matches.get(new_index).copied()
    }

    /// Find and set the closest match to the given frame.
    /// Useful when starting a search from the current playback position.
    pub fn set_closest_to_frame(&mut self, frame: u64) {
        if self.matches.is_empty() {
            self.current_index = None;
            return;
        }

        // Find the first match >= frame, or wrap to the first if none found
        let index = self.matches.iter().position(|&f| f >= frame).unwrap_or(0);

        self.current_index = Some(index);
    }

    /// Check if a frame is in the matches list.
    #[allow(dead_code)]
    pub fn contains_frame(&self, frame: u64) -> bool {
        self.matches.binary_search(&frame).is_ok()
    }
}

/// Find all frames that match the given search query.
///
/// Returns a list of unique frame numbers sorted in ascending order
/// where at least one event matches the query criteria.
pub fn find_matches(log: &InputLog, query: &SearchQuery) -> Vec<u64> {
    // If query is empty, return empty results (don't match everything)
    if query.is_empty() {
        return Vec::new();
    }

    // Collect matching frame numbers
    let mut frames: Vec<u64> = log
        .events
        .iter()
        .filter(|event| query.matches(event))
        .map(|event| event.frame)
        .collect();

    // Remove duplicates (multiple events in same frame) and sort
    frames.sort_unstable();
    frames.dedup();

    frames
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::log::{InputEvent, InputMapping, LogMetadata};

    fn create_test_log() -> InputLog {
        InputLog {
            metadata: LogMetadata {
                version: 1,
                target_fps: 60,
                frame_count: 100,
                created_at: None,
                source: None,
            },
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
                    state: ButtonState::Pressed,
                    value: [1.0, 0.0],
                },
                InputEvent {
                    frame: 1,
                    id: 0,
                    kind: InputKind::Button,
                    state: ButtonState::Held,
                    value: [1.0, 0.0],
                },
                InputEvent {
                    frame: 5,
                    id: 0,
                    kind: InputKind::Button,
                    state: ButtonState::Released,
                    value: [0.0, 0.0],
                },
                InputEvent {
                    frame: 10,
                    id: 1,
                    kind: InputKind::Button,
                    state: ButtonState::Pressed,
                    value: [1.0, 0.0],
                },
                InputEvent {
                    frame: 15,
                    id: 10,
                    kind: InputKind::Axis1D,
                    state: ButtonState::Released,
                    value: [0.75, 0.0],
                },
                InputEvent {
                    frame: 20,
                    id: 0,
                    kind: InputKind::Button,
                    state: ButtonState::Pressed,
                    value: [1.0, 0.0],
                },
            ],
        }
    }

    #[test]
    fn test_empty_query() {
        let log = create_test_log();
        let query = SearchQuery::new();
        assert!(query.is_empty());

        let results = find_matches(&log, &query);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_by_input_id() {
        let log = create_test_log();
        let query = SearchQuery::with_input_id(0);
        let results = find_matches(&log, &query);

        assert_eq!(results, vec![0, 1, 5, 20]);
    }

    #[test]
    fn test_search_by_button_state() {
        let log = create_test_log();
        let query = SearchQuery::new()
            .input_id(0)
            .button_state(ButtonState::Pressed);
        let results = find_matches(&log, &query);

        assert_eq!(results, vec![0, 20]);
    }

    #[test]
    fn test_search_by_kind() {
        let log = create_test_log();
        let query = SearchQuery::new().kind(InputKind::Axis1D);
        let results = find_matches(&log, &query);

        assert_eq!(results, vec![15]);
    }

    #[test]
    fn test_search_result_navigation() {
        let mut result = SearchResult::from_matches(vec![0, 5, 10, 20]);

        assert_eq!(result.count(), 4);
        assert_eq!(result.current_frame(), Some(0));
        assert_eq!(result.current_position(), Some(1));

        // Navigate forward
        assert_eq!(result.next(), Some(5));
        assert_eq!(result.current_position(), Some(2));
        assert_eq!(result.next(), Some(10));
        assert_eq!(result.next(), Some(20));

        // Wrap around
        assert_eq!(result.next(), Some(0));
        assert_eq!(result.current_position(), Some(1));

        // Navigate backward
        assert_eq!(result.prev(), Some(20));
        assert_eq!(result.prev(), Some(10));
    }

    #[test]
    fn test_search_result_closest() {
        let mut result = SearchResult::from_matches(vec![0, 5, 10, 20, 30]);

        result.set_closest_to_frame(7);
        assert_eq!(result.current_frame(), Some(10));

        result.set_closest_to_frame(0);
        assert_eq!(result.current_frame(), Some(0));

        result.set_closest_to_frame(100);
        // Should wrap to first when no match >= frame
        assert_eq!(result.current_frame(), Some(0));
    }

    #[test]
    fn test_search_result_contains() {
        let result = SearchResult::from_matches(vec![0, 5, 10, 20]);

        assert!(result.contains_frame(5));
        assert!(result.contains_frame(20));
        assert!(!result.contains_frame(7));
        assert!(!result.contains_frame(100));
    }

    #[test]
    fn test_query_builder() {
        let query = SearchQuery::new()
            .input_id(5)
            .kind(InputKind::Button)
            .button_state(ButtonState::Pressed);

        assert_eq!(query.input_id, Some(5));
        assert_eq!(query.kind, Some(InputKind::Button));
        assert_eq!(query.button_state, Some(ButtonState::Pressed));
    }
}

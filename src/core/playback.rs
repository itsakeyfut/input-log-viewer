//! Playback state management for controlling frame-by-frame playback.
//!
//! This module provides the PlaybackState structure that manages current frame position,
//! playback speed, loop settings, and timing logic for advancing frames.

use std::time::{Duration, Instant};

/// Default playback speed (1.0 = normal speed).
pub const DEFAULT_SPEED: f32 = 1.0;

/// Minimum allowed playback speed.
pub const MIN_SPEED: f32 = 0.1;

/// Maximum allowed playback speed.
pub const MAX_SPEED: f32 = 10.0;

/// Available speed presets for the UI.
pub const SPEED_OPTIONS: &[f32] = &[0.25, 0.5, 1.0, 2.0, 4.0];

/// Default target FPS when metadata is not available.
#[allow(dead_code)] // Will be used when implementing playback controls
pub const DEFAULT_TARGET_FPS: u32 = 60;

/// Manages playback state including frame position, speed, and timing.
#[derive(Debug, Clone)]
pub struct PlaybackState {
    /// Current frame number (0-indexed).
    pub current_frame: u64,

    /// Playback speed multiplier (1.0 = normal speed).
    /// Higher values = faster playback.
    pub speed: f32,

    /// Whether to loop back to the start when reaching the end.
    #[allow(dead_code)] // Will be used when implementing playback loop controls
    pub loop_enabled: bool,

    /// Optional start frame for range playback.
    /// If None, playback starts from frame 0.
    #[allow(dead_code)] // Will be used when implementing range playback
    pub range_start: Option<u64>,

    /// Optional end frame for range playback.
    /// If None, playback continues to the last frame.
    #[allow(dead_code)] // Will be used when implementing range playback
    pub range_end: Option<u64>,

    /// Last time the frame was advanced (for timing control).
    last_update: Instant,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaybackState {
    /// Create a new PlaybackState with default values.
    pub fn new() -> Self {
        Self {
            current_frame: 0,
            speed: DEFAULT_SPEED,
            loop_enabled: true,
            range_start: None,
            range_end: None,
            last_update: Instant::now(),
        }
    }

    /// Check if enough time has elapsed to advance to the next frame.
    ///
    /// This method determines whether the playback should move to the next frame
    /// based on the target FPS and current speed multiplier.
    ///
    /// # Arguments
    /// * `target_fps` - The target frames per second for playback.
    ///
    /// # Returns
    /// `true` if enough time has passed to advance the frame, `false` otherwise.
    pub fn should_advance(&self, target_fps: u32) -> bool {
        if target_fps == 0 || self.speed <= 0.0 {
            return false;
        }

        let frame_duration = Duration::from_secs_f32(1.0 / (target_fps as f32 * self.speed));
        self.last_update.elapsed() >= frame_duration
    }

    /// Mark that a frame advance has occurred, updating the timestamp.
    pub fn mark_advanced(&mut self) {
        self.last_update = Instant::now();
    }

    /// Get the effective start frame (accounting for range_start).
    #[allow(dead_code)] // Will be used when implementing range playback
    pub fn effective_start(&self) -> u64 {
        self.range_start.unwrap_or(0)
    }

    /// Get the effective end frame (accounting for range_end).
    ///
    /// # Arguments
    /// * `total_frames` - The total number of frames in the log.
    #[allow(dead_code)] // Will be used when implementing range playback
    pub fn effective_end(&self, total_frames: u64) -> u64 {
        self.range_end
            .map(|end| end.min(total_frames.saturating_sub(1)))
            .unwrap_or(total_frames.saturating_sub(1))
    }

    /// Set the current frame, clamping to valid range.
    ///
    /// # Arguments
    /// * `frame` - The desired frame number.
    /// * `total_frames` - The total number of frames in the log.
    pub fn set_frame(&mut self, frame: u64, total_frames: u64) {
        let max_frame = total_frames.saturating_sub(1);
        self.current_frame = frame.min(max_frame);
    }

    /// Advance to the next frame.
    ///
    /// # Arguments
    /// * `total_frames` - The total number of frames in the log.
    ///
    /// # Returns
    /// `true` if playback should continue, `false` if end was reached and loop is disabled.
    pub fn advance(&mut self, total_frames: u64) -> bool {
        let end_frame = self.effective_end(total_frames);

        if self.current_frame >= end_frame {
            if self.loop_enabled {
                self.current_frame = self.effective_start();
                self.mark_advanced();
                true
            } else {
                false
            }
        } else {
            self.current_frame += 1;
            self.mark_advanced();
            true
        }
    }

    /// Go to the previous frame.
    ///
    /// # Arguments
    /// * `total_frames` - The total number of frames in the log.
    pub fn previous(&mut self, total_frames: u64) {
        let start_frame = self.effective_start();

        if self.current_frame <= start_frame {
            if self.loop_enabled {
                self.current_frame = self.effective_end(total_frames);
            }
        } else {
            self.current_frame -= 1;
        }
    }

    /// Go to the start frame.
    pub fn go_to_start(&mut self) {
        self.current_frame = self.effective_start();
    }

    /// Go to the end frame.
    ///
    /// # Arguments
    /// * `total_frames` - The total number of frames in the log.
    pub fn go_to_end(&mut self, total_frames: u64) {
        self.current_frame = self.effective_end(total_frames);
    }

    /// Set playback speed, clamping to valid range.
    ///
    /// # Arguments
    /// * `speed` - The desired playback speed multiplier.
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.clamp(MIN_SPEED, MAX_SPEED);
    }

    /// Get the next speed preset from SPEED_OPTIONS.
    ///
    /// Returns the next higher speed preset, or the maximum if already at max.
    pub fn next_speed_preset(&self) -> f32 {
        for &preset in SPEED_OPTIONS {
            if preset > self.speed {
                return preset;
            }
        }
        // Already at or above max preset, return the last one
        *SPEED_OPTIONS.last().unwrap_or(&DEFAULT_SPEED)
    }

    /// Get the previous speed preset from SPEED_OPTIONS.
    ///
    /// Returns the next lower speed preset, or the minimum if already at min.
    pub fn prev_speed_preset(&self) -> f32 {
        for &preset in SPEED_OPTIONS.iter().rev() {
            if preset < self.speed {
                return preset;
            }
        }
        // Already at or below min preset, return the first one
        *SPEED_OPTIONS.first().unwrap_or(&DEFAULT_SPEED)
    }

    /// Set the playback range.
    ///
    /// # Arguments
    /// * `start` - Optional start frame for range playback.
    /// * `end` - Optional end frame for range playback.
    #[allow(dead_code)] // Will be used when implementing range playback
    pub fn set_range(&mut self, start: Option<u64>, end: Option<u64>) {
        self.range_start = start;
        self.range_end = end;
    }

    /// Clear the playback range (use full timeline).
    #[allow(dead_code)] // Will be used when implementing range playback
    pub fn clear_range(&mut self) {
        self.range_start = None;
        self.range_end = None;
    }

    /// Reset the timing for frame advance (call when starting playback).
    pub fn reset_timing(&mut self) {
        self.last_update = Instant::now();
    }

    /// Check if the current frame is at the start boundary.
    ///
    /// Returns `true` if we're at the effective start and cannot go further back
    /// (either loop is disabled, or we're implementing boundary-aware UI).
    pub fn is_at_start(&self) -> bool {
        self.current_frame <= self.effective_start()
    }

    /// Check if the current frame is at the end boundary.
    ///
    /// # Arguments
    /// * `total_frames` - The total number of frames in the log.
    ///
    /// Returns `true` if we're at the effective end and cannot go further forward
    /// (either loop is disabled, or we're implementing boundary-aware UI).
    pub fn is_at_end(&self, total_frames: u64) -> bool {
        self.current_frame >= self.effective_end(total_frames)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_new_default_values() {
        let state = PlaybackState::new();
        assert_eq!(state.current_frame, 0);
        assert_eq!(state.speed, 1.0);
        assert!(state.loop_enabled);
        assert!(state.range_start.is_none());
        assert!(state.range_end.is_none());
    }

    #[test]
    fn test_set_frame_clamps_to_valid_range() {
        let mut state = PlaybackState::new();

        state.set_frame(50, 100);
        assert_eq!(state.current_frame, 50);

        state.set_frame(150, 100);
        assert_eq!(state.current_frame, 99); // max is total - 1

        state.set_frame(0, 0);
        assert_eq!(state.current_frame, 0); // handles empty log gracefully
    }

    #[test]
    fn test_advance_increments_frame() {
        let mut state = PlaybackState::new();
        state.current_frame = 5;

        let continued = state.advance(100);
        assert!(continued);
        assert_eq!(state.current_frame, 6);
    }

    #[test]
    fn test_advance_loops_when_enabled() {
        let mut state = PlaybackState::new();
        state.current_frame = 99;
        state.loop_enabled = true;

        let continued = state.advance(100);
        assert!(continued);
        assert_eq!(state.current_frame, 0);
    }

    #[test]
    fn test_advance_stops_when_loop_disabled() {
        let mut state = PlaybackState::new();
        state.current_frame = 99;
        state.loop_enabled = false;

        let continued = state.advance(100);
        assert!(!continued);
        assert_eq!(state.current_frame, 99);
    }

    #[test]
    fn test_previous_decrements_frame() {
        let mut state = PlaybackState::new();
        state.current_frame = 10;

        state.previous(100);
        assert_eq!(state.current_frame, 9);
    }

    #[test]
    fn test_previous_loops_when_enabled() {
        let mut state = PlaybackState::new();
        state.current_frame = 0;
        state.loop_enabled = true;

        state.previous(100);
        assert_eq!(state.current_frame, 99);
    }

    #[test]
    fn test_previous_stays_at_start_when_loop_disabled() {
        let mut state = PlaybackState::new();
        state.current_frame = 0;
        state.loop_enabled = false;

        state.previous(100);
        assert_eq!(state.current_frame, 0);
    }

    #[test]
    fn test_go_to_start_and_end() {
        let mut state = PlaybackState::new();
        state.current_frame = 50;

        state.go_to_start();
        assert_eq!(state.current_frame, 0);

        state.go_to_end(100);
        assert_eq!(state.current_frame, 99);
    }

    #[test]
    fn test_range_playback() {
        let mut state = PlaybackState::new();
        state.set_range(Some(10), Some(20));

        assert_eq!(state.effective_start(), 10);
        assert_eq!(state.effective_end(100), 20);

        state.go_to_start();
        assert_eq!(state.current_frame, 10);

        state.go_to_end(100);
        assert_eq!(state.current_frame, 20);

        // Test loop within range
        state.loop_enabled = true;
        let continued = state.advance(100);
        assert!(continued);
        assert_eq!(state.current_frame, 10); // Looped back to range start
    }

    #[test]
    fn test_set_speed_clamps_values() {
        let mut state = PlaybackState::new();

        state.set_speed(2.0);
        assert_eq!(state.speed, 2.0);

        state.set_speed(0.01);
        assert_eq!(state.speed, MIN_SPEED);

        state.set_speed(100.0);
        assert_eq!(state.speed, MAX_SPEED);
    }

    #[test]
    fn test_should_advance_invalid_inputs() {
        let state = PlaybackState::new();

        // Zero FPS should not advance
        assert!(!state.should_advance(0));
    }

    #[test]
    fn test_should_advance_timing() {
        let mut state = PlaybackState::new();
        state.mark_advanced();

        // Should not advance immediately at 60 FPS
        assert!(!state.should_advance(60));

        // Wait for approximately one frame at 60 FPS (~17ms)
        thread::sleep(Duration::from_millis(20));

        // Should now be ready to advance
        assert!(state.should_advance(60));
    }

    #[test]
    fn test_should_advance_with_speed_multiplier() {
        let mut state = PlaybackState::new();
        state.speed = 2.0; // Double speed
        state.mark_advanced();

        // At 2x speed, frames should be twice as fast
        // At 60 FPS with 2x speed, frame duration is ~8.3ms
        thread::sleep(Duration::from_millis(10));

        assert!(state.should_advance(60));
    }

    #[test]
    fn test_clear_range() {
        let mut state = PlaybackState::new();
        state.set_range(Some(10), Some(20));

        state.clear_range();
        assert!(state.range_start.is_none());
        assert!(state.range_end.is_none());
    }

    #[test]
    fn test_is_at_start() {
        let mut state = PlaybackState::new();

        // At frame 0, should be at start
        state.current_frame = 0;
        assert!(state.is_at_start());

        // At frame 10, should not be at start
        state.current_frame = 10;
        assert!(!state.is_at_start());

        // With range set, should use effective_start
        state.set_range(Some(5), None);
        state.current_frame = 5;
        assert!(state.is_at_start());

        state.current_frame = 4;
        assert!(state.is_at_start()); // Below range start, still considered at start

        state.current_frame = 6;
        assert!(!state.is_at_start());
    }

    #[test]
    fn test_is_at_end() {
        let mut state = PlaybackState::new();
        let total_frames = 100;

        // At frame 99 (last frame), should be at end
        state.current_frame = 99;
        assert!(state.is_at_end(total_frames));

        // At frame 50, should not be at end
        state.current_frame = 50;
        assert!(!state.is_at_end(total_frames));

        // At frame 0, should not be at end
        state.current_frame = 0;
        assert!(!state.is_at_end(total_frames));

        // With range set, should use effective_end
        state.set_range(None, Some(50));
        state.current_frame = 50;
        assert!(state.is_at_end(total_frames));

        state.current_frame = 51;
        assert!(state.is_at_end(total_frames)); // Above range end, still considered at end

        state.current_frame = 49;
        assert!(!state.is_at_end(total_frames));
    }

    #[test]
    fn test_boundary_with_empty_log() {
        let state = PlaybackState::new();

        // With 0 total frames, should be at both boundaries
        assert!(state.is_at_start());
        assert!(state.is_at_end(0));
    }

    #[test]
    fn test_boundary_with_single_frame() {
        let mut state = PlaybackState::new();
        state.current_frame = 0;

        // With 1 total frame, frame 0 is both start and end
        assert!(state.is_at_start());
        assert!(state.is_at_end(1));
    }

    #[test]
    fn test_next_speed_preset() {
        let mut state = PlaybackState::new();

        // Starting at 1.0x (default), next should be 2.0x
        state.speed = 1.0;
        assert_eq!(state.next_speed_preset(), 2.0);

        // At 0.25x, next should be 0.5x
        state.speed = 0.25;
        assert_eq!(state.next_speed_preset(), 0.5);

        // At 0.5x, next should be 1.0x
        state.speed = 0.5;
        assert_eq!(state.next_speed_preset(), 1.0);

        // At 2.0x, next should be 4.0x
        state.speed = 2.0;
        assert_eq!(state.next_speed_preset(), 4.0);

        // At 4.0x (max), should stay at 4.0x
        state.speed = 4.0;
        assert_eq!(state.next_speed_preset(), 4.0);

        // At above max, should return max
        state.speed = 5.0;
        assert_eq!(state.next_speed_preset(), 4.0);

        // At a value between presets, should return next higher preset
        state.speed = 0.75;
        assert_eq!(state.next_speed_preset(), 1.0);

        state.speed = 1.5;
        assert_eq!(state.next_speed_preset(), 2.0);
    }

    #[test]
    fn test_prev_speed_preset() {
        let mut state = PlaybackState::new();

        // Starting at 1.0x (default), prev should be 0.5x
        state.speed = 1.0;
        assert_eq!(state.prev_speed_preset(), 0.5);

        // At 4.0x, prev should be 2.0x
        state.speed = 4.0;
        assert_eq!(state.prev_speed_preset(), 2.0);

        // At 2.0x, prev should be 1.0x
        state.speed = 2.0;
        assert_eq!(state.prev_speed_preset(), 1.0);

        // At 0.5x, prev should be 0.25x
        state.speed = 0.5;
        assert_eq!(state.prev_speed_preset(), 0.25);

        // At 0.25x (min), should stay at 0.25x
        state.speed = 0.25;
        assert_eq!(state.prev_speed_preset(), 0.25);

        // At below min, should return min
        state.speed = 0.1;
        assert_eq!(state.prev_speed_preset(), 0.25);

        // At a value between presets, should return next lower preset
        state.speed = 0.75;
        assert_eq!(state.prev_speed_preset(), 0.5);

        state.speed = 1.5;
        assert_eq!(state.prev_speed_preset(), 1.0);
    }
}

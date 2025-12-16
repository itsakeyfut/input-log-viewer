//! Playback controls module.
//!
//! This module handles the rendering and interaction of playback controls
//! including play/pause button, frame navigation, speed control, and timeline scrubber.

use eframe::egui;

use crate::core::playback::{PlaybackState, SPEED_OPTIONS};

/// User actions that can be triggered from the controls panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlAction {
    /// Toggle between play and pause
    TogglePlayPause,
    /// Go to the first frame
    GoToStart,
    /// Go to the previous frame
    PreviousFrame,
    /// Go to the next frame
    NextFrame,
    /// Go to the last frame
    GoToEnd,
    /// Change playback speed
    SetSpeed(f32),
    /// Seek to a specific frame (from scrubber)
    SeekToFrame(u64),
}

/// Renders playback controls and returns any actions triggered by user interaction.
pub struct ControlsRenderer<'a> {
    /// Whether controls should be enabled
    enabled: bool,
    /// Whether playback is currently active
    is_playing: bool,
    /// Current playback state
    playback: &'a PlaybackState,
    /// Total number of frames in the log
    total_frames: u64,
}

impl<'a> ControlsRenderer<'a> {
    /// Create a new controls renderer.
    pub fn new(
        enabled: bool,
        is_playing: bool,
        playback: &'a PlaybackState,
        total_frames: u64,
    ) -> Self {
        Self {
            enabled,
            is_playing,
            playback,
            total_frames,
        }
    }

    /// Render the controls and return any triggered action.
    pub fn render(&self, ui: &mut egui::Ui) -> Option<ControlAction> {
        let mut action: Option<ControlAction> = None;

        ui.vertical(|ui| {
            // Playback controls row
            ui.horizontal(|ui| {
                action = self.render_navigation_buttons(ui).or(action);
                ui.separator();
                self.render_frame_counter(ui);
                ui.separator();
                action = self.render_speed_control(ui).or(action);
            });

            ui.add_space(4.0);

            // Timeline scrubber row
            ui.horizontal(|ui| {
                action = self.render_scrubber(ui).or(action);
            });

            // Bookmarks row
            ui.horizontal(|ui| {
                ui.label("Bookmarks:");
                ui.label("(none)");
                // TODO: Display bookmarks in Phase 3
            });
        });

        action
    }

    /// Render navigation buttons and return any triggered action.
    fn render_navigation_buttons(&self, ui: &mut egui::Ui) -> Option<ControlAction> {
        let mut action: Option<ControlAction> = None;

        ui.add_enabled_ui(self.enabled, |ui| {
            if ui.button("⏮").on_hover_text("Go to start (Home)").clicked() {
                action = Some(ControlAction::GoToStart);
            }
            if ui
                .button("⏪")
                .on_hover_text("Previous frame (←)")
                .clicked()
            {
                action = Some(ControlAction::PreviousFrame);
            }

            // Play/pause button with icon based on current state
            let (btn_text, hover_text) = if self.is_playing {
                ("⏸", "Pause (Space)")
            } else {
                ("▶", "Play (Space)")
            };
            if ui.button(btn_text).on_hover_text(hover_text).clicked() {
                action = Some(ControlAction::TogglePlayPause);
            }

            if ui.button("⏩").on_hover_text("Next frame (→)").clicked() {
                action = Some(ControlAction::NextFrame);
            }
            if ui.button("⏭").on_hover_text("Go to end (End)").clicked() {
                action = Some(ControlAction::GoToEnd);
            }
        });

        action
    }

    /// Render the frame counter display.
    fn render_frame_counter(&self, ui: &mut egui::Ui) {
        ui.label(format!(
            "Frame: {} / {}",
            self.playback.current_frame, self.total_frames
        ));
    }

    /// Render speed control and return any triggered action.
    fn render_speed_control(&self, ui: &mut egui::Ui) -> Option<ControlAction> {
        let mut action: Option<ControlAction> = None;

        ui.add_enabled_ui(self.enabled, |ui| {
            ui.label("Speed:");
            let current_speed = self.playback.speed;
            egui::ComboBox::from_id_salt("speed_combo")
                .selected_text(format!("{:.2}x", current_speed))
                .width(60.0)
                .show_ui(ui, |ui| {
                    for &speed in SPEED_OPTIONS {
                        if ui
                            .selectable_label(
                                (current_speed - speed).abs() < 0.01,
                                format!("{:.2}x", speed),
                            )
                            .clicked()
                        {
                            action = Some(ControlAction::SetSpeed(speed));
                        }
                    }
                });
        });

        action
    }

    /// Render the timeline scrubber and return any triggered action.
    fn render_scrubber(&self, ui: &mut egui::Ui) -> Option<ControlAction> {
        let mut action: Option<ControlAction> = None;

        ui.add_enabled_ui(self.enabled, |ui| {
            let max_frame = self.total_frames.saturating_sub(1) as f32;
            let mut frame = self.playback.current_frame as f32;
            let response = ui.add(
                egui::Slider::new(&mut frame, 0.0..=max_frame.max(1.0))
                    .show_value(false)
                    .text(""),
            );
            if response.changed() {
                action = Some(ControlAction::SeekToFrame(frame as u64));
            }
        });

        action
    }
}

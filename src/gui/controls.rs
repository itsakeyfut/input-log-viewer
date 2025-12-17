//! Playback controls module.
//!
//! This module handles the rendering and interaction of playback controls
//! including play/pause button, frame navigation, speed control, and timeline scrubber.

use eframe::egui;

use crate::core::log::Bookmark;
use crate::core::playback::{PlaybackState, SPEED_OPTIONS};
use crate::gui::timeline::{MAX_VISIBLE_FRAMES, MIN_VISIBLE_FRAMES};

/// User actions that can be triggered from the controls panel.
#[derive(Debug, Clone, PartialEq)]
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
    /// Increase playback speed to next preset
    IncreaseSpeed,
    /// Decrease playback speed to previous preset
    DecreaseSpeed,
    /// Seek to a specific frame (from scrubber)
    SeekToFrame(u64),
    /// Toggle bookmark at current frame
    ToggleBookmark,
    /// Go to the next bookmark
    NextBookmark,
    /// Go to the previous bookmark
    PreviousBookmark,
    /// Jump to a specific bookmark by index
    JumpToBookmark(usize),
    /// Remove a bookmark by index (for future bookmark panel)
    #[allow(dead_code)]
    RemoveBookmark(usize),
    /// Add bookmark with label at current frame (for future bookmark panel)
    #[allow(dead_code)]
    AddBookmarkWithLabel(String),
    /// Set zoom level (visible frames)
    SetZoom(u64),
    /// Toggle auto-scroll (keep current frame visible during playback)
    ToggleAutoScroll,
    /// Toggle loop selection mode (loop playback within selected range)
    ToggleLoopSelection,
    /// Clear the current selection
    ClearSelection,
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
    /// Whether the current frame is at the start boundary
    at_start: bool,
    /// Whether the current frame is at the end boundary
    at_end: bool,
    /// Mutable frame value for inline editing
    frame_input: &'a mut u64,
    /// Bookmarks for the current session
    bookmarks: &'a [Bookmark],
    /// Whether the current frame has a bookmark
    has_bookmark_at_current: bool,
    /// Current visible frames (zoom level)
    visible_frames: u64,
    /// Mutable zoom value for inline editing
    zoom_input: &'a mut u64,
    /// Whether auto-scroll is enabled
    auto_scroll: bool,
    /// Current selection range (start, end) if any
    selection: Option<(u64, u64)>,
    /// Whether loop selection is enabled
    loop_selection: bool,
}

impl<'a> ControlsRenderer<'a> {
    /// Create a new controls renderer.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        enabled: bool,
        is_playing: bool,
        playback: &'a PlaybackState,
        total_frames: u64,
        frame_input: &'a mut u64,
        bookmarks: &'a [Bookmark],
        visible_frames: u64,
        zoom_input: &'a mut u64,
        auto_scroll: bool,
        selection: Option<(u64, u64)>,
        loop_selection: bool,
    ) -> Self {
        // Pre-compute boundary states for button disabling
        let at_start = playback.is_at_start();
        let at_end = playback.is_at_end(total_frames);
        let has_bookmark_at_current = bookmarks.iter().any(|b| b.frame == playback.current_frame);

        Self {
            enabled,
            is_playing,
            playback,
            total_frames,
            at_start,
            at_end,
            frame_input,
            bookmarks,
            has_bookmark_at_current,
            visible_frames,
            zoom_input,
            auto_scroll,
            selection,
            loop_selection,
        }
    }

    /// Render the controls and return any triggered action.
    pub fn render(&mut self, ui: &mut egui::Ui) -> Option<ControlAction> {
        let mut nav_action: Option<ControlAction> = None;
        let mut frame_action: Option<ControlAction> = None;
        let mut speed_action: Option<ControlAction> = None;
        let mut zoom_action: Option<ControlAction> = None;
        let mut auto_scroll_action: Option<ControlAction> = None;
        let mut selection_action: Option<ControlAction> = None;
        let mut scrubber_action: Option<ControlAction> = None;
        let mut bookmark_action: Option<ControlAction> = None;

        ui.vertical(|ui| {
            // Playback controls row
            ui.horizontal(|ui| {
                nav_action = self.render_navigation_buttons(ui);
                ui.separator();
                frame_action = self.render_frame_counter(ui);
                ui.separator();
                speed_action = self.render_speed_control(ui);
                ui.separator();
                zoom_action = self.render_zoom_control(ui);
                ui.separator();
                auto_scroll_action = self.render_auto_scroll_toggle(ui);
                ui.separator();
                selection_action = self.render_selection_controls(ui);
            });

            ui.add_space(4.0);

            // Timeline scrubber row
            ui.horizontal(|ui| {
                scrubber_action = self.render_scrubber(ui);
            });

            // Bookmarks row
            ui.horizontal(|ui| {
                bookmark_action = self.render_bookmarks_row(ui);
            });
        });

        // Return the first action that was triggered (priority order)
        nav_action
            .or(frame_action)
            .or(speed_action)
            .or(zoom_action)
            .or(auto_scroll_action)
            .or(selection_action)
            .or(scrubber_action)
            .or(bookmark_action)
    }

    /// Render navigation buttons and return any triggered action.
    fn render_navigation_buttons(&self, ui: &mut egui::Ui) -> Option<ControlAction> {
        let mut action: Option<ControlAction> = None;

        // Go to start button - disabled when already at start
        let start_enabled = self.enabled && !self.at_start;
        if ui
            .add_enabled(start_enabled, egui::Button::new("⏮"))
            .on_hover_text("Go to start (Home)")
            .clicked()
        {
            action = Some(ControlAction::GoToStart);
        }

        // Previous frame button - disabled when at start
        let prev_enabled = self.enabled && !self.at_start;
        if ui
            .add_enabled(prev_enabled, egui::Button::new("⏪"))
            .on_hover_text("Previous frame (←)")
            .clicked()
        {
            action = Some(ControlAction::PreviousFrame);
        }

        // Play/pause button - always enabled when controls are enabled
        let (btn_text, hover_text) = if self.is_playing {
            ("⏸", "Pause (Space)")
        } else {
            ("▶", "Play (Space)")
        };
        if ui
            .add_enabled(self.enabled, egui::Button::new(btn_text))
            .on_hover_text(hover_text)
            .clicked()
        {
            action = Some(ControlAction::TogglePlayPause);
        }

        // Next frame button - disabled when at end
        let next_enabled = self.enabled && !self.at_end;
        if ui
            .add_enabled(next_enabled, egui::Button::new("⏩"))
            .on_hover_text("Next frame (→)")
            .clicked()
        {
            action = Some(ControlAction::NextFrame);
        }

        // Go to end button - disabled when already at end
        let end_enabled = self.enabled && !self.at_end;
        if ui
            .add_enabled(end_enabled, egui::Button::new("⏭"))
            .on_hover_text("Go to end (End)")
            .clicked()
        {
            action = Some(ControlAction::GoToEnd);
        }

        action
    }

    /// Render the frame counter with editable input.
    fn render_frame_counter(&mut self, ui: &mut egui::Ui) -> Option<ControlAction> {
        let mut action: Option<ControlAction> = None;
        let max_frame = self.total_frames.saturating_sub(1);

        ui.add_enabled_ui(self.enabled, |ui| {
            ui.label("Frame:");

            // Sync frame_input with current playback frame
            *self.frame_input = self.playback.current_frame;

            let response = ui.add(
                egui::DragValue::new(self.frame_input)
                    .range(0..=max_frame)
                    .speed(1.0),
            );

            // Trigger seek when value changes (drag or direct input)
            if response.changed() {
                action = Some(ControlAction::SeekToFrame(*self.frame_input));
            }

            ui.label(format!("/ {}", self.total_frames.saturating_sub(1)));
        });

        action
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

    /// Render zoom control with editable input.
    fn render_zoom_control(&mut self, ui: &mut egui::Ui) -> Option<ControlAction> {
        let mut action: Option<ControlAction> = None;

        ui.add_enabled_ui(self.enabled, |ui| {
            ui.label("Zoom:");

            // Sync zoom_input with current visible frames
            *self.zoom_input = self.visible_frames;

            let response = ui.add(
                egui::DragValue::new(self.zoom_input)
                    .range(MIN_VISIBLE_FRAMES..=MAX_VISIBLE_FRAMES)
                    .speed(1.0)
                    .suffix(" frames"),
            );

            // Trigger zoom when value changes (drag or direct input)
            if response.changed() {
                action = Some(ControlAction::SetZoom(*self.zoom_input));
            }
        });

        action
    }

    /// Render auto-scroll toggle and return any triggered action.
    fn render_auto_scroll_toggle(&self, ui: &mut egui::Ui) -> Option<ControlAction> {
        let mut action: Option<ControlAction> = None;

        ui.add_enabled_ui(self.enabled, |ui| {
            let toggle_text = if self.auto_scroll {
                "Auto-scroll: ON"
            } else {
                "Auto-scroll: OFF"
            };

            let button = if self.auto_scroll {
                egui::Button::new(
                    egui::RichText::new(toggle_text).color(egui::Color32::from_rgb(100, 200, 100)),
                )
            } else {
                egui::Button::new(egui::RichText::new(toggle_text).color(egui::Color32::GRAY))
            };

            if ui
                .add(button)
                .on_hover_text("Toggle auto-scroll to keep current frame visible during playback")
                .clicked()
            {
                action = Some(ControlAction::ToggleAutoScroll);
            }
        });

        action
    }

    /// Render selection controls (loop selection toggle and clear button).
    fn render_selection_controls(&self, ui: &mut egui::Ui) -> Option<ControlAction> {
        let mut action: Option<ControlAction> = None;
        let has_selection = self.selection.is_some();

        ui.add_enabled_ui(self.enabled, |ui| {
            // Loop Selection toggle button (only enabled when there's a selection)
            let loop_text = if self.loop_selection {
                "Loop: Selection"
            } else {
                "Loop: All"
            };

            let loop_button = if self.loop_selection && has_selection {
                egui::Button::new(
                    egui::RichText::new(loop_text).color(egui::Color32::from_rgb(180, 100, 220)),
                )
            } else if has_selection {
                egui::Button::new(loop_text)
            } else {
                egui::Button::new(egui::RichText::new(loop_text).color(egui::Color32::DARK_GRAY))
            };

            if ui
                .add_enabled(has_selection, loop_button)
                .on_hover_text(if has_selection {
                    "Toggle looping within selected range"
                } else {
                    "Shift+drag on timeline to select a range"
                })
                .clicked()
            {
                action = Some(ControlAction::ToggleLoopSelection);
            }

            // Display selection range if any
            if let Some((start, end)) = self.selection {
                ui.label(
                    egui::RichText::new(format!("F{}-F{}", start, end))
                        .color(egui::Color32::from_rgb(180, 100, 220)),
                );

                // Clear selection button
                if ui
                    .button("✕")
                    .on_hover_text("Clear selection (Esc)")
                    .clicked()
                {
                    action = Some(ControlAction::ClearSelection);
                }
            }
        });

        action
    }

    /// Render the timeline scrubber and return any triggered action.
    fn render_scrubber(&self, ui: &mut egui::Ui) -> Option<ControlAction> {
        let mut action: Option<ControlAction> = None;

        ui.add_enabled_ui(self.enabled, |ui| {
            let max_frame = self.total_frames.saturating_sub(1);
            let max_frame_f32 = max_frame as f32;
            let mut frame = self.playback.current_frame as f32;

            // Render the slider
            ui.vertical(|ui| {
                let response = ui.add(
                    egui::Slider::new(&mut frame, 0.0..=max_frame_f32.max(1.0))
                        .show_value(false)
                        .text(""),
                );
                if response.changed() {
                    action = Some(ControlAction::SeekToFrame(frame as u64));
                }

                // Frame range labels below the slider
                ui.horizontal(|ui| {
                    ui.label("0");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("{}", max_frame));
                    });
                });
            });
        });

        action
    }

    /// Render the bookmarks row with toggle button and bookmark list.
    fn render_bookmarks_row(&self, ui: &mut egui::Ui) -> Option<ControlAction> {
        let mut action: Option<ControlAction> = None;

        ui.add_enabled_ui(self.enabled, |ui| {
            // Toggle bookmark button
            let toggle_text = if self.has_bookmark_at_current {
                "★ Remove"
            } else {
                "☆ Add"
            };
            let toggle_tooltip = if self.has_bookmark_at_current {
                "Remove bookmark at current frame (B)"
            } else {
                "Add bookmark at current frame (B)"
            };

            if ui
                .button(toggle_text)
                .on_hover_text(toggle_tooltip)
                .clicked()
            {
                action = Some(ControlAction::ToggleBookmark);
            }

            ui.separator();

            // Bookmark navigation buttons (only enabled if there are bookmarks)
            let has_bookmarks = !self.bookmarks.is_empty();

            ui.add_enabled_ui(has_bookmarks, |ui| {
                if ui
                    .button("◀ Prev")
                    .on_hover_text("Go to previous bookmark")
                    .clicked()
                {
                    action = Some(ControlAction::PreviousBookmark);
                }

                if ui
                    .button("Next ▶")
                    .on_hover_text("Go to next bookmark")
                    .clicked()
                {
                    action = Some(ControlAction::NextBookmark);
                }
            });

            ui.separator();

            // Display bookmark chips
            ui.label("Bookmarks:");

            if self.bookmarks.is_empty() {
                ui.label("(none)");
            } else {
                // Use a scroll area if there are many bookmarks
                egui::ScrollArea::horizontal()
                    .max_width(ui.available_width() - 10.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for (index, bookmark) in self.bookmarks.iter().enumerate() {
                                let is_current = bookmark.frame == self.playback.current_frame;
                                let label = if let Some(ref label) = bookmark.label {
                                    format!("★ F{} - {}", bookmark.frame, label)
                                } else {
                                    format!("★ F{}", bookmark.frame)
                                };

                                // Style the button differently if it's the current frame
                                let button = if is_current {
                                    egui::Button::new(
                                        egui::RichText::new(&label)
                                            .color(egui::Color32::from_rgb(255, 200, 100)),
                                    )
                                } else {
                                    egui::Button::new(&label)
                                };

                                if ui
                                    .add(button)
                                    .on_hover_text(format!("Jump to frame {}", bookmark.frame))
                                    .clicked()
                                {
                                    action = Some(ControlAction::JumpToBookmark(index));
                                }
                            }
                        });
                    });
            }
        });

        action
    }
}

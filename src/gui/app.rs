//! Main application state and GUI logic.
//!
//! This module defines the main application struct and implements the eframe::App trait
//! to provide the core GUI functionality for the input log viewer.

use eframe::egui;
use std::path::PathBuf;

use crate::core::log::InputLog;
use crate::core::parser;

use super::timeline::{TimelineConfig, TimelineRenderer};

/// Error information for the error state.
#[derive(Debug, Clone, PartialEq)]
pub struct AppError {
    /// Error message to display
    pub message: String,
    /// Whether the error is recoverable (can return to previous state)
    pub recoverable: bool,
}

impl AppError {
    /// Create a new recoverable error.
    pub fn recoverable(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            recoverable: true,
        }
    }

    /// Create a new non-recoverable error.
    #[allow(dead_code)] // Will be used when implementing fatal error handling
    pub fn fatal(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            recoverable: false,
        }
    }
}

/// Application state indicating the current loading status.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum AppState {
    /// No file has been loaded yet (initial state)
    #[default]
    NoFileLoaded,
    /// File loading in progress
    #[allow(dead_code)] // Will be used for async file loading
    Loading,
    /// A file has been successfully loaded and is ready for viewing
    Ready,
    /// Playback is in progress
    #[allow(dead_code)] // Will be used when implementing playback
    Playing,
    /// An error occurred
    Error(AppError),
}

impl AppState {
    /// Returns true if the application is in a state where file operations are allowed.
    pub fn can_open_file(&self) -> bool {
        matches!(
            self,
            AppState::NoFileLoaded | AppState::Ready | AppState::Playing | AppState::Error(_)
        )
    }

    /// Returns true if toolbar controls (filter, search) should be enabled.
    pub fn toolbar_enabled(&self) -> bool {
        matches!(self, AppState::Ready | AppState::Playing)
    }

    /// Returns true if playback controls should be enabled.
    pub fn controls_enabled(&self) -> bool {
        matches!(self, AppState::Ready | AppState::Playing)
    }

    /// Returns true if the timeline should be displayed.
    #[allow(dead_code)] // Will be used for conditional timeline rendering
    pub fn show_timeline(&self) -> bool {
        matches!(self, AppState::Ready | AppState::Playing)
    }

    /// Returns true if playback is currently active.
    pub fn is_playing(&self) -> bool {
        matches!(self, AppState::Playing)
    }
}

/// Kind of status message to display.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusKind {
    /// Success message (shown in green)
    Success,
    /// Error message (shown in red)
    Error,
}

/// A status message with its kind and timestamp.
#[derive(Debug, Clone)]
pub struct StatusMessage {
    /// The message text
    pub text: String,
    /// Kind of message (success/error)
    pub kind: StatusKind,
    /// When the message was created (for auto-dismiss)
    pub created_at: std::time::Instant,
}

impl StatusMessage {
    /// Create a new status message.
    pub fn new(text: impl Into<String>, kind: StatusKind) -> Self {
        Self {
            text: text.into(),
            kind,
            created_at: std::time::Instant::now(),
        }
    }

    /// Duration to show status messages before auto-dismissing.
    const DISPLAY_DURATION: std::time::Duration = std::time::Duration::from_secs(5);

    /// Check if the message should still be displayed.
    pub fn is_visible(&self) -> bool {
        self.created_at.elapsed() < Self::DISPLAY_DURATION
    }
}

/// Main application state and GUI logic.
pub struct InputLogViewerApp {
    /// Current application state
    state: AppState,
    /// Loaded input log data (Some when state is Ready)
    log: Option<InputLog>,
    /// Path to the currently loaded file
    loaded_file_path: Option<PathBuf>,
    /// Status message to display (success/error notifications)
    status_message: Option<StatusMessage>,
    /// Timeline rendering configuration
    timeline_config: TimelineConfig,
}

impl InputLogViewerApp {
    /// Create a new application instance.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            state: AppState::NoFileLoaded,
            log: None,
            loaded_file_path: None,
            status_message: None,
            timeline_config: TimelineConfig::default(),
        }
    }

    /// Open a file dialog and load the selected .ilj file.
    fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Input Log JSON", &["ilj"])
            .set_title("Open Input Log File")
            .pick_file()
        {
            self.load_file(path);
        }
    }

    /// Load an input log file from the given path.
    fn load_file(&mut self, path: PathBuf) {
        match std::fs::read_to_string(&path) {
            Ok(content) => match parser::parse_json(&content) {
                Ok(log) => {
                    let frame_count = log.metadata.frame_count;
                    let event_count = log.events.len();
                    self.log = Some(log);
                    self.loaded_file_path = Some(path.clone());
                    self.state = AppState::Ready;
                    self.status_message = Some(StatusMessage::new(
                        format!(
                            "Loaded: {} ({} frames, {} events)",
                            path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "file".to_string()),
                            frame_count,
                            event_count
                        ),
                        StatusKind::Success,
                    ));
                }
                Err(e) => {
                    self.set_error(format!("Parse error: {}", e));
                }
            },
            Err(e) => {
                self.set_error(format!("Failed to read file: {}", e));
            }
        }
    }

    /// Set an error state and display an error message.
    fn set_error(&mut self, message: String) {
        self.state = AppState::Error(AppError::recoverable(message.clone()));
        self.status_message = Some(StatusMessage::new(message, StatusKind::Error));
    }

    /// Clear error state and return to appropriate state.
    fn clear_error(&mut self) {
        if self.log.is_some() {
            self.state = AppState::Ready;
        } else {
            self.state = AppState::NoFileLoaded;
        }
    }
}

impl eframe::App for InputLogViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_toolbar(ctx);
        self.render_controls(ctx);
        self.render_timeline(ctx);
    }
}

impl InputLogViewerApp {
    /// Render the top toolbar section.
    ///
    /// Contains file loading, filter options, and search functionality.
    fn render_toolbar(&mut self, ctx: &egui::Context) {
        let can_open = self.state.can_open_file();
        let toolbar_enabled = self.state.toolbar_enabled();

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Input Log Viewer");
                ui.separator();

                // File loading button (enabled based on state)
                ui.add_enabled_ui(can_open, |ui| {
                    if ui.button("📂 Open File").clicked() {
                        self.open_file_dialog();
                    }
                });

                ui.separator();

                // Filter dropdown (enabled only when file is loaded)
                ui.add_enabled_ui(toolbar_enabled, |ui| {
                    ui.label("Filter:");
                    egui::ComboBox::from_id_salt("filter_combo")
                        .selected_text("All Inputs")
                        .show_ui(ui, |ui| {
                            let _ = ui.selectable_label(true, "All Inputs");
                            let _ = ui.selectable_label(false, "Buttons Only");
                            let _ = ui.selectable_label(false, "Axes Only");
                        });
                });

                ui.separator();

                // Search button (enabled only when file is loaded)
                ui.add_enabled_ui(toolbar_enabled, |ui| {
                    if ui.button("🔍 Search").clicked() {
                        // TODO: Implement search dialog in Phase 3
                    }
                });

                // Show status message in toolbar (right-aligned)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.render_status_message(ui);
                });
            });
        });
    }

    /// Render the status message if one is active.
    fn render_status_message(&mut self, ui: &mut egui::Ui) {
        // Check if we should dismiss the message
        let should_dismiss = self
            .status_message
            .as_ref()
            .is_some_and(|msg| !msg.is_visible());

        if should_dismiss {
            self.status_message = None;
            // Also clear error state if the error message expired
            if matches!(self.state, AppState::Error(_)) {
                self.clear_error();
            }
            return;
        }

        // Extract message info before rendering to avoid borrow issues
        let msg_info = self.status_message.as_ref().map(|msg| {
            let color = match msg.kind {
                StatusKind::Success => egui::Color32::from_rgb(76, 175, 80), // Green
                StatusKind::Error => egui::Color32::from_rgb(244, 67, 54),   // Red
            };
            (color, msg.text.clone())
        });

        if let Some((color, text)) = msg_info {
            let mut dismiss_clicked = false;

            ui.horizontal(|ui| {
                // Dismiss button
                if ui.small_button("✕").clicked() {
                    dismiss_clicked = true;
                }
                ui.colored_label(color, &text);
            });

            // Handle dismiss after the closure
            if dismiss_clicked {
                self.status_message = None;
                if matches!(self.state, AppState::Error(_)) {
                    self.clear_error();
                }
            }
        }
    }

    /// Render the bottom controls section.
    ///
    /// Contains playback controls, frame navigation, and speed settings.
    fn render_controls(&mut self, ctx: &egui::Context) {
        let controls_enabled = self.state.controls_enabled();
        let is_playing = self.state.is_playing();

        egui::TopBottomPanel::bottom("controls")
            .min_height(80.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Playback controls row
                    ui.horizontal(|ui| {
                        // Navigation buttons (disabled when no file or loading)
                        ui.add_enabled_ui(controls_enabled, |ui| {
                            if ui.button("⏮").on_hover_text("Go to start").clicked() {
                                // TODO: Implement in Phase 2
                            }
                            if ui.button("⏪").on_hover_text("Previous frame").clicked() {
                                // TODO: Implement in Phase 2
                            }
                            // Show pause icon when playing, play icon otherwise
                            let play_btn_text = if is_playing { "⏸" } else { "▶" };
                            let play_btn_hover = if is_playing { "Pause" } else { "Play" };
                            if ui
                                .button(play_btn_text)
                                .on_hover_text(play_btn_hover)
                                .clicked()
                            {
                                // TODO: Implement in Phase 2
                            }
                            if ui.button("⏩").on_hover_text("Next frame").clicked() {
                                // TODO: Implement in Phase 2
                            }
                            if ui.button("⏭").on_hover_text("Go to end").clicked() {
                                // TODO: Implement in Phase 2
                            }
                        });

                        ui.separator();

                        // Frame counter
                        let total_frames = self
                            .log
                            .as_ref()
                            .map(|l| l.metadata.frame_count)
                            .unwrap_or(0);
                        ui.label(format!(
                            "Frame: {} / {}",
                            self.timeline_config.current_frame, total_frames
                        ));

                        ui.separator();

                        // Speed control (disabled when no file or loading)
                        ui.add_enabled_ui(controls_enabled, |ui| {
                            ui.label("Speed:");
                            egui::ComboBox::from_id_salt("speed_combo")
                                .selected_text("1.0x")
                                .width(60.0)
                                .show_ui(ui, |ui| {
                                    let _ = ui.selectable_label(false, "0.25x");
                                    let _ = ui.selectable_label(false, "0.5x");
                                    let _ = ui.selectable_label(true, "1.0x");
                                    let _ = ui.selectable_label(false, "2.0x");
                                    let _ = ui.selectable_label(false, "4.0x");
                                });
                        });
                    });

                    ui.add_space(4.0);

                    // Timeline scrubber row (disabled when no file or loading)
                    ui.horizontal(|ui| {
                        ui.add_enabled_ui(controls_enabled, |ui| {
                            let mut frame: f32 = 0.0;
                            ui.add(
                                egui::Slider::new(&mut frame, 0.0..=100.0)
                                    .show_value(false)
                                    .text(""),
                            );
                        });
                    });

                    // Bookmarks row
                    ui.horizontal(|ui| {
                        ui.label("Bookmarks:");
                        ui.label("(none)");
                        // TODO: Display bookmarks in Phase 3
                    });
                });
            });
    }

    /// Render the center timeline section.
    ///
    /// Displays the main timeline view with input events.
    fn render_timeline(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.state {
                AppState::NoFileLoaded => {
                    self.render_no_file_placeholder(ui);
                }
                AppState::Loading => {
                    self.render_loading_placeholder(ui);
                }
                AppState::Ready | AppState::Playing => {
                    self.render_loaded_timeline(ui);
                }
                AppState::Error(_) => {
                    // Show placeholder even in error state
                    self.render_no_file_placeholder(ui);
                }
            }
        });
    }

    /// Render the placeholder view when no file is loaded.
    fn render_no_file_placeholder(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            ui.heading("📁 No File Loaded");
            ui.add_space(10.0);
            ui.label("Drag and drop an input log file (.ilj or .ilb) to get started.");
            ui.label("Or use the \"Open File\" button in the toolbar.");

            ui.add_space(20.0);

            // Show expected timeline layout as placeholder
            ui.separator();
            ui.add_space(10.0);
            ui.label("Timeline preview area:");
            ui.add_space(10.0);

            self.draw_placeholder_grid(ui);
        });
    }

    /// Render the loading placeholder view.
    fn render_loading_placeholder(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);

            ui.heading("⏳ Loading...");
            ui.add_space(20.0);
            ui.label("Please wait while the file is being loaded.");

            ui.add_space(20.0);
            ui.spinner();
        });
    }

    /// Render the timeline view when a file is loaded.
    fn render_loaded_timeline(&self, ui: &mut egui::Ui) {
        if let Some(ref log) = self.log {
            // Show file info header
            ui.horizontal(|ui| {
                ui.heading("📊 Timeline");
                ui.separator();

                if let Some(ref path) = self.loaded_file_path {
                    ui.label(format!(
                        "File: {}",
                        path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Unknown".to_string())
                    ));
                }

                ui.separator();
                ui.label(format!(
                    "Frames: {} | FPS: {} | Events: {}",
                    log.metadata.frame_count,
                    log.metadata.target_fps,
                    log.events.len()
                ));

                if let Some(ref source) = log.metadata.source {
                    ui.separator();
                    ui.label(format!("Source: {}", source));
                }
            });

            ui.separator();
            ui.add_space(5.0);

            // Render the timeline using TimelineRenderer
            let renderer = TimelineRenderer::new(log, &self.timeline_config);
            renderer.render(ui);
        }
    }

    /// Draw a placeholder grid for the timeline preview.
    fn draw_placeholder_grid(&self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(
            egui::vec2(available_size.x.min(800.0), 200.0),
            egui::Sense::hover(),
        );

        let rect = response.rect;
        let stroke = egui::Stroke::new(1.0, egui::Color32::GRAY);

        // Draw border
        painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Inside);

        // Draw horizontal grid lines (input rows)
        let row_height = rect.height() / 5.0;
        for i in 1..5 {
            let y = rect.top() + row_height * i as f32;
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                stroke,
            );
        }

        // Draw vertical grid lines (frame columns)
        let col_width = rect.width() / 10.0;
        for i in 1..10 {
            let x = rect.left() + col_width * i as f32;
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                stroke,
            );
        }

        // Draw placeholder labels
        let label_color = egui::Color32::LIGHT_GRAY;
        let labels = [
            "A Button",
            "B Button",
            "X Button",
            "Left Stick X",
            "Left Stick Y",
        ];
        for (i, label) in labels.iter().enumerate() {
            painter.text(
                egui::pos2(
                    rect.left() + 5.0,
                    rect.top() + row_height * (i as f32 + 0.5),
                ),
                egui::Align2::LEFT_CENTER,
                *label,
                egui::FontId::default(),
                label_color,
            );
        }

        // Draw center text
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "(Timeline will appear here)",
            egui::FontId::proportional(16.0),
            egui::Color32::DARK_GRAY,
        );
    }
}

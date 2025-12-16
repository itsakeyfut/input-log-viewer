//! Main application state and GUI logic.
//!
//! This module defines the main application struct and implements the eframe::App trait
//! to provide the core GUI functionality for the input log viewer.

use eframe::egui;
use std::path::PathBuf;

use crate::core::filter::FilterState;
use crate::core::log::{ButtonState, InputKind, InputLog};
use crate::core::parser;
use crate::core::playback::PlaybackState;
use crate::core::search::{SearchQuery, SearchResult, find_matches};

use super::controls::{ControlAction, ControlsRenderer};
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

/// State for the search dialog and results.
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    /// Whether the search dialog is currently open
    pub dialog_open: bool,
    /// Currently selected input ID for searching (index in mappings, not the actual ID)
    pub selected_input_index: usize,
    /// Currently selected button state for searching (0=Any, 1=Pressed, 2=Released)
    pub selected_state_index: usize,
    /// Current search results
    pub results: SearchResult,
    /// Whether a search has been performed
    pub has_searched: bool,
}

impl SearchState {
    /// Create a new search state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset the search state when a new file is loaded.
    pub fn reset(&mut self) {
        self.selected_input_index = 0;
        self.selected_state_index = 0;
        self.results = SearchResult::new();
        self.has_searched = false;
        // Keep dialog_open unchanged so user can continue searching
    }

    /// Clear search results and mark as not searched.
    pub fn clear_results(&mut self) {
        self.results = SearchResult::new();
        self.has_searched = false;
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
    /// Playback state for frame position and timing
    playback: PlaybackState,
    /// Filter state for input visibility
    filter: FilterState,
    /// Whether the filter popup is currently open
    filter_popup_open: bool,
    /// Frame input value for inline editing in controls
    frame_input_value: u64,
    /// Search state for input search functionality
    search: SearchState,
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
            playback: PlaybackState::new(),
            filter: FilterState::new(),
            filter_popup_open: false,
            frame_input_value: 0,
            search: SearchState::new(),
        }
    }

    /// Open a file dialog and load the selected input log file (.ilj or .ilb).
    fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Input Log Files", &["ilj", "ilb"])
            .add_filter("Input Log JSON", &["ilj"])
            .add_filter("Input Log Binary", &["ilb"])
            .set_title("Open Input Log File")
            .pick_file()
        {
            self.load_file(path);
        }
    }

    /// Load an input log file from the given path.
    ///
    /// Auto-detects the file format based on extension:
    /// - `.ilj` files are parsed as JSON
    /// - `.ilb` files are parsed as binary
    fn load_file(&mut self, path: PathBuf) {
        // Determine format from extension
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase());

        let parse_result = match extension.as_deref() {
            Some("ilj") => {
                // JSON format - read as string
                std::fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read file: {}", e))
                    .and_then(|content| {
                        parser::parse_json(&content).map_err(|e| format!("Parse error: {}", e))
                    })
            }
            Some("ilb") => {
                // Binary format - read as bytes
                std::fs::read(&path)
                    .map_err(|e| format!("Failed to read file: {}", e))
                    .and_then(|data| {
                        parser::parse_binary(&data).map_err(|e| format!("Parse error: {}", e))
                    })
            }
            _ => {
                Err("Unsupported file format. Please use .ilj (JSON) or .ilb (binary).".to_string())
            }
        };

        match parse_result {
            Ok(log) => {
                let frame_count = log.metadata.frame_count;
                let event_count = log.events.len();
                // Initialize filter with all inputs visible
                self.filter.initialize_from_log(&log);
                // Reset search state for new file
                self.search.reset();
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
                self.set_error(e);
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
        // Handle playback advancement when playing
        if self.state.is_playing() {
            if let Some(ref log) = self.log {
                let target_fps = log.metadata.target_fps;
                let total_frames = log.metadata.frame_count;

                if self.playback.should_advance(target_fps) {
                    let should_continue = self.playback.advance(total_frames);
                    if !should_continue {
                        // Playback ended (loop disabled and reached end)
                        self.state = AppState::Ready;
                    }
                }
            }
            // Keep requesting repaints while playing
            ctx.request_repaint();
        }

        let total_frames = self
            .log
            .as_ref()
            .map(|l| l.metadata.frame_count)
            .unwrap_or(0);

        // Handle keyboard shortcuts
        if let Some(action) = self.handle_keyboard_shortcuts(ctx) {
            self.handle_control_action(action, total_frames);
        }

        // Sync playback current_frame with timeline config for rendering
        self.timeline_config.current_frame = self.playback.current_frame;

        self.render_toolbar(ctx);
        self.render_controls(ctx);
        self.render_timeline(ctx);
    }
}

impl InputLogViewerApp {
    /// Handle keyboard shortcuts for playback control.
    ///
    /// Returns an action if a keyboard shortcut was triggered, None otherwise.
    /// Shortcuts only work when a file is loaded (controls_enabled).
    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) -> Option<ControlAction> {
        // Handle Ctrl+F to open search dialog (works when toolbar is enabled)
        if self.state.toolbar_enabled() {
            let open_search = ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::F));
            if open_search {
                self.search.dialog_open = true;
                return None;
            }
        }

        // Only process playback shortcuts when controls are enabled
        if !self.state.controls_enabled() {
            return None;
        }

        ctx.input(|i| {
            // Space: Toggle play/pause
            if i.key_pressed(egui::Key::Space) {
                return Some(ControlAction::TogglePlayPause);
            }

            // Left Arrow: Previous frame
            if i.key_pressed(egui::Key::ArrowLeft) {
                return Some(ControlAction::PreviousFrame);
            }

            // Right Arrow: Next frame
            if i.key_pressed(egui::Key::ArrowRight) {
                return Some(ControlAction::NextFrame);
            }

            // Home: Jump to first frame
            if i.key_pressed(egui::Key::Home) {
                return Some(ControlAction::GoToStart);
            }

            // End: Jump to last frame
            if i.key_pressed(egui::Key::End) {
                return Some(ControlAction::GoToEnd);
            }

            None
        })
    }

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

                // Filter dropdown button (enabled only when file is loaded)
                ui.add_enabled_ui(toolbar_enabled, |ui| {
                    let filter_button_text = if self.filter_popup_open {
                        "Filter ▲"
                    } else {
                        "Filter ▼"
                    };
                    let filter_response = ui.button(filter_button_text);
                    if filter_response.clicked() {
                        self.filter_popup_open = !self.filter_popup_open;
                    }

                    // Show filter count indicator
                    if let Some(ref log) = self.log {
                        let visible_count = self.filter.visible_ids.len();
                        let total_count = log.mappings.len();
                        if visible_count < total_count {
                            ui.label(format!("({}/{})", visible_count, total_count));
                        }
                    }
                });

                ui.separator();

                // Search button (enabled only when file is loaded)
                ui.add_enabled_ui(toolbar_enabled, |ui| {
                    if ui.button("🔍 Search").clicked() {
                        self.search.dialog_open = !self.search.dialog_open;
                    }
                });

                // Show status message in toolbar (right-aligned)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.render_status_message(ui);
                });
            });
        });

        // Render filter popup panel if open
        if self.filter_popup_open && toolbar_enabled {
            self.render_filter_popup(ctx);
        }

        // Render search dialog if open
        if self.search.dialog_open && toolbar_enabled {
            self.render_search_dialog(ctx);
        }
    }

    /// Render the filter popup panel.
    fn render_filter_popup(&mut self, ctx: &egui::Context) {
        let mut should_close = false;

        egui::Window::new("Input Filter")
            .id(egui::Id::new("filter_popup"))
            .collapsible(false)
            .resizable(true)
            .default_width(220.0)
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(160.0, 40.0))
            .show(ctx, |ui| {
                // Close button
                ui.horizontal(|ui| {
                    ui.heading("Filter Inputs");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕").clicked() {
                            should_close = true;
                        }
                    });
                });
                ui.separator();

                if let Some(ref log) = self.log {
                    // Input type filters
                    ui.label("Input Types:");

                    // Button type checkbox - prevent unchecking if it's the last enabled type
                    let button_can_toggle = self
                        .filter
                        .can_disable_type(crate::core::log::InputKind::Button);
                    ui.add_enabled_ui(button_can_toggle, |ui| {
                        ui.checkbox(&mut self.filter.show_button, "Button");
                    });

                    // Axis1D type checkbox - prevent unchecking if it's the last enabled type
                    let axis1d_can_toggle = self
                        .filter
                        .can_disable_type(crate::core::log::InputKind::Axis1D);
                    ui.add_enabled_ui(axis1d_can_toggle, |ui| {
                        ui.checkbox(&mut self.filter.show_axis1d, "Axis1D");
                    });

                    // Axis2D type checkbox - prevent unchecking if it's the last enabled type
                    let axis2d_can_toggle = self
                        .filter
                        .can_disable_type(crate::core::log::InputKind::Axis2D);
                    ui.add_enabled_ui(axis2d_can_toggle, |ui| {
                        ui.checkbox(&mut self.filter.show_axis2d, "Axis2D");
                    });

                    ui.separator();

                    // Individual input checkboxes
                    ui.label("Inputs:");
                    egui::ScrollArea::vertical()
                        .max_height(250.0)
                        .show(ui, |ui| {
                            for mapping in &log.mappings {
                                let mut is_visible = self.filter.visible_ids.contains(&mapping.id);
                                let label = if mapping.name.is_empty() {
                                    format!("Input {}", mapping.id)
                                } else {
                                    mapping.name.clone()
                                };

                                // Show color indicator if available
                                ui.horizontal(|ui| {
                                    if let Some(color) = mapping.color {
                                        let color32 =
                                            egui::Color32::from_rgb(color[0], color[1], color[2]);
                                        let (rect, _) = ui.allocate_exact_size(
                                            egui::vec2(12.0, 12.0),
                                            egui::Sense::hover(),
                                        );
                                        ui.painter().rect_filled(rect, 2.0, color32);
                                    }
                                    if ui.checkbox(&mut is_visible, &label).changed() {
                                        self.filter.set_id_visible(mapping.id, is_visible);
                                    }
                                });
                            }
                        });

                    ui.separator();

                    // Select All / Deselect All buttons
                    ui.horizontal(|ui| {
                        if ui.button("All").clicked() {
                            self.filter.select_all(&log.mappings);
                        }
                        if ui.button("None").clicked() {
                            self.filter.deselect_all();
                        }
                    });
                } else {
                    ui.label("No file loaded");
                }
            });

        if should_close {
            self.filter_popup_open = false;
        }
    }

    /// Render the search dialog window.
    fn render_search_dialog(&mut self, ctx: &egui::Context) {
        let mut should_close = false;
        let mut seek_to_frame: Option<u64> = None;
        let mut perform_search = false;

        egui::Window::new("Search Input")
            .id(egui::Id::new("search_dialog"))
            .collapsible(false)
            .resizable(false)
            .default_width(280.0)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 60.0))
            .show(ctx, |ui| {
                // Close button
                ui.horizontal(|ui| {
                    ui.heading("Search Input");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕").clicked() {
                            should_close = true;
                        }
                    });
                });
                ui.separator();

                if let Some(ref log) = self.log {
                    // Input selector dropdown
                    ui.horizontal(|ui| {
                        ui.label("Input:");
                        ui.add_space(10.0);
                        let selected_name = if self.search.selected_input_index < log.mappings.len()
                        {
                            let mapping = &log.mappings[self.search.selected_input_index];
                            if mapping.name.is_empty() {
                                format!("Input {}", mapping.id)
                            } else {
                                mapping.name.clone()
                            }
                        } else {
                            "Select input...".to_string()
                        };

                        egui::ComboBox::from_id_salt("search_input_combo")
                            .selected_text(selected_name)
                            .width(160.0)
                            .show_ui(ui, |ui| {
                                for (i, mapping) in log.mappings.iter().enumerate() {
                                    let label = if mapping.name.is_empty() {
                                        format!("Input {}", mapping.id)
                                    } else {
                                        mapping.name.clone()
                                    };
                                    if ui
                                        .selectable_label(
                                            self.search.selected_input_index == i,
                                            &label,
                                        )
                                        .clicked()
                                    {
                                        self.search.selected_input_index = i;
                                        // Clear previous results when selection changes
                                        self.search.clear_results();
                                    }
                                }
                            });
                    });

                    ui.add_space(4.0);

                    // Get the kind of the selected input to determine if state filter applies
                    let selected_kind = if self.search.selected_input_index < log.mappings.len() {
                        let mapping_id = log.mappings[self.search.selected_input_index].id;
                        log.events
                            .iter()
                            .find(|e| e.id == mapping_id)
                            .map(|e| e.kind)
                            .unwrap_or(InputKind::Button)
                    } else {
                        InputKind::Button
                    };

                    // State selector dropdown (only for Button type)
                    let is_button = selected_kind == InputKind::Button;
                    ui.add_enabled_ui(is_button, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("State:");
                            ui.add_space(10.0);
                            let state_options = ["Any", "Pressed", "Released"];
                            let selected_state = state_options
                                .get(self.search.selected_state_index)
                                .unwrap_or(&"Any");

                            egui::ComboBox::from_id_salt("search_state_combo")
                                .selected_text(*selected_state)
                                .width(160.0)
                                .show_ui(ui, |ui| {
                                    for (i, state) in state_options.iter().enumerate() {
                                        if ui
                                            .selectable_label(
                                                self.search.selected_state_index == i,
                                                *state,
                                            )
                                            .clicked()
                                        {
                                            self.search.selected_state_index = i;
                                            // Clear previous results when selection changes
                                            self.search.clear_results();
                                        }
                                    }
                                });
                        });
                    });

                    ui.add_space(8.0);

                    // Search button
                    ui.horizontal(|ui| {
                        if ui.button("Search").clicked() {
                            perform_search = true;
                        }
                        if self.search.has_searched && ui.button("Clear").clicked() {
                            self.search.clear_results();
                        }
                    });

                    ui.separator();

                    // Results section
                    if self.search.has_searched {
                        if self.search.results.is_empty() {
                            ui.label("No matches found");
                        } else {
                            let count = self.search.results.count();
                            let current = self.search.results.current_position().unwrap_or(0);
                            ui.label(format!("Results: {} / {} matches", current, count));

                            ui.add_space(4.0);

                            // Navigation buttons
                            ui.horizontal(|ui| {
                                if ui.button("< Prev").clicked()
                                    && let Some(frame) = self.search.results.prev()
                                {
                                    seek_to_frame = Some(frame);
                                }
                                if ui.button("Next >").clicked()
                                    && let Some(frame) = self.search.results.next()
                                {
                                    seek_to_frame = Some(frame);
                                }
                            });

                            // Show current match frame
                            if let Some(frame) = self.search.results.current_frame() {
                                ui.add_space(4.0);
                                ui.label(format!("Current: Frame {}", frame));
                            }
                        }
                    }
                } else {
                    ui.label("No file loaded");
                }
            });

        if should_close {
            self.search.dialog_open = false;
            // Clear search results when closing dialog to remove highlights
            self.search.clear_results();
        }

        // Perform search if requested
        if perform_search {
            self.perform_search();
        }

        // Seek to frame if navigation was used
        if let Some(frame) = seek_to_frame {
            let total_frames = self
                .log
                .as_ref()
                .map(|l| l.metadata.frame_count)
                .unwrap_or(0);
            self.playback.set_frame(frame, total_frames);
        }
    }

    /// Execute the search based on current search state.
    fn perform_search(&mut self) {
        if let Some(ref log) = self.log {
            if self.search.selected_input_index >= log.mappings.len() {
                return;
            }

            let mapping = &log.mappings[self.search.selected_input_index];
            let input_id = mapping.id;

            // Build the search query
            let mut query = SearchQuery::with_input_id(input_id);

            // Add button state filter if applicable
            if self.search.selected_state_index > 0 {
                let state = match self.search.selected_state_index {
                    1 => Some(ButtonState::Pressed),
                    2 => Some(ButtonState::Released),
                    _ => None,
                };
                if let Some(s) = state {
                    query = query.button_state(s);
                }
            }

            // Execute the search
            let matches = find_matches(log, &query);
            self.search.results = SearchResult::from_matches(matches);
            self.search.has_searched = true;

            // Navigate to the first result closest to current frame
            if !self.search.results.is_empty() {
                self.search
                    .results
                    .set_closest_to_frame(self.playback.current_frame);
                if let Some(frame) = self.search.results.current_frame() {
                    let total_frames = log.metadata.frame_count;
                    self.playback.set_frame(frame, total_frames);
                }
            }
        }
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
        let total_frames = self
            .log
            .as_ref()
            .map(|l| l.metadata.frame_count)
            .unwrap_or(0);

        // Capture action from controls renderer
        let mut action: Option<ControlAction> = None;
        let mut frame_input = self.frame_input_value;

        egui::TopBottomPanel::bottom("controls")
            .min_height(80.0)
            .show(ctx, |ui| {
                let mut renderer = ControlsRenderer::new(
                    controls_enabled,
                    is_playing,
                    &self.playback,
                    total_frames,
                    &mut frame_input,
                );
                action = renderer.render(ui);
            });

        self.frame_input_value = frame_input;

        // Handle control actions
        if let Some(action) = action {
            self.handle_control_action(action, total_frames);
        }
    }

    /// Handle a control action triggered by user interaction.
    fn handle_control_action(&mut self, action: ControlAction, total_frames: u64) {
        match action {
            ControlAction::TogglePlayPause => {
                if self.state.is_playing() {
                    // Switch to Ready (pause)
                    self.state = AppState::Ready;
                } else if self.state == AppState::Ready {
                    // Switch to Playing (play)
                    self.playback.reset_timing();
                    self.state = AppState::Playing;
                }
            }
            ControlAction::GoToStart => {
                self.playback.go_to_start();
            }
            ControlAction::PreviousFrame => {
                self.playback.previous(total_frames);
            }
            ControlAction::NextFrame => {
                let _ = self.playback.advance(total_frames);
            }
            ControlAction::GoToEnd => {
                self.playback.go_to_end(total_frames);
            }
            ControlAction::SetSpeed(speed) => {
                self.playback.set_speed(speed);
            }
            ControlAction::SeekToFrame(frame) => {
                self.playback.set_frame(frame, total_frames);
            }
        }
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

            // Render the timeline using TimelineRenderer with filter and search results
            let mut renderer = TimelineRenderer::new(log, &self.timeline_config, &self.filter);
            if self.search.has_searched && !self.search.results.is_empty() {
                renderer = renderer.with_search_results(&self.search.results);
            }
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

//! Main application state and GUI logic.
//!
//! This module defines the main application struct and implements the eframe::App trait
//! to provide the core GUI functionality for the input log viewer.

use eframe::egui;

/// Main application state and GUI logic.
pub struct InputLogViewerApp {
    // Placeholder for future state fields
    // Will be expanded in later phases to include:
    // - AppState (loading state)
    // - InputLog data
    // - PlaybackState
    // - ViewState
    // - Bookmarks
    // - Config
}

impl InputLogViewerApp {
    /// Create a new application instance.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {}
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
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Input Log Viewer");
                ui.separator();

                // File loading button placeholder
                if ui.button("📂 Open File").clicked() {
                    // TODO: Implement file dialog in Phase 2
                }

                ui.separator();

                // Filter dropdown placeholder
                ui.label("Filter:");
                egui::ComboBox::from_id_salt("filter_combo")
                    .selected_text("All Inputs")
                    .show_ui(ui, |ui| {
                        let _ = ui.selectable_label(true, "All Inputs");
                        let _ = ui.selectable_label(false, "Buttons Only");
                        let _ = ui.selectable_label(false, "Axes Only");
                    });

                ui.separator();

                // Search button placeholder
                if ui.button("🔍 Search").clicked() {
                    // TODO: Implement search dialog in Phase 3
                }
            });
        });
    }

    /// Render the bottom controls section.
    ///
    /// Contains playback controls, frame navigation, and speed settings.
    fn render_controls(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("controls")
            .min_height(80.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Playback controls row
                    ui.horizontal(|ui| {
                        // Navigation buttons
                        if ui.button("⏮").on_hover_text("Go to start").clicked() {
                            // TODO: Implement in Phase 2
                        }
                        if ui.button("⏪").on_hover_text("Previous frame").clicked() {
                            // TODO: Implement in Phase 2
                        }
                        if ui.button("▶").on_hover_text("Play/Pause").clicked() {
                            // TODO: Implement in Phase 2
                        }
                        if ui.button("⏩").on_hover_text("Next frame").clicked() {
                            // TODO: Implement in Phase 2
                        }
                        if ui.button("⏭").on_hover_text("Go to end").clicked() {
                            // TODO: Implement in Phase 2
                        }

                        ui.separator();

                        // Frame counter
                        ui.label("Frame: 0 / 0");

                        ui.separator();

                        // Speed control
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

                    ui.add_space(4.0);

                    // Timeline scrubber row
                    ui.horizontal(|ui| {
                        let mut frame: f32 = 0.0;
                        ui.add(
                            egui::Slider::new(&mut frame, 0.0..=100.0)
                                .show_value(false)
                                .text(""),
                        );
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
            // Timeline header
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                // Show placeholder content when no file is loaded
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

                // Draw placeholder timeline grid
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
                painter.text(
                    egui::pos2(rect.left() + 5.0, rect.top() + row_height * 0.5),
                    egui::Align2::LEFT_CENTER,
                    "A Button",
                    egui::FontId::default(),
                    label_color,
                );
                painter.text(
                    egui::pos2(rect.left() + 5.0, rect.top() + row_height * 1.5),
                    egui::Align2::LEFT_CENTER,
                    "B Button",
                    egui::FontId::default(),
                    label_color,
                );
                painter.text(
                    egui::pos2(rect.left() + 5.0, rect.top() + row_height * 2.5),
                    egui::Align2::LEFT_CENTER,
                    "X Button",
                    egui::FontId::default(),
                    label_color,
                );
                painter.text(
                    egui::pos2(rect.left() + 5.0, rect.top() + row_height * 3.5),
                    egui::Align2::LEFT_CENTER,
                    "Left Stick X",
                    egui::FontId::default(),
                    label_color,
                );
                painter.text(
                    egui::pos2(rect.left() + 5.0, rect.top() + row_height * 4.5),
                    egui::Align2::LEFT_CENTER,
                    "Left Stick Y",
                    egui::FontId::default(),
                    label_color,
                );

                // Draw center text
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "(Timeline will appear here)",
                    egui::FontId::proportional(16.0),
                    egui::Color32::DARK_GRAY,
                );
            });
        });
    }
}

//! GUI module for the input log viewer.
//!
//! This module contains the egui-based user interface components.

use eframe::egui;

/// Main application state and GUI logic.
pub struct InputLogViewerApp {
    // Placeholder for future state
}

impl InputLogViewerApp {
    /// Create a new application instance.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {}
    }
}

impl eframe::App for InputLogViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Input Log Viewer");
            ui.label("Drag and drop an input log file (.ilj or .ilb) to get started.");
        });
    }
}

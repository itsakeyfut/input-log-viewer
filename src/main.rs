//! Input Log Viewer - A lightweight input log viewer for game developers.

mod core;
mod gui;

use gui::InputLogViewerApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([640.0, 480.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "Input Log Viewer",
        options,
        Box::new(|cc| Ok(Box::new(InputLogViewerApp::new(cc)))),
    )
}

//! GUI module for the input log viewer.
//!
//! This module contains the egui-based user interface components
//! including the main application window, toolbar, timeline, and controls.

mod app;
mod controls;
mod dialogs;
mod timeline;

pub use app::InputLogViewerApp;

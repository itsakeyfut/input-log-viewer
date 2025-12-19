//! Dialog components for the input log viewer.
//!
//! This module provides reusable dialog components for displaying
//! errors, confirmations, and other user interactions.

// Allow dead code for utility methods that are designed for future use
#![allow(dead_code)]

use eframe::egui;

use crate::core::error::AppError;

/// Actions that can be triggered from the error dialog.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorDialogAction {
    /// User wants to retry the failed operation
    Retry,
    /// User dismissed the dialog
    Close,
}

/// State for managing the error dialog.
#[derive(Debug, Clone, Default)]
pub struct ErrorDialogState {
    /// Whether the dialog is currently open
    pub is_open: bool,
    /// The error to display, if any
    pub error: Option<AppError>,
    /// Whether the error details are expanded
    pub details_expanded: bool,
    /// Feedback message for clipboard operations
    pub clipboard_feedback: Option<ClipboardFeedback>,
}

/// Feedback for clipboard copy operation.
#[derive(Debug, Clone)]
pub struct ClipboardFeedback {
    /// Message to display
    pub message: String,
    /// When the feedback was created
    pub created_at: std::time::Instant,
    /// Whether the operation was successful
    pub success: bool,
}

impl ClipboardFeedback {
    /// Create a success feedback.
    pub fn success() -> Self {
        Self {
            message: "Copied to clipboard!".to_string(),
            created_at: std::time::Instant::now(),
            success: true,
        }
    }

    /// Create a failure feedback.
    pub fn failure(reason: &str) -> Self {
        Self {
            message: format!("Copy failed: {}", reason),
            created_at: std::time::Instant::now(),
            success: false,
        }
    }

    /// Check if the feedback should still be visible (auto-dismiss after 2 seconds).
    pub fn is_visible(&self) -> bool {
        self.created_at.elapsed() < std::time::Duration::from_secs(2)
    }
}

impl ErrorDialogState {
    /// Create a new error dialog state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the error dialog with the given error.
    pub fn show(&mut self, error: AppError) {
        self.is_open = true;
        self.error = Some(error);
        self.details_expanded = false;
        self.clipboard_feedback = None;
    }

    /// Close the error dialog.
    pub fn close(&mut self) {
        self.is_open = false;
        self.error = None;
        self.details_expanded = false;
        self.clipboard_feedback = None;
    }

    /// Check if the dialog is showing an error.
    pub fn has_error(&self) -> bool {
        self.is_open && self.error.is_some()
    }
}

/// Renderer for the error dialog.
pub struct ErrorDialogRenderer<'a> {
    state: &'a mut ErrorDialogState,
}

impl<'a> ErrorDialogRenderer<'a> {
    /// Create a new error dialog renderer.
    pub fn new(state: &'a mut ErrorDialogState) -> Self {
        Self { state }
    }

    /// Render the error dialog and return the action taken.
    ///
    /// Returns `Some(action)` if the user clicked a button, `None` otherwise.
    pub fn render(&mut self, ctx: &egui::Context) -> Option<ErrorDialogAction> {
        // Don't render if not open or no error
        if !self.state.is_open || self.state.error.is_none() {
            return None;
        }

        let mut action: Option<ErrorDialogAction> = None;
        let mut should_close = false;

        // Clone what we need from the error to avoid borrow issues
        let error = self.state.error.as_ref().unwrap();
        let title = error.dialog_title();
        let brief = error.brief_description();
        let detailed = error.detailed_info();
        let supports_retry = error.supports_retry();
        let is_recoverable = error.is_recoverable();

        // Create modal overlay
        let screen_rect = ctx.input(|i| i.viewport_rect());
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Background,
            egui::Id::new("error_dialog_overlay"),
        ));
        painter.rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(150));

        egui::Window::new(format!("⚠ {}", title))
            .id(egui::Id::new("error_dialog"))
            .collapsible(false)
            .resizable(true)
            .default_width(450.0)
            .min_width(350.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.add_space(8.0);

                // Brief description
                ui.label(egui::RichText::new(&brief).size(14.0));

                ui.add_space(12.0);

                // Details section (collapsible)
                let details_header = if self.state.details_expanded {
                    "▼ Details"
                } else {
                    "▶ Details"
                };

                if ui
                    .add(egui::Button::new(details_header).frame(false))
                    .clicked()
                {
                    self.state.details_expanded = !self.state.details_expanded;
                }

                if self.state.details_expanded {
                    ui.add_space(4.0);
                    egui::Frame::new()
                        .fill(egui::Color32::from_gray(30))
                        .inner_margin(8.0)
                        .corner_radius(4.0)
                        .show(ui, |ui| {
                            ui.style_mut().override_font_id = Some(egui::FontId::monospace(12.0));
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&detailed).color(egui::Color32::LIGHT_GRAY),
                                )
                                .wrap(),
                            );
                        });
                }

                ui.add_space(12.0);

                // Clipboard feedback
                if let Some(ref feedback) = self.state.clipboard_feedback
                    && feedback.is_visible()
                {
                    let color = if feedback.success {
                        egui::Color32::from_rgb(76, 175, 80) // Green
                    } else {
                        egui::Color32::from_rgb(244, 67, 54) // Red
                    };
                    ui.colored_label(color, &feedback.message);
                    ui.add_space(4.0);
                    ctx.request_repaint(); // Keep updating to check visibility
                }

                // Clean up expired feedback
                if self
                    .state
                    .clipboard_feedback
                    .as_ref()
                    .is_some_and(|f| !f.is_visible())
                {
                    self.state.clipboard_feedback = None;
                }

                ui.separator();

                // Action buttons
                ui.horizontal(|ui| {
                    // Copy Error button
                    if ui
                        .button("📋 Copy Error")
                        .on_hover_text("Copy error details to clipboard")
                        .clicked()
                    {
                        ui.ctx().copy_text(detailed.clone());
                        self.state.clipboard_feedback = Some(ClipboardFeedback::success());
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Close button (always available for recoverable errors)
                        if is_recoverable && ui.button("Close").clicked() {
                            should_close = true;
                            action = Some(ErrorDialogAction::Close);
                        }

                        // Retry button (only for errors that support retry)
                        if supports_retry
                            && ui
                                .button("🔄 Retry")
                                .on_hover_text("Try loading the file again")
                                .clicked()
                        {
                            should_close = true;
                            action = Some(ErrorDialogAction::Retry);
                        }
                    });
                });

                ui.add_space(4.0);
            });

        if should_close {
            self.state.close();
        }

        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_error_dialog_state_new() {
        let state = ErrorDialogState::new();
        assert!(!state.is_open);
        assert!(state.error.is_none());
        assert!(!state.details_expanded);
    }

    #[test]
    fn test_error_dialog_state_show() {
        let mut state = ErrorDialogState::new();
        let error = AppError::FileNotFound {
            path: PathBuf::from("/test/file.ilj"),
        };

        state.show(error.clone());
        assert!(state.is_open);
        assert!(state.error.is_some());
        assert!(!state.details_expanded);
        assert!(state.has_error());
    }

    #[test]
    fn test_error_dialog_state_close() {
        let mut state = ErrorDialogState::new();
        state.show(AppError::FileNotFound {
            path: PathBuf::from("/test/file.ilj"),
        });
        state.close();

        assert!(!state.is_open);
        assert!(state.error.is_none());
        assert!(!state.has_error());
    }

    #[test]
    fn test_clipboard_feedback_visibility() {
        let feedback = ClipboardFeedback::success();
        assert!(feedback.is_visible());

        // Note: We can't easily test the timeout without sleeping,
        // but we verify the initial state is visible
    }

    #[test]
    fn test_clipboard_feedback_failure() {
        let feedback = ClipboardFeedback::failure("test error");
        assert!(!feedback.success);
        assert!(feedback.message.contains("test error"));
    }
}

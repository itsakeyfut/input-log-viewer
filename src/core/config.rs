//! Configuration and settings module.
//!
//! This module handles persistent settings including color customization,
//! saving/loading configuration to disk, and default values.

use eframe::egui::Color32;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Settings filename for persistence.
const SETTINGS_FILENAME: &str = "settings.json";

/// Color settings for the application UI.
///
/// All colors can be customized by the user and are persisted to disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorSettings {
    // Button state colors
    /// Color for pressed button state (default: green)
    pub button_pressed: [u8; 3],
    /// Color for held button state (default: light gray)
    pub button_held: [u8; 3],
    /// Color for released button state (default: red)
    pub button_released: [u8; 3],

    // Input type colors
    /// Default color for Axis1D inputs (when not specified in log)
    pub axis1d: [u8; 3],
    /// Default color for Axis2D inputs (when not specified in log)
    pub axis2d: [u8; 3],

    // Highlight colors
    /// Current frame indicator color
    pub current_frame: [u8; 3],
    /// Selection highlight color
    pub selection: [u8; 3],
    /// Bookmark marker color
    pub bookmark: [u8; 3],

    // Search result colors
    /// Current search match highlight color
    pub search_current: [u8; 3],
    /// Other search matches highlight color
    pub search_other: [u8; 3],

    // Background colors
    /// Main timeline background color
    pub background: [u8; 3],
    /// Header background color
    pub header_background: [u8; 3],
    /// Label column background color
    pub label_background: [u8; 3],

    // Grid colors
    /// Grid line color
    pub grid: [u8; 3],
    /// Axis center line color
    pub axis_center: [u8; 3],

    // Scrollbar colors
    /// Scrollbar track color
    pub scrollbar_track: [u8; 3],
    /// Scrollbar thumb color
    pub scrollbar_thumb: [u8; 3],
    /// Scrollbar thumb border color
    pub scrollbar_border: [u8; 3],

    // Text colors
    /// Header text color
    pub text_header: [u8; 3],
    /// Label text color
    pub text_label: [u8; 3],
    /// Placeholder/inactive text color
    pub text_dim: [u8; 3],

    // Status colors
    /// Success status color
    pub status_success: [u8; 3],
    /// Error status color
    pub status_error: [u8; 3],

    // Control colors
    /// Auto-scroll enabled indicator color
    pub auto_scroll_enabled: [u8; 3],
    /// Loop selection enabled indicator color
    pub loop_enabled: [u8; 3],
}

impl Default for ColorSettings {
    fn default() -> Self {
        Self {
            // Button state colors
            button_pressed: [76, 175, 80],  // Green
            button_held: [211, 211, 211],   // Light gray
            button_released: [244, 67, 54], // Red

            // Input type colors
            axis1d: [100, 150, 200], // Light blue
            axis2d: [150, 100, 200], // Purple

            // Highlight colors
            current_frame: [255, 200, 100], // Orange/gold
            selection: [150, 80, 200],      // Purple
            bookmark: [255, 215, 0],        // Gold

            // Search result colors
            search_current: [100, 200, 255], // Cyan
            search_other: [255, 255, 100],   // Yellow

            // Background colors
            background: [30, 30, 35],        // Dark gray
            header_background: [40, 40, 45], // Slightly lighter
            label_background: [35, 35, 40],  // Between background and header

            // Grid colors
            grid: [50, 50, 55],        // Medium gray
            axis_center: [60, 60, 65], // Slightly lighter

            // Scrollbar colors
            scrollbar_track: [40, 40, 45],     // Same as header
            scrollbar_thumb: [80, 80, 90],     // Lighter gray
            scrollbar_border: [100, 100, 110], // Light border

            // Text colors
            text_header: [128, 128, 128], // Gray
            text_label: [211, 211, 211],  // Light gray
            text_dim: [105, 105, 105],    // Dark gray

            // Status colors
            status_success: [76, 175, 80], // Green
            status_error: [244, 67, 54],   // Red

            // Control colors
            auto_scroll_enabled: [100, 200, 100], // Light green
            loop_enabled: [180, 100, 220],        // Purple
        }
    }
}

impl ColorSettings {
    /// Convert a color array to egui Color32.
    #[inline]
    pub fn to_color32(color: [u8; 3]) -> Color32 {
        Color32::from_rgb(color[0], color[1], color[2])
    }

    /// Convert a color array to egui Color32 with alpha.
    #[inline]
    pub fn to_color32_alpha(color: [u8; 3], alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(color[0], color[1], color[2], alpha)
    }

    // Convenience methods to get Color32 values directly

    /// Get button pressed color as Color32.
    pub fn button_pressed_color(&self) -> Color32 {
        Self::to_color32(self.button_pressed)
    }

    /// Get button held color as Color32.
    pub fn button_held_color(&self) -> Color32 {
        Self::to_color32(self.button_held)
    }

    /// Get button released color as Color32.
    pub fn button_released_color(&self) -> Color32 {
        Self::to_color32(self.button_released)
    }

    /// Get current frame indicator color as Color32.
    pub fn current_frame_color(&self) -> Color32 {
        Self::to_color32(self.current_frame)
    }

    /// Get current frame indicator color with alpha.
    pub fn current_frame_color_alpha(&self, alpha: u8) -> Color32 {
        Self::to_color32_alpha(self.current_frame, alpha)
    }

    /// Get selection color as Color32.
    pub fn selection_color(&self) -> Color32 {
        Self::to_color32(self.selection)
    }

    /// Get selection color with alpha.
    pub fn selection_color_alpha(&self, alpha: u8) -> Color32 {
        Self::to_color32_alpha(self.selection, alpha)
    }

    /// Get bookmark color as Color32.
    pub fn bookmark_color(&self) -> Color32 {
        Self::to_color32(self.bookmark)
    }

    /// Get bookmark color with alpha.
    pub fn bookmark_color_alpha(&self, alpha: u8) -> Color32 {
        Self::to_color32_alpha(self.bookmark, alpha)
    }

    /// Get current search match color as Color32.
    pub fn search_current_color(&self) -> Color32 {
        Self::to_color32(self.search_current)
    }

    /// Get current search match color with alpha.
    pub fn search_current_color_alpha(&self, alpha: u8) -> Color32 {
        Self::to_color32_alpha(self.search_current, alpha)
    }

    /// Get other search matches color with alpha.
    pub fn search_other_color_alpha(&self, alpha: u8) -> Color32 {
        Self::to_color32_alpha(self.search_other, alpha)
    }

    /// Get background color as Color32.
    pub fn background_color(&self) -> Color32 {
        Self::to_color32(self.background)
    }

    /// Get header background color as Color32.
    pub fn header_background_color(&self) -> Color32 {
        Self::to_color32(self.header_background)
    }

    /// Get label background color as Color32.
    pub fn label_background_color(&self) -> Color32 {
        Self::to_color32(self.label_background)
    }

    /// Get grid color as Color32.
    pub fn grid_color(&self) -> Color32 {
        Self::to_color32(self.grid)
    }

    /// Get axis center line color as Color32.
    pub fn axis_center_color(&self) -> Color32 {
        Self::to_color32(self.axis_center)
    }

    /// Get axis1d color as Color32.
    pub fn axis1d_color(&self) -> Color32 {
        Self::to_color32(self.axis1d)
    }

    /// Get axis2d color as Color32.
    pub fn axis2d_color(&self) -> Color32 {
        Self::to_color32(self.axis2d)
    }

    /// Get scrollbar track color as Color32.
    pub fn scrollbar_track_color(&self) -> Color32 {
        Self::to_color32(self.scrollbar_track)
    }

    /// Get scrollbar thumb color as Color32.
    pub fn scrollbar_thumb_color(&self) -> Color32 {
        Self::to_color32(self.scrollbar_thumb)
    }

    /// Get scrollbar border color as Color32.
    pub fn scrollbar_border_color(&self) -> Color32 {
        Self::to_color32(self.scrollbar_border)
    }

    /// Get header text color as Color32.
    pub fn text_header_color(&self) -> Color32 {
        Self::to_color32(self.text_header)
    }

    /// Get label text color as Color32.
    pub fn text_label_color(&self) -> Color32 {
        Self::to_color32(self.text_label)
    }

    /// Get dim text color as Color32.
    pub fn text_dim_color(&self) -> Color32 {
        Self::to_color32(self.text_dim)
    }

    /// Get success status color as Color32.
    pub fn status_success_color(&self) -> Color32 {
        Self::to_color32(self.status_success)
    }

    /// Get error status color as Color32.
    pub fn status_error_color(&self) -> Color32 {
        Self::to_color32(self.status_error)
    }

    /// Get auto-scroll enabled color as Color32.
    pub fn auto_scroll_enabled_color(&self) -> Color32 {
        Self::to_color32(self.auto_scroll_enabled)
    }

    /// Get loop enabled color as Color32.
    pub fn loop_enabled_color(&self) -> Color32 {
        Self::to_color32(self.loop_enabled)
    }
}

/// Application settings including color customization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppSettings {
    /// Color customization settings
    pub colors: ColorSettings,
}

impl AppSettings {
    /// Get the settings file path in the user's config directory.
    fn get_settings_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut path| {
            path.push("input-log-viewer");
            path.push(SETTINGS_FILENAME);
            path
        })
    }

    /// Load settings from disk, returning defaults if loading fails.
    pub fn load() -> Self {
        Self::get_settings_path()
            .and_then(|path| std::fs::read_to_string(&path).ok())
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// Save settings to disk.
    ///
    /// Returns an error if saving fails.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_settings_path()
            .ok_or_else(|| "Could not determine config directory".to_string())?;

        // Ensure the parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        std::fs::write(&path, content)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        Ok(())
    }

    /// Reset all settings to defaults.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_settings_default() {
        let settings = ColorSettings::default();
        // Button states have expected colors
        assert_eq!(settings.button_pressed, [76, 175, 80]);
        assert_eq!(settings.button_released, [244, 67, 54]);
    }

    #[test]
    fn test_color32_conversion() {
        let color = [255, 128, 64];
        let color32 = ColorSettings::to_color32(color);
        assert_eq!(color32, Color32::from_rgb(255, 128, 64));
    }

    #[test]
    fn test_color32_alpha_conversion() {
        let color = [255, 128, 64];
        let color32 = ColorSettings::to_color32_alpha(color, 100);
        assert_eq!(color32, Color32::from_rgba_unmultiplied(255, 128, 64, 100));
    }

    #[test]
    fn test_color_settings_methods() {
        let settings = ColorSettings::default();

        // Test color methods return correct Color32 values
        assert_eq!(
            settings.button_pressed_color(),
            Color32::from_rgb(76, 175, 80)
        );
        assert_eq!(
            settings.button_released_color(),
            Color32::from_rgb(244, 67, 54)
        );
        assert_eq!(settings.bookmark_color(), Color32::from_rgb(255, 215, 0));
    }

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert_eq!(settings.colors, ColorSettings::default());
    }

    #[test]
    fn test_app_settings_reset() {
        let mut settings = AppSettings::default();
        settings.colors.button_pressed = [0, 0, 0];
        settings.reset();
        assert_eq!(settings.colors.button_pressed, [76, 175, 80]);
    }

    #[test]
    fn test_settings_serialization() {
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let restored: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings.colors, restored.colors);
    }
}

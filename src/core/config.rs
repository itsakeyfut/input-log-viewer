//! Configuration and settings module.
//!
//! This module handles persistent settings including color customization,
//! saving/loading configuration to disk, and default values.

use eframe::egui::Color32;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Settings filename for persistence.
const SETTINGS_FILENAME: &str = "config.json";

/// Maximum number of recent files to track.
const MAX_RECENT_FILES: usize = 10;

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

/// Application settings including color customization and user preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Color customization settings.
    pub colors: ColorSettings,

    /// Default playback speed (0.1 to 10.0).
    #[serde(default = "default_speed")]
    pub default_speed: f32,

    /// Whether loop playback is enabled by default.
    #[serde(default)]
    pub loop_enabled: bool,

    /// Recently opened files (most recent first).
    #[serde(default)]
    pub recent_files: Vec<PathBuf>,

    /// Window size to restore on startup (width, height).
    #[serde(default)]
    pub window_size: Option<(f32, f32)>,
}

/// Default playback speed.
fn default_speed() -> f32 {
    1.0
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            colors: ColorSettings::default(),
            default_speed: default_speed(),
            loop_enabled: false,
            recent_files: Vec::new(),
            window_size: None,
        }
    }
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

    /// Add a file to the recent files list.
    ///
    /// The file is moved to the front of the list. If it already exists, it is
    /// moved to the front. The list is capped at MAX_RECENT_FILES entries.
    pub fn add_recent_file(&mut self, path: PathBuf) {
        // Remove the path if it already exists (to move it to front)
        self.recent_files.retain(|p| p != &path);

        // Add to the front
        self.recent_files.insert(0, path);

        // Cap the list size
        self.recent_files.truncate(MAX_RECENT_FILES);
    }

    /// Clear the recent files list.
    pub fn clear_recent_files(&mut self) {
        self.recent_files.clear();
    }

    /// Get the default playback speed, clamped to valid range.
    pub fn get_default_speed(&self) -> f32 {
        self.default_speed.clamp(0.1, 10.0)
    }

    /// Set the default playback speed.
    pub fn set_default_speed(&mut self, speed: f32) {
        self.default_speed = speed.clamp(0.1, 10.0);
    }

    /// Set the window size.
    pub fn set_window_size(&mut self, width: f32, height: f32) {
        self.window_size = Some((width, height));
    }

    /// Get the config file path for display purposes.
    pub fn get_config_path() -> Option<PathBuf> {
        Self::get_settings_path()
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
        assert_eq!(settings.default_speed, 1.0);
        assert!(!settings.loop_enabled);
        assert!(settings.recent_files.is_empty());
        assert!(settings.window_size.is_none());
    }

    #[test]
    fn test_app_settings_reset() {
        let mut settings = AppSettings::default();
        settings.colors.button_pressed = [0, 0, 0];
        settings.default_speed = 2.0;
        settings.loop_enabled = true;
        settings.recent_files.push(PathBuf::from("/test/file.ilj"));
        settings.window_size = Some((800.0, 600.0));

        settings.reset();

        assert_eq!(settings.colors.button_pressed, [76, 175, 80]);
        assert_eq!(settings.default_speed, 1.0);
        assert!(!settings.loop_enabled);
        assert!(settings.recent_files.is_empty());
        assert!(settings.window_size.is_none());
    }

    #[test]
    fn test_settings_serialization() {
        let mut settings = AppSettings::default();
        settings.default_speed = 2.5;
        settings.loop_enabled = true;
        settings.recent_files.push(PathBuf::from("/test/file.ilj"));
        settings.window_size = Some((1024.0, 768.0));

        let json = serde_json::to_string(&settings).unwrap();
        let restored: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.colors, restored.colors);
        assert_eq!(restored.default_speed, 2.5);
        assert!(restored.loop_enabled);
        assert_eq!(restored.recent_files.len(), 1);
        assert_eq!(restored.window_size, Some((1024.0, 768.0)));
    }

    #[test]
    fn test_recent_files_add() {
        let mut settings = AppSettings::default();

        // Add first file
        settings.add_recent_file(PathBuf::from("/test/file1.ilj"));
        assert_eq!(settings.recent_files.len(), 1);
        assert_eq!(settings.recent_files[0], PathBuf::from("/test/file1.ilj"));

        // Add second file (should be at front)
        settings.add_recent_file(PathBuf::from("/test/file2.ilj"));
        assert_eq!(settings.recent_files.len(), 2);
        assert_eq!(settings.recent_files[0], PathBuf::from("/test/file2.ilj"));
        assert_eq!(settings.recent_files[1], PathBuf::from("/test/file1.ilj"));

        // Re-add first file (should move to front)
        settings.add_recent_file(PathBuf::from("/test/file1.ilj"));
        assert_eq!(settings.recent_files.len(), 2);
        assert_eq!(settings.recent_files[0], PathBuf::from("/test/file1.ilj"));
        assert_eq!(settings.recent_files[1], PathBuf::from("/test/file2.ilj"));
    }

    #[test]
    fn test_recent_files_max_limit() {
        let mut settings = AppSettings::default();

        // Add more than MAX_RECENT_FILES files
        for i in 0..15 {
            settings.add_recent_file(PathBuf::from(format!("/test/file{}.ilj", i)));
        }

        // Should be capped at MAX_RECENT_FILES (10)
        assert_eq!(settings.recent_files.len(), MAX_RECENT_FILES);
        // Most recent should be at front
        assert_eq!(settings.recent_files[0], PathBuf::from("/test/file14.ilj"));
    }

    #[test]
    fn test_recent_files_clear() {
        let mut settings = AppSettings::default();
        settings.add_recent_file(PathBuf::from("/test/file1.ilj"));
        settings.add_recent_file(PathBuf::from("/test/file2.ilj"));

        settings.clear_recent_files();

        assert!(settings.recent_files.is_empty());
    }

    #[test]
    fn test_default_speed_clamping() {
        let mut settings = AppSettings::default();

        // Set speed within range
        settings.set_default_speed(2.0);
        assert_eq!(settings.get_default_speed(), 2.0);

        // Set speed below minimum
        settings.set_default_speed(0.01);
        assert_eq!(settings.get_default_speed(), 0.1);

        // Set speed above maximum
        settings.set_default_speed(100.0);
        assert_eq!(settings.get_default_speed(), 10.0);
    }

    #[test]
    fn test_window_size() {
        let mut settings = AppSettings::default();
        assert!(settings.window_size.is_none());

        settings.set_window_size(1280.0, 720.0);
        assert_eq!(settings.window_size, Some((1280.0, 720.0)));
    }

    #[test]
    fn test_backward_compatible_deserialization() {
        // Test that old config files (without new fields) can still be loaded
        let old_json = r#"{"colors":{"button_pressed":[76,175,80],"button_held":[211,211,211],"button_released":[244,67,54],"axis1d":[100,150,200],"axis2d":[150,100,200],"current_frame":[255,200,100],"selection":[150,80,200],"bookmark":[255,215,0],"search_current":[100,200,255],"search_other":[255,255,100],"background":[30,30,35],"header_background":[40,40,45],"label_background":[35,35,40],"grid":[50,50,55],"axis_center":[60,60,65],"scrollbar_track":[40,40,45],"scrollbar_thumb":[80,80,90],"scrollbar_border":[100,100,110],"text_header":[128,128,128],"text_label":[211,211,211],"text_dim":[105,105,105],"status_success":[76,175,80],"status_error":[244,67,54],"auto_scroll_enabled":[100,200,100],"loop_enabled":[180,100,220]}}"#;

        let settings: AppSettings = serde_json::from_str(old_json).unwrap();

        // New fields should have defaults
        assert_eq!(settings.default_speed, 1.0);
        assert!(!settings.loop_enabled);
        assert!(settings.recent_files.is_empty());
        assert!(settings.window_size.is_none());
    }
}

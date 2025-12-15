//! Timeline rendering module.
//!
//! This module handles the visualization of input events over frames,
//! including drawing the frame grid, input rows, and event representations.

use eframe::egui::{self, Color32, Painter, Pos2, Rect, Stroke};
use std::collections::HashMap;

use crate::core::log::{ButtonState, InputEvent, InputKind, InputLog, InputMapping};

/// Default number of visible frames in the timeline.
pub const DEFAULT_VISIBLE_FRAMES: u64 = 100;

/// Height of each input row in pixels.
const ROW_HEIGHT: f32 = 32.0;

/// Width of the label column on the left side.
const LABEL_WIDTH: f32 = 120.0;

/// Padding inside cells.
const CELL_PADDING: f32 = 2.0;

/// Height of the frame number header.
const HEADER_HEIGHT: f32 = 20.0;

/// Configuration for timeline rendering.
pub struct TimelineConfig {
    /// First visible frame (for scrolling)
    pub scroll_offset: u64,
    /// Number of frames to display
    pub visible_frames: u64,
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self {
            scroll_offset: 0,
            visible_frames: DEFAULT_VISIBLE_FRAMES,
        }
    }
}

/// Timeline renderer that draws input events over frames.
pub struct TimelineRenderer<'a> {
    /// The input log to render
    log: &'a InputLog,
    /// Rendering configuration
    config: &'a TimelineConfig,
    /// Map from input ID to row index
    id_to_row: HashMap<u32, usize>,
    /// Map from input ID to its mapping (for name and color)
    id_to_mapping: HashMap<u32, &'a InputMapping>,
}

impl<'a> TimelineRenderer<'a> {
    /// Create a new timeline renderer for the given log.
    pub fn new(log: &'a InputLog, config: &'a TimelineConfig) -> Self {
        // Build ID to row mapping based on mappings order
        let id_to_row: HashMap<u32, usize> = log
            .mappings
            .iter()
            .enumerate()
            .map(|(i, m)| (m.id, i))
            .collect();

        // Build ID to mapping lookup
        let id_to_mapping: HashMap<u32, &InputMapping> =
            log.mappings.iter().map(|m| (m.id, m)).collect();

        Self {
            log,
            config,
            id_to_row,
            id_to_mapping,
        }
    }

    /// Get the color for an input ID, or a default color if not mapped.
    fn get_color(&self, id: u32) -> Color32 {
        self.id_to_mapping
            .get(&id)
            .and_then(|m| m.color)
            .map(|c| Color32::from_rgb(c[0], c[1], c[2]))
            .unwrap_or(Color32::LIGHT_GRAY)
    }

    /// Get the row index for an input ID.
    fn get_row(&self, id: u32) -> Option<usize> {
        self.id_to_row.get(&id).copied()
    }

    /// Calculate the total height needed for the timeline.
    pub fn calculate_height(&self) -> f32 {
        let num_rows = self.log.mappings.len().max(1);
        HEADER_HEIGHT + (num_rows as f32 * ROW_HEIGHT)
    }

    /// Render the complete timeline.
    pub fn render(&self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let num_rows = self.log.mappings.len().max(1);
        let grid_height = self.calculate_height().min(available_size.y - 10.0);

        let (response, painter) = ui.allocate_painter(
            egui::vec2(available_size.x - 10.0, grid_height),
            egui::Sense::hover(),
        );

        let rect = response.rect;

        // Calculate timeline area (excluding label column)
        let timeline_rect = Rect::from_min_max(
            Pos2::new(rect.left() + LABEL_WIDTH, rect.top() + HEADER_HEIGHT),
            rect.max,
        );

        // Draw components
        self.draw_background(&painter, rect);
        self.draw_frame_header(&painter, rect, timeline_rect);
        self.draw_row_labels(&painter, rect, num_rows);
        self.draw_grid(&painter, rect, timeline_rect, num_rows);
        self.draw_events(&painter, timeline_rect);
    }

    /// Draw the background and border.
    fn draw_background(&self, painter: &Painter, rect: Rect) {
        // Fill background
        painter.rect_filled(rect, 0.0, Color32::from_rgb(30, 30, 35));

        // Draw border
        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(1.0, Color32::DARK_GRAY),
            egui::StrokeKind::Inside,
        );
    }

    /// Draw the frame number header row.
    fn draw_frame_header(&self, painter: &Painter, rect: Rect, timeline_rect: Rect) {
        let frame_width = timeline_rect.width() / self.config.visible_frames as f32;
        let header_rect = Rect::from_min_max(
            Pos2::new(rect.left() + LABEL_WIDTH, rect.top()),
            Pos2::new(rect.right(), rect.top() + HEADER_HEIGHT),
        );

        // Draw header background
        painter.rect_filled(header_rect, 0.0, Color32::from_rgb(40, 40, 45));

        // Draw frame numbers at regular intervals
        let interval = self.calculate_frame_interval();
        let start_frame = self.config.scroll_offset;
        let end_frame = start_frame + self.config.visible_frames;

        // Align to interval
        let first_marker = ((start_frame / interval) * interval).max(start_frame);

        let mut frame = first_marker;
        while frame <= end_frame {
            let x = timeline_rect.left()
                + ((frame - start_frame) as f32 * frame_width)
                + frame_width / 2.0;

            if x >= timeline_rect.left() && x <= timeline_rect.right() {
                painter.text(
                    Pos2::new(x, rect.top() + HEADER_HEIGHT / 2.0),
                    egui::Align2::CENTER_CENTER,
                    format!("{}", frame),
                    egui::FontId::proportional(10.0),
                    Color32::GRAY,
                );
            }

            frame += interval;
        }

        // Draw separator line below header
        painter.line_segment(
            [
                Pos2::new(rect.left(), rect.top() + HEADER_HEIGHT),
                Pos2::new(rect.right(), rect.top() + HEADER_HEIGHT),
            ],
            Stroke::new(1.0, Color32::DARK_GRAY),
        );
    }

    /// Calculate the interval for frame number display based on visible frames.
    fn calculate_frame_interval(&self) -> u64 {
        if self.config.visible_frames <= 20 {
            1
        } else if self.config.visible_frames <= 50 {
            5
        } else if self.config.visible_frames <= 100 {
            10
        } else if self.config.visible_frames <= 500 {
            50
        } else {
            100
        }
    }

    /// Draw the row labels on the left side.
    fn draw_row_labels(&self, painter: &Painter, rect: Rect, num_rows: usize) {
        let label_rect = Rect::from_min_max(
            Pos2::new(rect.left(), rect.top() + HEADER_HEIGHT),
            Pos2::new(rect.left() + LABEL_WIDTH, rect.bottom()),
        );

        // Draw label column background
        painter.rect_filled(label_rect, 0.0, Color32::from_rgb(35, 35, 40));

        // Draw separator line
        painter.line_segment(
            [
                Pos2::new(rect.left() + LABEL_WIDTH, rect.top()),
                Pos2::new(rect.left() + LABEL_WIDTH, rect.bottom()),
            ],
            Stroke::new(1.0, Color32::DARK_GRAY),
        );

        // Draw each row label
        for i in 0..num_rows {
            let row_top = rect.top() + HEADER_HEIGHT + (i as f32 * ROW_HEIGHT);
            let row_center_y = row_top + ROW_HEIGHT / 2.0;

            if i < self.log.mappings.len() {
                let mapping = &self.log.mappings[i];
                let color = mapping
                    .color
                    .map(|c| Color32::from_rgb(c[0], c[1], c[2]))
                    .unwrap_or(Color32::LIGHT_GRAY);

                // Draw color indicator
                let indicator_rect = Rect::from_min_size(
                    Pos2::new(rect.left() + 4.0, row_center_y - 4.0),
                    egui::vec2(8.0, 8.0),
                );
                painter.rect_filled(indicator_rect, 2.0, color);

                // Draw label text
                painter.text(
                    Pos2::new(rect.left() + 16.0, row_center_y),
                    egui::Align2::LEFT_CENTER,
                    &mapping.name,
                    egui::FontId::proportional(12.0),
                    Color32::LIGHT_GRAY,
                );
            }
        }
    }

    /// Draw the grid lines.
    fn draw_grid(&self, painter: &Painter, rect: Rect, timeline_rect: Rect, num_rows: usize) {
        let frame_width = timeline_rect.width() / self.config.visible_frames as f32;

        // Draw horizontal row separators
        for i in 1..num_rows {
            let y = rect.top() + HEADER_HEIGHT + (i as f32 * ROW_HEIGHT);
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(0.5, Color32::from_rgb(50, 50, 55)),
            );
        }

        // Draw vertical frame lines at major intervals
        let interval = self.calculate_frame_interval();
        let start_frame = self.config.scroll_offset;
        let end_frame = start_frame + self.config.visible_frames;

        let first_marker = ((start_frame / interval) * interval).max(start_frame);

        let mut frame = first_marker;
        while frame <= end_frame {
            let x = timeline_rect.left() + ((frame - start_frame) as f32 * frame_width);

            if x >= timeline_rect.left() && x <= timeline_rect.right() {
                painter.line_segment(
                    [
                        Pos2::new(x, rect.top() + HEADER_HEIGHT),
                        Pos2::new(x, rect.bottom()),
                    ],
                    Stroke::new(0.5, Color32::from_rgb(50, 50, 55)),
                );
            }

            frame += interval;
        }
    }

    /// Draw all input events.
    fn draw_events(&self, painter: &Painter, timeline_rect: Rect) {
        let frame_width = timeline_rect.width() / self.config.visible_frames as f32;
        let start_frame = self.config.scroll_offset;
        let end_frame = start_frame + self.config.visible_frames;

        // Filter events to visible range and draw them
        for event in &self.log.events {
            // Skip events outside visible range
            if event.frame < start_frame || event.frame >= end_frame {
                continue;
            }

            // Skip events without a row mapping
            let row = match self.get_row(event.id) {
                Some(r) => r,
                None => continue,
            };

            let color = self.get_color(event.id);
            let x = timeline_rect.left() + ((event.frame - start_frame) as f32 * frame_width);
            let row_top = timeline_rect.top() + (row as f32 * ROW_HEIGHT);

            match event.kind {
                InputKind::Button => {
                    self.draw_button_event(painter, event, x, row_top, frame_width, color);
                }
                InputKind::Axis1D => {
                    self.draw_axis1d_event(painter, event, x, row_top, frame_width, color);
                }
                InputKind::Axis2D => {
                    self.draw_axis2d_event(painter, event, x, row_top, frame_width, color);
                }
            }
        }
    }

    /// Draw a button event as a rectangle.
    fn draw_button_event(
        &self,
        painter: &Painter,
        event: &InputEvent,
        x: f32,
        row_top: f32,
        frame_width: f32,
        color: Color32,
    ) {
        let cell_rect = Rect::from_min_size(
            Pos2::new(x + CELL_PADDING, row_top + CELL_PADDING),
            egui::vec2(
                frame_width - CELL_PADDING * 2.0,
                ROW_HEIGHT - CELL_PADDING * 2.0,
            ),
        );

        match event.state {
            ButtonState::Pressed | ButtonState::Held => {
                // Filled rectangle for pressed/held
                painter.rect_filled(cell_rect, 2.0, color);

                // Add a subtle highlight for "Pressed" to distinguish from "Held"
                if event.state == ButtonState::Pressed {
                    let highlight_rect =
                        Rect::from_min_size(cell_rect.min, egui::vec2(cell_rect.width(), 3.0));
                    painter.rect_filled(
                        highlight_rect,
                        2.0,
                        Color32::from_rgba_unmultiplied(255, 255, 255, 80),
                    );
                }
            }
            ButtonState::Released => {
                // Empty rectangle (just outline) for released
                painter.rect_stroke(
                    cell_rect,
                    2.0,
                    Stroke::new(1.0, color.gamma_multiply(0.5)),
                    egui::StrokeKind::Inside,
                );
            }
        }
    }

    /// Draw an Axis1D event as a horizontal bar.
    fn draw_axis1d_event(
        &self,
        painter: &Painter,
        event: &InputEvent,
        x: f32,
        row_top: f32,
        frame_width: f32,
        color: Color32,
    ) {
        let value = event.value[0];
        let abs_value = value.abs();

        if abs_value < 0.01 {
            // Draw minimal indicator for zero value
            let center_y = row_top + ROW_HEIGHT / 2.0;
            let indicator_rect = Rect::from_min_size(
                Pos2::new(x + frame_width / 2.0 - 1.0, center_y - 1.0),
                egui::vec2(2.0, 2.0),
            );
            painter.rect_filled(indicator_rect, 1.0, color.gamma_multiply(0.3));
            return;
        }

        let cell_height = ROW_HEIGHT - CELL_PADDING * 2.0;
        let bar_height = cell_height * 0.6;
        let center_y = row_top + ROW_HEIGHT / 2.0;

        // Calculate bar dimensions
        let max_bar_width = frame_width - CELL_PADDING * 2.0;
        let bar_width = max_bar_width * abs_value;

        // Center point of the cell
        let cell_center_x = x + frame_width / 2.0;

        // Draw bar from center, direction based on sign
        let bar_rect = if value >= 0.0 {
            // Positive: bar extends to the right
            Rect::from_min_size(
                Pos2::new(cell_center_x, center_y - bar_height / 2.0),
                egui::vec2(bar_width / 2.0, bar_height),
            )
        } else {
            // Negative: bar extends to the left
            Rect::from_min_size(
                Pos2::new(cell_center_x - bar_width / 2.0, center_y - bar_height / 2.0),
                egui::vec2(bar_width / 2.0, bar_height),
            )
        };

        // Draw the bar with color intensity based on value
        let intensity = 0.3 + (abs_value * 0.7);
        painter.rect_filled(bar_rect, 2.0, color.gamma_multiply(intensity));

        // Draw center line indicator
        painter.line_segment(
            [
                Pos2::new(cell_center_x, row_top + CELL_PADDING),
                Pos2::new(cell_center_x, row_top + ROW_HEIGHT - CELL_PADDING),
            ],
            Stroke::new(0.5, Color32::from_rgb(60, 60, 65)),
        );
    }

    /// Draw an Axis2D event as a combined representation.
    fn draw_axis2d_event(
        &self,
        painter: &Painter,
        event: &InputEvent,
        x: f32,
        row_top: f32,
        frame_width: f32,
        color: Color32,
    ) {
        let value_x = event.value[0];
        let value_y = event.value[1];
        let magnitude = (value_x * value_x + value_y * value_y).sqrt().min(1.0);

        if magnitude < 0.01 {
            // Draw minimal indicator for zero value
            let center_y = row_top + ROW_HEIGHT / 2.0;
            let center_x = x + frame_width / 2.0;
            let indicator_rect = Rect::from_min_size(
                Pos2::new(center_x - 1.0, center_y - 1.0),
                egui::vec2(2.0, 2.0),
            );
            painter.rect_filled(indicator_rect, 1.0, color.gamma_multiply(0.3));
            return;
        }

        let cell_size = (frame_width - CELL_PADDING * 2.0).min(ROW_HEIGHT - CELL_PADDING * 2.0);
        let center_x = x + frame_width / 2.0;
        let center_y = row_top + ROW_HEIGHT / 2.0;

        // Draw a small circle with a direction indicator
        let circle_radius = cell_size / 2.0 * 0.8;

        // Draw circle outline
        painter.circle_stroke(
            Pos2::new(center_x, center_y),
            circle_radius,
            Stroke::new(1.0, color.gamma_multiply(0.4)),
        );

        // Draw direction dot
        let dot_x = center_x + value_x * circle_radius * 0.8;
        let dot_y = center_y - value_y * circle_radius * 0.8; // Y inverted for screen coords
        let dot_radius = 2.0 + magnitude * 2.0;

        let intensity = 0.5 + (magnitude * 0.5);
        painter.circle_filled(
            Pos2::new(dot_x, dot_y),
            dot_radius,
            color.gamma_multiply(intensity),
        );
    }
}

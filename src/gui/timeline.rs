//! Timeline rendering module.
//!
//! This module handles the visualization of input events over frames,
//! including drawing the frame grid, input rows, and event representations.

use eframe::egui::{self, Color32, Painter, Pos2, Rect, Stroke};
use std::collections::HashMap;

use crate::core::config::ColorSettings;
use crate::core::filter::FilterState;
use crate::core::log::{Bookmark, ButtonState, InputEvent, InputKind, InputLog, InputMapping};
use crate::core::search::SearchResult;

/// Default number of visible frames in the timeline.
pub const DEFAULT_VISIBLE_FRAMES: u64 = 100;

/// Minimum number of visible frames (maximum zoom in).
pub const MIN_VISIBLE_FRAMES: u64 = 10;

/// Maximum number of visible frames (maximum zoom out).
pub const MAX_VISIBLE_FRAMES: u64 = 10000;

/// Zoom factor applied when scrolling (multiplier per scroll step).
const ZOOM_FACTOR: f32 = 1.15;

/// Height of each input row in pixels.
const ROW_HEIGHT: f32 = 32.0;

/// Width of the label column on the left side.
const LABEL_WIDTH: f32 = 120.0;

/// Padding inside cells.
const CELL_PADDING: f32 = 2.0;

/// Height of the frame number header.
const HEADER_HEIGHT: f32 = 20.0;

/// Height of the legend area at the bottom.
const LEGEND_HEIGHT: f32 = 30.0;

/// Height of the scrollbar area.
const SCROLLBAR_HEIGHT: f32 = 16.0;

/// Scroll speed in frames per scroll step.
const SCROLL_SPEED: f32 = 10.0;

/// Actions that the timeline can request from the application.
#[derive(Debug, Clone, PartialEq)]
pub enum ViewAction {
    /// Zoom the timeline view.
    Zoom {
        visible_frames: u64,
        scroll_offset: u64,
    },
    /// Scroll the timeline view.
    Scroll { scroll_offset: u64 },
    /// Start a range selection drag at the given frame.
    StartSelection { frame: u64 },
    /// Update the range selection during drag.
    UpdateSelection { frame: u64 },
    /// Finish the range selection.
    FinishSelection,
}

/// Configuration for timeline rendering.
pub struct TimelineConfig {
    /// First visible frame (for scrolling)
    pub scroll_offset: u64,
    /// Number of frames to display
    pub visible_frames: u64,
    /// Current frame position (for playback indicator)
    pub current_frame: u64,
    /// Total number of frames in the log
    pub total_frames: u64,
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self {
            scroll_offset: 0,
            visible_frames: DEFAULT_VISIBLE_FRAMES.max(1),
            current_frame: 0,
            total_frames: 0,
        }
    }
}

impl TimelineConfig {
    /// Calculate the zoom percentage (100% = DEFAULT_VISIBLE_FRAMES).
    pub fn zoom_percentage(&self) -> f32 {
        (DEFAULT_VISIBLE_FRAMES as f32 / self.visible_frames as f32) * 100.0
    }
}

/// Timeline renderer that draws input events over frames.
pub struct TimelineRenderer<'a> {
    /// The input log to render
    log: &'a InputLog,
    /// Rendering configuration
    config: &'a TimelineConfig,
    /// Filter state for input visibility
    filter: &'a FilterState,
    /// Color settings for UI elements
    colors: &'a ColorSettings,
    /// Search results for highlighting (optional)
    search_results: Option<&'a SearchResult>,
    /// Bookmarks to display on the timeline (optional)
    bookmarks: Option<&'a [Bookmark]>,
    /// Selection range for highlighting (optional)
    selection: Option<(u64, u64)>,
    /// Whether a selection drag is currently in progress
    selection_dragging: bool,
    /// Effective mappings including fallback entries for unmapped IDs
    effective_mappings: Vec<InputMapping>,
    /// Visible mappings based on current filter (indices into effective_mappings)
    visible_mapping_indices: Vec<usize>,
    /// Map from input ID to row index (among visible rows)
    id_to_row: HashMap<u32, usize>,
    /// Map from input ID to index in effective_mappings (for name and color)
    id_to_mapping_index: HashMap<u32, usize>,
}

impl<'a> TimelineRenderer<'a> {
    /// Create a new timeline renderer for the given log.
    pub fn new(
        log: &'a InputLog,
        config: &'a TimelineConfig,
        filter: &'a FilterState,
        colors: &'a ColorSettings,
    ) -> Self {
        // Get effective mappings (includes fallback entries for unmapped IDs)
        let effective_mappings = log.get_effective_mappings();

        // Build ID to kind mapping by scanning events
        let mut id_to_kind: HashMap<u32, InputKind> = HashMap::new();
        for event in &log.events {
            id_to_kind.entry(event.id).or_insert(event.kind);
        }

        // Build ID to mapping index lookup
        let id_to_mapping_index: HashMap<u32, usize> = effective_mappings
            .iter()
            .enumerate()
            .map(|(i, m)| (m.id, i))
            .collect();

        // Filter visible mappings based on filter state (store indices)
        let visible_mapping_indices: Vec<usize> = effective_mappings
            .iter()
            .enumerate()
            .filter(|(_, m)| {
                let kind = id_to_kind.get(&m.id).copied().unwrap_or(InputKind::Button);
                filter.is_visible(m.id, kind)
            })
            .map(|(i, _)| i)
            .collect();

        // Build ID to row mapping based on visible mappings order
        let id_to_row: HashMap<u32, usize> = visible_mapping_indices
            .iter()
            .enumerate()
            .map(|(row, &idx)| (effective_mappings[idx].id, row))
            .collect();

        Self {
            log,
            config,
            filter,
            colors,
            search_results: None,
            bookmarks: None,
            selection: None,
            selection_dragging: false,
            effective_mappings,
            visible_mapping_indices,
            id_to_row,
            id_to_mapping_index,
        }
    }

    /// Set search results for highlighting matching frames.
    pub fn with_search_results(mut self, results: &'a SearchResult) -> Self {
        self.search_results = Some(results);
        self
    }

    /// Set bookmarks for displaying on the timeline.
    pub fn with_bookmarks(mut self, bookmarks: &'a [Bookmark]) -> Self {
        self.bookmarks = Some(bookmarks);
        self
    }

    /// Set selection range for highlighting.
    pub fn with_selection(mut self, selection: Option<(u64, u64)>, is_dragging: bool) -> Self {
        self.selection = selection;
        self.selection_dragging = is_dragging;
        self
    }

    /// Get the color for an input ID, or a default color if not mapped.
    fn get_color(&self, id: u32) -> Color32 {
        self.id_to_mapping_index
            .get(&id)
            .and_then(|&idx| self.effective_mappings[idx].color)
            .map(|c| Color32::from_rgb(c[0], c[1], c[2]))
            .unwrap_or(self.colors.text_label_color())
    }

    /// Get the row index for an input ID.
    fn get_row(&self, id: u32) -> Option<usize> {
        self.id_to_row.get(&id).copied()
    }

    /// Calculate the total height needed for the timeline.
    pub fn calculate_height(&self) -> f32 {
        let num_rows = self.visible_mapping_indices.len().max(1);
        HEADER_HEIGHT + (num_rows as f32 * ROW_HEIGHT) + SCROLLBAR_HEIGHT + LEGEND_HEIGHT
    }

    /// Render the complete timeline and return any view actions triggered by user interaction.
    pub fn render(&self, ui: &mut egui::Ui) -> Option<ViewAction> {
        let available_size = ui.available_size();
        let num_rows = self.visible_mapping_indices.len().max(1);
        let grid_height = self.calculate_height().min(available_size.y - 10.0);

        let (response, painter) = ui.allocate_painter(
            egui::vec2(available_size.x - 10.0, grid_height),
            egui::Sense::click_and_drag(),
        );

        let rect = response.rect;

        // Calculate layout areas from bottom to top
        let legend_top = rect.bottom() - LEGEND_HEIGHT;
        let scrollbar_top = legend_top - SCROLLBAR_HEIGHT;
        let content_bottom = scrollbar_top;

        // Calculate timeline area (excluding label column, scrollbar and legend)
        let timeline_rect = Rect::from_min_max(
            Pos2::new(rect.left() + LABEL_WIDTH, rect.top() + HEADER_HEIGHT),
            Pos2::new(rect.right(), content_bottom),
        );

        // Calculate content rect (full width, excluding scrollbar and legend)
        let content_rect = Rect::from_min_max(rect.min, Pos2::new(rect.right(), content_bottom));

        // Calculate scrollbar area rect
        let scrollbar_rect = Rect::from_min_max(
            Pos2::new(rect.left() + LABEL_WIDTH, scrollbar_top),
            Pos2::new(rect.right(), legend_top),
        );

        // Calculate legend area rect
        let legend_area_rect = Rect::from_min_max(Pos2::new(rect.left(), legend_top), rect.max);

        // Draw components
        self.draw_background(&painter, rect);
        self.draw_frame_header(&painter, content_rect, timeline_rect);
        self.draw_row_labels(&painter, content_rect, num_rows);
        self.draw_grid(&painter, content_rect, timeline_rect, num_rows);
        self.draw_selection_highlight(&painter, content_rect, timeline_rect);
        self.draw_search_highlights(&painter, content_rect, timeline_rect);
        self.draw_bookmark_markers(&painter, rect, timeline_rect);
        self.draw_events(&painter, timeline_rect);
        self.draw_current_frame_indicator(&painter, content_rect, timeline_rect);
        self.draw_scrollbar(&painter, scrollbar_rect);
        self.draw_button_state_legend(&painter, legend_area_rect);
        self.draw_zoom_indicator(&painter, legend_area_rect);

        // Handle mouse interactions (scroll, zoom, scrollbar drag, selection)
        self.handle_mouse_interaction(ui, &response, timeline_rect, scrollbar_rect)
    }

    /// Handle mouse interactions: wheel scroll, Ctrl+wheel zoom, scrollbar drag, and timeline drag-to-pan.
    fn handle_mouse_interaction(
        &self,
        ui: &egui::Ui,
        response: &egui::Response,
        timeline_rect: Rect,
        scrollbar_rect: Rect,
    ) -> Option<ViewAction> {
        let ctx = ui.ctx();

        // Handle scrollbar drag
        if let Some(action) = self.handle_scrollbar_drag(response, scrollbar_rect) {
            return Some(action);
        }

        // Handle timeline drag-to-pan or Shift+drag for selection
        if let Some(action) = self.handle_timeline_drag(ui, response, timeline_rect) {
            return Some(action);
        }

        // Check if pointer is over the timeline or scrollbar area
        let pointer_pos = ctx.input(|i| i.pointer.hover_pos())?;
        let over_timeline = timeline_rect.contains(pointer_pos);
        let over_scrollbar = scrollbar_rect.contains(pointer_pos);

        if !over_timeline && !over_scrollbar {
            return None;
        }

        // Get scroll delta and modifier state
        let (scroll_delta_y, scroll_delta_x, ctrl_held, shift_held) = ctx.input(|i| {
            (
                i.raw_scroll_delta.y,
                i.raw_scroll_delta.x,
                i.modifiers.ctrl,
                i.modifiers.shift,
            )
        });

        // Ctrl+scroll = zoom
        if ctrl_held && scroll_delta_y != 0.0 {
            return self.handle_zoom(pointer_pos, scroll_delta_y, timeline_rect);
        }

        // Regular scroll (vertical or horizontal) = horizontal scroll
        // Shift+scroll also maps vertical to horizontal
        let scroll_delta = if shift_held || scroll_delta_x != 0.0 {
            -scroll_delta_x - scroll_delta_y
        } else {
            -scroll_delta_y
        };

        if scroll_delta != 0.0 {
            return self.handle_scroll(scroll_delta);
        }

        None
    }

    /// Handle scrollbar drag interaction.
    fn handle_scrollbar_drag(
        &self,
        response: &egui::Response,
        scrollbar_rect: Rect,
    ) -> Option<ViewAction> {
        if !response.dragged() {
            return None;
        }

        // Check if drag started in scrollbar area
        let drag_origin = response.interact_pointer_pos()?;
        if !scrollbar_rect.contains(drag_origin) {
            return None;
        }

        // Calculate scroll position from pointer position
        let pointer_pos = response.interact_pointer_pos()?;
        let track_width = scrollbar_rect.width();
        let thumb_width = self.calculate_scrollbar_thumb_width(track_width);

        // Calculate position relative to track, accounting for thumb size
        let usable_width = track_width - thumb_width;
        if usable_width <= 0.0 {
            return None;
        }

        let relative_x =
            (pointer_pos.x - scrollbar_rect.left() - thumb_width / 2.0).clamp(0.0, usable_width);
        let scroll_ratio = relative_x / usable_width;

        let max_scroll = self
            .config
            .total_frames
            .saturating_sub(self.config.visible_frames);
        let new_scroll_offset = (scroll_ratio * max_scroll as f32) as u64;

        Some(ViewAction::Scroll {
            scroll_offset: new_scroll_offset.min(max_scroll),
        })
    }

    /// Handle drag-to-pan interaction on the timeline area.
    /// Without Shift: pans the timeline.
    /// With Shift: selects a range.
    fn handle_timeline_drag(
        &self,
        ui: &egui::Ui,
        response: &egui::Response,
        timeline_rect: Rect,
    ) -> Option<ViewAction> {
        let ctx = ui.ctx();
        let shift_held = ctx.input(|i| i.modifiers.shift);

        // Handle Shift+drag for selection
        if shift_held {
            return self.handle_selection_drag(response, timeline_rect);
        }

        // Handle regular drag for panning
        if !response.dragged() {
            return None;
        }

        // Check if drag started in timeline area (not scrollbar)
        let drag_origin = response.interact_pointer_pos()?;
        if !timeline_rect.contains(drag_origin) {
            return None;
        }

        // Get the drag delta in pixels
        let drag_delta = response.drag_delta();
        if drag_delta.x.abs() < 0.1 {
            return None;
        }

        // Convert pixel delta to frame delta based on current zoom level
        let frame_width = timeline_rect.width() / self.config.visible_frames as f32;
        let frame_delta = -drag_delta.x / frame_width;

        // Calculate new scroll offset
        let current_scroll = self.config.scroll_offset as f64;
        let new_scroll = current_scroll + frame_delta as f64;

        let max_scroll = self
            .config
            .total_frames
            .saturating_sub(self.config.visible_frames) as f64;

        let new_scroll_offset = new_scroll.clamp(0.0, max_scroll) as u64;

        if new_scroll_offset == self.config.scroll_offset {
            return None;
        }

        Some(ViewAction::Scroll {
            scroll_offset: new_scroll_offset,
        })
    }

    /// Handle Shift+drag for range selection.
    fn handle_selection_drag(
        &self,
        response: &egui::Response,
        timeline_rect: Rect,
    ) -> Option<ViewAction> {
        let frame_width = timeline_rect.width() / self.config.visible_frames as f32;

        // Helper to convert x position to frame number
        let x_to_frame = |x: f32| -> u64 {
            let relative_x = (x - timeline_rect.left()).max(0.0);
            let frame_offset = (relative_x / frame_width) as u64;
            let frame = self.config.scroll_offset.saturating_add(frame_offset);
            frame.min(self.config.total_frames.saturating_sub(1))
        };

        // Check if drag started in timeline area
        let drag_origin = response.interact_pointer_pos()?;
        if !timeline_rect.contains(drag_origin) {
            return None;
        }

        // Handle drag start (first frame of drag)
        if response.drag_started() {
            let start_frame = x_to_frame(drag_origin.x);
            return Some(ViewAction::StartSelection { frame: start_frame });
        }

        // Handle ongoing drag
        if response.dragged() {
            let current_pos = response.interact_pointer_pos()?;
            let current_frame = x_to_frame(current_pos.x);
            return Some(ViewAction::UpdateSelection {
                frame: current_frame,
            });
        }

        // Handle drag release
        if response.drag_stopped() {
            return Some(ViewAction::FinishSelection);
        }

        None
    }

    /// Handle zoom interaction from Ctrl+scroll.
    fn handle_zoom(
        &self,
        pointer_pos: Pos2,
        scroll_delta: f32,
        timeline_rect: Rect,
    ) -> Option<ViewAction> {
        // Calculate zoom factor based on scroll direction
        let zoom_factor = if scroll_delta > 0.0 {
            1.0 / ZOOM_FACTOR // Scroll up = zoom in = fewer frames
        } else {
            ZOOM_FACTOR // Scroll down = zoom out = more frames
        };

        // Calculate new visible frames
        let new_visible_frames = (self.config.visible_frames as f32 * zoom_factor) as u64;
        let new_visible_frames = new_visible_frames.clamp(MIN_VISIBLE_FRAMES, MAX_VISIBLE_FRAMES);

        // If no change, skip
        if new_visible_frames == self.config.visible_frames {
            return None;
        }

        // Calculate the frame under the mouse cursor before zoom
        let frame_width = timeline_rect.width() / self.config.visible_frames as f32;
        let mouse_offset_x = (pointer_pos.x - timeline_rect.left()).max(0.0);
        let frame_under_mouse = self.config.scroll_offset as f32 + (mouse_offset_x / frame_width);

        // Calculate new frame width after zoom
        let new_frame_width = timeline_rect.width() / new_visible_frames as f32;

        // Calculate new scroll offset to keep the frame under mouse in the same screen position
        let new_scroll_offset = frame_under_mouse - (mouse_offset_x / new_frame_width);
        let new_scroll_offset = new_scroll_offset.max(0.0) as u64;

        // Clamp scroll offset to valid range
        let max_scroll = self.config.total_frames.saturating_sub(new_visible_frames);
        let new_scroll_offset = new_scroll_offset.min(max_scroll);

        Some(ViewAction::Zoom {
            visible_frames: new_visible_frames,
            scroll_offset: new_scroll_offset,
        })
    }

    /// Handle horizontal scroll from mouse wheel.
    fn handle_scroll(&self, scroll_delta: f32) -> Option<ViewAction> {
        let scroll_amount = (scroll_delta * SCROLL_SPEED / 10.0) as i64;
        if scroll_amount == 0 {
            return None;
        }

        let current_scroll = self.config.scroll_offset as i64;
        let new_scroll = current_scroll + scroll_amount;
        let max_scroll = self
            .config
            .total_frames
            .saturating_sub(self.config.visible_frames) as i64;

        let new_scroll_offset = new_scroll.clamp(0, max_scroll) as u64;

        if new_scroll_offset == self.config.scroll_offset {
            return None;
        }

        Some(ViewAction::Scroll {
            scroll_offset: new_scroll_offset,
        })
    }

    /// Calculate scrollbar thumb width based on visible/total frame ratio.
    fn calculate_scrollbar_thumb_width(&self, track_width: f32) -> f32 {
        if self.config.total_frames == 0 {
            return track_width;
        }
        let ratio = self.config.visible_frames as f32 / self.config.total_frames as f32;
        (track_width * ratio).max(20.0).min(track_width)
    }

    /// Draw the horizontal scrollbar.
    fn draw_scrollbar(&self, painter: &Painter, scrollbar_rect: Rect) {
        // Draw track background
        painter.rect_filled(scrollbar_rect, 2.0, self.colors.scrollbar_track_color());

        // Calculate thumb position and size
        let track_width = scrollbar_rect.width();
        let thumb_width = self.calculate_scrollbar_thumb_width(track_width);

        // Calculate thumb position
        let max_scroll = self
            .config
            .total_frames
            .saturating_sub(self.config.visible_frames);
        let scroll_ratio = if max_scroll > 0 {
            self.config.scroll_offset as f32 / max_scroll as f32
        } else {
            0.0
        };

        let usable_width = track_width - thumb_width;
        let thumb_x = scrollbar_rect.left() + (scroll_ratio * usable_width);

        // Draw thumb
        let thumb_rect = Rect::from_min_size(
            Pos2::new(thumb_x, scrollbar_rect.top() + 2.0),
            egui::vec2(thumb_width, scrollbar_rect.height() - 4.0),
        );
        painter.rect_filled(thumb_rect, 4.0, self.colors.scrollbar_thumb_color());

        // Draw thumb border on hover (simplified - always show slight border)
        painter.rect_stroke(
            thumb_rect,
            4.0,
            Stroke::new(1.0, self.colors.scrollbar_border_color()),
            egui::StrokeKind::Inside,
        );
    }

    /// Draw the background and border.
    fn draw_background(&self, painter: &Painter, rect: Rect) {
        // Fill background
        painter.rect_filled(rect, 0.0, self.colors.background_color());

        // Draw border
        painter.rect_stroke(
            rect,
            0.0,
            Stroke::new(1.0, self.colors.grid_color()),
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
        painter.rect_filled(header_rect, 0.0, self.colors.header_background_color());

        // Draw frame numbers at regular intervals
        let interval = self.calculate_frame_interval();
        let start_frame = self.config.scroll_offset;
        let end_frame = start_frame + self.config.visible_frames;

        // Align to interval (ceiling division to get next multiple >= start_frame)
        let first_marker = start_frame.div_ceil(interval) * interval;

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
                    self.colors.text_header_color(),
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
            Stroke::new(1.0, self.colors.grid_color()),
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
        } else if self.config.visible_frames <= 2000 {
            100
        } else if self.config.visible_frames <= 5000 {
            500
        } else {
            1000
        }
    }

    /// Draw the row labels on the left side.
    fn draw_row_labels(&self, painter: &Painter, rect: Rect, num_rows: usize) {
        let label_rect = Rect::from_min_max(
            Pos2::new(rect.left(), rect.top() + HEADER_HEIGHT),
            Pos2::new(rect.left() + LABEL_WIDTH, rect.bottom()),
        );

        // Draw label column background
        painter.rect_filled(label_rect, 0.0, self.colors.label_background_color());

        // Draw separator line
        painter.line_segment(
            [
                Pos2::new(rect.left() + LABEL_WIDTH, rect.top()),
                Pos2::new(rect.left() + LABEL_WIDTH, rect.bottom()),
            ],
            Stroke::new(1.0, self.colors.grid_color()),
        );

        // Draw each row label (using filtered visible mappings)
        for i in 0..num_rows {
            let row_top = rect.top() + HEADER_HEIGHT + (i as f32 * ROW_HEIGHT);
            let row_center_y = row_top + ROW_HEIGHT / 2.0;

            if i < self.visible_mapping_indices.len() {
                let mapping_idx = self.visible_mapping_indices[i];
                let mapping = &self.effective_mappings[mapping_idx];
                let color = mapping
                    .color
                    .map(|c| Color32::from_rgb(c[0], c[1], c[2]))
                    .unwrap_or(self.colors.text_label_color());

                // Draw color indicator
                let indicator_rect = Rect::from_min_size(
                    Pos2::new(rect.left() + 4.0, row_center_y - 4.0),
                    egui::vec2(8.0, 8.0),
                );
                painter.rect_filled(indicator_rect, 2.0, color);

                // Draw label text (uses mapping name which includes fallback)
                painter.text(
                    Pos2::new(rect.left() + 16.0, row_center_y),
                    egui::Align2::LEFT_CENTER,
                    &mapping.name,
                    egui::FontId::proportional(12.0),
                    self.colors.text_label_color(),
                );
            }
        }
    }

    /// Draw the grid lines.
    fn draw_grid(&self, painter: &Painter, rect: Rect, timeline_rect: Rect, num_rows: usize) {
        let frame_width = timeline_rect.width() / self.config.visible_frames as f32;
        let grid_color = self.colors.grid_color();

        // Draw horizontal row separators
        for i in 1..num_rows {
            let y = rect.top() + HEADER_HEIGHT + (i as f32 * ROW_HEIGHT);
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                Stroke::new(0.5, grid_color),
            );
        }

        // Draw vertical frame lines at major intervals
        let interval = self.calculate_frame_interval();
        let start_frame = self.config.scroll_offset;
        let end_frame = start_frame + self.config.visible_frames;

        // Align to interval (ceiling division to get next multiple >= start_frame)
        let first_marker = start_frame.div_ceil(interval) * interval;

        let mut frame = first_marker;
        while frame <= end_frame {
            let x = timeline_rect.left() + ((frame - start_frame) as f32 * frame_width);

            if x >= timeline_rect.left() && x <= timeline_rect.right() {
                painter.line_segment(
                    [
                        Pos2::new(x, rect.top() + HEADER_HEIGHT),
                        Pos2::new(x, rect.bottom()),
                    ],
                    Stroke::new(0.5, grid_color),
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

            // Skip events that are filtered out by type or ID filter
            if !self.filter.is_visible(event.id, event.kind) {
                continue;
            }

            // Skip events without a row mapping (among visible rows)
            let row = match self.get_row(event.id) {
                Some(r) => r,
                None => continue,
            };

            let color = self.get_color(event.id);
            let x = timeline_rect.left() + ((event.frame - start_frame) as f32 * frame_width);
            let row_top = timeline_rect.top() + (row as f32 * ROW_HEIGHT);

            match event.kind {
                InputKind::Button => {
                    self.draw_button_event(painter, x, row_top, frame_width, color, event.state);
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

    /// Draw a button event as a rectangle with state-specific styling.
    ///
    /// Visual styles:
    /// - Pressed: Green border with partial fill (button press started)
    /// - Held: Solid fill with mapping color (button held down)
    /// - Released: Red border, empty interior (button released)
    fn draw_button_event(
        &self,
        painter: &Painter,
        x: f32,
        row_top: f32,
        frame_width: f32,
        color: Color32,
        state: ButtonState,
    ) {
        let cell_rect = Rect::from_min_size(
            Pos2::new(x + CELL_PADDING, row_top + CELL_PADDING),
            egui::vec2(
                frame_width - CELL_PADDING * 2.0,
                ROW_HEIGHT - CELL_PADDING * 2.0,
            ),
        );

        match state {
            ButtonState::Pressed => {
                // Green border with partial fill to indicate button press start
                let pressed_color = self.colors.button_pressed_color();

                // Draw partial fill (50% opacity of the mapping color)
                let fill_color =
                    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 128);
                painter.rect_filled(cell_rect, 2.0, fill_color);

                // Draw green border
                painter.rect_stroke(
                    cell_rect,
                    2.0,
                    Stroke::new(2.0, pressed_color),
                    egui::StrokeKind::Inside,
                );
            }
            ButtonState::Held => {
                // Solid fill with mapping color for held state
                painter.rect_filled(cell_rect, 2.0, color);
            }
            ButtonState::Released => {
                // Red border with empty interior for released state
                let released_color = self.colors.button_released_color();

                // Draw empty rectangle with red border
                painter.rect_stroke(
                    cell_rect,
                    2.0,
                    Stroke::new(2.0, released_color),
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

        // Calculate bar dimensions using explicit half-cell width
        // This ensures magnitude 1.0 fills from center to cell edge
        let max_half_width = (frame_width - CELL_PADDING * 2.0) / 2.0;
        let half_width = max_half_width * abs_value;

        // Center point of the cell
        let cell_center_x = x + frame_width / 2.0;

        // Draw bar from center, direction based on sign
        let bar_rect = if value >= 0.0 {
            // Positive: bar extends to the right
            Rect::from_min_size(
                Pos2::new(cell_center_x, center_y - bar_height / 2.0),
                egui::vec2(half_width, bar_height),
            )
        } else {
            // Negative: bar extends to the left
            Rect::from_min_size(
                Pos2::new(cell_center_x - half_width, center_y - bar_height / 2.0),
                egui::vec2(half_width, bar_height),
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
            Stroke::new(0.5, self.colors.axis_center_color()),
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

    /// Draw highlight for selected frame range.
    fn draw_selection_highlight(&self, painter: &Painter, rect: Rect, timeline_rect: Rect) {
        let (sel_start, sel_end) = match self.selection {
            Some((start, end)) => (start, end),
            None => return,
        };

        let view_start = self.config.scroll_offset;
        let view_end = view_start + self.config.visible_frames;
        let frame_width = timeline_rect.width() / self.config.visible_frames as f32;

        // Check if selection overlaps with visible range
        if sel_end < view_start || sel_start >= view_end {
            return;
        }

        // Clamp selection to visible range
        let visible_sel_start = sel_start.max(view_start);
        let visible_sel_end = sel_end.min(view_end.saturating_sub(1));

        // Calculate x positions
        let x_start =
            timeline_rect.left() + ((visible_sel_start - view_start) as f32 * frame_width);
        let x_end =
            timeline_rect.left() + ((visible_sel_end - view_start + 1) as f32 * frame_width);

        // Selection colors - use a distinct color (purple/magenta) to differentiate from search
        let (fill_color, stroke_color) = if self.selection_dragging {
            // Lighter color while dragging
            (
                self.colors.selection_color_alpha(30),
                self.colors.selection_color_alpha(100),
            )
        } else {
            // Solid color when selection is complete
            (
                self.colors.selection_color_alpha(40),
                self.colors.selection_color_alpha(150),
            )
        };

        // Draw the selection highlight rectangle
        let highlight_rect = Rect::from_min_max(
            Pos2::new(x_start, rect.top() + HEADER_HEIGHT),
            Pos2::new(x_end, timeline_rect.bottom()),
        );
        painter.rect_filled(highlight_rect, 0.0, fill_color);
        painter.rect_stroke(
            highlight_rect,
            0.0,
            Stroke::new(2.0, stroke_color),
            egui::StrokeKind::Inside,
        );

        // Draw selection range text in the header area
        let selection_text = format!("F{}-F{}", sel_start, sel_end);
        let text_x = (x_start + x_end) / 2.0;
        let text_y = rect.top() + HEADER_HEIGHT / 2.0;

        // Only draw text if there's enough space
        if x_end - x_start > 40.0 {
            painter.text(
                Pos2::new(text_x, text_y),
                egui::Align2::CENTER_CENTER,
                selection_text,
                egui::FontId::proportional(10.0),
                self.colors.selection_color(),
            );
        }

        // Draw edge markers at selection boundaries
        let marker_color = self.colors.selection_color();

        // Left edge marker
        if visible_sel_start == sel_start && x_start >= timeline_rect.left() {
            painter.line_segment(
                [
                    Pos2::new(x_start, rect.top()),
                    Pos2::new(x_start, timeline_rect.bottom()),
                ],
                Stroke::new(2.0, marker_color),
            );
        }

        // Right edge marker
        if visible_sel_end == sel_end && x_end <= timeline_rect.right() {
            painter.line_segment(
                [
                    Pos2::new(x_end, rect.top()),
                    Pos2::new(x_end, timeline_rect.bottom()),
                ],
                Stroke::new(2.0, marker_color),
            );
        }
    }

    /// Draw highlights for search result frames.
    fn draw_search_highlights(&self, painter: &Painter, rect: Rect, timeline_rect: Rect) {
        let results = match self.search_results {
            Some(r) if !r.is_empty() => r,
            _ => return,
        };

        let start_frame = self.config.scroll_offset;
        let end_frame = start_frame + self.config.visible_frames;
        let frame_width = timeline_rect.width() / self.config.visible_frames as f32;

        // Draw highlight for each matching frame in the visible range
        for &frame in &results.matches {
            if frame < start_frame || frame >= end_frame {
                continue;
            }

            let x = timeline_rect.left() + ((frame - start_frame) as f32 * frame_width);

            // Determine if this is the current result
            let is_current = results.current_frame() == Some(frame);

            // Use different colors for current vs other matches
            let (fill_color, stroke_color) = if is_current {
                (
                    self.colors.search_current_color_alpha(40),
                    self.colors.search_current_color_alpha(150),
                )
            } else {
                (
                    self.colors.search_other_color_alpha(25),
                    self.colors.search_other_color_alpha(80),
                )
            };

            // Draw a vertical highlight bar for the frame column
            let highlight_rect = Rect::from_min_size(
                Pos2::new(x, rect.top() + HEADER_HEIGHT),
                egui::vec2(frame_width, rect.height() - HEADER_HEIGHT),
            );
            painter.rect_filled(highlight_rect, 0.0, fill_color);
            painter.rect_stroke(
                highlight_rect,
                0.0,
                Stroke::new(1.0, stroke_color),
                egui::StrokeKind::Inside,
            );

            // Draw a small marker at the top of the header for current result
            if is_current {
                let marker_width = 6.0;
                let marker_rect = Rect::from_min_size(
                    Pos2::new(x + frame_width / 2.0 - marker_width / 2.0, rect.top() + 2.0),
                    egui::vec2(marker_width, 4.0),
                );
                painter.rect_filled(marker_rect, 2.0, self.colors.search_current_color());
            }
        }
    }

    /// Draw bookmark markers (★) on the timeline.
    fn draw_bookmark_markers(&self, painter: &Painter, rect: Rect, timeline_rect: Rect) {
        let bookmarks = match self.bookmarks {
            Some(b) if !b.is_empty() => b,
            _ => return,
        };

        let start_frame = self.config.scroll_offset;
        let end_frame = start_frame + self.config.visible_frames;
        let frame_width = timeline_rect.width() / self.config.visible_frames as f32;

        // Bookmark marker color
        let bookmark_color = self.colors.bookmark_color();

        // Draw marker for each bookmark in the visible range
        for bookmark in bookmarks {
            if bookmark.frame < start_frame || bookmark.frame >= end_frame {
                continue;
            }

            let x = timeline_rect.left()
                + ((bookmark.frame - start_frame) as f32 * frame_width)
                + frame_width / 2.0;

            // Draw star marker at the top of the header
            painter.text(
                Pos2::new(x, rect.top() + HEADER_HEIGHT / 2.0 - 1.0),
                egui::Align2::CENTER_CENTER,
                "★",
                egui::FontId::proportional(12.0),
                bookmark_color,
            );

            // Draw a subtle vertical line from header to the content area
            let line_color = self.colors.bookmark_color_alpha(80);
            painter.line_segment(
                [
                    Pos2::new(x, rect.top() + HEADER_HEIGHT),
                    Pos2::new(x, timeline_rect.bottom()),
                ],
                Stroke::new(1.0, line_color),
            );
        }
    }

    /// Draw a vertical highlight line at the current frame position.
    fn draw_current_frame_indicator(&self, painter: &Painter, rect: Rect, timeline_rect: Rect) {
        let start_frame = self.config.scroll_offset;
        let end_frame = start_frame + self.config.visible_frames;

        // Only draw if current frame is within visible range
        if self.config.current_frame < start_frame || self.config.current_frame >= end_frame {
            return;
        }

        let frame_width = timeline_rect.width() / self.config.visible_frames as f32;
        let x = timeline_rect.left()
            + ((self.config.current_frame - start_frame) as f32 * frame_width)
            + frame_width / 2.0;

        // Draw the vertical highlight line (full height from header to bottom)
        let highlight_color = self.colors.current_frame_color_alpha(180);
        painter.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
            Stroke::new(2.0, highlight_color),
        );

        // Draw a small triangle marker at the top
        let triangle_size = 6.0;
        let triangle_points = [
            Pos2::new(x - triangle_size, rect.top()),
            Pos2::new(x + triangle_size, rect.top()),
            Pos2::new(x, rect.top() + triangle_size * 1.5),
        ];
        painter.add(egui::Shape::convex_polygon(
            triangle_points.to_vec(),
            highlight_color,
            Stroke::NONE,
        ));
    }

    /// Draw a legend explaining button state visual styles.
    ///
    /// Draws in the dedicated legend area at the bottom of the timeline.
    fn draw_button_state_legend(&self, painter: &Painter, legend_area: Rect) {
        // Legend configuration
        const ICON_SIZE: f32 = 14.0;
        const ICON_SPACING: f32 = 6.0;
        const ITEM_SPACING: f32 = 20.0;
        const TEXT_WIDTH: f32 = 52.0;

        // Legend items
        let items = [
            ("Pressed", self.colors.button_pressed_color()),
            ("Held", self.colors.button_held_color()),
            ("Released", self.colors.button_released_color()),
        ];

        // Calculate total width of legend items
        let item_width = ICON_SIZE + ICON_SPACING + TEXT_WIDTH;
        let total_width =
            (item_width * items.len() as f32) + (ITEM_SPACING * (items.len() - 1) as f32);

        // Position legend items at the right side of the legend area
        let start_x = legend_area.right() - total_width - 16.0;
        let y = legend_area.center().y;

        // Draw separator line at top of legend area
        painter.line_segment(
            [
                Pos2::new(legend_area.left(), legend_area.top()),
                Pos2::new(legend_area.right(), legend_area.top()),
            ],
            Stroke::new(1.0, self.colors.grid_color()),
        );

        // Draw legend items
        let mut x = start_x;

        for (label, border_color) in items {
            // Draw icon representing the state
            let icon_rect = Rect::from_center_size(
                Pos2::new(x + ICON_SIZE / 2.0, y),
                egui::vec2(ICON_SIZE, ICON_SIZE),
            );

            match label {
                "Pressed" => {
                    // Partial fill with green border
                    let fill_color = Color32::from_rgba_unmultiplied(150, 150, 150, 128);
                    painter.rect_filled(icon_rect, 2.0, fill_color);
                    painter.rect_stroke(
                        icon_rect,
                        2.0,
                        Stroke::new(2.0, border_color),
                        egui::StrokeKind::Inside,
                    );
                }
                "Held" => {
                    // Solid fill
                    painter.rect_filled(icon_rect, 2.0, self.colors.button_held_color());
                }
                "Released" => {
                    // Empty with red border
                    painter.rect_stroke(
                        icon_rect,
                        2.0,
                        Stroke::new(2.0, border_color),
                        egui::StrokeKind::Inside,
                    );
                }
                _ => {}
            }

            // Draw label text
            painter.text(
                Pos2::new(x + ICON_SIZE + ICON_SPACING, y),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(11.0),
                self.colors.text_label_color(),
            );

            x += item_width + ITEM_SPACING;
        }
    }

    /// Draw zoom indicator showing current zoom level and visible frames.
    fn draw_zoom_indicator(&self, painter: &Painter, legend_area: Rect) {
        let zoom_pct = self.config.zoom_percentage();
        let visible = self.config.visible_frames;

        // Format zoom text
        let zoom_text = format!("Zoom: {:.0}% ({} frames)", zoom_pct, visible);

        // Position at left side of legend area
        let x = legend_area.left() + 16.0;
        let y = legend_area.center().y;

        // Draw zoom indicator text
        painter.text(
            Pos2::new(x, y),
            egui::Align2::LEFT_CENTER,
            zoom_text,
            egui::FontId::proportional(11.0),
            self.colors.text_header_color(),
        );
    }
}

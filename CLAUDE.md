# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Input Log Viewer is a native desktop GUI application for game developers to visualize input logs frame-by-frame. It displays button presses, axis inputs, and timing data from recorded gameplay sessions.

## Build Commands

```bash
cargo build              # Development build
cargo build --release    # Optimized release build (LTO enabled)
cargo test               # Run all unit tests
cargo fmt                # Format code
cargo clippy             # Run linter
```

## Architecture

**MVC-like structure with immediate-mode GUI:**

```
src/
├─ main.rs          # eframe window setup, bootstraps InputLogViewerApp
├─ core/            # Data layer
│  ├─ log.rs        # InputLog, InputEvent, InputMapping, ButtonState, InputKind
│  ├─ parser.rs     # JSON parsing (parse_json), error handling
│  └─ playback.rs   # PlaybackState - frame position, timing, speed control
└─ gui/             # View layer
   ├─ app.rs        # InputLogViewerApp - main controller, AppState management
   ├─ controls.rs   # ControlsRenderer - playback buttons, speed selector, scrubber
   └─ timeline.rs   # TimelineRenderer - grid, events, frame indicators
```

**State flow:** User opens file → parser creates InputLog → TimelineRenderer visualizes it → ControlsRenderer provides interaction → PlaybackState tracks position/timing.

**AppState enum** controls what's enabled: `NoFileLoaded`, `Loading`, `Ready`, `Playing`, `Error`.

## Key Types

- `InputLog` - Container with metadata, mappings (ID→name/color), and frame events
- `InputEvent` - Button (Pressed/Held/Released) or Axis (1D value, 2D x/y)
- `PlaybackState` - Current frame, speed (0.1x-10.0x), timing logic, loop settings
- `ControlAction` - UI interaction enum dispatched from controls to app

## File Format

The app loads `.ilj` (JSON) files. See `assets/sample.ilj` for the expected structure. Binary `.ilb` support is planned.

## Specifications

- `specs/basic-specification.md` - Full feature spec (Japanese)
- `specs/mvp-phases.md` - Development roadmap (6 phases)
- `specs/dependencies.md` - Dependency selection rationale

## Current Development Phase

Phase 2 (playback controls) is complete. Phase 3 (filters, search, bookmarks) is next.

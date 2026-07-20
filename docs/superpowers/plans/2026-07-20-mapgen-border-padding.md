# Mapgen Border Padding Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deep-ocean breathing room around the mapgen viewport and start the desktop window maximized.

**Status:** Superseded for map breathing-room behavior. The render-only viewport padding from this plan was replaced by `2026-07-20-perlin-edge-buffer.md`, which shrinks Perlin land/shallow water away from the generated map edge. The wide-window centering and maximized startup portions remain useful.

**Architecture:** Original render-space padding approach was removed. Keep only the wide-window centered map area and maximized startup behavior from this plan.

**Tech Stack:** Rust 2024, Macroquad 0.4.15, Windows-only `windows-sys` for true maximize behavior.

## Global Constraints

- Keep map generation unchanged for this pass.
- Use tests before production code for viewport math.
- Preserve normal window chrome; do not switch to fullscreen unless explicitly requested later.
- Keep capture outputs in ignored folders.

---

### Task 1: Padded Map Viewport

**Files:**
- Modify: `src/scenes/mapgen.rs`
- Test: `src/scenes/mapgen/tests.rs`

**Interfaces:**
- Consumes: `map_source_rect(pan: Vec2, zoom: f32) -> Rect`, `clamp_pan(pan: Vec2, zoom: f32) -> Vec2`
- Produces: padded source rectangles whose min bounds can be negative and max bounds can exceed `MAP_SIZE`

- [x] **Step 1: Write failing tests**
- [x] **Step 2: Run the tests and verify they fail against old exact-map behavior**
- [x] **Step 3: Add virtual border constants and update source-rectangle/pan math**
- [x] **Step 4: Run viewport tests and full tests**
- [x] **Step 5: Center the square map inside wide map areas and paint unused map space as deep ocean**

### Task 2: Maximized Startup Window

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: Macroquad window title `TicTacToe`
- Produces: Windows-only maximize hook called once after Macroquad creates the window

- [x] **Step 1: Add direct `windows-sys` dependency with window messaging features**
- [x] **Step 2: Add `maximize_startup_window()` with a Windows implementation using `FindWindowW` and `ShowWindow(SW_MAXIMIZE)`**
- [x] **Step 3: Call it once after startup**
- [x] **Step 4: Run compile and tests**

### Task 3: Docs And Capture

**Files:**
- Modify: `ARCH.md`
- Modify: `GDD.md`
- Modify: `STATUS.md`
- Modify: `docs/superpowers/plans/2026-07-20-mapgen-border-padding.md`

- [x] **Step 1: Document the padded viewport behavior**
- [x] **Step 2: Capture mapgen biome view with a deterministic seed**
- [x] **Step 3: Inspect the capture and mark verification complete**

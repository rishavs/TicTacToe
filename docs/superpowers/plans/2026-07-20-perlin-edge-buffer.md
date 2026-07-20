# Perlin Edge Buffer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Shrink Perlin island land and shallow ocean away from the generated map edge without expanding the grid, leaving about two edge cells of deep ocean.

**Architecture:** Keep the existing square grid size and generation graph. Apply an edge-distance falloff inside the Perlin island mask so the outer cell band becomes ocean before the existing ocean/coast/shallow/deep classification runs.

**Tech Stack:** Rust 2024, Macroquad 0.4.15, `noise` crate.

## Global Constraints

- Perlin gets the two-cell deep-ocean edge buffer.
- Simplex remains unchanged, but its radial threshold values are named as config constants.
- Remove render-only fake viewport padding from camera/source-rectangle math.
- Keep wide-window centering with a neutral gray map area.
- Use tests before production changes.

---

### Task 1: Tests

**Files:**
- Modify: `src/scenes/mapgen/tests.rs`

- [x] Add tests for Perlin edge ocean buffer.
- [x] Add tests confirming Simplex is not forced to the same Perlin edge buffer rule.
- [x] Add tests reverting source-rectangle and pan math to generated map bounds.
- [x] Verify the new tests fail before implementation.

### Task 2: Implementation

**Files:**
- Modify: `src/scenes/mapgen.rs`
- Modify: `src/scenes/mapgen/noise.rs`

- [x] Remove `MAP_VIEWPORT_BORDER` from camera math.
- [x] Add Perlin-only edge-distance falloff in `IslandProfile::inside`.
- [x] Tune Perlin deep-ocean edge buffer down to two cells.
- [x] Name Simplex radial threshold constants; do not add the Perlin cell-buffer rule to Simplex.
- [x] Keep wide-window centering with a neutral gray map area.
- [x] Run focused tests.

### Task 3: Docs And Capture

**Files:**
- Modify: `ARCH.md`
- Modify: `GDD.md`
- Modify: `STATUS.md`
- Modify: `docs/superpowers/plans/2026-07-20-perlin-edge-buffer.md`

- [x] Document the Perlin edge buffer behavior.
- [x] Run `cargo fmt`, `cargo check`, and `cargo test`.
- [x] Capture and inspect Perlin mapgen biome view.

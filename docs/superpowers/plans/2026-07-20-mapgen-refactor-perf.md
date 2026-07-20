# Mapgen Refactor And Perf Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Separate mapgen rendering from generation and remove the synchronous generation stall from the Macroquad frame loop.

**Architecture:** Keep `src/scenes/mapgen.rs` responsible for scene state, input, deterministic generation, and tests. Move drawing and UI widget code to `src/scenes/mapgen/render.rs`. Start generation on a background worker and poll a channel from the frame loop; cache reusable noise generators in `IslandProfile`.

**Tech Stack:** Rust 2024, Macroquad 0.4.15, `noise`, `macroquad::rand::RandGenerator`, `std::sync::mpsc`, `std::thread`.

## Global Constraints

- Prefer Macroquad capabilities first, maintained Cargo libraries second, local code third.
- Keep map generation deterministic for the same seed and options.
- Keep rendering on the Macroquad main thread.
- Run `cargo fmt`, `cargo check`, `cargo test`, and visual capture.

---

### Task 1: Establish Baseline

**Files:**
- Read: `src/scenes/mapgen.rs`

**Interfaces:**
- Consumes: existing mapgen tests and debug capture environment.
- Produces: baseline timing notes for generation-heavy tests and mapgen screenshot launch.

- [x] Time generation-heavy tests with `cargo test map_generation -- --nocapture`.
- [x] Time a 4000-site mapgen screenshot launch.
- [x] Identify synchronous generation and per-sample noise construction as root causes.

### Task 2: Split Rendering

**Files:**
- Modify: `src/scenes/mapgen.rs`
- Create: `src/scenes/mapgen/render.rs`

**Interfaces:**
- Consumes: `MapgenScene`, `PolyMap`, `ViewMode`, and drawing helper data from `mapgen.rs`.
- Produces: `render::draw(scene: &mut MapgenScene, layout: MapgenLayout, source_rect: Rect)`.

- [x] Move Macroquad UI widgets and map drawing into `render.rs`.
- [x] Render polygons by center borders instead of repeated center-neighbor edge lookup.
- [x] Render borders/rivers by iterating each edge once.
- [x] Remove obsolete render helpers from `mapgen.rs`.

### Task 3: Make Generation Non-Blocking

**Files:**
- Modify: `src/scenes/mapgen.rs`
- Modify: `src/scenes/mapgen/render.rs`

**Interfaces:**
- Consumes: `GenerationRequest { seed_text, island_type, point_type, point_count }`.
- Produces: `MapgenScene::start_generation`, `MapgenScene::poll_generation`, and `Option<PolyMap>` scene storage.

- [x] Replace direct `PolyMap` scene storage with `Option<PolyMap>`.
- [x] Add a background worker that sends completed maps over `mpsc`.
- [x] Poll generation from `MapgenScene::update`.
- [x] Keep rendering active while a map is in progress.

### Task 4: Cache Noise Generators

**Files:**
- Modify: `src/scenes/mapgen.rs`

**Interfaces:**
- Consumes: `noise::Fbm<Perlin>` and `noise::OpenSimplex`.
- Produces: cached noise fields on `IslandProfile`.

- [x] Store reusable Perlin/fBm and OpenSimplex generators on `IslandProfile`.
- [x] Sample cached generators in `IslandProfile::inside`.
- [x] Keep test-only wrapper functions for deterministic noise coverage.

### Task 5: Verify

**Files:**
- Modify: `ARCH.md`
- Modify: `STATUS.md`
- Modify: `AGENTS.md`

**Interfaces:**
- Consumes: implemented refactor.
- Produces: updated docs and measured verification.

- [x] Run `cargo fmt`.
- [x] Run `cargo check`.
- [x] Run `cargo test`.
- [x] Re-run generation-heavy timing.
- [x] Capture and inspect mapgen screenshots.
- [x] Update architecture, status, and capture docs.

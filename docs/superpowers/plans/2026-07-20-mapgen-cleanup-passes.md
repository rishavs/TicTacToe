# Mapgen Cleanup Passes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Continue the mapgen cleanup in small, verifiable passes that reduce file size, remove dead-end abstractions, and preserve the non-blocking generation behavior.

**Architecture:** Keep `src/scenes/mapgen.rs` as the scene shell and worker coordinator. Move deterministic map data and generation code into focused child modules under `src/scenes/mapgen/`. Keep Macroquad drawing in `src/scenes/mapgen/render.rs`, and keep rendering on the main thread.

**Tech Stack:** Rust 2024, Macroquad 0.4.15, `noise`, `macroquad::rand::RandGenerator`, `std::sync::mpsc`, `std::thread`.

## Global Constraints

- Prefer Macroquad capabilities first, maintained Cargo libraries second, local code third.
- Preserve deterministic generation for the same seed and options.
- Keep rendering on the Macroquad main thread.
- Keep each pass independently shippable.
- After each pass run `cargo fmt`, `cargo check`, `cargo test`, and a Mapgen screenshot capture.

---

### Pass 1: Extract Map Model Types

**Files:**
- Modify: `src/scenes/mapgen.rs`
- Create: `src/scenes/mapgen/model.rs`
- Modify: `src/scenes/mapgen/render.rs`

**Interfaces:**
- Consumes: current `Center`, `Corner`, `Edge`, `NoisyEdge`, and `PolyMap` structs.
- Produces: `model::{Center, Corner, Edge, NoisyEdge, PolyMap}` available to scene, render, and tests.

- [x] Move `Center`, `Corner`, `Edge`, `NoisyEdge`, and `PolyMap` struct definitions into `model.rs`.
- [x] Mark only fields used outside `model.rs` as `pub(super)`.
- [x] Keep `PolyMap::generate` callable from `mapgen.rs`.
- [x] Update `render.rs` imports to use the model types through `super::*` or explicit `super::model::*`.
- [x] Run `cargo fmt`.
- [x] Run `cargo check`; expected: no warnings.
- [x] Run `cargo test`; expected: 31 tests pass.
- [x] Capture `.qa-captures/mapgen-pass-1-model.png` with `TICTACTOE_START_SCENE=mapgen`.

### Pass 2: Extract Generation Pipeline

**Files:**
- Modify: `src/scenes/mapgen.rs`
- Modify: `src/scenes/mapgen/model.rs`
- Create: `src/scenes/mapgen/generate.rs`

**Interfaces:**
- Consumes: `PolyMap`, `IslandType`, `PointType`, `GenerationRequest`, `MAP_SIZE`, `LAKE_THRESHOLD`, RNG helpers, and biome helpers.
- Produces: `generate::generate_map(seed_text: &str, island_type: IslandType, point_type: PointType, point_count: usize) -> PolyMap`.

- [x] Move `PolyMap::generate` and generation-only methods into `generate.rs`.
- [x] Keep `PolyMap` as a data container in `model.rs`.
- [x] Add `generate_map(...) -> PolyMap` as the single public generation entrypoint.
- [x] Update the background worker to call `generate_map(...)`.
- [x] Move tests that assert generation behavior into a `generate` test module or keep them in `mapgen.rs` while importing `generate_map`.
- [x] Run `cargo fmt`.
- [x] Run `cargo check`; expected: no warnings.
- [x] Run `cargo test`; expected: 31 tests pass.
- [x] Capture `.qa-captures/mapgen-pass-2-generate.png`.

### Pass 3: Extract Noise, RNG, And Seed Helpers

**Files:**
- Modify: `src/scenes/mapgen.rs`
- Modify: `src/scenes/mapgen/generate.rs`
- Create: `src/scenes/mapgen/random.rs`
- Create: `src/scenes/mapgen/noise.rs`
- Create: `src/scenes/mapgen/seed.rs`

**Interfaces:**
- Consumes: `map_rng`, `map_random_*`, `IslandProfile`, noise sampling helpers, `parse_seed`, `random_seed_text`, and seed input helpers.
- Produces: focused helper modules:
  - `random::{MapRng, map_rng, map_random_i32, map_random_f32}`
  - `noise::{IslandProfile, fractal_noise_2d, simplex_fractal_noise_2d}`
  - `seed::{parse_seed, random_seed_text, is_seed_char, push_seed_char}`

- [x] Move Macroquad RNG wrappers into `random.rs`.
- [x] Move `IslandProfile` and noise helpers into `noise.rs`.
- [x] Move seed parsing and seed input helpers into `seed.rs`.
- [x] Keep test-only functions behind `#[cfg(test)]`.
- [x] Remove any imports made redundant by the move.
- [x] Run `cargo fmt`.
- [x] Run `cargo check`; expected: no warnings.
- [x] Run `cargo test`; expected: 31 tests pass.
- [x] Capture `.qa-captures/mapgen-pass-3-helpers.png`.

### Pass 4: Extract Biome And Color Logic

**Files:**
- Modify: `src/scenes/mapgen.rs`
- Modify: `src/scenes/mapgen/generate.rs`
- Modify: `src/scenes/mapgen/render.rs`
- Create: `src/scenes/mapgen/biome.rs`

**Interfaces:**
- Consumes: `get_biome`, `biome_color`, `interpolate_color`, and `calculate_lighting`.
- Produces:
  - `biome::get_biome(center: &Center) -> &'static str`
  - `biome::biome_color(biome: &str) -> u32`
  - `biome::interpolate_color(color0: u32, color1: u32, f: f32) -> u32`
  - `biome::calculate_lighting(...) -> f32`

- [x] Move biome classification and color math into `biome.rs`.
- [x] Update generation to call `biome::get_biome`.
- [x] Update rendering slope color logic to call `biome` helpers.
- [x] Run `cargo fmt`.
- [x] Run `cargo check`; expected: no warnings.
- [x] Run `cargo test`; expected: 31 tests pass.
- [x] Capture `.qa-captures/mapgen-pass-4-biome.png`.

### Pass 5: Slim Scene Shell And Tests

**Files:**
- Modify: `src/scenes/mapgen.rs`
- Modify: child modules under `src/scenes/mapgen/`
- Optional create: `src/scenes/mapgen/tests.rs`

**Interfaces:**
- Consumes: all extracted modules.
- Produces: `mapgen.rs` focused on public scene `update()`, `MapgenScene`, `GenerationRequest`, layout/input, and worker polling.

- [x] Move module-specific tests next to the modules they exercise.
- [x] Keep scene/input tests in `mapgen.rs` or `tests.rs`.
- [x] Remove any re-export or helper that only exists because code used to live in one file.
- [x] Re-run the line count for `mapgen.rs`; target: less than 600 lines.
- [x] Run `cargo fmt`.
- [x] Run `cargo check`; expected: no warnings.
- [x] Run `cargo test`; expected: 31 tests pass.
- [x] Capture `.qa-captures/mapgen-pass-5-scene-shell.png`.
- [x] Update `ARCH.md` and `STATUS.md` to reflect final module layout.

### Stop Conditions

- Stop after any pass if generation determinism changes unexpectedly.
- Stop after any pass if the 32000-point launch no longer reaches the loading frame quickly.
- Stop before adding new crates unless Macroquad lacks the capability and the crate is clearly maintained, popular, and a real simplification.

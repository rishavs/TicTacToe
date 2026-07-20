# Island Generation (Pass 2 — Simplex Noise Biomes) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the uniform green grid with a procedurally-generated island using 2-layer simplex noise (elevation + moisture) with radial falloff, producing 16 biome types (2 water + 14 land) colored by a palette.

**Architecture:** The `noise` crate provides `OpenSimplex`. `generator.rs` implements `generate_island(grid, seed)` which fills the Grid's tile Vec with biome colors. The mapgen scene calls this instead of uniform fill. Everything else (Grid, to_texture, rendering) is unchanged from Pass 1.

**Tech Stack:** Rust edition 2024, macroquad 0.4.15, noise 0.9

## Global Constraints

- Grid size: 256×256
- 16 biome types: 2 water (Deep, Shallow) + 14 land (Badlands, Mountain, Snow, Glacier, Steppe, Highland, Taiga, Moor, Desert, Savannah, Grassland, Forest, Beach, Coast, Marsh, Swamp)
- Two OpenSimplex layers: elevation at frequency 1/64, moisture at frequency 1/48
- Radial falloff with exponent 1.5, center of grid
- Water threshold: elevation < 0.35 = Deep, < 0.42 = Shallow
- Land band thresholds: Low [0.42, 0.55), Mid [0.55, 0.70), High [0.70, 0.82), Peak [0.82, 1.0]
- Moisture bands: Arid [0, 0.25), Dry [0.25, 0.50), Temperate [0.50, 0.75), Wet [0.75, 1.0]
- Seed: hardcoded 42 (will be parameterized later)
- Noise values mapped from [-1, 1] to [0, 1] via `(v + 1.0) / 2.0`

---

### Task 1: Add `noise` dependency

**Files:**
- Modify: `Cargo.toml`

**Interfaces:**
- Produces: `noise` crate available for import

- [ ] **Step 1: Add noise to `Cargo.toml`**

```toml
[package]
name = "tictactoe"
version = "0.1.0"
edition = "2024"

[dependencies]
macroquad = "0.4.15"
noise = "0.9"
```

- [ ] **Step 2: Verify dependency resolves**

Run: `cargo check`
Expected: noise crate downloads and compiles, no errors in existing code.

---

### Task 2: Implement `src/mapgen/generator.rs`

**Files:**
- Modify: `src/mapgen/generator.rs` (replace stub)

**Interfaces:**
- Consumes: `Grid` from `crate::mapgen::Grid`
- Produces: `pub fn generate_island(grid: &mut Grid, seed: u32)`

- [ ] **Step 1: Replace `src/mapgen/generator.rs`**

```rust
use crate::mapgen::Grid;
use macroquad::prelude::*;
use noise::{NoiseFn, OpenSimplex};

const ELEV_FREQ: f64 = 1.0 / 64.0;
const MOIST_FREQ: f64 = 1.0 / 48.0;

const WATER_DEEP: f64 = 0.35;
const WATER_SHALLOW: f64 = 0.42;
const BAND_LOW: f64 = 0.42;
const BAND_MID: f64 = 0.55;
const BAND_HIGH: f64 = 0.70;
const BAND_PEAK: f64 = 0.82;

const MOIST_ARID: f64 = 0.25;
const MOIST_DRY: f64 = 0.50;
const MOIST_TEMP: f64 = 0.75;

fn biome_color(elevation: f64, moisture: f64) -> Color {
    if elevation < WATER_DEEP {
        return Color::new(0.05, 0.1, 0.35, 1.0);
    }
    if elevation < WATER_SHALLOW {
        return Color::new(0.15, 0.3, 0.55, 1.0);
    }

    match () {
        _ if elevation >= BAND_PEAK && moisture < MOIST_ARID => Color::new(0.7, 0.4, 0.2, 1.0),
        _ if elevation >= BAND_PEAK && moisture < MOIST_DRY => Color::new(0.5, 0.45, 0.4, 1.0),
        _ if elevation >= BAND_PEAK && moisture < MOIST_TEMP => Color::new(0.95, 0.95, 0.95, 1.0),
        _ if elevation >= BAND_PEAK => Color::new(0.85, 0.9, 0.95, 1.0),

        _ if elevation >= BAND_HIGH && moisture < MOIST_ARID => Color::new(0.6, 0.55, 0.35, 1.0),
        _ if elevation >= BAND_HIGH && moisture < MOIST_DRY => Color::new(0.55, 0.5, 0.3, 1.0),
        _ if elevation >= BAND_HIGH && moisture < MOIST_TEMP => Color::new(0.25, 0.45, 0.3, 1.0),
        _ if elevation >= BAND_HIGH => Color::new(0.35, 0.3, 0.4, 1.0),

        _ if elevation >= BAND_MID && moisture < MOIST_ARID => Color::new(0.9, 0.8, 0.4, 1.0),
        _ if elevation >= BAND_MID && moisture < MOIST_DRY => Color::new(0.7, 0.7, 0.3, 1.0),
        _ if elevation >= BAND_MID && moisture < MOIST_TEMP => Color::new(0.3, 0.65, 0.2, 1.0),
        _ if elevation >= BAND_MID => Color::new(0.15, 0.45, 0.1, 1.0),

        _ if moisture < MOIST_ARID => Color::new(0.85, 0.8, 0.5, 1.0),
        _ if moisture < MOIST_DRY => Color::new(0.7, 0.68, 0.45, 1.0),
        _ if moisture < MOIST_TEMP => Color::new(0.35, 0.5, 0.3, 1.0),
        _ => Color::new(0.25, 0.4, 0.25, 1.0),
    }
}

pub fn generate_island(grid: &mut Grid, seed: u32) {
    let elev_noise = OpenSimplex::new(seed);
    let moist_noise = OpenSimplex::new(seed.wrapping_add(1));

    let half = grid.width as f64 / 2.0;
    let max_dist = half * 1.42;

    for y in 0..grid.height {
        for x in 0..grid.width {
            let nx = x as f64 * ELEV_FREQ;
            let ny = y as f64 * ELEV_FREQ;
            let mut elevation = (elev_noise.get([nx, ny]) + 1.0) / 2.0;

            let dx = x as f64 - half;
            let dy = y as f64 - half;
            let dist = (dx * dx + dy * dy).sqrt();
            let falloff = (dist / max_dist).powf(1.5).min(1.0);
            elevation *= 1.0 - falloff;

            let mx = x as f64 * MOIST_FREQ;
            let my = y as f64 * MOIST_FREQ;
            let moisture = (moist_noise.get([mx, my]) + 1.0) / 2.0;

            let idx = grid.width * y + x;
            grid.tiles[idx] = biome_color(elevation, moisture);
        }
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: generator.rs compiles. Unused function warning is fine until Task 3 wires it in.

---

### Task 3: Wire `generate_island` into the mapgen scene

**Files:**
- Modify: `src/scenes/mapgen.rs` (lines 17–25)

**Interfaces:**
- Consumes: `generate_island` from `crate::mapgen::generator`
- Modifies: MapgenState initialization

- [ ] **Step 1: Update state initialization in `src/scenes/mapgen.rs`**

Change the `STATE.get_or_init` closure from:

```rust
let grid = Grid::new(GRID_SIZE, GRID_SIZE, GREEN);
```

To:

```rust
use crate::mapgen::generator;

let mut grid = Grid::new(GRID_SIZE, GRID_SIZE, BLACK);
generator::generate_island(&mut grid, 42);
```

Full replacement — the `use` should go at the top of the file with other imports, and the closure body changes. The complete updated file:

```rust
use crate::scenes::Scene;
use crate::mapgen::Grid;
use crate::mapgen::generator;
use macroquad::prelude::*;
use std::sync::{Mutex, OnceLock};

const GRID_SIZE: usize = 256;

struct MapgenState {
    texture: Texture2D,
    pan: Vec2,
    zoom: f32,
}

static STATE: OnceLock<Mutex<MapgenState>> = OnceLock::new();

pub fn update() -> Option<Scene> {
    let state = STATE.get_or_init(|| {
        let mut grid = Grid::new(GRID_SIZE, GRID_SIZE, BLACK);
        generator::generate_island(&mut grid, 42);
        let texture = grid.to_texture();
        Mutex::new(MapgenState {
            texture,
            pan: Vec2::ZERO,
            zoom: 500.0,
        })
    });

    let mut state = state.lock().unwrap();

    let dt = get_frame_time();
    let pan_speed = 300.0 * dt * (state.zoom / 500.0);

    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        state.pan.y -= pan_speed;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        state.pan.y += pan_speed;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        state.pan.x -= pan_speed;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        state.pan.x += pan_speed;
    }

    let (_, wy) = mouse_wheel();
    state.zoom = (state.zoom - wy * 50.0).clamp(100.0, 2000.0);

    let center = vec3(
        GRID_SIZE as f32 / 2.0 + state.pan.x,
        0.0,
        GRID_SIZE as f32 / 2.0 + state.pan.y,
    );

    clear_background(DARKGRAY);

    let height = state.zoom;
    let offset = height * 0.7;
    let camera = Camera3D {
        position: vec3(center.x, height, center.z + offset),
        up: vec3(0.0, 1.0, 0.0),
        target: center,
        ..Default::default()
    };
    set_camera(&camera);

    draw_grid(64, 4.0, BLACK, GRAY);

    let plane_center = GRID_SIZE as f32 / 2.0;
    draw_plane(
        vec3(plane_center, 0.0, plane_center),
        vec2(GRID_SIZE as f32, GRID_SIZE as f32),
        Some(&state.texture),
        WHITE,
    );

    set_default_camera();

    if is_key_pressed(KeyCode::Escape) {
        return Some(Scene::MainMenu);
    }

    None
}
```

- [ ] **Step 2: Build and verify**

Run: `cargo build`
Expected: Successful compilation, zero warnings.

---

### Task 4: Final verification

- [ ] **Step 1: Run `cargo check` for zero warnings**

Run: `cargo check`
Expected: Clean output.

- [ ] **Step 2: Confirm binary exists**

Run: `Test-Path -LiteralPath "target\debug\tictactoe.exe"`
Expected: `True`

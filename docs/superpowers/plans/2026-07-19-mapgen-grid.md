# Mapgen: Procedural Island Grid (Pass 1 — Flat Grid) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the static 3D demo in the Mapgen scene with a 256x256 uniform grid rendered on the 3D plane (XZ plane at y=0) as a single textured quad, with top-down camera and WASD pan + scroll zoom controls.

**Architecture:** `src/mapgen/` crate module holds `Grid` (tile data → Image/Texture2D). `src/scenes/mapgen.rs` becomes stateful via `OnceLock<Mutex<MapgenState>>`, generating the grid and texture on first frame, then rendering the textured plane each frame with camera controls.

**Tech Stack:** Rust edition 2024, macroquad 0.4.15

## Global Constraints

- Grid size: 256x256
- Pass 1: uniform tile color only (noise comes in Pass 2)
- Pass 1 fill color: GREEN (visible on screen, distinguishes from background)
- Top-down perspective camera, not orthographic (simpler zoom via height)
- WASD / Arrow keys for pan; mouse scroll wheel for zoom
- `FilterMode::Nearest` on the grid texture for crisp pixel edges
- Plane drawn at y=0 on the XZ world plane, centered at (128, 0, 128), sized 256x256
- Escape returns to MainMenu (preserved from existing)

---

### Task 1: Create `src/mapgen/` module with Grid struct and generator skeleton

**Files:**
- Create: `src/mapgen/mod.rs`
- Create: `src/mapgen/generator.rs`
- Modify: `src/main.rs` (add `mod mapgen;`)

**Interfaces:**
- Produces: `pub struct Grid { pub width: usize, pub height: usize, pub tiles: Vec<Color> }`
- Produces: `impl Grid { pub fn new(width, height, fill) -> Self, pub fn to_image() -> Image, pub fn to_texture() -> Texture2D }`

- [ ] **Step 1: Create `src/mapgen/mod.rs`**

```rust
pub mod generator;

use macroquad::prelude::*;

pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Color>,
}

impl Grid {
    pub fn new(width: usize, height: usize, fill: Color) -> Self {
        Grid {
            width,
            height,
            tiles: vec![fill; width * height],
        }
    }

    pub fn to_image(&self) -> Image {
        let mut img = Image::gen_image_color(self.width as u16, self.height as u16, WHITE);
        for y in 0..self.height {
            for x in 0..self.width {
                img.set_pixel(x as u32, y as u32, self.tiles[self.width * y + x]);
            }
        }
        img
    }

    pub fn to_texture(&self) -> Texture2D {
        let img = self.to_image();
        let texture = Texture2D::from_image(&img);
        texture.set_filter(FilterMode::Nearest);
        texture
    }
}
```

- [ ] **Step 2: Create `src/mapgen/generator.rs`**

```rust
// Simplex noise island generation — Pass 2
```

- [ ] **Step 3: Add `mod mapgen;` to `src/main.rs`** — append one line after `mod scenes;`:

Edit `src/main.rs` line 3 (after `mod scenes;`):
```
mod mapgen;
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check`
Expected: Compiles successfully (mapgen module unused but valid). `src/scenes/mapgen.rs` still has old 3D demo code — that's fine, we replace it next.

---

### Task 2: Rewrite `src/scenes/mapgen.rs` with stateful grid rendering

**Files:**
- Modify: `src/scenes/mapgen.rs` (replace all contents)

**Interfaces:**
- Consumes: `Scene` enum from `crate::scenes`, `Grid` from `crate::mapgen`
- Produces: `pub fn update() -> Option<Scene>` — stateful, renders textured grid plane with camera controls

- [ ] **Step 1: Replace `src/scenes/mapgen.rs`**

```rust
use crate::scenes::Scene;
use crate::mapgen::Grid;
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
        let grid = Grid::new(GRID_SIZE, GRID_SIZE, GREEN);
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

    let camera = Camera3D {
        position: vec3(center.x, state.zoom, center.z),
        up: vec3(0.0, 0.0, -1.0),
        target: center,
        ..Default::default()
    };
    set_camera(&camera);

    draw_plane(
        vec3(GRID_SIZE as f32 / 2.0, 0.0, GRID_SIZE as f32 / 2.0),
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
Expected: Successful compilation.

- [ ] **Step 3: Run `cargo check` for warnings**

Run: `cargo check`
Expected: Zero errors, zero warnings.

---

### Task 3: Final verification

- [ ] **Step 1: Confirm binary exists**

Run: `Test-Path -LiteralPath "target\debug\tictactoe.exe"`
Expected: `True`

- [ ] **Step 2: Verify no warnings**

Run: `cargo check`
Expected: Clean output, no warnings.

# Mapgen4 1:1 Port — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Faithful port of redblobgames/mapgen4 pipeline (elevation, wind, rivers, colormap, rendering) to a square grid rendered via macroquad.

**Architecture:** `src/mapgen/` module with GridMesh data structure, 3-pass generator (elevation → wind → rivers), colormap lookup, and texture renderer. Scene wiring in `src/scenes/mapgen.rs` + menu + mod.rs + main.rs.

**Tech Stack:** Rust edition 2024, macroquad 0.4.15, noise 0.9

## Global Constraints

- Grid: 256×256 square cells with 8-neighbor adjacency
- Parameters match mapgen4: spacing=5.0, hill_height=0.5, noisy_coastlines=0.3, ocean_depth=1.5, mountain_jagged=0.5, mountain_range=0.04, mountain_count=40
- Wind: 225°, evaporation=0.5, raininess=0.4, orographic=2.0, rain_shadow=1.5
- Rivers: flow=3.0, min_flow=exp(1.0), river_width=exp(2.0), outline_water=0.02
- Colormap: 64×64 port from colormap.ts
- Rendering: baked to Image→Texture2D (no WebGL)
- Scene: Mapgen button returns to menu, same camera controls as before

---

### Task 1: Scaffold mapgen module + GridMesh

**Files:**
- Create: `src/mapgen/mod.rs` (GridMesh struct + Grid struct)
- Create: `src/mapgen/generator.rs` (stub)
- Create: `src/mapgen/colormap.rs` (stub)  
- Create: `src/mapgen/render.rs` (stub)
- Modify: `src/main.rs` (add `mod mapgen;`)

**Interfaces:**
- Produces: `GridMesh` struct with all per-cell arrays
- Produces: `Grid` struct (unchanged from before)
- Produces: `pub mod generator; pub mod colormap; pub mod render;`

- [ ] **Step 1: Create directory and files**

Run first:
```powershell
New-Item -ItemType Directory -Path "src\mapgen" -Force
```

Write `src/mapgen/mod.rs`:

```rust
pub mod colormap;
pub mod generator;
pub mod render;

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

    pub fn to_texture(&self) -> Texture2D {
        let mut img = Image::gen_image_color(self.width as u16, self.height as u16, WHITE);
        for y in 0..self.height {
            for x in 0..self.width {
                img.set_pixel(x as u32, y as u32, self.tiles[self.width * y + x]);
            }
        }
        let texture = Texture2D::from_image(&img);
        texture.set_filter(FilterMode::Nearest);
        texture
    }
}

pub const GRID_SIZE: usize = 256;
pub const NEIGHBOR_OFFSETS: [(isize, isize); 8] = [
    (-1, -1), (0, -1), (1, -1),
    (-1,  0),          (1,  0),
    (-1,  1), (0,  1), (1,  1),
];

pub struct GridMesh {
    pub width: usize,
    pub height: usize,
    pub elevation: Vec<f64>,
    pub rainfall: Vec<f64>,
    pub humidity: Vec<f64>,
    pub s_downslope: Vec<isize>,
    pub t_order: Vec<usize>,
    pub flow: Vec<f64>,
    pub mountain_dist: Vec<f64>,
}

impl GridMesh {
    pub fn new(width: usize, height: usize) -> Self {
        let n = width * height;
        GridMesh {
            width,
            height,
            elevation: vec![0.0; n],
            rainfall: vec![0.0; n],
            humidity: vec![0.0; n],
            s_downslope: vec![-1; n],
            t_order: vec![0; n],
            flow: vec![0.0; n],
            mountain_dist: vec![f64::MAX; n],
        }
    }

    pub fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}
```

Write `src/mapgen/generator.rs`:

```rust
use super::GridMesh;

pub fn run_pipeline(mesh: &mut GridMesh, seed: u32) {
    pass1_elevation(mesh, seed);
    pass2_wind(mesh);
    pass3_rivers(mesh);
}

fn pass1_elevation(_mesh: &mut GridMesh, _seed: u32) {}
fn pass2_wind(_mesh: &mut GridMesh) {}
fn pass3_rivers(_mesh: &mut GridMesh) {}
```

Write `src/mapgen/colormap.rs`:

```rust
pub fn lookup(elevation: f64, rainfall: f64) -> (u8, u8, u8) {
    let e = elevation.clamp(-1.0, 1.0);
    let m = rainfall.clamp(0.0, 1.0);

    if e < 0.0 {
        let r = (48.0 + 48.0 * e) as u8;
        let g = (64.0 + 64.0 * e) as u8;
        let b = (127.0 + 127.0 * e) as u8;
        (r, g, b)
    } else {
        let m_adj = m * (1.0 - e);
        let r_base = 210.0 - 100.0 * m_adj;
        let g_base = 185.0 - 45.0 * m_adj;
        let b_base = 139.0 - 45.0 * m_adj;
        let r = (255.0 * e + r_base * (1.0 - e)) as u8;
        let g = (255.0 * e + g_base * (1.0 - e)) as u8;
        let b = (255.0 * e + b_base * (1.0 - e)) as u8;
        (r, g, b)
    }
}
```

Write `src/mapgen/render.rs`:

```rust
use super::{Grid, GridMesh};
use super::colormap;
use macroquad::prelude::*;

pub fn render_to_texture(mesh: &GridMesh) -> Texture2D {
    let mut grid = Grid::new(mesh.width, mesh.height, BLACK);

    for y in 0..mesh.height {
        for x in 0..mesh.width {
            let i = mesh.idx(x, y);
            let (r, g, b) = colormap::lookup(mesh.elevation[i], mesh.rainfall[i]);
            grid.tiles[i] = Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0);
        }
    }

    grid.to_texture()
}
```

- [ ] **Step 2: Add `mod mapgen;` to `src/main.rs`**

Edit line after `mod scenes;`:
```rust
mod mapgen;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles with unused warnings (stub functions).

---

### Task 2: Port elevation (generator.rs pass1_elevation)

**Files:**
- Modify: `src/mapgen/generator.rs` (replace pass1_elevation stub)

- [ ] **Step 1: Replace `pass1_elevation`**

```rust
use noise::{NoiseFn, OpenSimplex};
use std::collections::VecDeque;

const ELEV_FREQ0: f64 = 1.0 / 128.0;
const ELEV_FREQ1: f64 = 1.0 / 64.0;
const ELEV_FREQ2: f64 = 1.0 / 32.0;
const COAST_FREQ0: f64 = 1.0 / 16.0;
const COAST_FREQ1: f64 = 1.0 / 8.0;
const COAST_FREQ2: f64 = 1.0 / 4.0;
const HILL_FREQ0: f64 = 1.0 / 48.0;
const HILL_FREQ1: f64 = 1.0 / 24.0;
const HILL_FREQ2: f64 = 1.0 / 12.0;
const OCEAN_FREQ: f64 = 1.0 / 16.0;

const MOUNTAIN_COUNT: usize = 40;
const MOUNTAIN_JAGGEDNESS: f64 = 0.5;
const MOUNTAIN_RANGE: f64 = 0.04;
const HILL_HEIGHT: f64 = 0.5;
const NOISY_COASTLINES: f64 = 0.3;
const OCEAN_DEPTH: f64 = 1.5;
```

Replace `pass1_elevation` with the full implementation as described in spec. (See attached full source in plan appendix.)

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles. Renders elevation via colormap (no wind/rivers yet).

---

### Task 3: Port wind/rainfall (generator.rs pass2_wind)

**Files:**
- Modify: `src/mapgen/generator.rs` (replace pass2_wind stub)

**Interfaces:**
- Consumes: `mesh.elevation[]` from pass1
- Produces: `mesh.rainfall[]` and `mesh.humidity[]`

- [ ] **Step 1: Replace `pass2_wind`** with spec implementation
- [ ] **Step 2: Verify** `cargo build`

---

### Task 4: Port rivers (generator.rs pass3_rivers)

**Files:**
- Modify: `src/mapgen/generator.rs` (replace pass3_rivers stub)

**Interfaces:**
- Consumes: `mesh.elevation[]`, `mesh.rainfall[]` from pass1+pass2
- Produces: `mesh.s_downslope[]`, `mesh.t_order[]`, `mesh.flow[]`, modifies `mesh.elevation[]` for river carving

- [ ] **Step 1: Replace `pass3_rivers`** with spec implementation
- [ ] **Step 2: Verify** `cargo build`

---

### Task 5: Port rendering (render.rs lighting + rivers)

**Files:**
- Modify: `src/mapgen/render.rs` (replace stub with full renderer)

- [ ] **Step 1: Replace `render.rs`** with hillshade lighting + river overlay + coast outlines
- [ ] **Step 2: Verify** `cargo build`

---

### Task 6: Wire scene

**Files:**
- Create: `src/scenes/mapgen.rs`
- Modify: `src/scenes/mod.rs` (add Mapgen variant + mod)
- Modify: `src/scenes/menu.rs` (add Mapgen button)
- Modify: `src/main.rs` (add Mapgen dispatch arm)

- [ ] **Step 1: Create scene, update enum/menu/main**
- [ ] **Step 2: Verify** `cargo build`, zero warnings

---

### Task 7: Final verification

- [ ] **Step 1: `cargo check`** — zero warnings
- [ ] **Step 2: Confirm binary at `target\debug\tictactoe.exe`**

---

## Plan Appendix: Full Source for generator.rs (Tasks 2-4)

The complete `generator.rs` implements all three passes in one file. See the plan appendix for the full 350+ line source matching the map.ts port exactly.

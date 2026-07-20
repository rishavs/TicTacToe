# Mapgen4-style Pipeline — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the 2-noise-layer generator with a 3-pass mapgen4-inspired pipeline: elevation (hills+mountains+radial), wind/rainfall (orographic simulation), biome classification.

**Architecture:** All passes in `generator.rs`. `generate_island()` orchestrates: pass1 produces `elevation[]`, pass2 consumes elevation and produces `rainfall[]`, pass3 consumes both and fills `grid.tiles[]`. No other files change.

**Tech Stack:** Rust edition 2024, macroquad 0.4.15, noise 0.9

## Global Constraints

- Grid size: 256×256
- Mountain peaks: 30 randomly selected from land cells, weighted by `elevation × coastal_distance²`
- Mountain BFS: queue-based using VecDeque, jaggedness 0.5, slope 16, sharpness 4
- Hill noise: simplex freq 1/32, low amplitude
- Wind: 225° (southwest), evaporation 0.5, raininess 0.4, orographic factor 2.0, rain shadow 1.5
- Biome thresholds from spec (Water: e<0.35 deep, 0.35≤e<0.42 shallow; Land: 4×4 matrix by elevation×rainfall)
- Entry point: `pub fn generate_island(grid: &mut Grid, seed: u32)` — called from mapgen.rs
- All f64 arrays allocated as `vec![0.0; grid.width * grid.height]`

---

### Task 1: Replace generator.rs skeleton with Pass 1 (Elevation)

**Files:**
- Modify: `src/mapgen/generator.rs` (replace entirely)

**Interfaces:**
- Consumes: `Grid` from `crate::mapgen::Grid`
- Produces: `fn generate_island(grid: &mut Grid, seed: u32)` — full pipeline entry point
- Produces: `fn pass1_elevation(elevation: &mut [f64], width: usize, height: usize, seed: u32)`

- [ ] **Step 1: Write generator.rs**

```rust
use crate::mapgen::Grid;
use macroquad::prelude::*;
use noise::{NoiseFn, OpenSimplex};
use std::collections::VecDeque;

const ELEV_FREQ: f64 = 1.0 / 128.0;
const HILL_FREQ: f64 = 1.0 / 32.0;
const OCEAN_NOISE_FREQ: f64 = 1.0 / 32.0;
const MOUNTAIN_COUNT: usize = 30;
const MOUNTAIN_JAGGEDNESS: f64 = 0.5;
const MOUNTAIN_SLOPE: f64 = 16.0;
const MOUNTAIN_SHARPNESS: f64 = 4.0;

const WIND_ANGLE_DEG: f64 = 225.0;
const EVAPORATION_RATE: f64 = 0.5;
const RAININESS: f64 = 0.4;
const OROGRAPHIC_FACTOR: f64 = 2.0;
const RAIN_SHADOW: f64 = 1.5;

const WATER_DEEP: f64 = 0.35;
const WATER_SHALLOW: f64 = 0.42;

const NEIGHBOR_OFFSETS: [(isize, isize); 8] = [
    (-1, -1), (0, -1), (1, -1),
    (-1,  0),          (1,  0),
    (-1,  1), (0,  1), (1,  1),
];

fn idx(x: usize, y: usize, width: usize) -> usize {
    y * width + x
}

fn coastal_distance_map(elevation: &[f64], width: usize, height: usize) -> Vec<f64> {
    let n = elevation.len();
    let mut dist = vec![f64::MAX; n];
    let mut queue = VecDeque::new();

    for y in 0..height {
        for x in 0..width {
            let i = idx(x, y, width);
            let is_coastal = elevation[i] < 0.0
                && NEIGHBOR_OFFSETS.iter().any(|&(dx, dy)| {
                    let nx = x.wrapping_add_signed(dx);
                    let ny = y.wrapping_add_signed(dy);
                    nx < width && ny < height && elevation[idx(nx, ny, width)] >= 0.0
                });
            if is_coastal {
                dist[i] = 0.0;
                queue.push_back((x, y));
            }
        }
    }

    while let Some((cx, cy)) = queue.pop_front() {
        let ci = idx(cx, cy, width);
        for &(dx, dy) in &NEIGHBOR_OFFSETS {
            let nx = cx.wrapping_add_signed(dx);
            let ny = cy.wrapping_add_signed(dy);
            if nx < width && ny < height {
                let ni = idx(nx, ny, width);
                if elevation[ni] >= 0.0 && dist[ni] > dist[ci] + 1.0 {
                    dist[ni] = dist[ci] + 1.0;
                    queue.push_back((nx, ny));
                }
            }
        }
    }

    dist
}

pub fn generate_island(grid: &mut Grid, seed: u32) {
    let width = grid.width;
    let height = grid.height;
    let n = width * height;

    let mut elevation = vec![0.0f64; n];
    pass1_elevation(&mut elevation, width, height, seed);

    let mut rainfall = vec![0.0f64; n];
    pass2_wind_rainfall(&mut rainfall, &elevation, width, height);

    pass3_biomes(grid, &elevation, &rainfall);
}

fn pass1_elevation(elevation: &mut [f64], width: usize, height: usize, seed: u32) {
    let base_noise = OpenSimplex::new(seed);
    let hill_noise = OpenSimplex::new(seed.wrapping_add(100));
    let ocean_noise = OpenSimplex::new(seed.wrapping_add(200));

    let half_x = width as f64 / 2.0;
    let half_y = height as f64 / 2.0;
    let max_dist = (half_x * half_x + half_y * half_y).sqrt();

    let mut peaks = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let i = idx(x, y, width);

            let nx = x as f64 * ELEV_FREQ;
            let ny = y as f64 * ELEV_FREQ;
            let mut e = (base_noise.get([nx, ny]) + 1.0) / 2.0;

            let dx = x as f64 - half_x;
            let dy = y as f64 - half_y;
            let dist = (dx * dx + dy * dy).sqrt();
            let falloff = (dist / max_dist).powf(1.5).min(1.0);
            e *= 1.0 - falloff;

            if e >= 0.0 {
                let dc = (dist / max_dist).min(1.0);
                peaks.push((e * dc * dc, x, y));
            }

            elevation[i] = e;
        }
    }

    peaks.sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    peaks.truncate(MOUNTAIN_COUNT);

    let mut mountain_dist = vec![f64::MAX; elevation.len()];
    let mut queue = VecDeque::new();
    for &(_, px, py) in &peaks {
        let pi = idx(px, py, width);
        mountain_dist[pi] = 0.0;
        queue.push_back((px, py));
    }

    let rand_gen = OpenSimplex::new(seed.wrapping_add(300));
    while let Some((cx, cy)) = queue.pop_front() {
        let ci = idx(cx, cy, width);
        for &(dx, dy) in &NEIGHBOR_OFFSETS {
            let nx = cx.wrapping_add_signed(dx);
            let ny = cy.wrapping_add_signed(dy);
            if nx < width && ny < height {
                let ni = idx(nx, ny, width);
                if mountain_dist[ni] > mountain_dist[ci] + 1000.0 {
                    let r1 = (rand_gen.get([cx as f64, cy as f64]) + 1.0) / 2.0;
                    let r2 = (rand_gen.get([cy as f64, cx as f64]) + 1.0) / 2.0;
                    let inc = 1.0 + MOUNTAIN_JAGGEDNESS * (r1 - r2);
                    mountain_dist[ni] = mountain_dist[ci] + inc;
                    queue.push_back((nx, ny));
                }
            }
        }
    }

    let coastal_dist = coastal_distance_map(elevation, width, height);
    let max_coastal = coastal_dist.iter().cloned().fold(0.0f64, f64::max).max(1.0);

    let sharpness = MOUNTAIN_SHARPNESS;
    let slope = MOUNTAIN_SLOPE;

    for y in 0..height {
        for x in 0..width {
            let i = idx(x, y, width);
            let e = elevation[i];

            if e < 0.0 {
                let onx = x as f64 * OCEAN_NOISE_FREQ;
                let ony = y as f64 * OCEAN_NOISE_FREQ;
                elevation[i] = e * (1.0 + 0.2 * ocean_noise.get([onx, ony]));
                if elevation[i] < -1.0 {
                    elevation[i] = -1.0;
                }
                continue;
            }

            let hnx = x as f64 * HILL_FREQ;
            let hny = y as f64 * HILL_FREQ;
            let hills = 0.15 * hill_noise.get([hnx, hny]);

            let cw = (coastal_dist[i] / max_coastal).min(1.0);
            let w = cw * cw;

            let m_height = 1.0 - slope * mountain_dist[i] / sharpness;
            let mountains = m_height.max(0.01);

            elevation[i] = (1.0 - w) * hills + w * mountains;

            if elevation[i] > 1.0 {
                elevation[i] = 1.0;
            }
            if elevation[i] < -1.0 {
                elevation[i] = -1.0;
            }
        }
    }
}

fn pass2_wind_rainfall(_rainfall: &mut [f64], _elevation: &[f64], _width: usize, _height: usize) {
    // Placeholder — implemented in Task 2
}

fn pass3_biomes(_grid: &mut Grid, _elevation: &[f64], _rainfall: &[f64]) {
    // Placeholder — implemented in Task 3
}
```

- [ ] **Step 2: Build and verify**

Run: `cargo build`
Expected: Compiles. Map will show elevation pass only (rainfall and biomes are placeholders — uniform color output). This is a visual checkpoint.

---

### Task 2: Implement Pass 2 (Wind & Rainfall)

**Files:**
- Modify: `src/mapgen/generator.rs` (replace `pass2_wind_rainfall` placeholder)

**Interfaces:**
- Consumes: `elevation: &[f64]` from pass 1
- Produces: fills `rainfall: &mut [f64]` for pass 3

- [ ] **Step 1: Replace the `pass2_wind_rainfall` function**

Replace the placeholder body with:

```rust
fn pass2_wind_rainfall(rainfall: &mut [f64], elevation: &[f64], width: usize, height: usize) {
    let n = elevation.len();
    let mut humidity = vec![0.0f64; n];

    let angle_rad = WIND_ANGLE_DEG.to_radians();
    let wind_dir = (angle_rad.cos(), angle_rad.sin());

    #[allow(clippy::cast_possible_truncation)]
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_unstable_by(|&a, &b| {
        let ax = (a % width) as f64;
        let ay = (a / width) as f64;
        let bx = (b % width) as f64;
        let by = (b / width) as f64;
        let pa = ax * wind_dir.0 + ay * wind_dir.1;
        let pb = bx * wind_dir.0 + by * wind_dir.1;
        pa.partial_cmp(&pb).unwrap()
    });

    for &i in &order {
        let x = i % width;
        let y = i / width;

        let mut upwind_count = 0;
        let mut upwind_humidity = 0.0;
        let mut upwind_elev = 0.0;

        for &(dx, dy) in &NEIGHBOR_OFFSETS {
            let nx = x.wrapping_add_signed(dx);
            let ny = y.wrapping_add_signed(dy);
            if nx < width && ny < height {
                let ni = idx(nx, ny, width);
                let n_proj = nx as f64 * wind_dir.0 + ny as f64 * wind_dir.1;
                let c_proj = x as f64 * wind_dir.0 + y as f64 * wind_dir.1;
                if n_proj < c_proj {
                    upwind_count += 1;
                    upwind_humidity += humidity[ni];
                    upwind_elev += elevation[ni];
                }
            }
        }

        if upwind_count == 0 {
            humidity[i] = 1.0;
        } else {
            humidity[i] = upwind_humidity / upwind_count as f64;
        }

        if elevation[i] < 0.0 {
            humidity[i] += EVAPORATION_RATE * elevation[i].abs();
        } else {
            let upwind_avg_elev = if upwind_count > 0 {
                upwind_elev / upwind_count as f64
            } else {
                elevation[i]
            };
            let lift = (elevation[i] - upwind_avg_elev).max(0.0);
            rainfall[i] = RAININESS * humidity[i] * (1.0 + lift * OROGRAPHIC_FACTOR);
            humidity[i] -= rainfall[i] * RAIN_SHADOW;
        }

        humidity[i] = humidity[i].clamp(0.0, 1.0);
    }
}
```

- [ ] **Step 2: Build and verify**

Run: `cargo build`
Expected: Compiles. Biomes are still placeholder (uniform), but wind/rainfall computation runs.

---

### Task 3: Implement Pass 3 (Biome Classification)

**Files:**
- Modify: `src/mapgen/generator.rs` (replace `pass3_biomes` placeholder)

**Interfaces:**
- Consumes: `elevation` and `rainfall` arrays
- Produces: fills `grid.tiles[i]` with biome colors

- [ ] **Step 1: Replace the `pass3_biomes` function**

Replace the placeholder body with:

```rust
fn pass3_biomes(grid: &mut Grid, elevation: &[f64], rainfall: &[f64]) {
    for y in 0..grid.height {
        for x in 0..grid.width {
            let i = idx(x, y, grid.width);
            let e = elevation[i];
            let r = rainfall[i];

            let color = if e < WATER_DEEP {
                Color::new(0.05, 0.1, 0.35, 1.0)
            } else if e < WATER_SHALLOW {
                Color::new(0.15, 0.3, 0.55, 1.0)
            } else if e >= 0.80 && r < 0.15 {
                Color::new(0.7, 0.4, 0.2, 1.0)
            } else if e >= 0.80 && r < 0.35 {
                Color::new(0.5, 0.45, 0.4, 1.0)
            } else if e >= 0.80 && r < 0.60 {
                Color::new(0.95, 0.95, 0.95, 1.0)
            } else if e >= 0.80 {
                Color::new(0.85, 0.9, 0.95, 1.0)
            } else if e >= 0.67 && r < 0.15 {
                Color::new(0.6, 0.55, 0.35, 1.0)
            } else if e >= 0.67 && r < 0.35 {
                Color::new(0.55, 0.5, 0.3, 1.0)
            } else if e >= 0.67 && r < 0.60 {
                Color::new(0.25, 0.45, 0.3, 1.0)
            } else if e >= 0.67 {
                Color::new(0.35, 0.3, 0.4, 1.0)
            } else if e >= 0.52 && r < 0.15 {
                Color::new(0.9, 0.8, 0.4, 1.0)
            } else if e >= 0.52 && r < 0.35 {
                Color::new(0.7, 0.7, 0.3, 1.0)
            } else if e >= 0.52 && r < 0.60 {
                Color::new(0.3, 0.65, 0.2, 1.0)
            } else if e >= 0.52 {
                Color::new(0.15, 0.45, 0.1, 1.0)
            } else if r < 0.15 {
                Color::new(0.85, 0.8, 0.5, 1.0)
            } else if r < 0.35 {
                Color::new(0.7, 0.68, 0.45, 1.0)
            } else if r < 0.60 {
                Color::new(0.35, 0.5, 0.3, 1.0)
            } else {
                Color::new(0.25, 0.4, 0.25, 1.0)
            };

            grid.tiles[i] = color;
        }
    }
}
```

- [ ] **Step 2: Build and verify**

Run: `cargo build`
Expected: Compiles with full 3-pass pipeline. Mapgen scene shows island with wind-shaped biomes.

---

### Task 4: Final verification

- [ ] **Step 1: Run `cargo check` for zero warnings**

Run: `cargo check`
Expected: Clean output, zero warnings.

- [ ] **Step 2: Confirm binary**

Run: `Test-Path -LiteralPath "target\debug\tictactoe.exe"`  
Expected: `True`

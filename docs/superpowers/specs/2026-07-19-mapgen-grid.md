# Mapgen: Procedural Island Grid (Pass 1 — Flat Grid)

**Date:** 2026-07-19
**Status:** draft

## Overview

Replace the static 3D demo in the Mapgen scene with a 256x256 procedurally-generated island map rendered as a flat colored grid on the 3D plane. Pass 1 establishes the grid infrastructure: tile data model, texture-based rendering, and top-down camera controls. Uniform tile color only — noise generation comes in Pass 2.

## Architecture

### New module: `src/mapgen/`

```
src/
  mapgen/
    mod.rs         -- Grid struct, grid→texture conversion
    generator.rs   -- Simplex noise (skeleton for Pass 2)
```

### Grid data model (`mapgen/mod.rs`)

```rust
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Color>,
}
```

Tiles stored row-major. `Grid::new(width, height, fill_color)` creates a uniform grid. Later: `generator::apply_island_noise(grid, seed)` will overwrite tile colors via simplex noise.

### Scene state (`mapgen.rs`)

The update function becomes stateful using a `static OnceLock<Mutex<MapgenState>>`:

```rust
struct MapgenState {
    grid: Grid,
    texture: Texture2D,
    camera_offset: Vec2,
    zoom: f32,
}
```

On first frame: generate `Grid`, convert to `Texture2D`, initialize camera. Subsequent frames: handle input, render.

### Rendering

- Orthographic camera looking straight down at the grid center
- One `draw_plane()` call at y=0 sized `vec2(256, 256)` with the grid texture
- Grid→texture conversion iterates tiles and writes pixels into a `macroquad::texture::Image`, then uploads as `Texture2D`

### Camera controls

- **Zoom:** Mouse scroll wheel adjusts zoom level (ortho scale or camera distance). Clamped to min/max.
- **Pan:** Arrow keys or WASD move camera offset. Speed scales with zoom level.

### Navigation

Escape returns to MainMenu (already implemented).

## Dependencies

- `macroquad = "0.4.15"` (already in Cargo.toml)
- `std::sync::{OnceLock, Mutex}` (stdlib, no new deps)

## Out of Scope for Pass 1

- Simplex noise generation (Pass 2)
- Tile type classification (water/beach/grass/hill/mountain) (Pass 3)
- Grid lines overlay (Pass 4+)
- Tile interaction / mouse picking
- Map regeneration hotkey
- Camera rotation (top-down only)

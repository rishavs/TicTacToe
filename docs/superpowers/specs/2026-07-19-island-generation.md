# Mapgen: Island Generation (Pass 2 — Simplex Noise Biomes)

**Date:** 2026-07-19
**Status:** draft

## Overview

Replace the uniform green grid with a procedurally-generated island using simplex noise. Two noise layers (elevation + moisture) combined with radial falloff produce 16 biome types: 2 water + 14 land, classified into a 4×4 elevation/moisture matrix + water threshold.

## Architecture

### New dependency

`noise = "0.9"` in `Cargo.toml` — provides `OpenSimplex` noise generator.

### Generator (`src/mapgen/generator.rs`)

```rust
pub fn generate_island(grid: &mut Grid, seed: u32)
```

**Algorithm:**

1. Create two `OpenSimplex` noise generators (elevation + moisture) seeded from `seed`
2. For each cell (x, y) in 256×256:
   - Sample elevation noise at `(x/64.0, y/64.0)` → `[-1, 1]`, map to `[0, 1]`
   - Sample moisture noise at `(x/48.0, y/48.0)` → `[-1, 1]`, map to `[0, 1]`
   - Compute radial falloff: `distance_from_center / max_distance` → `[0, 1]`
   - Apply falloff to elevation: `elevation *= 1.0 - falloff^1.5`
   - Classify by thresholds (see Biome Matrix)
   - Set tile color from palette

### Radial falloff

Center of island = grid center (128, 128). Max distance = half diagonal ≈ 181. Falloff exponent 1.5 gives a gradual shoreline without excessive inland water.

### Biome Matrix (4×4 land + 2 water)

Water thresholds (elevation):
- Deep Water:  `e < 0.35`
- Shallow Water: `0.35 ≤ e < 0.42`

Land biomes (elevation ≥ 0.42), cross-referenced with moisture:

| Elevation \ Moisture | Arid (m < 0.25) | Dry (m < 0.5) | Temperate (m < 0.75) | Wet (m ≥ 0.75) |
|---|---|---|---|---|
| **Elev ≥ 0.82** (Peak) | Badlands | Mountain | Snow | Glacier |
| **0.70–0.82** (High) | Steppe | Highland | Taiga | Moor |
| **0.55–0.70** (Mid) | Desert | Savannah | Grassland | Forest |
| **0.42–0.55** (Low) | Beach | Coast | Marsh | Swamp |

### Color Palette

| Tile | Color (r, g, b) |
|---|---|
| Deep Water | `(0.05, 0.1, 0.35)` |
| Shallow Water | `(0.15, 0.3, 0.55)` |
| Beach | `(0.85, 0.8, 0.5)` |
| Coast | `(0.7, 0.68, 0.45)` |
| Marsh | `(0.35, 0.5, 0.3)` |
| Swamp | `(0.25, 0.4, 0.25)` |
| Desert | `(0.9, 0.8, 0.4)` |
| Savannah | `(0.7, 0.7, 0.3)` |
| Grassland | `(0.3, 0.65, 0.2)` |
| Forest | `(0.15, 0.45, 0.1)` |
| Steppe | `(0.6, 0.55, 0.35)` |
| Highland | `(0.55, 0.5, 0.3)` |
| Taiga | `(0.25, 0.45, 0.3)` |
| Moor | `(0.35, 0.3, 0.4)` |
| Badlands | `(0.7, 0.4, 0.2)` |
| Mountain | `(0.5, 0.45, 0.4)` |
| Snow | `(0.95, 0.95, 0.95)` |
| Glacier | `(0.85, 0.9, 0.95)` |

### Scene integration

In `src/scenes/mapgen.rs`, replace `Grid::new(GRID_SIZE, GRID_SIZE, GREEN)` with a `generate_island` call. Seed is hardcoded for now (regeneration hotkey out of scope); re-entering the scene from menu re-generates with same seed (OnceLock caches it).

### File changes

- Modify: `Cargo.toml` — add `noise = "0.9"`
- Modify: `src/mapgen/generator.rs` — full implementation
- Modify: `src/mapgen/mod.rs` — no changes needed (Grid already supports per-tile colors)
- Modify: `src/scenes/mapgen.rs` — call `generate_island` instead of uniform fill

## Out of Scope

- Regeneration hotkey / new seed UI
- Moisture noise re-tuning
- Coastline smoothing (cellular automata)
- Tile interaction / mouse picking

## Dependencies

- `macroquad = "0.4.15"` (existing)
- `noise = "0.9"` (new)

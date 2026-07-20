# Mapgen: mapgen4-style Pipeline (Elevation + Wind + Biomes)

**Date:** 2026-07-19
**Status:** draft

## Overview

Replace the current 2-noise-layer approach with a 3-pass mapgen4-inspired pipeline: (1) hill+mountain blended elevation with radial falloff, (2) wind-driven rainfall simulation with orographic effects, (3) biome classification by elevation×rainfall. Rivers deferred to a future pass.

## Architecture

All generation logic lives in `src/mapgen/generator.rs`. The file is split into clear sections with helper functions for each pass. `generate_island(grid, seed)` remains the single entry point.

### Pass 1: Elevation

1. **Base noise**: simplex noise at frequency 1/128, mapped to [0, 1]
2. **Radial falloff**: same as current — `(dist/max_dist)^1.5`, ensures ocean at edges
3. **Ocean depth**: for cells where `elevation < 0`, add medium-scale noise (freq 1/32, amplitude 0.2)
4. **Mountain peaks**: sample 100 random land cells, score by `elevation × coastal_distance²`, keep top 30
5. **Mountain distance field**: BFS from peaks. For each neighbor, `distance += spacing * (1 + jaggedness * (rand - rand))`. Spacing = 1 cell = 1 unit. Jaggedness = 0.5
6. **Hill noise**: low-amplitude simplex at freq 1/32, scaled to ~0.15
7. **Mountain shape**: `m = 1 - slope * distance / sharpness`, clamped to [0.01, 1.0]. slope=16, sharpness=4
8. **Blend**: for each land cell, `coastal_w = distance_from_coast / max_inland_dist`. Then `elevation = (1 - w²) * hills + w² * mountains`. Coastal cells stay hilly, inland cells become mountainous.
9. Clamp final elevation to [-1, 1]

**Parameters:** mountain_count=30, jaggedness=0.5, slope=16, sharpness=4

### Pass 2: Wind & Rainfall

Per-cell arrays: `humidity[f64]`, `rainfall[f64]`, `wind_proj[f64]`.

1. **Wind projection**: for each cell, `proj = x*cos(θ) + y*sin(θ)` with θ = 225° (southwest wind)
2. **Sort**: create index array sorted by `proj` ascending (upwind first)
3. **Process** each cell in wind order:
   - If it's on the map boundary (or adjacent to ocean boundary): `humidity[c] = 1.0`
   - Else: `humidity[c] = average(humidity[upwind_neighbors])`
   - If water cell (`elevation[c] < 0`): `humidity[c] += 0.5 * |elevation[c]|` (evaporation)
   - If land cell: compute orographic lift
     - `upwind_elev = average(elevation[upwind_neighbors])`
     - `lift = max(0, elevation[c] - upwind_elev)`
     - `rainfall[c] = 0.4 * humidity[c] * (1 + lift * 2.0)`
     - `humidity[c] -= rainfall[c] * 1.5` (rain shadow — drier air continues)
   - Clamp humidity to [0, 1]

**Parameters:** wind_angle=225, evaporation_rate=0.5, raininess=0.4, orographic_factor=2.0, rain_shadow=1.5

### Pass 3: Biome Classification

Water thresholds unchanged. Land: 4×4 matrix using rainfall instead of moisture noise.

**Water:**
- Deep: `e < 0.35` — (0.05, 0.1, 0.35)
- Shallow: `0.35 ≤ e < 0.42` — (0.15, 0.3, 0.55)

**Land (e ≥ 0.42):**

| Elevation \ Rainfall | Arid r<0.15 | Dry r<0.35 | Temperate r<0.60 | Wet r≥0.60 |
|---|---|---|---|---|
| **Peak** e≥0.80 | Badlands | Mountain | Snow | Glacier |
| **High** 0.67-0.80 | Steppe | Highland | Taiga | Moor |
| **Mid** 0.52-0.67 | Desert | Savannah | Grassland | Forest |
| **Low** 0.42-0.52 | Beach | Coast | Marsh | Swamp |

Colors unchanged from current palette.

## File changes

- **Modify**: `src/mapgen/generator.rs` — replace entire contents with 3-pass pipeline

No other files change. Grid, to_texture, scene rendering all unchanged.

## Dependencies

- `noise = "0.9"` (existing)
- `macroquad = "0.4.15"` (existing)

## Out of Scope

- River generation (future pass)
- Terrain painting / shaping tools
- Regeneration hotkey
- Parameter tuning UI

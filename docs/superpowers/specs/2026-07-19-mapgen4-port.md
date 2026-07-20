# Mapgen: 1:1 Port of mapgen4 to Square Grid

**Date:** 2026-07-19
**Status:** draft

## Overview

Faithful port of redblobgames/mapgen4 to a square grid rendered via macroquad. Ports map.ts (elevation + rainfall + rivers), geometry.ts, colormap.ts, and render.ts. Adapts the Delaunay/Voronoi dual-mesh to an 8-neighbor square grid.

## Architecture

### Files
```
src/mapgen/
  mod.rs          — Grid struct (existing), GridMesh struct (new)
  generator.rs    — Elevation, rainfall, rivers (port of map.ts)
  colormap.rs     — Color lookup table (port of colormap.ts)
  render.rs       — Grid → Image/Texture2D with lighting + rivers
```

### Data Model (`GridMesh`)

Square cells with 8-neighbor adjacency (NEIGHBOR_OFFSETS 0..7). Each cell stores elevation, rainfall, humidity, downslope direction, flow, and mountain distance:

```
elevation: Vec<f64>     — final per-cell elevation [-1, 1]
rainfall:  Vec<f64>     — per-cell moisture/rainfall [0, 1]
humidity:  Vec<f64>     — wind humidity per cell [0, 1]
s_downslope: Vec<isize> — -1=ocean, 0..7=direction to lowest neighbor
t_order:   Vec<usize>   — topological sort (ocean→inland by elevation)
flow:      Vec<f64>     — accumulated river flow
mountain_dist: Vec<f64> — BFS distance from peaks
```

### Dependencies

- `macroquad = "0.4.15"` (existing)
- `noise = "0.9"` (existing)
- `std::collections::VecDeque` (stdlib)

## Pass 1: Elevation

Ported from `map.ts::assignElevation()`.

1. 3-octave fractal simplex noise at 1/128, 1/64, 1/32
2. Soft radial falloff: `base = 1 - (dist/max_dist)^2`
3. Coastline noise: 3-frequency noise × `(1 - e⁴) × NOISY_COASTLINES` applied to near-boundary cells
4. Select 40 mountain peaks from land cells weighted by elevation × (1-dist/max_dist)
5. BFS mountain distance field with jaggedness
6. Hills formula: `eh = (1 + noisiness×n_high + (1-noisiness)×n_med) × hill_height`
7. Mountains: `em = 1 - dist × mountain_range`, clamped to 0.01
8. Blend: `weight = e²` → land elevation = `(1-weight)×hills + weight×mountains`
9. Ocean: `e × (ocean_depth + noise)`

Parameters: hill_height=0.5, noisy_coastlines=0.3, ocean_depth=1.5, mountain_jagged=0.5, mountain_range=0.04, mountain_count=40

## Pass 2: Wind & Rainfall

Ported from `map.ts::assignRainfall()`.

1. Sort cells by position projected onto wind axis (225° = southwest)
2. For each cell in wind order:
   - Humidity = average of upwind neighbor humidities, or 1.0 if no upwind data
   - Water: humidity += evaporation_rate × |elevation|
   - Land: compute orographic lift, rainfall = raininess × humidity × (1 + lift×2.0), humidity -= rainfall × rain_shadow
   - Clamp humidity to [0, 1]

Parameters: wind_angle=225, evaporation=0.5, raininess=0.4, orographic_factor=2.0, rain_shadow=1.5

## Pass 3: Rivers

Ported from `map.ts::assignRivers()`.

1. **Downslope**: priority queue seeded from ocean cells, propagate inland by elevation. Each land cell points to its lowest-elevation neighbor.
2. **Moisture**: `moisture[i] = rainfall[i]` (simplified from 3-region average)
3. **Flow**: init `flow[i] = flow_constant × moisture²`, accumulate from leaves to roots along downslope tree (reverse topological order)
4. **River carving**: `elevation[i] -= outline_water × (1 - (1-flow_rel)²)` for cells with flow > min_flow to create visible river valleys

Parameters: flow=3.0, min_flow=exp(1.0), river_width=exp(2.0), outline_water=0.02

## Pass 4: Rendering

Ported from `render.ts + colormap.ts + geometry.ts`.

1. **Colormap**: 2D table indexed by `(elevation_normalized, rainfall)` mapping to RGB color. Port the colormap data from mapgen4's `colormap.ts` (256×1 texture with biome colors at specific elevation stops).
2. **Hillshade**: compute local slope from elevation neighborhood, dot product with light direction (225°) → multiply RGB by `ambient + max(0, dot(normal, light))`
3. **River rendering**: cells with flow > min_flow get blended with river blue, proportional to flow
4. **Coast outline**: detect cells where elevation crosses water threshold in neighborhood, apply dark outline
5. Output: `Image` with final pixel colors → `Texture2D::from_image(&image)`

### Scene integration

`src/scenes/mapgen.rs` returns. On first frame:
1. Create `GridMesh` (256×256)
2. Call `generator::run_pipeline(&mut mesh, seed)` — runs all 3 generation passes
3. Call `render::render_to_texture(&mesh)` → `Texture2D`
4. Display via `draw_plane()` (existing camera setup)

## Out of Scope

- Interactive painting / terrain shaping tools
- WebGL multi-pass rendering (baked to texture instead)
- Worker thread (single-threaded generation)
- Oblique projection (standard perspective camera)
- Dynamic viewport parameters (hardcoded)

## Spec vs Reference mapping

| mapgen4 | TicTacToe |
|---|---|
| `map.ts: assignElevation` | `generator.rs: pass1_elevation()` |
| `map.ts: assignRainfall` | `generator.rs: pass2_wind()` |
| `map.ts: assignRivers` | `generator.rs: pass3_rivers()` |
| `colormap.ts` | `colormap.rs` |
| `render.ts` (WebGL) | `render.rs` (Image/Texture2D) |
| `geometry.ts: setMapGeometry` | `render.rs: render_to_texture()` |
| `mesh.ts` (Delaunay) | `mod.rs: GridMesh` (square grid) |

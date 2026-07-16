---
status: draft
created: 2026-07-15
tags:
- mapgen
- rendering
- ui
created_at: 2026-07-15T18:24:35.746150700Z
updated_at: 2026-07-15T18:24:39.380580500Z
---
# Wire Noisy Edges, Fills, and Lighting

Wire the 3 existing checkboxes (Edges, Fills, Light) on the map viewer panel to actual rendering effects.

## 1. Noisy Edges

**Data generation** — already exists in `noisyedges.go`. Wire into pipeline:
- Call `assignNoisyEdges(m, rng)` from `Generate()`, store result on `GameMap.NoisyEdges`
- `GameMap` gets new field `NoisyEdges *NoisyEdges`

**Rendering** — when `noisyEdgesOn` checkbox is checked:
- After drawing tile rects, draw noisy edge segments as thin lines at coast boundaries
- Use `vector.StrokeLine` or `ebitenutil.DrawLine` with the subdivided point arrays
- Line color: white or contrasting, 1px wide

## 2. Fills

**Current behavior:** tiles are always rendered with biome colors. Fills OFF means wireframe mode.
- Fills ON (default): biome-colored filled rects (unchanged)
- Fills OFF: neutral dark background, draw biome boundary lines only
- Skip `vector.DrawFilledRect` for tile interiors when fills are off

## 3. Lighting

**Compute per-tile light level** from neighboring elevation differential:
- New `assignLighting(m)` function in `lighting.go` (or `elevation.go`)
- `Tile` gets new `Light float64` field
- For each land tile, compute slope toward light source (northwest):
  ```
  dx = tile[x+1].Elevation - tile[x-1].Elevation
  dy = tile[x-1].Elevation - tile[x+1].Elevation
  light = 0.5 + (dx + dy) * 0.5  // range ~0-1
  ```
- Clamp to [0.2, 1.0] so shadows aren't pure black

**Rendering** — when `lightingOn` checkbox is checked:
- Multiply biome color RGB by `tile.Light` before drawing
- Creates 3D hillshade effect

## Files

| File | Change |
|------|--------|
| `tilemap.go` | Add `Light float64` to Tile, `NoisyEdges *NoisyEdges` to GameMap |
| `generator.go` | Add `assignNoisyEdges` + `assignLighting` calls |
| `scene/mapgen.go` | Wire 3 checkbox fields to Draw logic |
| `lighting.go` (new) | `assignLighting(m)` function |

## Out of Scope

- Advanced lighting (ambient occlusion, specular)
- Noisy edge smoothing / anti-aliasing
- Fills OFF outlines (rendered as simple thin lines over biome boundaries)

## Verification

- Build: `go build ./...`
- Visual: checkboxes toggle rendering modes in real-time (no regeneration needed)

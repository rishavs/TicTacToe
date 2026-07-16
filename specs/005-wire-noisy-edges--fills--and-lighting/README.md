---
status: complete
created: 2026-07-15
tags:
- mapgen
- rendering
- ui
created_at: 2026-07-15T18:24:35.746150700Z
updated_at: 2026-07-16T03:54:49.567072800Z
---

# Wire Noisy Edges, Fills, and Lighting

Reconcile the map viewer render-toggle work with the current code and design decision.

## Current Status

Complete as a closed decision. Lighting is wired into the map viewer. Edges and Fills viewer modes were tried earlier, did not provide enough benefit, and were removed from the active code path. Noisy-edge generation was also removed during cleanup because it was only supporting the dropped edge-rendering path.

## Final Implemented State

### Lighting Data and Rendering

- `Tile` has `Light float64`.
- `src/mapgen/lighting.go` computes land light from neighboring elevation differentials and clamps through configurable `MapConfig` values.
- `Generate()` calls `assignLighting(m, cfg)`.
- `MapgenScene` has `lightingOn` state.
- The map viewer exposes a `Light` checkbox.
- `Draw()` multiplies biome/elevation color by `tile.Light` when lighting is enabled.

### Current Render Controls

- `MapgenScene` has `biomesOn` state.
- The map viewer exposes a `Biomes` checkbox.
- With Biomes enabled, tiles render with the biome palette.
- With Biomes disabled, tiles render as grayscale elevation.
- The viewer exposes a `Light` checkbox.

## Explicitly Dropped

### Edges Toggle

- Tried previously.
- Did not provide enough visual or gameplay value.
- Removed from code and no longer planned.
- Supporting noisy-edge generation code was removed in spec 006.

### Fills Toggle

- Tried previously.
- Did not provide enough visual or gameplay value.
- Removed from code and no longer planned.

## Verification

- Current code matches the final scope: Biomes and Light controls remain; Edges and Fills controls are absent.
- `go test ./...` passes.
- `go vet ./...` passes.
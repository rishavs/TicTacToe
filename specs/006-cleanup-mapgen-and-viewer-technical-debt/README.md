---
status: complete
created: 2026-07-16
priority: medium
tags:
- cleanup
- mapgen
- ui
- refactor
created_at: 2026-07-16T03:36:23.668031600Z
updated_at: 2026-07-16T03:55:03.495933200Z
---

# Cleanup mapgen and viewer technical debt

Implement the cleanup/refactor opportunities identified in the code review, keeping behavior stable and avoiding speculative features.

## Current Status

Complete.

## Implemented

- Removed dead river-width state and widening path:
  - Removed `Tile.RiverWidth`.
  - Removed `MapConfig.RiverMaxWidth`.
  - Removed `widenRivers` and its generator call.
- Removed noisy-edge generation from the active code path:
  - Removed `GameMap.NoisyEdges`.
  - Removed `assignNoisyEdges` and `src/mapgen/noisyedges.go`.
  - Removed the generator call that created noisy edges every map.
- Centralized scene resolution constants:
  - Added `src/scene/constants.go` with `InternalWidth`, `InternalHeight`, and `WindowScale`.
  - Updated `main`, menu, placeholder, and map viewer code to use shared constants.
- Moved active tuning values into `MapConfig`:
  - Spring elevation bounds.
  - Water-depth noise scale/amplitude/threshold.
  - Lighting ambient/slope/min/max values.
- Reused OpenSimplex generators for FBM island noise instead of recreating generators for every tile sample.
- Removed unused helpers:
  - D8 offsets and `D8Neighbor`.
  - `Camera.ZoomAt`.
- Added a small `GameMap.EachN4` helper and used it in simple neighbor scans.
- Simplified `MapgenScene`:
  - Extracted map config construction.
  - Extracted camera input update.
  - Extracted delayed regeneration handling.
  - Extracted map drawing.
  - Extracted tile color calculation.
- Debounced slider-driven map regeneration so slider drags do not rebuild the map on every widget change event.

## Design Notes

- Edges/Fills remain intentionally dropped; no rendering mode was reintroduced.
- The cleanup preserves the existing visible viewer controls: Biomes and Light.
- Mapgen remains independent from Ebiten rendering.

## Verification

- `go test ./...` passes.
- `go vet ./...` passes.

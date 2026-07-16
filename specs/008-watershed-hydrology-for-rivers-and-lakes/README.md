---
status: complete
created: 2026-07-16
tags:
- mapgen
- hydrology
- terrain
- procedural-generation
breaking: true
created_at: 2026-07-16T07:40:43.271066500Z
updated_at: 2026-07-16T09:54:07.618714100Z
completed_at: 2026-07-16T09:54:07.618714100Z
transitions:
- status: planned
  at: 2026-07-16T09:54:07.121106200Z
- status: in-progress
  at: 2026-07-16T09:54:07.339766900Z
- status: complete
  at: 2026-07-16T09:54:07.618714100Z
---
# Watershed Hydrology for Rivers and Lakes

Replace the current random-spring river pass with a deterministic hydrology layer that produces believable lakes, watersheds, drainage networks, variable river scale, and moisture inputs.

## Current Status

Complete. Implemented in the active code path.

## Implemented

- Added deterministic rainfall.
- Added explicit inland lake basins with `LakeID`, surface level, outlet tile, tile count, and flow summary.
- Added D8 flow directions.
- Added flow accumulation from downstream rainfall and lake catchments.
- Added watershed IDs and watershed summaries.
- Replaced random spring-line rivers with flow-threshold rivers.
- Added `RiverScale` based on accumulated upstream flow.
- Reworked moisture around rainfall plus proximity to rivers, lakes, and ocean coast.
- Updated normal map rendering so rivers use coherent variable-scale water overlays instead of bright fixed debug strokes.
- Added focused hydrology tests for tributary accumulation and generated-map hydrology layers.

## Problem

The current mapgen output makes rivers and lakes feel disconnected from the terrain:

- Rivers are selected from random spring candidates and traced along a BFS downslope parent.
- River rendering is a fixed bright overlay, so channels look like straight blue strokes.
- Lakes only exist when enclosed water happens to survive the island mask; they are not explicit drainage basins.
- Shallow ocean, lakes, and rivers use visibly different water treatments.
- Moisture is diffused after the artificial rivers are marked, so terrain wetness inherits river artifacts.

## Goals

- Produce explicit lake basins with lake identity, surface level, and outlet/spill point data.
- Compute watersheds so each land tile can be traced to a lake, ocean outlet, or terminal basin.
- Generate rivers from rainfall and flow accumulation rather than random spring selection.
- Allow tributaries, confluences, and variable river scale based on upstream flow.
- Rebuild moisture from rainfall, flow, lakes, rivers, ocean/coast influence, and elevation.
- Keep the system deterministic: same seed + same config = same map.
- Keep mapgen independent from Ebiten rendering.
- Add debug-friendly intermediate data for QA capture and future viewer toggles.

## Non-Goals

- No full time-stepped fluid simulation.
- No sediment transport or expensive multi-iteration erosion loop in this spec.
- No return to noisy edges or fill modes from the dropped rendering experiment.
- No TMX export changes unless needed by the new data model.

## Proposed Pipeline

1. Generate island water/ocean/coast as today.
2. Generate and redistribute initial elevation as today, with cleanup if needed for stable drainage.
3. Compute local downhill direction for land using N4 or D8 based on quality/performance testing.
4. Detect sinks and basins where downhill flow cannot reach ocean.
5. Fill or label basins into lakes:
   - assign `LakeID`
   - assign lake surface level
   - find spill/outlet tile when overflow can escape
6. Resolve drainage so every land tile has a path to:
   - ocean outlet
   - lake inlet/outlet
   - terminal basin if no outlet exists
7. Assign `WatershedID` by tracing each tile to its final outlet/lake/basin.
8. Generate rainfall as a deterministic field from noise, latitude/temperature, elevation, and optional future windward effects.
9. Accumulate flow downstream:
   - each land tile contributes rainfall
   - upstream rainfall sums into downstream tiles
   - lake catchments collect and release through outlets where available
10. Mark rivers where accumulated flow crosses thresholds.
11. Derive river scale from accumulated flow for rendering/gameplay data.
12. Recompute moisture from rainfall, river/lake proximity, ocean/coast influence, and elevation.
13. Assign biomes from the updated moisture/temperature/water data.
14. Render water with a coherent palette so shallow ocean, lakes, and rivers read as the same hydrology system.

## Data Model Candidates

`Tile` may gain:

```go
Rainfall      float64
Flow          float64
FlowDir       int
WatershedID   int
LakeID        int
RiverScale    float64
```

A separate lake/watershed summary may be cleaner than overloading tiles:

```go
type Lake struct {
  ID           int
  TileCount    int
  SurfaceLevel float64
  Outlet       int
}

type Watershed struct {
  ID       int
  Outlet   int
  LakeID   int
  Area     int
  Flow     float64
}
```

Choose the smallest structure that keeps mapgen readable and testable.

## Viewer and Debug Needs

- Keep the normal biome view as the default.
- Add data that can support future debug modes:
  - flow accumulation heatmap
  - watershed ID coloring
  - lake basin coloring
  - rainfall/moisture heatmap
- Rivers should use water colors in normal rendering, with debug overlays reserved for analysis.
- QA capture should be used to compare seeded outputs before and after the hydrology change.

## Implementation Phases

### Phase 1: Visual Coherence and Metrics

- Unify river/lake/ocean palette enough that current output is easier to inspect.
- Add simple debug/log metrics for lakes, watersheds, river tile count, and max flow once the data exists.

### Phase 2: Drainage and Basins

- Compute downhill directions from elevation.
- Detect sinks/basins.
- Label lakes and outlets.
- Add deterministic tests for basin and outlet behavior on small handcrafted maps.

### Phase 3: Watersheds and Flow

- Assign watershed IDs.
- Generate rainfall.
- Accumulate downstream flow.
- Replace random spring river selection with flow-threshold river generation.
- Add deterministic tests for tributaries, confluence, and watershed assignment.

### Phase 4: Moisture and Biome Reconciliation

- Recompute moisture from hydrology outputs.
- Tune biome thresholds if needed.
- Update `ARCH.md`, `GDD.md`, `STATUS.md`, and `CONV.md` with the final pipeline.

## Acceptance Criteria

- Rivers form connected drainage networks rather than isolated straight strokes.
- Tributaries can merge into larger channels.
- River scale increases with accumulated upstream flow.
- Lakes are explicit and visible on normal generated maps.
- Some rivers can flow into and/or out of lakes via outlets.
- Watershed IDs are deterministic and internally consistent.
- Moisture and biomes reflect hydrology outputs instead of the old spring-line artifacts.
- Normal rendering uses a coherent water palette.
- Map generation remains fast enough for the existing viewer regenerate workflow.
- `go test ./...` and `go vet ./...` pass with repo-local `GOCACHE`.

## Open Questions

- Should downhill flow use N4 for grid clarity or D8 for more organic drainage?
- Should rainfall include simple prevailing wind/rain shadow effects in this spec, or be deferred?
- Should basin filling alter elevation, or only label/render lake surfaces over unchanged terrain?
- What viewer debug toggles are worth adding immediately versus keeping as data-only support?

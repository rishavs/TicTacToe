# ARCHITECTURE

> Living document — update as packages, systems, and data flow evolve.

## Tech Stack

- **Language:** Go
- **Engine:** Ebiten
- **Genre:** Turn-based tactics

## Architecture

Layered: Scene Manager → Scenes → Systems. All game logic in pure-data packages (no Ebiten dependency). Rendering is thin — reads game state, never mutates it.

## Package Structure

```
src/
  main.go          — entry point, creates scene.Manager, runs Ebiten game loop
  qa.go            — dev QA flags, direct scene launch, one-frame PNG capture
  scene/            — Scene interface, Manager (current scene switching)
    constants.go
    manager.go
    scene.go
    menu.go
    placeholder.go
    resources.go
  mapgen/           — procedural map generation (no Ebiten dependency)
    tilemap.go      — GameMap, Tile, MapConfig, BiomeType (20 biomes), N4/D8 helpers
    island.go       — FBM opensimplex noise + Chebyshev distance → water vs land
    water.go        — ocean flood-fill, topological coast detection, shallow/deep water depth
    elevation.go    — BFS from water/land boundary, lake handling, BFS parent as downslope, quadratic redistribution
    hydrology.go    — rainfall, explicit lakes, flow directions, watersheds, flow accumulation, rivers
    moisture.go     — rainfall + water-proximity moisture, linear redistribution
    biomes.go       — temperature (1.0-elevation + lat bias), 20-biome mapgen2 classification
    lighting.go     — hillshade-style per-tile light values
    generator.go    — pipeline orchestrator
    tmx.go          — TMX type stubs (Tiled compatibility)
  camera/           — viewport camera for large maps
    camera.go       — Camera (pan, zoom, world-to-screen, viewport calc)
```

## Data Flow

```
Input (mouse/keyboard)
  → Scene.Update() → game logic
  → Scene.Draw() → Ebiten render

Map generation:
  MapConfig → mapgen.Generate() → *GameMap (pure function, no Ebiten)
  → Camera filters visible tiles
  → Scene draws tiles to screen
```

## Mapgen Logic

`mapgen.Generate(cfg)` is the single entry point for terrain generation. It creates a `GameMap`, seeds a deterministic PCG RNG from `MapConfig.Seed`, then runs the pipeline below in order. Keep this section updated whenever `src/mapgen/generator.go`, `MapConfig`, `Tile`, or biome classification changes.

### Data Model

- `GameMap` is a row-major square tile grid: width, height, flat `[]Tile`, seed, explicit lake summaries, and watershed summaries.
- `Tile` stores generated terrain state: elevation, rainfall, flow, moisture, river scale, temperature, light, biome, downslope/flow directions, watershed/lake IDs, and booleans for water/ocean/coast/shallow/river.
- `MapConfig` owns active generation tunables: size, seed, island shape, octave amplitudes, lake selection, rainfall, river threshold/scale, moisture/temperature bias, water-depth noise controls, and lighting controls.
- Mapgen has no Ebiten dependency. Scenes read `GameMap`; they do not participate in generation.

### Pipeline

1. `assignIslandWater` computes initial land/water from FBM OpenSimplex noise plus Chebyshev distance falloff.
2. `assignOcean` flood-fills water connected to map edges and marks it ocean.
3. `assignCoast` marks land tiles that touch ocean in N4 topology.
4. `assignElevation` runs randomized N4 BFS from water/land boundaries, handles lake traversal with zero distance increment, and stores BFS parent direction as `Downslope`.
5. `redistributeElevation` sorts land tiles and applies the lowland-heavy elevation redistribution curve.
6. `assignHydrology` resets hydrology state and computes deterministic rainfall.
7. `assignLakes` selects low, wet inland basins, marks them as explicit lakes, records lake surface levels, and finds spill/outlet tiles.
8. `assignFlowDirections` computes D8 downhill flow directions while preventing lake outlets from immediately draining back into their own lake.
9. `assignFlowAccumulation` sums rainfall through the downstream graph, including lake catchments and outlet release.
10. `assignWatersheds` traces each non-ocean tile to a lake, ocean outlet, or terminal basin and records watershed summaries.
11. `assignRivers` marks river tiles from accumulated flow and derives `RiverScale` from upstream volume.
12. `assignWaterDepth` uses configurable low-frequency noise plus elevation to mark shallow ocean water; explicit lakes keep their basin-derived shallow/deep state.
13. `assignMoisture` blends rainfall with proximity to rivers, lakes, and coast, then `redistributeMoisture` remaps land moisture over `[MoistureBias, 1+MoistureBias]`.
14. `assignTemperature` computes temperature as `1.0 - elevation + lerp(NorthTempBias, SouthTempBias, latitude)`.
15. `assignBiomes` classifies 20 biomes from ocean/water/shallow/coast flags plus temperature and moisture thresholds.
16. `assignLighting` computes hillshade-style per-tile light from neighboring elevation differentials and clamps with config values.

### Rendering Boundary

`scene.MapgenScene` converts controls into `MapConfig`, calls `mapgen.Generate`, fits a camera over the generated grid, and renders visible tiles using always-on biome colors plus lighting. Rivers render as calmer variable-scale water overlays, sharing the lake/ocean palette instead of a bright debug color. Terrain is drawn first, then rivers render in a dedicated overlay pass as antialiased downstream centerline strokes, so neighboring terrain tiles cannot chop river strokes into disconnected dots or segments. Edges/Fills modes and noisy-edge generation were intentionally dropped after experimentation; Biomes and Light controls were also removed because both effects are always desired.

## QA Capture Harness

The executable supports dev-only visual capture flags:

```bash
go run ./src --qa-scene mapgen --qa-seed 42 --qa-capture .qa-captures/mapgen.png
```

- `--qa-scene` selects the initial scene: `menu`, `mapgen`, `new`, `battle`, or `settings`.
- `--qa-seed` controls deterministic mapgen setup.
- `--qa-capture` saves one rendered 320x180 PNG and exits.
- Without `--qa-capture`, the game launches normally, optionally starting at `--qa-scene`.
- Captures are for local visual QA/debugging and should stay in ignored capture folders unless explicitly requested.

## Key Systems

| System | Package | Purpose |
|--------|---------|---------|
| Scene Manager | `scene` | Scene stack/switching, delegates Update/Draw |
| Map Generator | `mapgen` | Procedural island generation (deterministic, pure data) |
| Camera | `camera` | Scroll/zoom viewport into large tile maps |
| QA Capture | `main` | Direct scene launch and one-frame PNG capture for visual debugging |

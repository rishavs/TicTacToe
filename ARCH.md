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
  scene/            — Scene interface, Manager (current scene switching)
    constants.go
    manager.go
    scene.go
    menu.go
    placeholder.go
    resources.go
  mapgen/           — procedural map generation (no Ebiten dependency)
    tilemap.go      — GameMap, Tile, MapConfig, BiomeType (20 biomes), N4 helpers
    island.go       — FBM opensimplex noise + Chebyshev distance → water vs land
    water.go        — ocean flood-fill, topological coast detection, shallow/deep water depth
    elevation.go    — BFS from water/land boundary, lake handling, BFS parent as downslope, quadratic redistribution
    rivers.go       — N-spring river sources, downslope tracing
    moisture.go     — riverbanks + lakeshores seeds, BFS, sqrt falloff, linear redistribution
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

- `GameMap` is a row-major square tile grid: width, height, flat `[]Tile`, and seed.
- `Tile` stores generated terrain state: elevation, moisture, temperature, light, biome, downslope direction, and booleans for water/ocean/coast/shallow/river.
- `MapConfig` owns active generation tunables: size, seed, island shape, octave amplitudes, river count and spring elevation bounds, moisture/temperature bias, water-depth noise controls, and lighting controls.
- Mapgen has no Ebiten dependency. Scenes read `GameMap`; they do not participate in generation.

### Pipeline

1. `assignIslandWater` computes initial land/water from FBM OpenSimplex noise plus Chebyshev distance falloff.
2. `assignOcean` flood-fills water connected to map edges and marks it ocean.
3. `assignCoast` marks land tiles that touch ocean in N4 topology.
4. `assignElevation` runs randomized N4 BFS from water/land boundaries, handles lake traversal with zero distance increment, and stores BFS parent direction as `Downslope`.
5. `assignWaterDepth` uses configurable low-frequency noise plus elevation to mark shallow water.
6. `redistributeElevation` sorts land tiles and applies the lowland-heavy elevation redistribution curve.
7. `findSprings` selects non-water river sources within configurable elevation bounds and away from water.
8. `randomShuffle` shuffles springs deterministically with the generation RNG, then `NumRivers` limits the active set.
9. `assignRiverFlow` traces each spring along `Downslope` toward ocean and marks river tiles.
10. `findMoistureSeeds` seeds moisture from riverbanks, lake tiles, and lakeshores.
11. `assignMoisture` diffuses moisture through land with BFS and square-root falloff.
12. `redistributeMoisture` sorts land tiles and remaps moisture over `[MoistureBias, 1+MoistureBias]`.
13. `assignTemperature` computes temperature as `1.0 - elevation + lerp(NorthTempBias, SouthTempBias, latitude)`.
14. `assignBiomes` classifies 20 biomes from ocean/water/shallow/coast flags plus temperature and moisture thresholds.
15. `assignLighting` computes hillshade-style per-tile light from neighboring elevation differentials and clamps with config values.

### Rendering Boundary

`scene.MapgenScene` converts controls into `MapConfig`, calls `mapgen.Generate`, fits a camera over the generated grid, and renders visible tiles using biome/elevation colors plus optional lighting. Biomes and Light are the only active viewer render toggles. Edges/Fills modes and noisy-edge generation were intentionally dropped after experimentation.

## Key Systems

| System | Package | Purpose |
|--------|---------|---------|
| Scene Manager | `scene` | Scene stack/switching, delegates Update/Draw |
| Map Generator | `mapgen` | Procedural island generation (deterministic, pure data) |
| Camera | `camera` | Scroll/zoom viewport into large tile maps |

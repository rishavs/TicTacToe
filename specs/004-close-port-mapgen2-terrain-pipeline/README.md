---
status: complete
created: 2026-07-15
tags:
- mapgen
- procedural-generation
- terrain
- port
created_at: 2026-07-15T17:09:21.673786100Z
updated_at: 2026-07-16T03:54:37.912097500Z
---

# Close-port mapgen2 Terrain Pipeline

Bring Stoneheart's mapgen closer to redblobgames/mapgen2 while keeping the project on a square grid instead of Voronoi regions. This implementation is present in `src/mapgen/` and is used by the map viewer scene.

## Current Status

Complete as of the current codebase. The generator now uses the mapgen2-style water-first terrain pipeline, topological coast detection, BFS-derived elevation and downslope, explicit spring rivers, moisture diffusion, temperature classification, shallow/deep water biomes, and hillshade lighting.

## Implemented Pipeline

1. `assignIslandWater` - FBM simplex noise blended toward 0.5 by `IslandRoundness`, with Chebyshev distance falloff using `IslandInflate`.
2. `assignOcean` - flood-fill from map edges through water tiles.
3. `assignCoast` - land tiles adjacent to ocean by 4-neighbor topology.
4. `assignElevation` - BFS from the water/land boundary, randomized neighbor order, lake traversal with zero distance increment, and BFS parent stored as `Downslope`.
5. `assignWaterDepth` - configurable low-frequency noise blended with elevation to set `IsShallow` for water tiles.
6. `redistributeElevation` - sorted land redistribution using the mapgen2-style lowland-heavy curve.
7. `findSprings` - river sources selected from non-water tiles between configurable elevation bounds with no water neighbor.
8. `assignRiverFlow` - shuffled spring list traces along BFS parent directions toward ocean.
9. `findMoistureSeeds` - seeds include riverbanks, lake tiles, and lakeshores.
10. `assignMoisture` - BFS through land only, water set to full moisture, square-root falloff.
11. `redistributeMoisture` - sorted land redistribution over `[MoistureBias, 1+MoistureBias]`.
12. `assignTemperature` - `1.0 - elevation + lerp(NorthTempBias, SouthTempBias, latitude)` with no clamping.
13. `assignBiomes` - 20-biome classification: deep/shallow ocean, deep/shallow lake, marsh, ice, beach, and 14 land biomes.
14. `assignLighting` - configurable hillshade-style per-tile `Light` from neighboring elevation differentials.

## Data Model

`Tile` currently includes:

```go
Elevation   float64
Moisture    float64
Temperature float64
Light       float64
Biome       BiomeType
Downslope   int
IsWater     bool
IsOcean     bool
IsCoast     bool
IsShallow   bool
IsRiver     bool
```

`GameMap` currently includes width, height, tiles, and seed.

`MapConfig` currently includes size, seed, island shape controls, noise amplitudes, `NumRivers`, spring elevation bounds, moisture/temperature biases, water-depth controls, and lighting controls.

## Biomes

The code has 20 biome enum values:

- Deep Ocean
- Shallow Ocean
- Deep Lake
- Shallow Lake
- Marsh
- Ice
- Beach
- Snow
- Tundra
- Bare
- Scorched
- Taiga
- Shrubland
- Temperate Desert
- Temperate Rain Forest
- Temperate Deciduous Forest
- Grassland
- Tropical Rain Forest
- Tropical Seasonal Forest
- Subtropical Desert

## Verification

- `go test ./...` passes; packages currently have no test files.
- `go vet ./...` passes.
- Map viewer uses generated biome colors and lighting.

## Follow-up Notes

- Dedicated deterministic unit tests for mapgen steps are still desirable future hardening work.
- Noisy-edge rendering and Fill/Edge viewer modes were intentionally dropped in spec 005 and removed from the active code path in spec 006.
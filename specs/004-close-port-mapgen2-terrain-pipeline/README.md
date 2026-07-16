---
status: draft
created: 2026-07-15
tags:
- mapgen
- procedural-generation
- terrain
- port
created_at: 2026-07-15T17:09:21.673786100Z
updated_at: 2026-07-15T18:16:45.144548700Z
---

# Close-port mapgen2 Terrain Pipeline

Bring stoneheart's mapgen to closely match redblobgames/mapgen2. Only allowed differences: square grid instead of voronoi, no icon rendering, no huge-region display.

## P0 — Breakthrough Fixes

### 1. Island Formula — Water/Land from Noise (not elevation threshold)

**Target (mapgen2 water.js):**
- Water is computed directly from noise + distance, not from elevation
- Formula: `lerp(noise, 0.5, round) - (1-inflate) * dist² < 0`
- Distance = max(|nx|,|ny|) (Chebyshev), normalized to [-1,1] from center

### 2. Coast Detection — Topological, Not Elevation

**Target (mapgen2 biomes.js):**
- Coast = land tile that has at least one ocean-adjacent 4-neighbor
- Separate `assignCoast` step, runs after ocean assignment, before elevation

### 3. Lake Elevation Handling in BFS

**Target (mapgen2 elevation.js):**
- Lake-adjacent traversal gets distance increment = 0 (same elevation)
- Lake regions unshifted to front of BFS queue
- Revisit allowed: if shorter distance found, update it

### 4. Downslope Graph from BFS (Not Separate D8)

**Target (mapgen2 elevation.js + rivers.js):**
- Downslope direction is the parent pointer from the elevation BFS (N4 direction, 0-3)
- Flow follows BFS parent pointers to coast
- Removed separate `assignFlowDirection` function

### 5. River Generation — Explicit N-Spring Approach

**Target (mapgen2 rivers.js):**
- Springs: tiles with elevation 0.3–0.9, not adjacent to any water
- Fisher-Yates shuffle, pick N (NumRivers config, default 30)
- Each spring contributes 1 flow along its downslope path
- Trace from spring following BFS parent until reaching ocean
- `widenRivers`: perpendicular spread for wider channels (stoneheart-only feature)

### 6. Moisture — Riverbanks + Lakeshores as Seeds, sqrt Falloff

**Target (mapgen2 moisture.js):**
- Seeds: riverbanks (4-neighbors of river tiles) + lakeshores + lake tiles
- BFS only through non-water land tiles; water tiles get moisture = 1.0
- Falloff: `1.0 - sqrt(d/maxDist)` (square root)
- Linear redistribution over [bias, 1+bias] range

### 7. Temperature — Full Elevation Subtraction, No Clamping

**Target (mapgen2 biomes.js):**
- Formula: `temperature = 1.0 - elevation + lerp(biasNorth, biasSouth, lat)`
- No clamping — out-of-range values are expected
- Latitude: 0 at top (north) to 1 at bottom (south)

### 8. Full 20-Biome Classification with Water Depth

**Target (mapgen2 biomes.js + stoneheart water depth):**
- 4 water depth biomes split by IsShallow:
  - Deep Ocean (IsOcean, !IsShallow) — dark blue, soft game boundary
  - Shallow Ocean (IsOcean, IsShallow)
  - Deep Lake (IsWater, !IsOcean, !marsh/ice, !IsShallow)
  - Shallow Lake (IsWater, !IsOcean, !marsh/ice, IsShallow)
  - Marsh (IsWater, temp > 0.9)
  - Ice (IsWater, temp < 0.2)
- Coast: BEACH (topological IsCoast flag)
- 14 land biomes with exact mapgen2 moisture/temperature thresholds:

```
if temperature < 0.2:  (cold)
    moisture > 0.50 → SNOW
    moisture > 0.33 → TUNDRA
    moisture > 0.16 → BARE
    else → SCORCHED

if temperature < 0.4:  (cool)
    moisture > 0.66 → TAIGA
    moisture > 0.33 → SHRUBLAND
    else → TEMPERATE_DESERT

if temperature < 0.7:  (warm)
    moisture > 0.83 → TEMPERATE_RAIN_FOREST
    moisture > 0.50 → TEMPERATE_DECIDUOUS_FOREST
    moisture > 0.16 → GRASSLAND
    else → TEMPERATE_DESERT

else:  (hot)
    moisture > 0.66 → TROPICAL_RAIN_FOREST
    moisture > 0.33 → TROPICAL_SEASONAL_FOREST
    moisture > 0.16 → GRASSLAND
    else → SUBTROPICAL_DESERT
```

### 9. BFS Randomization in Elevation

**Target (mapgen2 elevation.js):**
- Each BFS step shuffles neighbor order using PRNG
- `iOffset = rng.IntN(4)`, iterate `(i + iOffset) % 4`

### 10. Noisy Edges

**Target (mapgen2 noisy-edges.js):**
- Recursive midpoint subdivision for coast edges
- Data generated, rendering deferred
- New `src/mapgen/noisyedges.go`

### 11. Shallow/Deep Water Depth

**Stoneheart addition:**
- `assignWaterDepth` — noise-blended elevation threshold: `Elevation + noise*0.15 > -0.25`
- Low-frequency opensimplex noise (~200-tile wavelength) creates organic bay/headland shapes
- `IsShallow` boolean on Tile

## Config (`MapConfig`)

```go
type MapConfig struct {
    Width, Height      int
    Seed               int64
    IslandRoundness    float64   // mapgen2 "round" — lerp noise toward 0.5
    IslandInflate      float64   // mapgen2 "inflate" — distance penalty multiplier
    NoiseAmplitudes    []float64 // [0.5, 0.25, 0.125, 0.0625]
    NumRivers          int       // how many springs (default 30)
    RiverMaxWidth      int       // max river spread (default 4)
    MoistureBias       float64   // drives redistribution [bias, 1+bias]
    NorthTempBias      float64   // temperature shift at north pole
    SouthTempBias      float64   // temperature shift at south pole
}
```

## Biome Enum (20 values)

```go
const (
    BiomeDeepOcean              BiomeType = iota // 0
    BiomeShallowOcean                            // 1
    BiomeDeepLake                                // 2
    BiomeShallowLake                             // 3
    BiomeMarsh                                   // 4
    BiomeIce                                     // 5
    BiomeBeach                                   // 6
    BiomeSnow                                    // 7
    BiomeTundra                                  // 8
    BiomeBare                                    // 9
    BiomeScorched                                // 10
    BiomeTaiga                                   // 11
    BiomeShrubland                               // 12
    BiomeTemperateDesert                         // 13
    BiomeTemperateRainForest                     // 14
    BiomeTemperateDeciduousForest                // 15
    BiomeGrassland                               // 16
    BiomeTropicalRainForest                      // 17
    BiomeTropicalSeasonalForest                  // 18
    BiomeSubtropicalDesert                       // 19
)
```

## Tile Struct

```go
type Tile struct {
    Elevation   float64
    Moisture    float64
    Temperature float64
    Biome       BiomeType
    Downslope   int       // BFS parent direction (0-3 N4, -1 none)
    IsWater     bool
    IsOcean     bool
    IsCoast     bool
    IsShallow   bool      // water depth flag
    IsRiver     bool
    RiverWidth  int
}
```

## Pipeline Order

```
assignIslandWater       — noise + Chebyshev distance → IsWater flag
assignOcean             — flood-fill from map edges → IsOcean flag
assignCoast             — land tiles adjacent to ocean → IsCoast flag
assignElevation         — BFS from water/land boundary, lake handling, BFS parent, randomization
assignWaterDepth        — noise-blended elevation threshold → IsShallow flag
redistributeElevation   — quadratic redistribution (low elevations more common)
findSprings             — candidate river sources (elev 0.3–0.9, no water neighbor)
assignRiverFlow         — Fisher-Yates shuffle springs, trace BFS parent to coast
widenRivers             — perpendicular spread for wider channels
findMoistureSeeds       — riverbanks + lakeshores + lake tiles
assignMoisture          — BFS through land only, sqrt falloff
redistributeMoisture    — linear redistribution over [bias, 1+bias]
assignTemperature       — 1.0 - elevation + lerp(biasN, biasS, lat), no clamping
assignBiomes            — 20-biome classification (4 water depth + 16 land)
assignNoisyEdges        — recursive subdivision for coast edges (data only)
```

## Verification

- Build: `go build ./...`
- Visual check: map viewer shows biome-colored tiles with shallow/deep water zones
- Unit tests for each generator step (deterministic output for fixed seed)

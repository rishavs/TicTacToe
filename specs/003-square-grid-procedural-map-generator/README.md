---
status: complete
created: 2026-07-15
tags:
- mapgen
- procedural-generation
- terrain
created_at: 2026-07-15T11:39:02.047606400Z
updated_at: 2026-07-15T11:50:49.226712Z
---
# Square-grid Procedural Map Generator

Procedural map generator for turn-based tactics on a huge square grid. Adapted from Red Blob Games' mapgen2 algorithm pipeline to a grid topology.

## Game Specs

- **Internal resolution:** 320×180 (pixel art)
- **Tile size:** 16×16 px
- **Map size:** Large (configurable, target ~256×256 to 1024×1024)
- **Output format:** Data model designed for TMX (Tiled) compatibility; export implemented later

## Pipeline

```
Island Shape (simplex noise + radial falloff)
  → Water/Ocean/Lake assignment
  → BFS elevation from coasts
  → Elevation redistribution
  → Flow direction (D8 downslope)
  → Flow accumulation
  → River tiles (threshold-based, width via accumulation)
  → Moisture diffusion (multi-source BFS)
  → Temperature (latitude + elevation)
  → Biome lookup
```

## Data Model

```go
type Tile struct {
  Elevation       float64   // -1.0 (deep ocean) to 1.0 (peak)
  Moisture        float64   // 0.0 (arid) to 1.0 (wet)
  Temperature     float64   // 0.0 (cold) to 1.0 (hot)
  Biome           BiomeType
  FlowDir         int       // D8 direction index (0-7), -1 for water
  FlowAccum       int       // accumulated flow count
  IsWater         bool
  IsOcean         bool
  IsLake          bool
  IsCoast         bool
  IsRiver         bool
  RiverWidth      int       // 0 = no river, 1-3+ for display
}

type GameMap struct {
  Width, Height int
  Tiles         []Tile
  Seed          int64
}
```

## Package Structure

```
src/mapgen/
  generator.go    — top-level pipeline, orchestrates all steps
  tilemap.go      — GameMap, Tile, MapConfig, helpers
  island.go       — simplex noise + radial falloff → elevation field
  water.go        — threshold → water; flood-fill → ocean vs lake
  elevation.go    — BFS from coasts, quadratic redistribution
  rivers.go       — D8 flow direction, flow accumulation, river carving (width)
  moisture.go     — multi-source BFS moisture from rivers + coasts
  biomes.go       — temperature by lat+elv, biome classification
  tmx.go          — (stub) convert GameMap → TMX data structure (later)

src/camera/
  camera.go       — Camera struct (X, Y offset, zoom level), viewport calculations
```

## Algorithm Details

### Island Shape (`island.go`)

- Fractal simplex noise (4 octaves, persistence ~0.5)
- Radial falloff: `1 - (dist_from_center / max_radius)^2`, clamped to [0,1]
- Island value: `noise_value * radial_falloff`
- Threshold (configurable, ~0.3) splits water vs land

Library: `github.com/ojrac/opensimplex-go` for simplex noise.

### Ocean vs Lake (`water.go`)

- Flood-fill (BFS) from all edge tiles that are water → mark as ocean
- Remaining water tiles → lakes

### Elevation (`elevation.go`)

- BFS from all coast tiles:
  - Land: positive elevation = coast_distance / max_land_distance
  - Ocean: negative elevation = -coast_distance / max_ocean_distance
- Track parent pointers per tile (used for flow direction base)
- Redistribute: sort land elevations, apply quadratic `1-(1-x)^2` curve

### Rivers (`rivers.go`)

- **D8 flow direction**: For each land tile, compute steepest downslope neighbor among 8 neighbors. This gives a flow direction index (0-7). Water tiles have no direction.
- **Flow accumulation**: Each land tile contributes count 1. Walk upstream-to-downstream (topological order) accumulating counts. A tile's flow accumulation = 1 + sum of all tiles that drain into it.
- **River threshold**: Tiles with accumulation > threshold become rivers.
- **River width**: Width = floor(log2(accumulation / threshold)) + 1, clamped to a max width. At width > 1, adjacent perpendicular tiles are also marked as river, creating wider channels at the mouth.
- Springs emerge naturally from the accumulation threshold — no random source selection needed.

### Moisture (`moisture.go`)

- Sources: river tiles + ocean coast tiles
- Multi-source BFS outward, BFS distance → moisture value
- Redistribute with configurable bias parameter

### Temperature (`biomes.go`)

- Latitude: `1 - |y/height - 0.5| * 2` (equator warm, poles cold)
- Elevation modifier: -0.1 per unit elevation
- North/south bias parameters

### Biome Classification (`biomes.go`)

| Elevation | Moisture | Temp | Biome |
|-----------|----------|------|-------|
| < 0 | any | any | Water (ocean/lake) |
| ~0 | any | any | Coast/Beach |
| > 0 | low | high | Desert |
| > 0 | high | high | Grassland |
| > 0 | high | medium | Forest |
| > 0 | medium | med | Hills |
| > 0 | med | low | Tundra |
| > 0 | low | low | Snow |
| > 0 | high | low | Taiga |

Thresholds from `MapConfig`, tunable.

### Camera (`camera.go`)

```go
type Camera struct {
  X, Y float64    // world offset in pixels
  Zoom float64    // 1.0 = default, 2.0 = 2x zoom, etc.
}
```

- `Viewport() (minX, minY, maxX, maxY)` — returns visible tile range
- Pan via arrow keys / WASD
- Zoom in/out via scroll wheel or +/- keys (clamped, e.g. 0.5x to 4x)
- Smooth scrolling optional (ease toward target position)

## Config

```go
type MapConfig struct {
  Width, Height     int
  Seed              int64
  IslandRoundness   float64     // radial falloff exponent (0.5–1.0)
  IslandInflation   float64     // land vs water bias (0.3–0.6)
  NoiseAmplitudes   []float64   // per-octave amplitudes
  RiverThreshold    int         // flow accumulation threshold for river
  RiverMaxWidth     int         // maximum river tile width
  MoistureBias      float64
  NorthTempBias     float64
  SouthTempBias     float64
  ElevThreshold     float64     // water/land cutoff
}
```

## Design Decisions

- **Square grid** → Hex is a data-structure change (neighbors, coordinates) without affecting algorithms; deferred
- **Row-major flat slice** → cache-friendly iteration, easy serialization to TMX
- **Pure functions** → each pipeline step takes and returns map state; testable without Ebiten
- **Deterministic** → same seed + config = identical map
- **D8 flow over random springs** → more realistic river networks with self-organizing width
- **No rendering in mapgen** → mapgen produces data; camera + scene renders viewport
- **Biome as enum** → fast switch-based rendering
- **TMX-ready data model** → tile IDs map to tileset indices; separate layers for ground, river, objects

## Out of Scope (this spec)

- TMX file export (design for it, implement later)
- Pathfinding on generated map
- Per-tile props (trees, rocks, buildings)
- Save/load map data
- Smooth camera transitions

## Files Created

```
src/mapgen/generator.go
src/mapgen/tilemap.go
src/mapgen/island.go
src/mapgen/water.go
src/mapgen/elevation.go
src/mapgen/rivers.go
src/mapgen/moisture.go
src/mapgen/biomes.go
src/mapgen/tmx.go            (stub — TMX type definitions only)
src/camera/camera.go
```

## Files Modified

- `go.mod` — add `github.com/ojrac/opensimplex-go`
- `STATUS.md` — add mapgen items
- `GDD.md` — fill in Maps & Terrain section

# GAME DESIGN DOCUMENT

> Living document — update as game mechanics, rules, and design decisions are made.

## Overview

- **Title:** Stoneheart
- **Genre:** Turn-based tactics
- **Engine:** Ebiten (Go)
- **Perspective:** TBD

## Core Game Loop

*TBD*

## Combat System

*TBD*

## Units & Characters

*TBD*

## Maps & Terrain

- **Grid:** Square tile grid (TMX-compatible data model, export later)
- **Internal resolution:** 320×180 (pixel art)
- **Tile size:** 16×16 px
- **Map size:** Configurable, target 256×256 to 1024×1024
- **Generation:** Procedural island generator (close port of Red Blob Games' mapgen2 to square grid)

### Biomes

20 biomes: 4 water depth zones + 16 land types classified by temperature × moisture matrix (mapgen2 thresholds):

| Biome | Conditions |
|-------|-----------|
| Deep Ocean | IsOcean, !IsShallow |
| Shallow Ocean | IsOcean, IsShallow |
| Deep Lake | IsWater, !IsOcean, marsh/ice excluded, !IsShallow |
| Shallow Lake | IsWater, !IsOcean, marsh/ice excluded, IsShallow |
| Marsh | IsWater, temperature > 0.9 |
| Ice | IsWater, temperature < 0.2 |
| Beach | IsCoast (land with ocean neighbor) |
| Snow | Temp < 0.2, moisture > 0.50 |
| Tundra | Temp < 0.2, moisture > 0.33 |
| Bare | Temp < 0.2, moisture > 0.16 |
| Scorched | Temp < 0.2, else |
| Taiga | Temp 0.2–0.4, moisture > 0.66 |
| Shrubland | Temp 0.2–0.4, moisture > 0.33 |
| Temperate Desert | Temp 0.2–0.4, else |
| Temperate Rain Forest | Temp 0.4–0.7, moisture > 0.83 |
| Temperate Deciduous Forest | Temp 0.4–0.7, moisture > 0.50 |
| Grassland | Temp 0.4–0.7, moisture > 0.16; or Temp ≥ 0.7, moisture > 0.16 |
| Tropical Rain Forest | Temp ≥ 0.7, moisture > 0.66 |
| Tropical Seasonal Forest | Temp ≥ 0.7, moisture > 0.33 |
| Subtropical Desert | Temp ≥ 0.7, else |

### Generation Pipeline

1. `assignIslandWater` — FBM simplex noise + Chebyshev distance: `lerp(noise, 0.5, round) - (1-inflate)*dist² < 0`
2. `assignOcean` — flood-fill from map edges (4-neighbor)
3. `assignCoast` — topological: land tile with an ocean neighbor
4. `assignElevation` — BFS from water-land boundary, randomized neighbor order, lake handling (increment=0), BFS parent as downslope
5. `redistributeElevation` — quadratic redistribution: low elevations more common
6. `assignHydrology` — rainfall, explicit lake basins, D8 flow directions, flow accumulation, watershed IDs, and river scale
7. `assignWaterDepth` — noise-blended ocean depth; explicit lakes keep basin-derived shallow/deep state
8. `assignMoisture` — rainfall blended with proximity to rivers, lakes, and ocean coast
9. `redistributeMoisture` — linear redistribution over [bias, 1+bias]
10. `assignTemperature` — `1.0 - elevation + lerp(biasNorth, biasSouth, latitude)`, no clamping
11. `assignBiomes` — 20-biome classification from ocean/water/shallow/coast/temperature/moisture
12. `assignLighting` — hillshade-style per-tile light level from neighboring elevation differentials

### Hydrology

- Rivers are no longer random spring lines. They are marked where accumulated downstream rainfall exceeds the configured flow threshold.
- Lakes are explicit inland basins with lake IDs, surface levels, and outlet tiles.
- Watershed IDs trace non-ocean tiles to a lake, ocean outlet, or terminal basin.
- River scale comes from accumulated flow and is used by rendering as a variable water overlay.
- Moisture uses rainfall plus proximity to rivers, lakes, and ocean coast before biome classification.

## Progression & Mechanics

*TBD*

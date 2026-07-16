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
5. `assignWaterDepth` — noise-blended elevation threshold: `Elevation + noise*0.15 > -0.25` → IsShallow
6. `redistributeElevation` — quadratic redistribution: low elevations more common
7. `findSprings` — candidate river sources (elevation 0.3–0.9, no water neighbor)
8. `assignRiverFlow` — Fisher-Yates shuffle springs, trace BFS parent to coast, mark IsRiver
9. `findMoistureSeeds` — riverbanks (4-neighbors of river tiles) + lakeshores + lake tiles
10. `assignMoisture` — BFS through land only, sqrt falloff: `1.0 - sqrt(d/maxDist)`
11. `redistributeMoisture` — linear redistribution over [bias, 1+bias]
12. `assignTemperature` — `1.0 - elevation + lerp(biasNorth, biasSouth, latitude)`, no clamping
13. `assignBiomes` — 20-biome classification from ocean/water/shallow/coast/temperature/moisture
14. `assignLighting` — hillshade-style per-tile light level from neighboring elevation differentials

## Progression & Mechanics

*TBD*

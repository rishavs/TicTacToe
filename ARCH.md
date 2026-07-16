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
    rivers.go       — N-spring river sources, downslope tracing, perpendicular spread
    moisture.go     — riverbanks + lakeshores seeds, BFS, sqrt falloff, linear redistribution
    biomes.go       — temperature (1.0-elevation + lat bias), 20-biome mapgen2 classification
    generator.go    — 14-step pipeline orchestrator
    noisyedges.go   — recursive midpoint subdivision for coast edges
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

## Key Systems

| System | Package | Purpose |
|--------|---------|---------|
| Scene Manager | `scene` | Scene stack/switching, delegates Update/Draw |
| Map Generator | `mapgen` | Procedural island generation (deterministic, pure data) |
| Camera | `camera` | Scroll/zoom viewport into large tile maps |

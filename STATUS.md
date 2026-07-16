# Stoneheart — Status

> Running checklist of completed work and pending todos. Keep updated after every meaningful change.

## Project Setup
- [x] Go module initialized (`go.mod` with Ebiten v2.9.9)
- [x] Boilerplate Ebiten game in `src/main.go` ("Hello, World!")
- [x] `assets/` directory created (empty)
- [x] LeanSpec MCP connection verified
- [x] Specs directory structure (`specs/`) with README
- [x] Added `golang.org/x/image` dependency for text rendering
- [x] Agent Go checks use repo-local `.gocache/` instead of the default user cache

## Scene / Menu System
- [x] `src/scene/` package created with `Scene` interface
- [x] Scene `Manager` for switching between scenes
- [x] **ebitenui** dependency added (`github.com/ebitenui/ebitenui v0.7.3`)
- [x] `src/scene/resources.go` — shared theme (button images, font faces)
- [x] Main menu with 5 buttons using ebitenui widgets (New, Mapgen, Battle, Settings, Quit)
- [x] Placeholder scenes for each menu option with Back button (ebitenui)
- [x] `src/main.go` updated to use scene manager
- [x] Removed manual `vector` rendering, `basicfont`, `inpututil`

## Documentation
- [x] `AGENTS.md` — project instructions & standards
- [x] `ARCH.md` — architecture document, package structure, data flow, mapgen logic/pipeline
- [x] `GDD.md` — stub created (TBD)
- [x] `CONV.md` — conversation log started
- [x] `STATUS.md` — this file

## Completed Specs
- [x] **001 — ebitengine-mcp integration** — Attempted, hit Windows bugs, reverted & archived
- [x] **002 — Main menu & scene system** — Scene manager, 5-button main menu (ebitenui), placeholder scenes with back button
- [x] **003 — Square-grid procedural map generator** — GameMap types, island noise, water/ocean/lake, BFS elevation, D8 flow accumulation rivers (width), moisture, temperature, biomes, camera/viewport
- [x] **004 — Close-port mapgen2 terrain pipeline** — Current code reconciled and spec closed. Island formula (Chebyshev+noise→water), topological coast, lake elevation handling, BFS-derived downslope, N-spring rivers, sqrt moisture falloff + redistribution, 1.0-elevation temperature, 20-biome classification, BFS randomization, lighting data
- [x] **005 — Wire noisy edges, fills, and lighting** — Closed as a design decision. Lighting and biome rendering remain; Edges/Fills were tried, gave no meaningful benefit, and are intentionally not planned.
- [x] **006 — Cleanup mapgen and viewer technical debt** — Removed dead river-width/noisy-edge paths, centralized resolution constants, moved active tuning values into MapConfig, reused noise generators, and split map viewer helpers
- [x] **007 — Dev screenshot capture and direct scene launch** — Added QA flags for direct scene launch, deterministic mapgen seed, and one-frame PNG capture
- [x] **008 — Watershed hydrology for rivers and lakes** — Replaced random spring-line rivers with rainfall, explicit lakes, D8 flow directions, watersheds, flow accumulation, variable river scale, and hydrology-based moisture
- [x] **009 — Always-on map viewer biomes and lighting** — Removed Biomes/Light controls; mapgen viewer always renders biome colors with lighting

- [x] **010 - Try 512x512 map viewer generation** - Mapgen viewer now generates 512x512 maps and supports fit-to-view with lower camera min zoom
- [x] **011 - Connect river rendering between tiles** - Added flow-aware river connectors to avoid dotted broken lines
- [x] **012 - River centerline rendering** - Moved river drawing to a dedicated overlay pass with downstream centerline strokes so rivers are continuous instead of chopped by later terrain tiles

## Map Generator
- [x] `src/mapgen/` package (10 files)
- [x] `src/mapgen/tilemap.go` — GameMap, Tile (Downslope, FlowDir, Flow, Rainfall, RiverScale, WatershedID, LakeID, Light, IsShallow), MapConfig hydrology tunables, 20 BiomeType enum, N4/D8 helpers
- [x] `src/mapgen/island.go` — `assignIslandWater`: FBM noise + Chebyshev distance, lerp(noise,0.5,round) - (1-inflate)*dist² < 0
- [x] `src/mapgen/water.go` — `assignOcean` (flood-fill from edges), `assignCoast` (topological: land tile adjacent to ocean)
- [x] `src/mapgen/elevation.go` — BFS from water/land boundary, lake handling (increment=0), randomized neighbor order, BFS parent as downslope, quadratic redistribution
- [x] `src/mapgen/hydrology.go` — rainfall, explicit lake basins/outlets, D8 flow directions, flow accumulation, watershed IDs, flow-threshold rivers
- [x] `src/mapgen/moisture.go` — rainfall + river/lake/coast proximity moisture, linear redistribution over [bias,1+bias]
- [x] `src/mapgen/biomes.go` — `assignTemperature` (1.0 - elevation + lerp(biasN,biasS,lat)), `assignBiomes` (20-biome mapgen2-style thresholds)
- [x] `src/mapgen/generator.go` — current pipeline: islandWater→ocean→coast→elevation→redistribute→hydrology→waterDepth→moisture→redistribute→temperature→biomes→lighting
- [x] `src/mapgen/tmx.go` — TMX type stubs
- [x] `github.com/ojrac/opensimplex-go` dependency

## Camera System
- [x] `src/camera/camera.go` — Camera struct with pan, zoom, viewport calc

## Map Viewer Scene
- [x] `src/scene/mapgen.go` — MapgenScene: generates 512×512 map, renders biome-colored tiles
- [x] Camera integration: arrow/WASD pan, scroll/keys zoom
- [x] Back button returns to main menu
- [x] Mapgen button in main menu wired to MapgenScene
- [x] Right panel with sliders (Wet/Dry, N-Cold/Hot, S-Cold/Hot, Smooth); Biomes and Light are always on with no controls
- [x] Edges/Fills viewer modes intentionally dropped after experiment (`005`)
- [x] Rivers render as coherent variable-scale water overlays instead of fixed bright debug lines
- [x] River rendering runs as a second overlay pass and draws downstream centerline strokes so terrain tiles cannot cut rivers into dots or segments
- [x] Default map lighting ambient raised to 0.65 for brighter lit terrain
- [x] Regenerate button, F-key fit, zoom range 0.02×–16×
- [x] QA capture mode: `--qa-scene`, `--qa-seed`, `--qa-capture`

## Theme / UI Polish
- [x] Font swapped from goregular to Press Start 2P (pixel font, embedded via embed.FS)
- [x] Resolution set to 320×180
- [x] Built-in ebitenui dark theme (`themes.GetBasicDarkTheme()`) with Press Start 2P font override
- [x] Standalone `ScrollContainer` with vertical scrollbar slider + mouse wheel wiring
- [x] Compact 6×6 checkbox images for tight panel

## Slider Fixes & Size Reduction
- [x] Content sliders had `PreferredSize()` width=0 (horizontal) → invisible + unclickable
- [x] Fixed with `RowLayoutData{Stretch: true}` + `MinSize(0, 16)` → functional sliders
- [x] Panel width reduced 80 → 50px, checkboxes 10×10 → 6×6
- [x] Slider height 16→8px, handle 10→6px; scrollbar 10→6px wide, handle 12→8px
- [x] Button padding tightened via theme ({4,4,1,1}), grid spacing 1→0
- [x] Font stays at crisp 8px Press Start 2P (6px was blurry)

## Active / Next Up
- Tune watershed/lake generation by visual QA across more seeds after playtesting

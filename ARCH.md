# ARCHITECTURE

> Living document - update as modules, systems, and data flow evolve.

## Tech Stack

- **Project:** TicTacToe
- **Language:** Rust 2024
- **Engine/toolkit:** Macroquad 0.4.15
- **Procedural noise:** `noise`
- **Seeded randomness:** Macroquad's owned `macroquad::rand::RandGenerator`
- **Desktop window polish:** Windows-only startup maximize hook via `windows-sys`
- **Current focus:** Scene shell plus procedural map-generation viewer

## Module Layout

```text
src/
  main.rs              - Macroquad config, maximized startup, main loop, scene dispatch, screenshot capture
  scenes/
    mod.rs             - Scene enum
    menu.rs            - Main menu and scene navigation buttons
    play.rs            - Play placeholder scene
    battle.rs          - Battle placeholder scene
    settings.rs        - Settings placeholder scene
    mapgen.rs          - Mapgen scene state, input, background worker, scene-level tests
    mapgen/biome.rs    - Biome classification, color interpolation, slope lighting
    mapgen/generate.rs - Deterministic map generation pipeline and map query methods
    mapgen/model.rs    - Map data structs: centers, corners, edges, noisy edges, PolyMap
    mapgen/noise.rs    - Island profiles and cached procedural noise sampling
    mapgen/random.rs   - Macroquad owned RNG wrappers for deterministic generation
    mapgen/render.rs   - Mapgen drawing, sidebar UI, histograms, map edge/polygon rendering
    mapgen/seed.rs     - Seed parsing, random seed text, seed input helpers
    mapgen/tests.rs    - Mapgen unit tests
```

There are no active Go/Ebiten modules and no LeanSpec-managed specs in the current checkout.

## Runtime Flow

```text
Macroquad window setup
  -> choose initial Scene
  -> per frame:
       match Scene
       call scenes::<scene>::update()
       apply returned Scene transition, if any
       optional screenshot capture tick
       next_frame().await
```

`src/main.rs` owns the frame loop. On Windows it asks the OS to maximize the normal resizable window once after Macroquad creates it; this is intentionally not fullscreen. Each scene module exposes `pub fn update() -> Option<Scene>`. Returning `Some(scene)` changes scenes; returning `None` keeps the current scene active.

## Scenes

| Scene | Module | Current behavior |
|-------|--------|------------------|
| MainMenu | `src/scenes/menu.rs` | Draws a centered Macroquad UI window branded as TicTacToe with Play, Mapgen, Battle, Settings, and Quit buttons. |
| Play | `src/scenes/play.rs` | Placeholder label; Escape returns to MainMenu. |
| Mapgen | `src/scenes/mapgen.rs` | Interactive procedural island/map viewer with sidebar controls, panning, zooming, regeneration, deterministic seeds, and unit-tested helpers. |
| Battle | `src/scenes/battle.rs` | Placeholder label; Escape returns to MainMenu. |
| Settings | `src/scenes/settings.rs` | Placeholder label; Escape returns to MainMenu. |

## Mapgen Scene

`src/scenes/mapgen.rs` contains scene state, input, and background generation coordination. Child modules under `src/scenes/mapgen/` hold the deterministic model, generation pipeline, rendering, biome/color math, noise, RNG, seed helpers, and tests.

- `MapgenScene` stores selected seed, island type, point type, point count, view mode, current generated map, pending generation job, pan, zoom, and status.
- `PolyMap` stores generated centers, corners, edges, noisy edges, and histogram data in `mapgen/model.rs`.
- `Center`, `Corner`, `Edge`, and `NoisyEdge` model the graph used by the map renderer. Centers also store ocean depth classification state, including `shallow_ocean` and `ocean_distance`.
- `IslandType` supports radial, perlin, and simplex shaping.
- `PointType` currently supports square point layout.
- `ViewMode` supports biome and slope-style debug views.
- Seed parsing lives in `mapgen/seed.rs`; pan/zoom math remains in the scene shell; biome classification lives in `mapgen/biome.rs`; graph generation and ocean-depth assignment live in `mapgen/generate.rs`.
- Rendering and Macroquad UI widgets live in `mapgen/render.rs`; rendering reads map state and does not build maps. In wide windows, the square map is centered inside a neutral gray map area instead of stretching or leaving unused white space.
- Regeneration runs on a background worker thread and sends the finished `PolyMap` back to the scene through a channel. The previous map remains visible while a new map is building.
- `noise` supplies Perlin/fBm and OpenSimplex noise; small local wrappers normalize output into the mapgen-friendly `0.0..=1.0` range.
- `IslandProfile` caches reusable noise generators so map sampling does not recreate noise objects per corner.
- `macroquad::rand::RandGenerator` supplies deterministic seeded randomness through small local wrapper functions. Mapgen uses owned RNG instances instead of Macroquad's global RNG state.

The scene uses a narrow static cache:

```rust
static STATE: OnceLock<Mutex<MapgenScene>> = OnceLock::new();
```

This is intentional for the current Macroquad scene model: the mapgen scene keeps state across frames while scene modules remain simple `update()` functions. Avoid broadening this pattern unless the scene system is refactored.

## Mapgen Data Flow

```text
debug env / UI controls
  -> MapgenScene::regenerate
  -> background worker calls PolyMap::generate(seed, island type, point type, point count)
  -> graph construction, elevation, Perlin edge-buffer shaping, ocean/coast/land assignment
  -> ocean-depth assignment, moisture, rivers, biomes, noisy edges
  -> worker sends completed PolyMap over channel
  -> render module draws visible polygons into the square map viewport through Macroquad
```

Generation is deterministic for the same seed and options. Perlin maps apply an edge-distance land falloff so about two edge cells remain deep ocean after the shallow shelf is assigned, without expanding the grid. Simplex keeps its current island size and uses named radial threshold constants rather than a cell-based edge buffer. Ocean depth is assigned with a breadth-first distance from land through connected ocean centers, then softened with deterministic coordinate jitter so shallow water follows the island shape without becoming an exact outline. The test suite includes checks for seed parsing, layout math, point generation, determinism, graph links, elevation/moisture ranges, biome categories, Perlin edge buffering, shallow/deep ocean placement, and drainage behavior.

## Debug Launch And Capture

Initial scene:

```powershell
$env:TICTACTOE_START_SCENE = "mapgen"
cargo run
```

Supported values: `menu`, `play`, `mapgen`, `battle`, `settings`.

Screenshot capture:

```powershell
$env:TICTACTOE_START_SCENE = "mapgen"
$env:TICTACTOE_SCREENSHOT = ".qa-captures\mapgen.png"
$env:TICTACTOE_SCREENSHOT_FRAMES = "3"
cargo run
```

Mapgen controls:

```powershell
$env:TICTACTOE_MAPGEN_SEED = "85882-8"
$env:TICTACTOE_MAPGEN_ISLAND = "perlin"
$env:TICTACTOE_MAPGEN_POINTS = "square"
$env:TICTACTOE_MAPGEN_COUNT = "4000"
$env:TICTACTOE_MAPGEN_VIEW = "biomes"
```

Screenshot paths should stay in ignored capture folders unless the user explicitly asks to keep them.

## Boundaries

- Rendering and UI depend directly on Macroquad and stay in `src/scenes/mapgen/render.rs`.
- Mapgen generation logic is separated from the scene shell and remains pure/unit-tested where practical.
- If mapgen grows further, the likely next architectural improvement is to move scene-agnostic modules out of `src/scenes/` into a game/domain-level mapgen package.
- `target/`, `temp/`, and capture folders are local output and should not be committed.

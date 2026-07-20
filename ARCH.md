# ARCHITECTURE

> Living document - update as modules, systems, and data flow evolve.

## Tech Stack

- **Project:** TicTacToe
- **Language:** Rust 2024
- **Engine/toolkit:** Macroquad 0.4.15
- **Current focus:** Scene shell plus procedural map-generation viewer

## Module Layout

```text
src/
  main.rs              - Macroquad config, main loop, scene dispatch, screenshot capture
  scenes/
    mod.rs             - Scene enum
    menu.rs            - Main menu and scene navigation buttons
    play.rs            - Play placeholder scene
    battle.rs          - Battle placeholder scene
    settings.rs        - Settings placeholder scene
    mapgen.rs          - Mapgen viewer, generation model, rendering helpers, tests
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

`src/main.rs` owns the frame loop. Each scene module exposes `pub fn update() -> Option<Scene>`. Returning `Some(scene)` changes scenes; returning `None` keeps the current scene active.

## Scenes

| Scene | Module | Current behavior |
|-------|--------|------------------|
| MainMenu | `src/scenes/menu.rs` | Draws a centered Macroquad UI window branded as TicTacToe with Play, Mapgen, Battle, Settings, and Quit buttons. |
| Play | `src/scenes/play.rs` | Placeholder label; Escape returns to MainMenu. |
| Mapgen | `src/scenes/mapgen.rs` | Interactive procedural island/map viewer with sidebar controls, panning, zooming, regeneration, deterministic seeds, and unit-tested helpers. |
| Battle | `src/scenes/battle.rs` | Placeholder label; Escape returns to MainMenu. |
| Settings | `src/scenes/settings.rs` | Placeholder label; Escape returns to MainMenu. |

## Mapgen Scene

`src/scenes/mapgen.rs` is currently the largest module. It contains both viewer UI and generation logic:

- `MapgenScene` stores selected seed, island type, point type, point count, view mode, generated map, pan, zoom, and status.
- `PolyMap` stores generated centers, corners, edges, noisy edges, and histogram data.
- `Center`, `Corner`, `Edge`, and `NoisyEdge` model the graph used by the map renderer.
- `IslandType` supports radial, perlin, and simplex shaping.
- `PointType` currently supports square point layout.
- `ViewMode` supports biome and slope-style debug views.
- `PmPrng`, noise helpers, seed parsing, pan/zoom math, biome classification, graph linking, and rendering helpers live in the same file.

The scene uses a narrow static cache:

```rust
static STATE: OnceLock<Mutex<MapgenScene>> = OnceLock::new();
```

This is intentional for the current Macroquad scene model: the mapgen scene keeps state across frames while scene modules remain simple `update()` functions. Avoid broadening this pattern unless the scene system is refactored.

## Mapgen Data Flow

```text
debug env / UI controls
  -> MapgenScene::new or Regenerate
  -> PolyMap::generate(seed, island type, point type, point count)
  -> graph construction, elevation, moisture, rivers, biomes, noisy edges
  -> draw visible polygons and overlays through Macroquad
```

Generation is deterministic for the same seed and options. The test suite includes checks for seed parsing, layout math, point generation, determinism, graph links, elevation/moisture ranges, biome categories, and drainage behavior.

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

- Rendering and UI currently depend directly on Macroquad.
- Mapgen logic is embedded in `src/scenes/mapgen.rs`, but many helpers are pure and unit-tested.
- If mapgen grows further, the likely next architectural improvement is to split generation/model code from scene drawing into a dedicated module.
- `target/`, `temp/`, and capture folders are local output and should not be committed.

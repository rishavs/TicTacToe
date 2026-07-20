# TicTacToe - Status

> Running checklist of completed work and pending todos. Keep updated after meaningful changes.

## Project Setup

- [x] Rust project initialized with `Cargo.toml` and `Cargo.lock`
- [x] Package renamed to `tictactoe`
- [x] Macroquad dependency added (`macroquad = "0.4.15"`)
- [x] Procedural noise uses the `noise` crate
- [x] Deterministic map randomness uses Macroquad's owned `macroquad::rand::RandGenerator`
- [x] Rust 2024 edition configured
- [x] `target/`, `temp/`, and capture folders ignored by git
- [x] LeanSpec removed from the active workflow

## Documentation

- [x] `AGENTS.md` updated for Rust + Macroquad workflow
- [x] `ARCH.md` updated to match current source layout
- [x] `GDD.md` updated to describe current game/design state
- [x] `STATUS.md` rebuilt as current project status
- [x] Historical planning notes live under `docs/superpowers/specs/` and `docs/superpowers/plans/`

## Runtime Shell

- [x] Macroquad window configured in `src/main.rs`
- [x] Window title renamed to TicTacToe
- [x] Main loop dispatches through the `Scene` enum
- [x] Windows desktop startup maximizes the normal resizable window without switching to fullscreen
- [x] Debug start scene uses `TICTACTOE_START_SCENE`
- [x] Screenshot capture uses `TICTACTOE_SCREENSHOT` and `TICTACTOE_SCREENSHOT_FRAMES`

## Scene System

- [x] `src/scenes/mod.rs` defines `Scene`
- [x] `src/scenes/menu.rs` provides main menu navigation
- [x] Main menu branding renamed to TicTacToe
- [x] `src/scenes/play.rs` placeholder scene
- [x] `src/scenes/battle.rs` placeholder scene
- [x] `src/scenes/settings.rs` placeholder scene
- [x] Escape returns placeholder scenes to the main menu

## Mapgen Scene

- [x] `src/scenes/mapgen.rs` contains the active map-generation viewer
- [x] Mapgen rendering split into `src/scenes/mapgen/render.rs`
- [x] Mapgen model, generation, biome, noise, RNG, seed, and tests split into focused child modules
- [x] `src/scenes/mapgen.rs` slimmed to the scene shell and worker coordination
- [x] Scene state cached with `OnceLock<Mutex<MapgenScene>>`
- [x] Map regeneration runs on a background worker so the Macroquad window keeps repainting
- [x] Seed input and random seed generation
- [x] Debug environment controls renamed to `TICTACTOE_MAPGEN_*`
- [x] Island shape options: perlin, simplex
- [x] Square point layout kept as the always-on internal layout
- [x] Island size options: 4000, 8000, 16000, 32000
- [x] Shallow sea size options: narrow, normal, wide, very wide
- [x] View modes include biome and slopes
- [x] Sidebar shows a biome color/count list instead of histograms
- [x] Pan and zoom support
- [x] Perlin island shaping leaves about two deep-ocean cells at the generated map edge
- [x] Wide mapgen windows center the square map inside a neutral gray map area
- [x] Deterministic generation tests
- [x] Custom hand-rolled noise internals replaced with the `noise` crate
- [x] Island profile caches reusable noise generators for much faster generation
- [x] Custom hand-rolled PRNG replaced with Macroquad's owned RNG
- [x] Renderer iterates center borders and edges directly instead of repeatedly searching neighbor edges
- [x] Ocean biome split into shallow and deep ocean, with shallow water following the island coastline using deterministic coast-distance jitter
- [x] Shallow sea size control expands shallow water from the narrow default while preserving deep ocean
- [x] Narrow deep-ocean fingers inside bays are rounded into shallow ocean without broadening the whole ocean
- [x] Enclosed deep-ocean pockets fully surrounded by shallow ocean are promoted to shallow without removing open deep ocean
- [x] Disconnected islands are joined to the mainland with island-size shallow-ocean corridors while preserving border-connected deep ocean
- [x] Tests cover layout math, seed parsing, point generation, graph links, elevation/moisture normalization, biome categories, shallow/deep ocean placement, bay rounding, island shallow connectivity, anti-thread bridge corridors, and drainage-loop behavior

## Removed / Stale Items

- [x] Go/Ebiten workflow removed from project instructions
- [x] LeanSpec workflow removed from project instructions
- [x] `CONV.md` no longer listed as required project documentation
- [x] Willow UI references removed from root docs
- [x] Root docs no longer describe deleted `src/mapgen/`, `src/mapgen4/`, `src/camera/`, or Go package paths

## Active / Next Up

- [ ] Decide whether TicTacToe is a literal game direction, a temporary rename, or a placeholder for a future rename
- [ ] Decide whether scene-agnostic mapgen modules should move out of `src/scenes/` into a domain-level package
- [ ] Add real Play/Battle/Settings behavior when the game direction is confirmed
- [ ] Add visual QA captures for scene/layout changes when useful

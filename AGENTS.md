# TicTacToe

> A Rust + Macroquad game project currently focused on scene navigation and procedural map-generation experiments.

## Project Documentation

- **ARCH.md** - Current architecture, module layout, runtime flow, debug hooks, and system boundaries.
- **GDD.md** - Current game design direction and mechanics decisions.
- **STATUS.md** - Running checklist of completed work, current state, and next steps.
- **docs/superpowers/specs/** - Lightweight design notes for larger changes.
- **docs/superpowers/plans/** - Implementation plans for larger changes.

Keep ARCH.md, GDD.md, and STATUS.md current after meaningful changes. LeanSpec has been removed from this project; do not create or manage `.lean-spec/` or `specs/` content.

## Toolchain

This project uses Rust 2024 and Macroquad.

Common commands:

```powershell
cargo test
cargo run
cargo build
```

Guidelines:

- Prefer capabilities in this order: Macroquad APIs first, then well-maintained Cargo libraries, then local code when no good library exists.
- Use Macroquad APIs for rendering, input, UI, screenshots, window setup, and owned deterministic randomness unless a local module already provides a better abstraction.
- Use the `noise` crate for procedural noise instead of hand-rolled noise math.
- Use Macroquad's owned `macroquad::rand::RandGenerator` for deterministic seeded map generation; avoid global RNG state.

## Debug Launch / Visual Capture

Use Macroquad's built-in debug environment variables when you need to open a
specific scene or save a screenshot for review. This is the preferred visual QA
path for scene layout, mapgen output, and menu checks.

### Launch Normally

Run the game from the repo root:

```powershell
cargo run
```

The game opens at the main menu. From there:

- Click `Play` to open the Play placeholder scene.
- Click `Mapgen` to open the procedural map viewer.
- Click `Battle` to open the Battle placeholder scene.
- Click `Settings` to open the Settings placeholder scene.
- Press `Escape` in placeholder scenes to return to the main menu.

### Launch Directly To A Scene

Set `TICTACTOE_START_SCENE` before `cargo run`:

```powershell
$env:TICTACTOE_START_SCENE = "mapgen"
cargo run
```

Supported values:

- `menu`
- `play`
- `mapgen`
- `battle`
- `settings`

### Capture A Screenshot For Review

Set the start scene, output path, and optional frame delay. The frame delay lets
Macroquad draw a few frames before capture.

```powershell
$env:TICTACTOE_START_SCENE = "mapgen"
$env:TICTACTOE_SCREENSHOT = ".qa-captures\mapgen.png"
$env:TICTACTOE_SCREENSHOT_FRAMES = "3"
cargo run
```

The app saves the PNG and exits automatically. Keep review captures in
`.qa-captures/`, `.tmp-qa-captures/`, or `captures/`; those folders are ignored
by git.

Mapgen builds maps on a background worker. For high point counts, use a larger
`TICTACTOE_SCREENSHOT_FRAMES` value so the capture records the completed map
instead of the in-progress loading frame.

Examples:

```powershell
# Main menu capture
$env:TICTACTOE_START_SCENE = "menu"
$env:TICTACTOE_SCREENSHOT = ".qa-captures\menu.png"
$env:TICTACTOE_SCREENSHOT_FRAMES = "3"
cargo run
```

```powershell
# Mapgen capture with deterministic controls
$env:TICTACTOE_START_SCENE = "mapgen"
$env:TICTACTOE_MAPGEN_SEED = "85882-8"
$env:TICTACTOE_MAPGEN_ISLAND = "perlin"
$env:TICTACTOE_MAPGEN_POINTS = "square"
$env:TICTACTOE_MAPGEN_COUNT = "16000"
$env:TICTACTOE_MAPGEN_VIEW = "biomes"
$env:TICTACTOE_SCREENSHOT = ".qa-captures\mapgen-seed-85882-8.png"
$env:TICTACTOE_SCREENSHOT_FRAMES = "3"
cargo run
```

After a capture, inspect the saved PNG directly before claiming visual behavior
is correct.

## Design Principles

| Principle | Details |
|-----------|---------|
| **Current code is the source of truth** | Root docs must describe the Rust/Macroquad code that exists now. Avoid carrying forward stale Go/Ebiten assumptions. |
| **Separate logic and rendering where practical** | Keep deterministic generation and simulation helpers testable without needing a live window. Rendering should read state and draw it. |
| **Deterministic map generation** | Same seed and same controls should produce the same map. Preserve this for tests, visual debugging, and future replayability. |
| **Small, explicit state** | Prefer clear structs, enums, and explicit transitions over hidden side effects. If global scene state is needed for Macroquad, keep it narrow and documented. |
| **Config over constants when behavior is tunable** | Keep player-facing and debug-facing controls represented as data or well-named constants. |
| **Simple boundaries first** | The current project is compact. Add modules only when they make the code easier to test, explain, or change. |

## Current Architecture Expectations

- `src/main.rs` owns Macroquad setup, the main loop, scene dispatch, and screenshot capture.
- `src/scenes/mod.rs` defines the `Scene` enum.
- `src/scenes/menu.rs`, `play.rs`, `battle.rs`, and `settings.rs` are simple scene modules.
- `src/scenes/mapgen.rs` currently contains the mapgen viewer, generation model, rendering helpers, debug controls, and tests.
- The main menu should show TicTacToe branding.
- Environment variable names should use the `TICTACTOE_` prefix.

## Planning Workflow

LeanSpec is not used here.

For small bug fixes, doc corrections, and self-contained refactors:
- Make the change directly.
- Update ARCH.md, GDD.md, and STATUS.md if the project shape or design changed.
- Run the smallest useful verification command, usually `cargo test`.

For multi-part features or design decisions:
- Write a concise Markdown spec under `docs/superpowers/specs/YYYY-MM-DD-topic.md`.
- If the implementation is non-trivial, write an implementation plan under `docs/superpowers/plans/YYYY-MM-DD-topic.md`.
- Keep the spec focused on intent and acceptance criteria; keep the plan focused on steps and verification.

## Rust Standards

| Guideline | Details |
|-----------|---------|
| **Use enums for states** | Scene/state transitions should be explicit and pattern-matchable. |
| **Keep tests near pure logic** | Map generation helpers, parsing, clamping, layout math, and deterministic algorithms should have unit tests. |
| **Avoid broad globals** | Static state such as the mapgen scene cache is acceptable only when it is narrow, intentional, and protected. |
| **No filesystem reads in the hot loop** | Load or derive resources outside per-frame draw/update paths. |
| **Profile before optimizing** | Map generation and rendering can get expensive; measure before introducing complex caching or concurrency. |
| **Prefer standard library and current dependencies** | Do not add crates unless they solve a real problem better than the existing stack. |

## Documentation Hygiene

- Keep STATUS.md honest: checked items should reflect files and behavior that exist in the current checkout.
- Keep ARCH.md architectural, not a changelog.
- Keep GDD.md about Game logic direction, not implementation minutiae.

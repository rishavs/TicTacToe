# GAME DESIGN DOCUMENT

> Living document - update as game mechanics, rules, and design decisions are made.

## Overview

- **Title:** TicTacToe
- **Current genre:** Experimental game shell with procedural map-generation viewer
- **Runtime:** Rust + Macroquad
- **Current playable surface:** Main menu, placeholder scenes, and interactive Mapgen scene

The project is temporarily named TicTacToe. Existing map-generation work is still useful exploration, but the tactical/combat design from the older project has not been carried forward as implemented gameplay.

## Current Game Loop

1. Launch into the main menu.
2. Choose Play, Mapgen, Battle, Settings, or Quit.
3. Placeholder scenes show their scene name and return to the menu with Escape.
4. Mapgen opens an interactive procedural island viewer.
5. Mapgen users can adjust seed/options, regenerate, pan, zoom, and switch debug views.
6. On desktop, the game starts in a maximized normal window for easier map inspection.

## Implemented Scenes

| Scene | Design status |
|-------|---------------|
| Main menu | Implemented navigation shell. |
| Play | Placeholder; no rules yet. |
| Mapgen | Implemented procedural map viewer and experiment surface. |
| Battle | Placeholder; no combat rules yet. |
| Settings | Placeholder; no settings model yet. |

## Mapgen Design

Mapgen is the most developed design area. It is an exploratory island generator inspired by Red Blob style map-generation work and older Flash/SWF demo behavior.

Current controls:

- Seed text input, with random seed generation.
- Island shape: perlin and simplex.
- Island size: 16000, 32000, 64000, 128000 square-layout cells.
- Shallow sea size: narrow, normal, wide.
- Bay rounding: light, normal, strong.
- View mode: biome and slope-oriented debug rendering.
- Right-panel biome list with color swatches and cell counts.
- Pan and zoom for map inspection.

Current generation concepts:

- Deterministic seed parsing from text values such as `85882-8`.
- Square point selection and region construction.
- Center/corner/edge graph linking.
- Island profile shaping.
- Elevation and moisture assignment.
- River/drainage handling that produces `River` base biome cells.
- Biome classification, including separate shallow ocean, deep ocean, lake, marsh, and river water biomes.
- Current land biome vocabulary includes snow, tundra, highlands, peak, taiga, shrubland, rocky plains, forest, meadow, grassland, desert, rainforest, and woodland.
- Noisy edge paths for more organic region boundaries.
- Color interpolation and biome palettes for rendering.

Ocean design:

- Deep ocean represents open water farther away from land.
- Shallow ocean follows the island coastline roughly, but not exactly.
- The shallow band is based on ocean distance from land plus deterministic map-space jitter, so the same seed always produces the same coastal shelf. `Wide` is the default/current baseline; smaller shallow sea settings preserve more near-land deep ocean.
- Narrow deep-ocean fingers and concave deep-water inlets inside bays can be rounded into shallow ocean with the bay rounding control. `Light` is the default; `Normal` and `Strong` progressively close larger bites. The cleanup closes the land-plus-shallow-water mask, trims exposed edges back, then fills any remaining near-land pinches. This is intended to make the shallow boundary read as a rounded shelf rather than a concave polygon, while keeping border-connected open deep ocean as the impassable boundary.
- Small deep-ocean pockets fully enclosed by shallow ocean are treated as shallow ocean; open/border-connected deep ocean remains deep.
- Every island is connected back to the mainland through shallow ocean corridors. Corridor thickness scales with island size to avoid fragile one-cell bridge threads while keeping deep ocean as an impassable boundary.
- River cells are base biome tiles rather than decorative edge lines. They are intended to support future boat movement and Tiled export as real terrain. River width is capped at one to three cells, with weak streams using one cell and stronger confluences widening modestly.
- Shallow/deep ocean and river cells are classification states only for now; they do not yet affect implemented movement, combat, resources, or win/loss rules.
- Perlin maps keep land away from the generated map edge with an edge-distance falloff, leaving about five cells of deep-ocean buffer outside the shallow shelf without increasing grid size.
- Simplex maps keep their current island scale; their shape uses threshold constants rather than the Perlin cell-buffer rule.

## Combat, Rules, And Progression

No TicTacToe rules, battle rules, unit rules, win/loss conditions, or progression systems are implemented yet.

Before adding gameplay rules, decide whether this project should become:

- a literal TicTacToe game,
- a tactics prototype using the temporary TicTacToe name,
- or a map-generation sandbox that will later be renamed again.

## UI Direction

- Use Macroquad UI for the current simple menu and debug controls.
- Keep the first screen usable; no landing-page style shell.
- Keep debug controls dense and functional for map inspection.
- Prefer direct labels and immediate feedback over decorative UI until the actual game direction settles.

## Design Constraints

- Same seed plus same mapgen options should produce the same result.
- Rendering should remain responsive while inspecting generated maps.
- Map inspection should start with a generous viewport: maximized desktop window and Perlin maps that leave visible ocean breathing room at the generated map edge.
- Placeholder scenes should stay simple until they receive real rules.
- Documentation should describe the current Rust/Macroquad project, not the removed Go/Ebiten version.

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
- Island shape: radial, perlin, simplex.
- Point layout: square.
- Point count: 4000, 8000, 16000, 32000.
- View mode: biome and slope-oriented debug rendering.
- Pan and zoom for map inspection.

Current generation concepts:

- Deterministic seed parsing from text values such as `85882-8`.
- Square point selection and region construction.
- Center/corner/edge graph linking.
- Island profile shaping.
- Elevation and moisture assignment.
- River/drainage handling.
- Biome classification.
- Noisy edge paths for more organic region boundaries.
- Color interpolation and biome palettes for rendering.

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
- Placeholder scenes should stay simple until they receive real rules.
- Documentation should describe the current Rust/Macroquad project, not the removed Go/Ebiten version.

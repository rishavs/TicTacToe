# Scene System & Main Menu

**Date:** 2026-07-19
**Status:** draft

## Overview

Add a scene system and main menu to the TicTacToe turn-based tactics game. The game transitions between scenes via enum-based dispatch: MainMenu, Play, Mapgen, Battle, Settings. Quit exits the application.

## Architecture

### Scene Enum

Defined in `src/scenes/mod.rs`:

```rust
pub enum Scene {
    MainMenu,
    Play,
    Mapgen,
    Battle,
    Settings,
}
```

### Dispatch

`main.rs` holds a `Scene` variable. Each frame, it matches on the current scene and calls the corresponding module's `update()` function. If `update()` returns `Some(Scene)`, the active scene changes.

```rust
let mut scene = Scene::MainMenu;
loop {
    let next = match scene {
        Scene::MainMenu => scenes::menu::update(),
        Scene::Play      => scenes::play::update(),
        Scene::Mapgen    => scenes::mapgen::update(),
        Scene::Battle    => scenes::battle::update(),
        Scene::Settings  => scenes::settings::update(),
    };
    if let Some(new_scene) = next {
        scene = new_scene;
    }
    next_frame().await;
}
```

### Module structure

```
src/
  main.rs
  scenes/
    mod.rs       -- Scene enum, shared constants
    menu.rs      -- MainMenu scene
    play.rs      -- Play scene (empty placeholder)
    mapgen.rs    -- Mapgen scene (3D view from original main.rs)
    battle.rs    -- Battle scene (empty placeholder)
    settings.rs  -- Settings scene (empty placeholder)
```

Each scene module exports a single `pub fn update() -> Option<Scene>`.

### Navigation

- **MainMenu** → buttons navigate to each scene. Quit button calls `std::process::exit(0)`.
- **All other scenes** → Escape key returns to `Scene::MainMenu`.

## UI

Uses `macroquad::ui` (`root_ui()`, `button()`, `label()`) — no custom GUI framework.

### Main Menu Layout

- Window title: `"TicTacToe"`
- Centered vertical column of buttons, each ~200x40, 10px spacing
- Buttons: Play, Mapgen, Battle, Settings, Quit
- No decorative styling in v1

### Placeholder Scenes

- Play, Battle, Settings: centered label with the scene name. Escape returns to menu.

### Mapgen Scene

- Preserves the existing 3D scene from `main.rs`: grid, wireframe cubes, solid cube, sphere, camera, and "WELCOME TO 3D WORLD" text
- Escape returns to menu

## Dependencies

- `macroquad = "0.4.15"` (already in Cargo.toml, includes `macroquad::ui`)

## Out of Scope

- Fade transitions or animations (future)
- Actual gameplay logic in Play/Battle scenes (future)
- Settings UI (future)
- Map interaction in Mapgen (future)

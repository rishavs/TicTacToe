# Scene System & Main Menu — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor the single-scene macroquad app into a multi-scene architecture with a main menu using `macroquad::ui`.

**Architecture:** Enum-based scene dispatch. `main.rs` holds a `Scene` enum and matches on it each frame to call the corresponding scene module's `pub fn update() -> Option<Scene>`. Each scene is a module under `src/scenes/`. Navigation uses `macroquad::ui` buttons on the main menu and Escape key on all other scenes.

**Tech Stack:** Rust edition 2024, macroquad 0.4.15 (includes `macroquad::ui`)

## Global Constraints

- Use `macroquad::ui` (`root_ui()`, `widgets::Window`, `widgets::Group`, `ui.button()`, `ui.label()`) for all UI. No custom GUI framework.
- Rust edition 2024 (already set in `Cargo.toml`)
- Scene switching is immediate (no transitions)
- Escape key returns to MainMenu from all non-menu scenes
- Quit button calls `std::process::exit(0)`

---

## File Structure

```
src/
  main.rs            -- Scene enum import, dispatch loop, no scene logic
  scenes/
    mod.rs           -- pub enum Scene, pub mod declarations
    menu.rs          -- pub fn update() -> Option<Scene>
    play.rs          -- pub fn update() -> Option<Scene>
    mapgen.rs        -- pub fn update() -> Option<Scene>
    battle.rs        -- pub fn update() -> Option<Scene>
    settings.rs      -- pub fn update() -> Option<Scene>
```

### Task 1: Create the scenes module skeleton (`src/scenes/mod.rs`)

**Files:**
- Create: `src/scenes/mod.rs`

**Interfaces:**
- Produces: `pub enum Scene { MainMenu, Play, Mapgen, Battle, Settings }`
- Produces: `pub mod menu; pub mod play; pub mod mapgen; pub mod battle; pub mod settings;`

- [ ] **Step 1: Create `src/scenes/mod.rs`**

```rust
pub mod battle;
pub mod mapgen;
pub mod menu;
pub mod play;
pub mod settings;

pub enum Scene {
    MainMenu,
    Play,
    Mapgen,
    Battle,
    Settings,
}
```

- [ ] **Step 2: Verify file exists and compiles (module resolution only)**

Run: `cargo check`
Expected: Errors about missing submodules (menu.rs etc. don't exist yet). The `mod.rs` itself should be valid.

---

### Task 2: Create the main menu scene (`src/scenes/menu.rs`)

**Files:**
- Create: `src/scenes/menu.rs`

**Interfaces:**
- Consumes: `Scene` enum from `scenes/mod.rs`
- Produces: `pub fn update() -> Option<Scene>`

- [ ] **Step 1: Create `src/scenes/menu.rs`**

```rust
use crate::scenes::Scene;
use macroquad::prelude::*;

pub fn update() -> Option<Scene> {
    clear_background(DARKGRAY);

    let button_w = 200.0;
    let button_h = 40.0;
    let total_h = 5.0 * button_h + 30.0;
    let pos = vec2(
        (screen_width() - button_w) / 2.0,
        (screen_height() - total_h) / 2.0,
    );

    let mut next_scene = None;

    widgets::Window::new(hash!(), pos, vec2(button_w, total_h))
        .titlebar(false)
        .movable(false)
        .ui(&mut *root_ui(), |ui| {
            ui.label(None, "TicTacToe");
            if ui.button(None, "Play") {
                next_scene = Some(Scene::Play);
            }
            if ui.button(None, "Mapgen") {
                next_scene = Some(Scene::Mapgen);
            }
            if ui.button(None, "Battle") {
                next_scene = Some(Scene::Battle);
            }
            if ui.button(None, "Settings") {
                next_scene = Some(Scene::Settings);
            }
            if ui.button(None, "Quit") {
                std::process::exit(0);
            }
        });

    next_scene
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: menu.rs compiles. Other scene modules still missing (play.rs, etc.).

---

### Task 3: Create placeholder scenes (`play.rs`, `battle.rs`, `settings.rs`)

**Files:**
- Create: `src/scenes/play.rs`
- Create: `src/scenes/battle.rs`
- Create: `src/scenes/settings.rs`

**Interfaces:**
- Consumes: `Scene` enum from `scenes/mod.rs`
- Produces (each file): `pub fn update() -> Option<Scene>`

- [ ] **Step 1: Create `src/scenes/play.rs`**

```rust
use crate::scenes::Scene;
use macroquad::prelude::*;

pub fn update() -> Option<Scene> {
    clear_background(DARKGRAY);

    let label = "PLAY";
    let font_size = 40.0;
    let text_dims = measure_text(label, None, font_size as u16, 1.0);
    draw_text(
        label,
        (screen_width() - text_dims.width) / 2.0,
        screen_height() / 2.0,
        font_size,
        WHITE,
    );

    if is_key_pressed(KeyCode::Escape) {
        return Some(Scene::MainMenu);
    }

    None
}
```

- [ ] **Step 2: Create `src/scenes/battle.rs`** (identical pattern, different label)

```rust
use crate::scenes::Scene;
use macroquad::prelude::*;

pub fn update() -> Option<Scene> {
    clear_background(DARKGRAY);

    let label = "BATTLE";
    let font_size = 40.0;
    let text_dims = measure_text(label, None, font_size as u16, 1.0);
    draw_text(
        label,
        (screen_width() - text_dims.width) / 2.0,
        screen_height() / 2.0,
        font_size,
        WHITE,
    );

    if is_key_pressed(KeyCode::Escape) {
        return Some(Scene::MainMenu);
    }

    None
}
```

- [ ] **Step 3: Create `src/scenes/settings.rs`** (identical pattern, different label)

```rust
use crate::scenes::Scene;
use macroquad::prelude::*;

pub fn update() -> Option<Scene> {
    clear_background(DARKGRAY);

    let label = "SETTINGS";
    let font_size = 40.0;
    let text_dims = measure_text(label, None, font_size as u16, 1.0);
    draw_text(
        label,
        (screen_width() - text_dims.width) / 2.0,
        screen_height() / 2.0,
        font_size,
        WHITE,
    );

    if is_key_pressed(KeyCode::Escape) {
        return Some(Scene::MainMenu);
    }

    None
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check`
Expected: All scene modules compile. Only `main.rs` still has the old code (no dispatch loop yet).

---

### Task 4: Create the Mapgen scene with existing 3D code (`src/scenes/mapgen.rs`)

**Files:**
- Create: `src/scenes/mapgen.rs`

**Interfaces:**
- Consumes: `Scene` enum from `scenes/mod.rs`
- Produces: `pub fn update() -> Option<Scene>`

- [ ] **Step 1: Create `src/scenes/mapgen.rs`** (moves all 3D rendering from main.rs, adds Escape handler)

```rust
use crate::scenes::Scene;
use macroquad::prelude::*;

pub fn update() -> Option<Scene> {
    clear_background(LIGHTGRAY);

    set_camera(&Camera3D {
        position: vec3(-20., 15., 0.),
        up: vec3(0., 1., 0.),
        target: vec3(0., 0., 0.),
        ..Default::default()
    });

    draw_grid(20, 1., BLACK, GRAY);

    draw_cube_wires(vec3(0., 1., -6.), vec3(2., 2., 2.), DARKGREEN);
    draw_cube_wires(vec3(0., 1., 6.), vec3(2., 2., 2.), DARKBLUE);
    draw_cube_wires(vec3(2., 1., 2.), vec3(2., 2., 2.), YELLOW);

    draw_cube(vec3(2., 0., -2.), vec3(0.4, 0.4, 0.4), None, BLACK);

    draw_sphere(vec3(-8., 0., 0.), 1., None, BLUE);

    set_default_camera();
    draw_text("WELCOME TO 3D WORLD", 10.0, 20.0, 30.0, BLACK);

    if is_key_pressed(KeyCode::Escape) {
        return Some(Scene::MainMenu);
    }

    None
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: mapgen.rs compiles. Only `main.rs` is still the old single-scene code.

---

### Task 5: Update `main.rs` with scene dispatch loop

**Files:**
- Modify: `src/main.rs` (replace entire contents)

**Interfaces:**
- Consumes: `Scene` enum and all scene modules from `scenes/`
- Produces: Entry point — macroquad main loop with scene dispatch

- [ ] **Step 1: Replace `src/main.rs`**

```rust
use macroquad::prelude::*;

mod scenes;

use scenes::Scene;

fn window_conf() -> Conf {
    Conf {
        window_title: "TicTacToe".to_owned(),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut scene = Scene::MainMenu;

    loop {
        let next = match scene {
            Scene::MainMenu => scenes::menu::update(),
            Scene::Play => scenes::play::update(),
            Scene::Mapgen => scenes::mapgen::update(),
            Scene::Battle => scenes::battle::update(),
            Scene::Settings => scenes::settings::update(),
        };

        if let Some(new_scene) = next {
            scene = new_scene;
        }

        next_frame().await;
    }
}
```

- [ ] **Step 2: Build and verify**

Run: `cargo build`
Expected: Successful compilation, no warnings.

- [ ] **Step 3: Verify the binary exists**

Run: `Test-Path -LiteralPath "target\debug\tictactoe.exe"`
Expected: `True`

---

### Task 6: Final verification

- [ ] **Step 1: Run `cargo check` for warnings**

Run: `cargo check`
Expected: Zero errors, zero warnings.

- [ ] **Step 2: Run `cargo clippy` (if available)**

Run: `cargo clippy -- -D warnings` (note: if clippy is not installed, skip)
Expected: No clippy warnings.


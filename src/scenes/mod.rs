pub mod battle;
pub mod mapgen;
pub mod menu;
pub mod play;
pub mod settings;

use macroquad::prelude::*;

pub enum Scene {
    MainMenu,
    Play,
    Mapgen,
    Battle,
    Settings,
}

fn placeholder_scene(label: &str) -> Option<Scene> {
    clear_background(DARKGRAY);
    draw_centered_label(label);

    if is_key_pressed(KeyCode::Escape) {
        Some(Scene::MainMenu)
    } else {
        None
    }
}

fn draw_centered_label(label: &str) {
    let font_size = 40.0;
    let text_dims = measure_text(label, None, font_size as u16, 1.0);
    draw_text(
        label,
        (screen_width() - text_dims.width) / 2.0,
        screen_height() / 2.0,
        font_size,
        WHITE,
    );
}

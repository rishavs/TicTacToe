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

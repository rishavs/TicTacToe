use crate::scenes::Scene;
use macroquad::prelude::*;
use macroquad::ui::widgets;
use macroquad::ui::{hash, root_ui};

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
        .ui(&mut root_ui(), |ui| {
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

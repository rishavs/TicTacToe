use macroquad::prelude::*;

mod scenes;

use scenes::Scene;

fn window_conf() -> Conf {
    Conf {
        window_title: "TicTacToe".to_owned(),
        window_width: 1024,
        window_height: 768,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut scene = debug_start_scene().unwrap_or(Scene::MainMenu);
    let mut debug_capture = DebugCapture::from_env();

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

        if let Some(capture) = &mut debug_capture {
            capture.tick();
        }

        next_frame().await;
    }
}

fn debug_start_scene() -> Option<Scene> {
    match std::env::var("TICTACTOE_START_SCENE")
        .ok()?
        .to_lowercase()
        .as_str()
    {
        "mapgen" => Some(Scene::Mapgen),
        "play" => Some(Scene::Play),
        "battle" => Some(Scene::Battle),
        "settings" => Some(Scene::Settings),
        "menu" => Some(Scene::MainMenu),
        _ => None,
    }
}

struct DebugCapture {
    path: String,
    frames_remaining: u32,
}

impl DebugCapture {
    fn from_env() -> Option<Self> {
        Some(Self {
            path: std::env::var("TICTACTOE_SCREENSHOT").ok()?,
            frames_remaining: std::env::var("TICTACTOE_SCREENSHOT_FRAMES")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(3),
        })
    }

    fn tick(&mut self) {
        if self.frames_remaining > 0 {
            self.frames_remaining -= 1;
            return;
        }

        let image = get_screen_data();
        image.export_png(&self.path);
        std::process::exit(0);
    }
}

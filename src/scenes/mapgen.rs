use crate::scenes::Scene;
use macroquad::prelude::*;
use std::env;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::{Mutex, OnceLock};
use std::thread;

mod biome;
mod generate;
mod model;
mod noise;
mod random;
mod render;
mod seed;

#[cfg(test)]
mod tests;

#[cfg(test)]
use self::noise::{fractal_noise_2d, simplex_fractal_noise_2d};
#[cfg(test)]
use generate::generate_map;
use generate::generate_map_with_shallow_sea;
use model::{BiomeCount, Center, Corner, Edge, NoisyEdge, PolyMap};
use noise::{IslandProfile, PERLIN_DEEP_OCEAN_EDGE_BUFFER_CELLS};
#[cfg(test)]
use random::map_random_u32;
use random::{MapRng, map_random_f32, map_random_i32, map_rng};
#[cfg(test)]
use seed::is_seed_char;
use seed::{parse_seed, push_seed_char, random_seed_text};

const MAP_SIZE: f32 = 600.0;
const DEFAULT_SEED_TEXT: &str = "85882-8";
const SIDEBAR_COLOR: Color = Color::new(0.72, 0.72, 0.64, 1.0);
const LAKE_THRESHOLD: f32 = 0.3;
const MIN_ZOOM: f32 = 1.0;
const MAX_ZOOM: f32 = 8.0;
const PAN_SPEED: f32 = 280.0;

static STATE: OnceLock<Mutex<MapgenScene>> = OnceLock::new();

pub fn update() -> Option<Scene> {
    let state = STATE.get_or_init(|| Mutex::new(MapgenScene::new()));
    let mut state = state.lock().unwrap();

    clear_background(WHITE);
    state.update();

    if is_key_pressed(KeyCode::Escape) {
        return Some(Scene::MainMenu);
    }

    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IslandType {
    Perlin,
    Simplex,
}

impl IslandType {
    const ALL: [Self; 2] = [Self::Perlin, Self::Simplex];

    fn label(self) -> &'static str {
        match self {
            Self::Perlin => "Perlin",
            Self::Simplex => "Simplex",
        }
    }

    fn from_debug_env(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "perlin" => Some(Self::Perlin),
            "simplex" => Some(Self::Simplex),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PointType {
    Square,
}

impl PointType {
    fn from_debug_env(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "square" => Some(Self::Square),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ViewMode {
    Biome,
    Slopes,
}

impl ViewMode {
    const ALL: [Self; 2] = [Self::Biome, Self::Slopes];

    fn label(self) -> &'static str {
        match self {
            Self::Biome => "Biomes",
            Self::Slopes => "2D slopes",
        }
    }

    fn from_debug_env(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "biome" | "biomes" => Some(Self::Biome),
            "slopes" | "2d slopes" | "2d" => Some(Self::Slopes),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShallowSeaSize {
    Narrow,
    Normal,
    Wide,
}

impl ShallowSeaSize {
    const ALL: [Self; 3] = [Self::Narrow, Self::Normal, Self::Wide];

    fn label(self) -> &'static str {
        match self {
            Self::Narrow => "Narrow",
            Self::Normal => "Normal",
            Self::Wide => "Wide",
        }
    }

    fn from_debug_env(value: &str) -> Option<Self> {
        match value
            .trim()
            .to_lowercase()
            .replace([' ', '_', '-'], "")
            .as_str()
        {
            "narrow" => Some(Self::Narrow),
            "normal" => Some(Self::Normal),
            "wide" => Some(Self::Wide),
            _ => None,
        }
    }

    fn guaranteed_shallow_distance(self) -> i32 {
        match self {
            Self::Narrow => 1,
            Self::Normal => 2,
            Self::Wide => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BayRounding {
    Light,
    Normal,
    Strong,
}

impl BayRounding {
    const ALL: [Self; 3] = [Self::Light, Self::Normal, Self::Strong];

    fn label(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Normal => "Normal",
            Self::Strong => "Strong",
        }
    }

    fn from_debug_env(value: &str) -> Option<Self> {
        match value
            .trim()
            .to_lowercase()
            .replace([' ', '_', '-'], "")
            .as_str()
        {
            "light" => Some(Self::Light),
            "normal" => Some(Self::Normal),
            "strong" => Some(Self::Strong),
            _ => None,
        }
    }

    fn base_closing_radius(self, shallow_sea_size: ShallowSeaSize) -> usize {
        match self {
            Self::Light => 1,
            Self::Normal => 2,
            Self::Strong => match shallow_sea_size {
                ShallowSeaSize::Narrow => 4,
                ShallowSeaSize::Normal => 5,
                ShallowSeaSize::Wide => 6,
            },
        }
    }

    fn concavity_closing_radius(
        self,
        shallow_sea_size: ShallowSeaSize,
        grid_width: usize,
    ) -> usize {
        let base = self.base_closing_radius(shallow_sea_size);
        if base == 0 {
            return 0;
        }
        ((base as f32 * grid_width as f32 / 63.0).round() as usize).max(base)
    }

    fn concavity_closing_max_distance(
        self,
        shallow_sea_size: ShallowSeaSize,
        grid_width: usize,
    ) -> i32 {
        let radius = self.concavity_closing_radius(shallow_sea_size, grid_width);
        shallow_sea_size.guaranteed_shallow_distance() + radius as i32 + 2
    }
}

const POINT_COUNTS: [usize; 4] = [16000, 32000, 64000, 128000];
const DEFAULT_ISLAND_TYPE: IslandType = IslandType::Perlin;
const DEFAULT_POINT_TYPE: PointType = PointType::Square;
const DEFAULT_POINT_COUNT: usize = 16000;
const DEFAULT_SHALLOW_SEA_SIZE: ShallowSeaSize = ShallowSeaSize::Wide;
const DEFAULT_BAY_ROUNDING: BayRounding = BayRounding::Light;
const DEFAULT_VIEW_MODE: ViewMode = ViewMode::Biome;

fn island_button_x_positions() -> &'static [f32] {
    &[0.0, 68.0]
}

struct MapgenLayout {
    left_panel_rect: Rect,
    map_area_rect: Rect,
    map_rect: Rect,
    right_panel_rect: Rect,
}

fn mapgen_layout(width: f32, height: f32) -> MapgenLayout {
    let panel_w = (width * 0.2).clamp(248.0, 260.0);
    let map_w = (width - panel_w * 2.0).max(1.0);
    let map_size = map_w.min(height);
    let map_x = panel_w + ((map_w - map_size) * 0.5).max(0.0);
    let map_y = ((height - map_size) * 0.5).max(0.0);
    MapgenLayout {
        left_panel_rect: Rect::new(0.0, 0.0, panel_w, height),
        map_area_rect: Rect::new(panel_w, 0.0, map_w, height),
        map_rect: Rect::new(map_x, map_y, map_size, map_size),
        right_panel_rect: Rect::new(panel_w + map_w, 0.0, panel_w, height),
    }
}

fn seed_field_rect(panel: Rect) -> Rect {
    Rect::new(panel.x + 79.0, 28.0, 88.0, 24.0)
}

fn map_source_rect(pan: Vec2, zoom: f32) -> Rect {
    let zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
    let source_size = MAP_SIZE / zoom;
    let max_origin = (MAP_SIZE - source_size).max(0.0);
    let center = vec2(MAP_SIZE / 2.0, MAP_SIZE / 2.0) + clamp_pan(pan, zoom);
    let origin = vec2(center.x - source_size / 2.0, center.y - source_size / 2.0);
    Rect::new(
        origin.x.clamp(0.0, max_origin),
        origin.y.clamp(0.0, max_origin),
        source_size,
        source_size,
    )
}

fn clamp_pan(pan: Vec2, zoom: f32) -> Vec2 {
    let half_range = (MAP_SIZE - MAP_SIZE / zoom.clamp(MIN_ZOOM, MAX_ZOOM)) / 2.0;
    vec2(
        pan.x.clamp(-half_range, half_range),
        pan.y.clamp(-half_range, half_range),
    )
}

struct MapgenScene {
    seed_text: String,
    seed_edit_text: String,
    seed_input_active: bool,
    seed_replace_on_type: bool,
    island_type: IslandType,
    point_type: PointType,
    point_count: usize,
    shallow_sea_size: ShallowSeaSize,
    bay_rounding: BayRounding,
    view_mode: ViewMode,
    map: Option<PolyMap>,
    generation: Option<GenerationJob>,
    pan: Vec2,
    zoom: f32,
}

struct GenerationJob {
    receiver: Receiver<PolyMap>,
}

#[derive(Clone)]
struct GenerationRequest {
    seed_text: String,
    island_type: IslandType,
    point_type: PointType,
    point_count: usize,
    shallow_sea_size: ShallowSeaSize,
    bay_rounding: BayRounding,
}

impl MapgenScene {
    fn new() -> Self {
        let seed_text =
            env::var("TICTACTOE_MAPGEN_SEED").unwrap_or_else(|_| DEFAULT_SEED_TEXT.to_string());
        let island_type = env::var("TICTACTOE_MAPGEN_ISLAND")
            .ok()
            .and_then(|value| IslandType::from_debug_env(&value))
            .unwrap_or(DEFAULT_ISLAND_TYPE);
        let point_type = env::var("TICTACTOE_MAPGEN_POINTS")
            .ok()
            .and_then(|value| PointType::from_debug_env(&value))
            .unwrap_or(DEFAULT_POINT_TYPE);
        let point_count = env::var("TICTACTOE_MAPGEN_COUNT")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| POINT_COUNTS.contains(value))
            .unwrap_or(DEFAULT_POINT_COUNT);
        let shallow_sea_size = env::var("TICTACTOE_MAPGEN_SHALLOW_SEA")
            .ok()
            .and_then(|value| ShallowSeaSize::from_debug_env(&value))
            .unwrap_or(DEFAULT_SHALLOW_SEA_SIZE);
        let bay_rounding = env::var("TICTACTOE_MAPGEN_BAY_ROUNDING")
            .ok()
            .and_then(|value| BayRounding::from_debug_env(&value))
            .unwrap_or(DEFAULT_BAY_ROUNDING);
        let view_mode = env::var("TICTACTOE_MAPGEN_VIEW")
            .ok()
            .and_then(|value| ViewMode::from_debug_env(&value))
            .unwrap_or(DEFAULT_VIEW_MODE);
        let mut scene = Self {
            seed_edit_text: seed_text.clone(),
            seed_text,
            seed_input_active: false,
            seed_replace_on_type: false,
            island_type,
            point_type,
            point_count,
            shallow_sea_size,
            bay_rounding,
            view_mode,
            map: None,
            generation: None,
            pan: Vec2::ZERO,
            zoom: MIN_ZOOM,
        };
        scene.regenerate();
        scene
    }

    fn update(&mut self) {
        self.poll_generation();
        let layout = mapgen_layout(screen_width(), screen_height());
        self.handle_seed_input(layout.left_panel_rect);
        if !self.seed_input_active {
            self.handle_viewport_keys();
        }
        let source_rect = map_source_rect(self.pan, self.zoom);

        render::draw(self, layout, source_rect);
    }

    fn handle_seed_input(&mut self, panel: Rect) {
        let field = seed_field_rect(panel);
        let (mouse_x, mouse_y) = mouse_position();
        let mouse = vec2(mouse_x, mouse_y);

        if is_mouse_button_pressed(MouseButton::Left) {
            if field.contains(mouse) {
                self.seed_input_active = true;
                self.seed_edit_text = self.seed_text.clone();
                self.seed_replace_on_type = true;
            } else {
                self.seed_input_active = false;
                self.seed_edit_text = self.seed_text.clone();
                self.seed_replace_on_type = false;
            }
        }

        if !self.seed_input_active {
            return;
        }

        while let Some(ch) = get_char_pressed() {
            push_seed_char(&mut self.seed_edit_text, &mut self.seed_replace_on_type, ch);
        }

        if is_key_pressed(KeyCode::Backspace) {
            self.seed_replace_on_type = false;
            self.seed_edit_text.pop();
        }
        if is_key_pressed(KeyCode::Delete) {
            self.seed_replace_on_type = false;
            self.seed_edit_text.clear();
        }
        if is_key_pressed(KeyCode::Escape) {
            self.seed_input_active = false;
            self.seed_edit_text = self.seed_text.clone();
            self.seed_replace_on_type = false;
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
            self.seed_text = self.seed_edit_text.trim().to_string();
            self.seed_input_active = false;
            self.seed_replace_on_type = false;
            self.regenerate();
        }
    }

    fn handle_viewport_keys(&mut self) {
        let dt = get_frame_time();
        let pan_step = PAN_SPEED * dt / self.zoom;
        let mut delta = Vec2::ZERO;

        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            delta.x -= pan_step;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            delta.x += pan_step;
        }
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            delta.y -= pan_step;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            delta.y += pan_step;
        }

        self.pan += delta;

        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            self.zoom = (self.zoom * 1.25).min(MAX_ZOOM);
        }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            self.zoom = (self.zoom / 1.25).max(MIN_ZOOM);
        }
        if is_key_pressed(KeyCode::Home) || is_key_pressed(KeyCode::Key0) {
            self.zoom = MIN_ZOOM;
            self.pan = Vec2::ZERO;
        }

        self.pan = clamp_pan(self.pan, self.zoom);
    }

    fn regenerate(&mut self) {
        if self.seed_text.trim().is_empty() {
            self.seed_text = random_seed_text();
        }
        let request = GenerationRequest {
            seed_text: self.seed_text.clone(),
            island_type: self.island_type,
            point_type: self.point_type,
            point_count: self.point_count,
            shallow_sea_size: self.shallow_sea_size,
            bay_rounding: self.bay_rounding,
        };
        self.start_generation(request);
    }

    fn apply_random_seed_text(&mut self, seed_text: String) {
        self.seed_text = seed_text;
        self.seed_edit_text = self.seed_text.clone();
        self.seed_input_active = false;
        self.seed_replace_on_type = false;
    }

    fn start_generation(&mut self, request: GenerationRequest) {
        let (sender, receiver) = mpsc::channel();
        let worker_request = request.clone();
        thread::spawn(move || {
            let map = generate_map_with_shallow_sea(
                &worker_request.seed_text,
                worker_request.island_type,
                worker_request.point_type,
                worker_request.point_count,
                worker_request.shallow_sea_size,
                worker_request.bay_rounding,
            );
            let _ = sender.send(map);
        });
        self.generation = Some(GenerationJob { receiver });
    }

    fn poll_generation(&mut self) {
        let Some(result) = self.generation.as_ref().map(|job| job.receiver.try_recv()) else {
            return;
        };
        match result {
            Ok(map) => {
                self.map = Some(map);
                self.generation = None;
                self.pan = clamp_pan(self.pan, self.zoom);
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.generation = None;
            }
        }
    }
}

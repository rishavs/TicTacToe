use crate::scenes::Scene;
use macroquad::prelude::*;
use macroquad::ui::widgets;
use macroquad::ui::{hash, root_ui};
use std::collections::{HashMap, VecDeque};
use std::env;
use std::f32::consts::PI;
use std::sync::{Mutex, OnceLock};

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
    Radial,
    Perlin,
    Simplex,
}

impl IslandType {
    const ALL: [Self; 3] = [Self::Radial, Self::Perlin, Self::Simplex];

    fn label(self) -> &'static str {
        match self {
            Self::Radial => "Radial",
            Self::Perlin => "Perlin",
            Self::Simplex => "Simplex",
        }
    }

    fn from_debug_env(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "radial" => Some(Self::Radial),
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
    const ALL: [Self; 1] = [Self::Square];

    fn label(self) -> &'static str {
        match self {
            Self::Square => "Square",
        }
    }

    fn needs_more_randomness(self) -> bool {
        true
    }

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

const POINT_COUNTS: [usize; 4] = [4000, 8000, 16000, 32000];
const DEFAULT_ISLAND_TYPE: IslandType = IslandType::Perlin;
const DEFAULT_POINT_TYPE: PointType = PointType::Square;
const DEFAULT_POINT_COUNT: usize = 4000;
const DEFAULT_VIEW_MODE: ViewMode = ViewMode::Biome;

fn island_button_x_positions() -> &'static [f32] {
    &[0.0, 58.0, 124.0]
}

struct MapgenLayout {
    map_rect: Rect,
    sidebar_rect: Rect,
}

fn mapgen_layout(width: f32, height: f32) -> MapgenLayout {
    let sidebar_w = (width * 0.25).clamp(190.0, 260.0);
    let map_w = (width - sidebar_w).max(1.0);
    let map_size = map_w.min(height);
    MapgenLayout {
        map_rect: Rect::new(0.0, 0.0, map_size, map_size),
        sidebar_rect: Rect::new(map_w, 0.0, sidebar_w, height),
    }
}

fn seed_field_rect(sidebar: Rect) -> Rect {
    Rect::new(sidebar.x + 79.0, 28.0, 66.0, 24.0)
}

fn is_seed_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'
}

fn push_seed_char(seed_text: &mut String, replace_on_type: &mut bool, ch: char) -> bool {
    if !is_seed_char(ch) || seed_text.len() >= 16 {
        return false;
    }
    if *replace_on_type {
        seed_text.clear();
        *replace_on_type = false;
    }
    seed_text.push(ch);
    true
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
    view_mode: ViewMode,
    map: PolyMap,
    pan: Vec2,
    zoom: f32,
    status: String,
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
        let view_mode = env::var("TICTACTOE_MAPGEN_VIEW")
            .ok()
            .and_then(|value| ViewMode::from_debug_env(&value))
            .unwrap_or(DEFAULT_VIEW_MODE);
        let map = PolyMap::generate(&seed_text, island_type, point_type, point_count);
        Self {
            seed_edit_text: seed_text.clone(),
            seed_text,
            seed_input_active: false,
            seed_replace_on_type: false,
            island_type,
            point_type,
            point_count,
            view_mode,
            map,
            pan: Vec2::ZERO,
            zoom: MIN_ZOOM,
            status: String::new(),
        }
    }

    fn update(&mut self) {
        let layout = mapgen_layout(screen_width(), screen_height());
        self.handle_seed_input(layout.sidebar_rect);
        if !self.seed_input_active {
            self.handle_viewport_keys();
        }
        let source_rect = map_source_rect(self.pan, self.zoom);

        self.draw_map(layout.map_rect, source_rect);
        draw_rectangle(
            layout.sidebar_rect.x,
            layout.sidebar_rect.y,
            layout.sidebar_rect.w,
            layout.sidebar_rect.h,
            SIDEBAR_COLOR,
        );
        self.draw_histograms(layout.sidebar_rect);
        self.draw_controls(layout.sidebar_rect);
        self.draw_seed_field(layout.sidebar_rect);
        self.draw_footer(layout.sidebar_rect);
    }

    fn draw_map(&self, map_rect: Rect, source_rect: Rect) {
        draw_rectangle(
            map_rect.x,
            map_rect.y,
            map_rect.w,
            map_rect.h,
            color_from_u32(0x44447a),
        );

        self.draw_polygons(map_rect, source_rect);
        self.draw_edges(map_rect, source_rect);
    }

    fn draw_polygons(&self, map_rect: Rect, source_rect: Rect) {
        for center in &self.map.centers {
            if !center_visible(center, source_rect) {
                continue;
            }
            for &neighbor in &center.neighbors {
                let Some(edge_id) = self.map.lookup_edge_from_center(center.index, neighbor) else {
                    continue;
                };
                let noisy_edge = self.map.noisy_edges.get(edge_id);
                let Some((path0, path1)) =
                    noisy_edge.and_then(|edge| edge.path0.as_ref().zip(edge.path1.as_ref()))
                else {
                    continue;
                };
                let color = color_from_u32(self.map.triangle_color(
                    self.view_mode,
                    center.index,
                    edge_id,
                ));
                draw_path_wedge(center.point, path0, map_rect, source_rect, color);
                draw_path_wedge(center.point, path1, map_rect, source_rect, color);
            }
        }
    }

    fn draw_edges(&self, map_rect: Rect, source_rect: Rect) {
        for center in &self.map.centers {
            for &neighbor in &center.neighbors {
                let Some(edge_id) = self.map.lookup_edge_from_center(center.index, neighbor) else {
                    continue;
                };
                let edge = &self.map.edges[edge_id];
                let (Some(_), Some(_)) = (edge.v0, edge.v1) else {
                    continue;
                };
                let a = center;
                let b = &self.map.centers[neighbor];

                let noisy_edge = self.map.noisy_edges.get(edge.index);
                let Some((path0, path1)) =
                    noisy_edge.and_then(|edge| edge.path0.as_ref().zip(edge.path1.as_ref()))
                else {
                    continue;
                };

                if a.ocean != b.ocean {
                    draw_noisy_edge_path(
                        path0,
                        path1,
                        map_rect,
                        source_rect,
                        2.0,
                        color_from_u32(0x33335a),
                    );
                } else if a.water != b.water && a.biome != "ICE" && b.biome != "ICE" {
                    draw_noisy_edge_path(
                        path0,
                        path1,
                        map_rect,
                        source_rect,
                        1.0,
                        color_from_u32(0x225588),
                    );
                } else if a.water || b.water {
                    continue;
                } else if edge.river > 0 {
                    draw_noisy_edge_path(
                        path0,
                        path1,
                        map_rect,
                        source_rect,
                        (edge.river as f32).sqrt(),
                        color_from_u32(0x225588),
                    );
                }
            }
        }
    }

    fn draw_histograms(&self, sidebar: Rect) {
        let x = sidebar.x + 25.0;
        let y = 230.0;
        let width = (sidebar.w - 50.0).max(120.0);

        draw_text("Distribution:", sidebar.x + 50.0, y, 18.0, BLACK);
        draw_distribution(x, y + 12.0, width, 18.0, &self.map.land_histogram());
        draw_distribution(x, y + 36.0, width, 18.0, &self.map.biome_histogram());
        draw_histogram(x, y + 88.0, width, 28.0, &self.map.elevation_histogram());
        draw_histogram(x, y + 126.0, width, 18.0, &self.map.moisture_histogram());
    }

    fn draw_controls(&mut self, sidebar: Rect) {
        let x = sidebar.x + 16.0;
        let mut needs_regenerate = false;

        widgets::Window::new(
            hash!("mapgen_island_shape"),
            vec2(x, 4.0),
            vec2(224.0, 86.0),
        )
        .titlebar(false)
        .movable(false)
        .ui(&mut root_ui(), |ui| {
            ui.label(Some(vec2(52.0, 2.0)), "Island Shape:");
            ui.label(Some(vec2(4.0, 26.0)), "Shape #");
            if ui.button(Some(vec2(133.0, 22.0)), "Random") {
                self.seed_text = random_seed_text();
                self.seed_edit_text = self.seed_text.clone();
                self.seed_input_active = false;
                self.seed_replace_on_type = false;
                needs_regenerate = true;
            }

            for (index, island_type) in IslandType::ALL.into_iter().enumerate() {
                let button_x = island_button_x_positions()[index];
                let label = selected_label(island_type.label(), island_type == self.island_type);
                if ui.button(Some(vec2(button_x, 54.0)), label.as_str()) {
                    self.island_type = island_type;
                    needs_regenerate = true;
                }
            }
        });

        widgets::Window::new(
            hash!("mapgen_point_selection"),
            vec2(x, 104.0),
            vec2(232.0, 86.0),
        )
        .titlebar(false)
        .movable(false)
        .ui(&mut root_ui(), |ui| {
            ui.label(Some(vec2(54.0, 2.0)), "Point Selection:");
            for (index, point_type) in PointType::ALL.into_iter().enumerate() {
                let button_x = [0.0][index];
                let label = selected_label(point_type.label(), point_type == self.point_type);
                if ui.button(Some(vec2(button_x, 28.0)), label.as_str()) {
                    self.point_type = point_type;
                    needs_regenerate = true;
                }
            }
            for (index, count) in POINT_COUNTS.into_iter().enumerate() {
                let button_x = [0.0, 56.0, 112.0, 168.0][index];
                let label = selected_label(&count.to_string(), count == self.point_count);
                if ui.button(Some(vec2(button_x, 58.0)), label.as_str()) {
                    self.point_count = count;
                    needs_regenerate = true;
                }
            }
        });

        widgets::Window::new(
            hash!("mapgen_view"),
            vec2(x + 12.0, 390.0),
            vec2(200.0, 62.0),
        )
        .titlebar(false)
        .movable(false)
        .ui(&mut root_ui(), |ui| {
            ui.label(Some(vec2(82.0, 2.0)), "View:");
            for (index, mode) in ViewMode::ALL.into_iter().enumerate() {
                let col = index as f32;
                let label = selected_label(mode.label(), mode == self.view_mode);
                if ui.button(Some(vec2(col * 95.0, 28.0)), label.as_str()) {
                    self.view_mode = mode;
                    self.status = format!("View: {}", mode.label());
                }
            }
        });

        if needs_regenerate {
            self.regenerate();
        }
    }

    fn handle_seed_input(&mut self, sidebar: Rect) {
        let field = seed_field_rect(sidebar);
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

    fn draw_seed_field(&self, sidebar: Rect) {
        let field = seed_field_rect(sidebar);
        let border = if self.seed_input_active { BLACK } else { GRAY };
        let value = if self.seed_input_active {
            self.seed_edit_text.as_str()
        } else {
            self.seed_text.as_str()
        };
        let text = if self.seed_input_active && (get_time() * 2.0) as i32 % 2 == 0 {
            format!("{}|", value)
        } else {
            value.to_string()
        };

        draw_rectangle(
            field.x,
            field.y,
            field.w,
            field.h,
            Color::new(0.94, 0.94, 0.82, 1.0),
        );
        draw_rectangle_lines(field.x, field.y, field.w, field.h, 1.0, border);
        draw_text(&text, field.x + 4.0, field.y + 17.0, 18.0, BLACK);
    }

    fn draw_footer(&self, sidebar: Rect) {
        draw_text(
            format!("Zoom {:.1}x", self.zoom),
            sidebar.x + 16.0,
            724.0,
            16.0,
            BLACK,
        );
        draw_text(&self.status, sidebar.x + 16.0, 744.0, 16.0, BLACK);
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
        self.map = PolyMap::generate(
            &self.seed_text,
            self.island_type,
            self.point_type,
            self.point_count,
        );
        self.pan = clamp_pan(self.pan, self.zoom);
        self.status = format!(
            "{} / {} / {} sites",
            self.island_type.label(),
            self.point_type.label(),
            self.map.centers.len()
        );
    }
}

#[derive(Clone)]
struct Center {
    index: usize,
    point: Vec2,
    water: bool,
    ocean: bool,
    coast: bool,
    border: bool,
    biome: &'static str,
    elevation: f32,
    moisture: f32,
    neighbors: Vec<usize>,
    borders: Vec<usize>,
    corners: Vec<usize>,
}

#[derive(Clone)]
struct Corner {
    index: usize,
    point: Vec2,
    ocean: bool,
    water: bool,
    coast: bool,
    border: bool,
    elevation: f32,
    moisture: f32,
    touches: Vec<usize>,
    protrudes: Vec<usize>,
    adjacent: Vec<usize>,
    river: i32,
    downslope: usize,
    watershed: usize,
    watershed_size: i32,
}

#[derive(Clone)]
struct Edge {
    index: usize,
    d0: Option<usize>,
    d1: Option<usize>,
    v0: Option<usize>,
    v1: Option<usize>,
    midpoint: Vec2,
    river: i32,
}

#[derive(Clone, Default)]
struct NoisyEdge {
    path0: Option<Vec<Vec2>>,
    path1: Option<Vec<Vec2>>,
}

struct PolyMap {
    point_type: PointType,
    map_random: PmPrng,
    island_shape: IslandProfile,
    centers: Vec<Center>,
    corners: Vec<Corner>,
    edges: Vec<Edge>,
    noisy_edges: Vec<NoisyEdge>,
    center_watersheds: Vec<Option<usize>>,
    edge_by_corners: HashMap<(usize, usize), usize>,
}

impl PolyMap {
    fn generate(
        seed_text: &str,
        island_type: IslandType,
        point_type: PointType,
        point_count: usize,
    ) -> Self {
        let (shape_seed, variant) = parse_seed(seed_text);
        let island_shape = IslandProfile::new(island_type, shape_seed);
        let points = select_points(point_type, point_count, shape_seed);
        let mut map = Self::from_points(points, point_type, point_count, variant, island_shape);
        map.assign_corner_elevations();
        map.assign_ocean_coast_and_land();
        map.redistribute_elevations();
        map.assign_polygon_elevations();
        map.calculate_downslopes();
        map.calculate_watersheds();
        map.create_rivers();
        map.assign_corner_moisture();
        map.redistribute_moisture();
        map.assign_polygon_moisture();
        map.assign_biomes();
        map.create_center_watersheds();
        map.build_noisy_edges();
        map
    }

    fn from_points(
        points: Vec<Vec2>,
        point_type: PointType,
        point_count: usize,
        variant: u32,
        island_shape: IslandProfile,
    ) -> Self {
        let regions = build_regions(&points, point_type, point_count);
        let mut centers: Vec<Center> = points
            .iter()
            .enumerate()
            .map(|(index, &point)| Center {
                index,
                point,
                water: false,
                ocean: false,
                coast: false,
                border: false,
                biome: "OCEAN",
                elevation: 0.0,
                moisture: 0.0,
                neighbors: Vec::new(),
                borders: Vec::new(),
                corners: Vec::new(),
            })
            .collect();
        let mut corners: Vec<Corner> = Vec::new();
        let mut edges: Vec<Edge> = Vec::new();
        let mut corner_lookup: HashMap<(i32, i32), usize> = HashMap::new();
        let mut edge_by_corners: HashMap<(usize, usize), usize> = HashMap::new();

        for (center_id, region) in regions.iter().enumerate() {
            if region.len() < 3 {
                continue;
            }
            let mut corner_ids = Vec::with_capacity(region.len());
            for &point in region {
                let corner_id = make_corner(&mut corners, &mut corner_lookup, point);
                push_unique(&mut corners[corner_id].touches, center_id);
                corner_ids.push(corner_id);
            }
            centers[center_id].corners = corner_ids.clone();

            for i in 0..corner_ids.len() {
                let v0 = corner_ids[i];
                let v1 = corner_ids[(i + 1) % corner_ids.len()];
                if v0 == v1 {
                    continue;
                }
                let key = sorted_pair(v0, v1);
                let edge_id = if let Some(&edge_id) = edge_by_corners.get(&key) {
                    if edges[edge_id].d0 != Some(center_id) && edges[edge_id].d1.is_none() {
                        edges[edge_id].d1 = Some(center_id);
                    }
                    edge_id
                } else {
                    let edge_id = edges.len();
                    edge_by_corners.insert(key, edge_id);
                    edges.push(Edge {
                        index: edge_id,
                        d0: Some(center_id),
                        d1: None,
                        v0: Some(v0),
                        v1: Some(v1),
                        midpoint: (corners[v0].point + corners[v1].point) * 0.5,
                        river: 0,
                    });
                    push_unique(&mut corners[v0].adjacent, v1);
                    push_unique(&mut corners[v1].adjacent, v0);
                    push_unique(&mut corners[v0].protrudes, edge_id);
                    push_unique(&mut corners[v1].protrudes, edge_id);
                    edge_id
                };
                push_unique(&mut centers[center_id].borders, edge_id);
            }
        }

        for edge in &edges {
            if let (Some(d0), Some(d1)) = (edge.d0, edge.d1) {
                push_unique(&mut centers[d0].neighbors, d1);
                push_unique(&mut centers[d1].neighbors, d0);
            }
        }

        improve_corners(&mut centers, &mut corners, &mut edges);

        Self {
            point_type,
            map_random: PmPrng::new_raw(variant),
            island_shape,
            centers,
            corners,
            edges,
            noisy_edges: Vec::new(),
            center_watersheds: Vec::new(),
            edge_by_corners,
        }
    }

    fn assign_corner_elevations(&mut self) {
        let mut queue = VecDeque::new();
        for corner in &mut self.corners {
            corner.water = !self.island_shape.inside(vec2(
                2.0 * (corner.point.x / MAP_SIZE - 0.5),
                2.0 * (corner.point.y / MAP_SIZE - 0.5),
            ));
            if corner.border {
                corner.elevation = 0.0;
                queue.push_back(corner.index);
            } else {
                corner.elevation = f32::INFINITY;
            }
        }

        while let Some(q) = queue.pop_front() {
            let adjacent = self.corners[q].adjacent.clone();
            for s in adjacent {
                let mut new_elevation = 0.01 + self.corners[q].elevation;
                if !self.corners[q].water && !self.corners[s].water {
                    new_elevation += 1.0;
                    if self.point_type.needs_more_randomness() {
                        new_elevation += self.map_random.next_double();
                    }
                }
                if new_elevation < self.corners[s].elevation {
                    self.corners[s].elevation = new_elevation;
                    queue.push_back(s);
                }
            }
        }
    }

    fn assign_ocean_coast_and_land(&mut self) {
        let mut queue = VecDeque::new();
        for center in &mut self.centers {
            let mut num_water = 0usize;
            for &corner_id in &center.corners {
                let corner = &mut self.corners[corner_id];
                if corner.border {
                    center.border = true;
                    center.ocean = true;
                    corner.water = true;
                    queue.push_back(center.index);
                }
                if corner.water {
                    num_water += 1;
                }
            }
            center.water = center.ocean
                || (!center.corners.is_empty()
                    && num_water as f32 >= center.corners.len() as f32 * LAKE_THRESHOLD);
        }

        while let Some(p) = queue.pop_front() {
            let neighbors = self.centers[p].neighbors.clone();
            for r in neighbors {
                if self.centers[r].water && !self.centers[r].ocean {
                    self.centers[r].ocean = true;
                    queue.push_back(r);
                }
            }
        }

        for p in 0..self.centers.len() {
            let mut num_ocean = 0;
            let mut num_land = 0;
            for &r in &self.centers[p].neighbors {
                if self.centers[r].ocean {
                    num_ocean += 1;
                }
                if !self.centers[r].water {
                    num_land += 1;
                }
            }
            self.centers[p].coast = num_ocean > 0 && num_land > 0;
        }

        for q in 0..self.corners.len() {
            let mut num_ocean = 0;
            let mut num_land = 0;
            for &p in &self.corners[q].touches {
                if self.centers[p].ocean {
                    num_ocean += 1;
                }
                if !self.centers[p].water {
                    num_land += 1;
                }
            }
            let touches = self.corners[q].touches.len();
            self.corners[q].ocean = touches > 0 && num_ocean == touches;
            self.corners[q].coast = num_ocean > 0 && num_land > 0;
            self.corners[q].water =
                self.corners[q].border || ((num_land != touches) && !self.corners[q].coast);
        }
    }

    fn redistribute_elevations(&mut self) {
        let mut locations: Vec<usize> = self
            .corners
            .iter()
            .filter(|corner| !corner.ocean && !corner.coast)
            .map(|corner| corner.index)
            .collect();
        locations.sort_by(|&a, &b| {
            self.corners[a]
                .elevation
                .total_cmp(&self.corners[b].elevation)
        });
        if locations.len() > 1 {
            let scale_factor = 1.1_f32;
            for (i, &corner_id) in locations.iter().enumerate() {
                let y = i as f32 / (locations.len() - 1) as f32;
                let x = scale_factor.sqrt() - (scale_factor * (1.0 - y)).sqrt();
                self.corners[corner_id].elevation = x.min(1.0);
            }
        }
        for corner in &mut self.corners {
            if corner.ocean || corner.coast {
                corner.elevation = 0.0;
            }
        }
    }

    fn assign_polygon_elevations(&mut self) {
        for center in &mut self.centers {
            center.elevation = average(
                center
                    .corners
                    .iter()
                    .map(|&corner_id| self.corners[corner_id].elevation),
            );
        }
    }

    fn calculate_downslopes(&mut self) {
        for q in 0..self.corners.len() {
            let mut r = q;
            for &s in &self.corners[q].adjacent {
                if self.corners[s].elevation <= self.corners[r].elevation {
                    r = s;
                }
            }
            self.corners[q].downslope = r;
        }
    }

    fn calculate_watersheds(&mut self) {
        for q in 0..self.corners.len() {
            self.corners[q].watershed = q;
            if !self.corners[q].ocean && !self.corners[q].coast {
                self.corners[q].watershed = self.corners[q].downslope;
            }
            self.corners[q].watershed_size = 0;
        }

        for _ in 0..100 {
            let mut changed = false;
            for q in 0..self.corners.len() {
                let watershed = self.corners[q].watershed;
                if !self.corners[q].ocean
                    && !self.corners[q].coast
                    && !self.corners[watershed].coast
                {
                    let r = self.corners[self.corners[q].downslope].watershed;
                    if !self.corners[r].ocean {
                        self.corners[q].watershed = r;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }

        for q in 0..self.corners.len() {
            let watershed = self.corners[q].watershed;
            self.corners[watershed].watershed_size += 1;
        }
    }

    fn create_rivers(&mut self) {
        for _ in 0..(MAP_SIZE as usize / 2) {
            if self.corners.is_empty() {
                return;
            }
            let mut q = self
                .map_random
                .next_int_range(0, self.corners.len() as i32 - 1) as usize;
            if self.corners[q].ocean
                || self.corners[q].elevation < 0.3
                || self.corners[q].elevation > 0.9
            {
                continue;
            }
            let mut visited = vec![false; self.corners.len()];
            while !self.corners[q].coast {
                if visited[q] {
                    break;
                }
                visited[q] = true;
                let downslope = self.corners[q].downslope;
                if q == downslope {
                    break;
                }
                if let Some(edge_id) = self.lookup_edge_from_corner(q, downslope) {
                    self.edges[edge_id].river += 1;
                    self.corners[q].river += 1;
                    self.corners[downslope].river += 1;
                }
                q = downslope;
            }
        }
    }

    fn lookup_edge_from_corner(&self, q: usize, s: usize) -> Option<usize> {
        self.edge_by_corners.get(&sorted_pair(q, s)).copied()
    }

    fn lookup_edge_from_center(&self, p: usize, r: usize) -> Option<usize> {
        self.centers[p].borders.iter().copied().find(|&edge_id| {
            let edge = &self.edges[edge_id];
            (edge.d0 == Some(p) && edge.d1 == Some(r)) || (edge.d0 == Some(r) && edge.d1 == Some(p))
        })
    }

    fn assign_corner_moisture(&mut self) {
        let mut queue = VecDeque::new();
        for q in 0..self.corners.len() {
            if (self.corners[q].water || self.corners[q].river > 0) && !self.corners[q].ocean {
                self.corners[q].moisture = if self.corners[q].river > 0 {
                    (0.2 * self.corners[q].river as f32).min(3.0)
                } else {
                    1.0
                };
                queue.push_back(q);
            } else {
                self.corners[q].moisture = 0.0;
            }
        }

        while let Some(q) = queue.pop_front() {
            let adjacent = self.corners[q].adjacent.clone();
            for r in adjacent {
                let new_moisture = self.corners[q].moisture * 0.9;
                if new_moisture > self.corners[r].moisture {
                    self.corners[r].moisture = new_moisture;
                    queue.push_back(r);
                }
            }
        }

        for corner in &mut self.corners {
            if corner.ocean || corner.coast {
                corner.moisture = 1.0;
            }
        }
    }

    fn redistribute_moisture(&mut self) {
        let mut locations: Vec<usize> = self
            .corners
            .iter()
            .filter(|corner| !corner.ocean && !corner.coast)
            .map(|corner| corner.index)
            .collect();
        locations.sort_by(|&a, &b| {
            self.corners[a]
                .moisture
                .total_cmp(&self.corners[b].moisture)
        });
        if locations.len() > 1 {
            for (i, &corner_id) in locations.iter().enumerate() {
                self.corners[corner_id].moisture = i as f32 / (locations.len() - 1) as f32;
            }
        }
    }

    fn assign_polygon_moisture(&mut self) {
        for center in &mut self.centers {
            center.moisture = average(center.corners.iter().map(|&corner_id| {
                self.corners[corner_id].moisture = self.corners[corner_id].moisture.min(1.0);
                self.corners[corner_id].moisture
            }));
        }
    }

    fn assign_biomes(&mut self) {
        for center in &mut self.centers {
            center.biome = get_biome(center);
        }
    }

    fn create_center_watersheds(&mut self) {
        self.center_watersheds = vec![None; self.centers.len()];
        for center in &self.centers {
            let mut lowest: Option<usize> = None;
            for &corner_id in &center.corners {
                if lowest
                    .map(|candidate| {
                        self.corners[corner_id].elevation < self.corners[candidate].elevation
                    })
                    .unwrap_or(true)
                {
                    lowest = Some(corner_id);
                }
            }
            self.center_watersheds[center.index] =
                lowest.map(|corner_id| self.corners[corner_id].watershed);
        }
    }

    fn build_noisy_edges(&mut self) {
        self.noisy_edges = vec![NoisyEdge::default(); self.edges.len()];
        for center in &self.centers {
            for &edge_id in &center.borders {
                let edge = &self.edges[edge_id];
                let (Some(d0), Some(d1), Some(v0), Some(v1)) = (edge.d0, edge.d1, edge.v0, edge.v1)
                else {
                    continue;
                };
                if self.noisy_edges[edge.index].path0.is_some() {
                    continue;
                }

                let f = 0.5;
                let v0_point = self.corners[v0].point;
                let v1_point = self.corners[v1].point;
                let d0_point = self.centers[d0].point;
                let d1_point = self.centers[d1].point;
                let t = flash_interpolate(v0_point, d0_point, f);
                let q = flash_interpolate(v0_point, d1_point, f);
                let r = flash_interpolate(v1_point, d0_point, f);
                let s = flash_interpolate(v1_point, d1_point, f);

                let mut min_length = 10.0;
                if self.centers[d0].biome != self.centers[d1].biome {
                    min_length = 3.0;
                }
                if self.centers[d0].ocean && self.centers[d1].ocean {
                    min_length = 100.0;
                }
                if self.centers[d0].coast || self.centers[d1].coast || edge.river > 0 {
                    min_length = 1.0;
                }

                self.noisy_edges[edge.index].path0 = Some(build_noisy_line_segments(
                    &mut self.map_random,
                    v0_point,
                    t,
                    edge.midpoint,
                    q,
                    min_length,
                ));
                self.noisy_edges[edge.index].path1 = Some(build_noisy_line_segments(
                    &mut self.map_random,
                    v1_point,
                    r,
                    edge.midpoint,
                    s,
                    min_length,
                ));
            }
        }
    }

    fn triangle_color(&self, mode: ViewMode, center_id: usize, edge_id: usize) -> u32 {
        let center = &self.centers[center_id];
        let base = biome_color(center.biome);
        match mode {
            ViewMode::Biome => base,
            ViewMode::Slopes => self.color_with_slope(base, center_id, edge_id),
        }
    }

    fn color_with_slope(&self, color: u32, center_id: usize, edge_id: usize) -> u32 {
        let edge = &self.edges[edge_id];
        let center = &self.centers[center_id];
        let (Some(v0), Some(v1)) = (edge.v0, edge.v1) else {
            return 0x44447a;
        };
        if center.water {
            return color;
        }
        let mut blended = color;
        if let Some(neighbor) = self.other_center(edge_id, center_id)
            && center.water == self.centers[neighbor].water
        {
            blended = interpolate_color(color, biome_color(self.centers[neighbor].biome), 0.4);
        }

        let light = calculate_lighting(
            center.point,
            center.elevation,
            self.corners[v0].point,
            self.corners[v0].elevation,
            self.corners[v1].point,
            self.corners[v1].elevation,
        );
        let color_low = interpolate_color(blended, 0x333333, 0.7);
        let color_high = interpolate_color(blended, 0xffffff, 0.3);
        if light < 0.5 {
            interpolate_color(color_low, blended, light * 2.0)
        } else {
            interpolate_color(blended, color_high, light * 2.0 - 1.0)
        }
    }

    fn other_center(&self, edge_id: usize, center_id: usize) -> Option<usize> {
        let edge = &self.edges[edge_id];
        if edge.d0 == Some(center_id) {
            edge.d1
        } else if edge.d1 == Some(center_id) {
            edge.d0
        } else {
            None
        }
    }

    fn land_histogram(&self) -> Vec<(u32, f32)> {
        let mut buckets = [0usize; 4];
        for center in &self.centers {
            let bucket = if center.ocean {
                0
            } else if center.coast {
                1
            } else if center.water {
                2
            } else {
                3
            };
            buckets[bucket] += 1;
        }
        histogram_pairs(&[0x44447a, 0xa09077, 0x336699, 0x679459], &buckets)
    }

    fn biome_histogram(&self) -> Vec<(u32, f32)> {
        let biomes = [
            "BEACH",
            "LAKE",
            "ICE",
            "MARSH",
            "SNOW",
            "TUNDRA",
            "BARE",
            "SCORCHED",
            "TAIGA",
            "SHRUBLAND",
            "TEMPERATE_DESERT",
            "TEMPERATE_RAIN_FOREST",
            "TEMPERATE_DECIDUOUS_FOREST",
            "GRASSLAND",
            "SUBTROPICAL_DESERT",
            "TROPICAL_RAIN_FOREST",
            "TROPICAL_SEASONAL_FOREST",
        ];
        let mut buckets = vec![0usize; biomes.len()];
        for center in &self.centers {
            if let Some(index) = biomes.iter().position(|&biome| biome == center.biome) {
                buckets[index] += 1;
            }
        }
        biomes
            .iter()
            .zip(buckets)
            .map(|(&biome, count)| (biome_color(biome), count as f32))
            .collect()
    }

    fn elevation_histogram(&self) -> Vec<(u32, f32)> {
        let mut buckets = [0usize; 10];
        for center in &self.centers {
            if !center.ocean {
                let bucket = (center.elevation * 10.0).floor().clamp(0.0, 9.0) as usize;
                buckets[bucket] += 1;
            }
        }
        (0..10)
            .map(|i| {
                (
                    interpolate_color(0x679459, 0x88aa55, i as f32 * 0.1),
                    buckets[i] as f32,
                )
            })
            .collect()
    }

    fn moisture_histogram(&self) -> Vec<(u32, f32)> {
        let mut buckets = [0usize; 10];
        for center in &self.centers {
            if !center.water {
                let bucket = (center.moisture * 10.0).floor().clamp(0.0, 9.0) as usize;
                buckets[bucket] += 1;
            }
        }
        (0..10)
            .map(|i| {
                (
                    interpolate_color(0xa09077, 0x225588, i as f32 * 0.1),
                    buckets[i] as f32,
                )
            })
            .collect()
    }
}

#[derive(Clone, Copy)]
struct IslandProfile {
    kind: IslandType,
    seed: u32,
    bumps: i32,
    start_angle: f32,
    dip_angle: f32,
    dip_width: f32,
}

impl IslandProfile {
    fn new(kind: IslandType, seed: u32) -> Self {
        let mut rng = PmPrng::new(seed);
        Self {
            kind,
            seed,
            bumps: rng.next_int_range(1, 6),
            start_angle: rng.next_double_range(0.0, 2.0 * PI),
            dip_angle: rng.next_double_range(0.0, 2.0 * PI),
            dip_width: rng.next_double_range(0.2, 0.7),
        }
    }

    fn inside(self, q: Vec2) -> bool {
        match self.kind {
            IslandType::Radial => {
                let angle = q.y.atan2(q.x);
                let length = 0.5 * (q.x.abs().max(q.y.abs()) + q.length());
                let bumps = self.bumps as f32;
                let mut r1 = 0.5
                    + 0.40
                        * (self.start_angle + bumps * angle + ((bumps + 3.0) * angle).cos()).sin();
                let mut r2 = 0.7
                    - 0.20
                        * (self.start_angle + bumps * angle - ((bumps + 2.0) * angle).sin()).sin();
                if (angle - self.dip_angle).abs() < self.dip_width
                    || (angle - self.dip_angle + 2.0 * PI).abs() < self.dip_width
                    || (angle - self.dip_angle - 2.0 * PI).abs() < self.dip_width
                {
                    r1 = 0.2;
                    r2 = 0.2;
                }
                length < r1 || (length > r1 * 1.07 && length < r2)
            }
            IslandType::Perlin => {
                let c = fractal_noise_2d((q.x + 1.0) * 128.0, (q.y + 1.0) * 128.0, self.seed);
                c > 0.3 + 0.3 * q.length_squared()
            }
            IslandType::Simplex => {
                let c = simplex_fractal_noise_2d(q.x * 2.2, q.y * 2.2, self.seed);
                c > 0.34 + 0.34 * q.length_squared()
            }
        }
    }
}

fn select_points(point_type: PointType, point_count: usize, _seed: u32) -> Vec<Vec2> {
    match point_type {
        PointType::Square => generate_square_points(point_count),
    }
}

fn generate_square_points(point_count: usize) -> Vec<Vec2> {
    let n = (point_count as f32).sqrt() as usize;
    let mut points = Vec::with_capacity(n * n);
    for x in 0..n {
        for y in 0..n {
            points.push(vec2(
                (0.5 + x as f32) / n as f32 * MAP_SIZE,
                (0.5 + y as f32) / n as f32 * MAP_SIZE,
            ));
        }
    }
    points
}

fn build_regions(_points: &[Vec2], _point_type: PointType, point_count: usize) -> Vec<Vec<Vec2>> {
    build_square_regions(point_count)
}

fn build_square_regions(point_count: usize) -> Vec<Vec<Vec2>> {
    let n = (point_count as f32).sqrt() as usize;
    let cell = MAP_SIZE / n as f32;
    let mut regions = Vec::with_capacity(n * n);
    for x in 0..n {
        for y in 0..n {
            let left = x as f32 * cell;
            let right = (x + 1) as f32 * cell;
            let top = y as f32 * cell;
            let bottom = (y + 1) as f32 * cell;
            regions.push(vec![
                vec2(left, top),
                vec2(right, top),
                vec2(right, bottom),
                vec2(left, bottom),
            ]);
        }
    }
    regions
}

fn make_corner(
    corners: &mut Vec<Corner>,
    lookup: &mut HashMap<(i32, i32), usize>,
    point: Vec2,
) -> usize {
    let clamped = vec2(point.x.clamp(0.0, MAP_SIZE), point.y.clamp(0.0, MAP_SIZE));
    let key = (
        (clamped.x * 1000.0).round() as i32,
        (clamped.y * 1000.0).round() as i32,
    );
    if let Some(&corner_id) = lookup.get(&key) {
        return corner_id;
    }
    let border = clamped.x <= 0.001
        || clamped.x >= MAP_SIZE - 0.001
        || clamped.y <= 0.001
        || clamped.y >= MAP_SIZE - 0.001;
    let index = corners.len();
    corners.push(Corner {
        index,
        point: clamped,
        ocean: false,
        water: false,
        coast: false,
        border,
        elevation: 0.0,
        moisture: 0.0,
        touches: Vec::new(),
        protrudes: Vec::new(),
        adjacent: Vec::new(),
        river: 0,
        downslope: index,
        watershed: index,
        watershed_size: 0,
    });
    lookup.insert(key, index);
    index
}

fn improve_corners(centers: &mut [Center], corners: &mut [Corner], edges: &mut [Edge]) {
    let new_points: Vec<Vec2> = corners
        .iter()
        .map(|corner| {
            if corner.border {
                corner.point
            } else {
                corner
                    .touches
                    .iter()
                    .fold(Vec2::ZERO, |acc, &center_id| acc + centers[center_id].point)
                    / corner.touches.len() as f32
            }
        })
        .collect();

    for (corner, point) in corners.iter_mut().zip(new_points) {
        corner.point = point;
    }
    for edge in edges {
        if let (Some(v0), Some(v1)) = (edge.v0, edge.v1) {
            edge.midpoint = (corners[v0].point + corners[v1].point) * 0.5;
        }
    }
}

fn get_biome(center: &Center) -> &'static str {
    if center.ocean {
        "OCEAN"
    } else if center.water {
        if center.elevation < 0.1 {
            "MARSH"
        } else if center.elevation > 0.8 {
            "ICE"
        } else {
            "LAKE"
        }
    } else if center.coast {
        "BEACH"
    } else if center.elevation > 0.8 {
        if center.moisture > 0.50 {
            "SNOW"
        } else if center.moisture > 0.33 {
            "TUNDRA"
        } else if center.moisture > 0.16 {
            "BARE"
        } else {
            "SCORCHED"
        }
    } else if center.elevation > 0.6 {
        if center.moisture > 0.66 {
            "TAIGA"
        } else if center.moisture > 0.33 {
            "SHRUBLAND"
        } else {
            "TEMPERATE_DESERT"
        }
    } else if center.elevation > 0.3 {
        if center.moisture > 0.83 {
            "TEMPERATE_RAIN_FOREST"
        } else if center.moisture > 0.50 {
            "TEMPERATE_DECIDUOUS_FOREST"
        } else if center.moisture > 0.16 {
            "GRASSLAND"
        } else {
            "TEMPERATE_DESERT"
        }
    } else if center.moisture > 0.66 {
        "TROPICAL_RAIN_FOREST"
    } else if center.moisture > 0.33 {
        "TROPICAL_SEASONAL_FOREST"
    } else if center.moisture > 0.16 {
        "GRASSLAND"
    } else {
        "SUBTROPICAL_DESERT"
    }
}

fn biome_color(biome: &str) -> u32 {
    match biome {
        "OCEAN" => 0x44447a,
        "COAST" => 0x33335a,
        "LAKESHORE" => 0x225588,
        "LAKE" => 0x336699,
        "RIVER" => 0x225588,
        "MARSH" => 0x2f6666,
        "ICE" => 0x99ffff,
        "BEACH" => 0xa09077,
        "SNOW" => 0xffffff,
        "TUNDRA" => 0xbbbbaa,
        "BARE" => 0x888888,
        "SCORCHED" => 0x555555,
        "TAIGA" => 0x99aa77,
        "SHRUBLAND" => 0x889977,
        "TEMPERATE_DESERT" => 0xc9d29b,
        "TEMPERATE_RAIN_FOREST" => 0x448855,
        "TEMPERATE_DECIDUOUS_FOREST" => 0x679459,
        "GRASSLAND" => 0x88aa55,
        "SUBTROPICAL_DESERT" => 0xd2b98b,
        "TROPICAL_RAIN_FOREST" => 0x337755,
        "TROPICAL_SEASONAL_FOREST" => 0x559944,
        _ => 0x000000,
    }
}

fn calculate_lighting(center: Vec2, center_e: f32, a: Vec2, a_e: f32, b: Vec2, b_e: f32) -> f32 {
    let ab = vec3(a.x - center.x, a.y - center.y, a_e - center_e);
    let ac = vec3(b.x - center.x, b.y - center.y, b_e - center_e);
    let mut normal = ab.cross(ac);
    if normal.z < 0.0 {
        normal = -normal;
    }
    let normal = normal.normalize_or_zero();
    (0.5 + 35.0 * normal.dot(vec3(-1.0, -1.0, 0.0))).clamp(0.0, 1.0)
}

fn draw_distribution(x: f32, y: f32, width: f32, height: f32, buckets: &[(u32, f32)]) {
    let total: f32 = buckets.iter().map(|(_, count)| *count).sum();
    if total <= 0.0 {
        return;
    }
    let mut cursor = x;
    for &(color, count) in buckets {
        if count <= 0.0 {
            continue;
        }
        let w = count / total * width;
        draw_rectangle(cursor, y, (w - 1.0).max(0.0), height, color_from_u32(color));
        cursor += w;
    }
}

fn draw_histogram(x: f32, y: f32, width: f32, height: f32, buckets: &[(u32, f32)]) {
    let max_count = buckets
        .iter()
        .map(|(_, count)| *count)
        .fold(0.0_f32, f32::max);
    if max_count <= 0.0 {
        return;
    }
    let bar_w = width / buckets.len() as f32;
    for (index, &(color, count)) in buckets.iter().enumerate() {
        let h = height * count / max_count;
        draw_rectangle(
            x + index as f32 * bar_w,
            y + height - h,
            (bar_w - 1.0).max(0.0),
            h,
            color_from_u32(color),
        );
    }
}

fn histogram_pairs(colors: &[u32], buckets: &[usize]) -> Vec<(u32, f32)> {
    colors
        .iter()
        .zip(buckets)
        .map(|(&color, &count)| (color, count as f32))
        .collect()
}

fn center_visible(center: &Center, source_rect: Rect) -> bool {
    let margin = 35.0;
    center.point.x >= source_rect.x - margin
        && center.point.x <= source_rect.x + source_rect.w + margin
        && center.point.y >= source_rect.y - margin
        && center.point.y <= source_rect.y + source_rect.h + margin
}

fn to_screen(point: Vec2, map_rect: Rect, source_rect: Rect) -> Vec2 {
    vec2(
        map_rect.x + (point.x - source_rect.x) / source_rect.w * map_rect.w,
        map_rect.y + (point.y - source_rect.y) / source_rect.h * map_rect.h,
    )
}

fn draw_line_between(
    a: Vec2,
    b: Vec2,
    map_rect: Rect,
    source_rect: Rect,
    width: f32,
    color: Color,
) {
    let a = to_screen(a, map_rect, source_rect);
    let b = to_screen(b, map_rect, source_rect);
    draw_line(a.x, a.y, b.x, b.y, width, color);
}

fn draw_path_wedge(center: Vec2, path: &[Vec2], map_rect: Rect, source_rect: Rect, color: Color) {
    if path.len() < 2 {
        return;
    }
    let center = to_screen(center, map_rect, source_rect);
    for segment in path.windows(2) {
        draw_triangle(
            center,
            to_screen(segment[0], map_rect, source_rect),
            to_screen(segment[1], map_rect, source_rect),
            color,
        );
    }
}

fn draw_noisy_edge_path(
    path0: &[Vec2],
    path1: &[Vec2],
    map_rect: Rect,
    source_rect: Rect,
    width: f32,
    color: Color,
) {
    draw_path_lines(path0, false, map_rect, source_rect, width, color);
    draw_path_lines(path1, true, map_rect, source_rect, width, color);
}

fn draw_path_lines(
    path: &[Vec2],
    reverse: bool,
    map_rect: Rect,
    source_rect: Rect,
    width: f32,
    color: Color,
) {
    if path.len() < 2 {
        return;
    }
    if reverse {
        for segment in path.windows(2).rev() {
            draw_line_between(segment[1], segment[0], map_rect, source_rect, width, color);
        }
    } else {
        for segment in path.windows(2) {
            draw_line_between(segment[0], segment[1], map_rect, source_rect, width, color);
        }
    }
}

fn selected_label(label: &str, selected: bool) -> String {
    if selected {
        format!("[{}]", label)
    } else {
        label.to_string()
    }
}

fn random_seed_text() -> String {
    let mut rng = PmPrng::new((get_time().to_bits() as u32).max(1));
    format!(
        "{}-{}",
        rng.next_int_range(0, 100_000),
        rng.next_int_range(1, 9)
    )
}

fn parse_seed(seed_text: &str) -> (u32, u32) {
    let trimmed = seed_text.trim();
    let mut parts = trimmed.splitn(2, '-');
    let first = parts.next().unwrap_or_default();
    let second = parts.next();
    if !first.is_empty() && first.chars().all(|ch| ch.is_ascii_digit()) {
        let seed = first.parse::<u32>().unwrap_or(1);
        let variant = second
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(0);
        if seed != 0 {
            return (seed, variant);
        }
    }

    let mut seed = 0u32;
    for ch in trimmed.chars() {
        seed = (seed << 4) | ch as u32;
    }
    (seed % 100_000, 1)
}

fn push_unique<T: PartialEq>(items: &mut Vec<T>, item: T) {
    if !items.contains(&item) {
        items.push(item);
    }
}

fn sorted_pair(a: usize, b: usize) -> (usize, usize) {
    if a < b { (a, b) } else { (b, a) }
}

fn average(values: impl Iterator<Item = f32>) -> f32 {
    let mut total = 0.0;
    let mut count = 0usize;
    for value in values {
        total += value;
        count += 1;
    }
    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

fn fractal_noise_2d(x: f32, y: f32, seed: u32) -> f32 {
    let mut sum = 0.0;
    let mut amplitude = 1.0;
    let mut amplitude_sum = 0.0;
    for octave in 0..8 {
        let scale = 1.0 / (64.0 / (1usize << octave) as f32);
        sum += value_noise(x * scale, y * scale, seed + octave as u32 * 101) * amplitude;
        amplitude_sum += amplitude;
        amplitude *= 0.5;
    }
    sum / amplitude_sum
}

fn simplex_fractal_noise_2d(x: f32, y: f32, seed: u32) -> f32 {
    let mut sum = 0.0;
    let mut amplitude = 1.0;
    let mut amplitude_sum = 0.0;
    let mut frequency = 1.0;
    for octave in 0..5 {
        sum += simplex_noise_2d(x * frequency, y * frequency, seed + octave * 997) * amplitude;
        amplitude_sum += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    0.5 + 0.5 * (sum / amplitude_sum).clamp(-1.0, 1.0)
}

fn simplex_noise_2d(xin: f32, yin: f32, seed: u32) -> f32 {
    const F2: f32 = 0.366_025_4;
    const G2: f32 = 0.211_324_87;

    let s = (xin + yin) * F2;
    let i = (xin + s).floor();
    let j = (yin + s).floor();
    let t = (i + j) * G2;
    let x0 = xin - (i - t);
    let y0 = yin - (j - t);

    let (i1, j1) = if x0 > y0 { (1.0, 0.0) } else { (0.0, 1.0) };
    let x1 = x0 - i1 + G2;
    let y1 = y0 - j1 + G2;
    let x2 = x0 - 1.0 + 2.0 * G2;
    let y2 = y0 - 1.0 + 2.0 * G2;

    let ii = i as i32;
    let jj = j as i32;
    let gi0 = simplex_gradient(seed, ii, jj);
    let gi1 = simplex_gradient(seed, ii + i1 as i32, jj + j1 as i32);
    let gi2 = simplex_gradient(seed, ii + 1, jj + 1);

    70.0 * (simplex_corner(gi0, x0, y0) + simplex_corner(gi1, x1, y1) + simplex_corner(gi2, x2, y2))
}

fn simplex_corner(gradient: Vec2, x: f32, y: f32) -> f32 {
    let mut t = 0.5 - x * x - y * y;
    if t < 0.0 {
        return 0.0;
    }
    t *= t;
    t * t * gradient.dot(vec2(x, y))
}

fn value_noise(x: f32, y: f32, seed: u32) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let xf = smoothstep(x - x0 as f32);
    let yf = smoothstep(y - y0 as f32);
    let v00 = hash_unit(seed, x0, y0);
    let v10 = hash_unit(seed, x0 + 1, y0);
    let v01 = hash_unit(seed, x0, y0 + 1);
    let v11 = hash_unit(seed, x0 + 1, y0 + 1);
    let a = lerp(v00, v10, xf);
    let b = lerp(v01, v11, xf);
    lerp(a, b, yf)
}

fn hash_unit(seed: u32, x: i32, y: i32) -> f32 {
    let mut h = seed ^ (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    (h ^ (h >> 16)) as f32 / u32::MAX as f32
}

fn simplex_gradient(seed: u32, x: i32, y: i32) -> Vec2 {
    const GRADIENTS: [Vec2; 8] = [
        Vec2::new(1.0, 0.0),
        Vec2::new(-1.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(0.0, -1.0),
        Vec2::new(0.707_106_77, 0.707_106_77),
        Vec2::new(-0.707_106_77, 0.707_106_77),
        Vec2::new(0.707_106_77, -0.707_106_77),
        Vec2::new(-0.707_106_77, -0.707_106_77),
    ];
    let index = (hash_unit(seed ^ 0x9e37_79b9, x, y) * GRADIENTS.len() as f32) as usize;
    GRADIENTS[index.min(GRADIENTS.len() - 1)]
}

fn smoothstep(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn flash_interpolate(a: Vec2, b: Vec2, f: f32) -> Vec2 {
    b + (a - b) * f
}

fn build_noisy_line_segments(
    rng: &mut PmPrng,
    a: Vec2,
    b: Vec2,
    c: Vec2,
    d: Vec2,
    min_length: f32,
) -> Vec<Vec2> {
    fn subdivide(
        rng: &mut PmPrng,
        points: &mut Vec<Vec2>,
        a: Vec2,
        b: Vec2,
        c: Vec2,
        d: Vec2,
        min_length: f32,
    ) {
        if a.distance(c) < min_length || b.distance(d) < min_length {
            return;
        }

        let p = rng.next_double_range(0.2, 0.8);
        let q = rng.next_double_range(0.2, 0.8);
        let e = flash_interpolate(a, d, p);
        let f = flash_interpolate(b, c, p);
        let g = flash_interpolate(a, b, q);
        let i = flash_interpolate(d, c, q);
        let h = flash_interpolate(e, f, q);
        let s = 1.0 - rng.next_double_range(-0.4, 0.4);
        let t = 1.0 - rng.next_double_range(-0.4, 0.4);

        subdivide(
            rng,
            points,
            a,
            flash_interpolate(g, b, s),
            h,
            flash_interpolate(e, d, t),
            min_length,
        );
        points.push(h);
        subdivide(
            rng,
            points,
            h,
            flash_interpolate(f, c, s),
            c,
            flash_interpolate(i, d, t),
            min_length,
        );
    }

    let mut points = vec![a];
    subdivide(rng, &mut points, a, b, c, d, min_length);
    points.push(c);
    points
}

fn interpolate_color(color0: u32, color1: u32, f: f32) -> u32 {
    let f = f.clamp(0.0, 1.0);
    let r = ((1.0 - f) * ((color0 >> 16) as f32) + f * ((color1 >> 16) as f32)).min(255.0) as u32;
    let g = ((1.0 - f) * (((color0 >> 8) & 0xff) as f32) + f * (((color1 >> 8) & 0xff) as f32))
        .min(255.0) as u32;
    let b = ((1.0 - f) * ((color0 & 0xff) as f32) + f * ((color1 & 0xff) as f32)).min(255.0) as u32;
    (r << 16) | (g << 8) | b
}

fn color_from_u32(color: u32) -> Color {
    Color::new(
        ((color >> 16) & 0xff) as f32 / 255.0,
        ((color >> 8) & 0xff) as f32 / 255.0,
        (color & 0xff) as f32 / 255.0,
        1.0,
    )
}

#[derive(Clone, Copy)]
struct PmPrng {
    seed: u32,
}

impl PmPrng {
    fn new(seed: u32) -> Self {
        Self {
            seed: seed.clamp(1, 2_147_483_646),
        }
    }

    fn new_raw(seed: u32) -> Self {
        Self {
            seed: seed % 2_147_483_647,
        }
    }

    fn next_int(&mut self) -> u32 {
        self.seed = ((self.seed as u64 * 16_807) % 2_147_483_647) as u32;
        self.seed
    }

    fn next_double(&mut self) -> f32 {
        self.next_int() as f32 / 2_147_483_647.0
    }

    fn next_int_range(&mut self, min: i32, max: i32) -> i32 {
        let min = min as f32 - 0.4999;
        let max = max as f32 + 0.4999;
        (min + ((max - min) * self.next_double())).round() as i32
    }

    fn next_double_range(&mut self, min: f32, max: f32) -> f32 {
        min + ((max - min) * self.next_double())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn quantize(value: f32) -> i32 {
        (value * 10_000.0).round() as i32
    }

    fn hash_point(hasher: &mut DefaultHasher, point: Vec2) {
        quantize(point.x).hash(hasher);
        quantize(point.y).hash(hasher);
    }

    fn map_fingerprint(map: &PolyMap) -> u64 {
        let mut hasher = DefaultHasher::new();
        map.centers.len().hash(&mut hasher);
        map.corners.len().hash(&mut hasher);
        map.edges.len().hash(&mut hasher);
        for center in &map.centers {
            center.index.hash(&mut hasher);
            hash_point(&mut hasher, center.point);
            center.water.hash(&mut hasher);
            center.ocean.hash(&mut hasher);
            center.coast.hash(&mut hasher);
            center.border.hash(&mut hasher);
            center.biome.hash(&mut hasher);
            quantize(center.elevation).hash(&mut hasher);
            quantize(center.moisture).hash(&mut hasher);
            center.neighbors.len().hash(&mut hasher);
            center.borders.len().hash(&mut hasher);
            center.corners.len().hash(&mut hasher);
        }
        for edge in &map.edges {
            edge.index.hash(&mut hasher);
            edge.d0.hash(&mut hasher);
            edge.d1.hash(&mut hasher);
            edge.v0.hash(&mut hasher);
            edge.v1.hash(&mut hasher);
            edge.river.hash(&mut hasher);
            hash_point(&mut hasher, edge.midpoint);
        }
        for noisy_edge in &map.noisy_edges {
            noisy_edge.path0.as_ref().map(Vec::len).hash(&mut hasher);
            noisy_edge.path1.as_ref().map(Vec::len).hash(&mut hasher);
        }
        hasher.finish()
    }

    #[test]
    fn default_seed_matches_swf_demo() {
        assert_eq!(parse_seed(DEFAULT_SEED_TEXT), (85882, 8));
    }

    #[test]
    fn default_scene_configuration_is_trimmed() {
        assert_eq!(DEFAULT_ISLAND_TYPE, IslandType::Perlin);
        assert_eq!(DEFAULT_POINT_TYPE, PointType::Square);
        assert_eq!(DEFAULT_POINT_COUNT, 4000);
        assert_eq!(DEFAULT_VIEW_MODE, ViewMode::Biome);
    }

    #[test]
    fn debug_env_accepts_only_remaining_controls() {
        assert_eq!(
            IslandType::from_debug_env("RADIAL"),
            Some(IslandType::Radial)
        );
        assert_eq!(
            IslandType::from_debug_env("perlin"),
            Some(IslandType::Perlin)
        );
        assert_eq!(
            IslandType::from_debug_env("simplex"),
            Some(IslandType::Simplex)
        );
        assert_eq!(PointType::from_debug_env("Square"), Some(PointType::Square));
        assert_eq!(ViewMode::from_debug_env("biomes"), Some(ViewMode::Biome));
        assert_eq!(
            ViewMode::from_debug_env("2D slopes"),
            Some(ViewMode::Slopes)
        );
    }

    #[test]
    fn seed_input_accepts_demo_seed_characters() {
        assert!("12345-6_Az".chars().all(is_seed_char));
        assert!(!"12.34".chars().all(is_seed_char));
    }

    #[test]
    fn seed_input_replaces_existing_seed_on_first_typed_character() {
        let mut text = DEFAULT_SEED_TEXT.to_string();
        let mut replace_on_type = true;

        for ch in "12345-6".chars() {
            assert!(push_seed_char(&mut text, &mut replace_on_type, ch));
        }

        assert_eq!(text, "12345-6");
        assert!(!replace_on_type);
    }

    #[test]
    fn seed_input_hitbox_matches_demo_panel_position() {
        let sidebar = Rect::new(768.0, 0.0, 256.0, 768.0);
        let field = seed_field_rect(sidebar);

        assert!(field.contains(vec2(848.0, 29.0)));
        assert!(field.contains(vec2(912.0, 51.0)));
        assert!(!field.contains(vec2(846.0, 29.0)));
    }

    #[test]
    fn square_point_selector_uses_floor_sqrt_grid_counts() {
        assert_eq!(generate_square_points(4000).len(), 63 * 63);
        assert_eq!(generate_square_points(8000).len(), 89 * 89);
        assert_eq!(generate_square_points(16000).len(), 126 * 126);
        assert_eq!(generate_square_points(32000).len(), 178 * 178);
    }

    #[test]
    fn map_generation_builds_graph_with_land_and_water() {
        let map = PolyMap::generate(
            DEFAULT_SEED_TEXT,
            IslandType::Perlin,
            PointType::Square,
            4000,
        );

        assert!(!map.centers.is_empty());
        assert!(!map.corners.is_empty());
        assert!(!map.edges.is_empty());
        assert!(map.centers.iter().any(|center| center.ocean));
        assert!(map.centers.iter().any(|center| !center.water));
    }

    #[test]
    fn island_shape_buttons_change_inside_function() {
        let radial = IslandProfile::new(IslandType::Radial, 85882);
        let perlin = IslandProfile::new(IslandType::Perlin, 85882);

        assert_ne!(
            radial.inside(vec2(-0.55, -0.2)),
            perlin.inside(vec2(-0.55, -0.2))
        );
    }

    #[test]
    fn demo_control_labels_match_reference() {
        let island_labels: Vec<_> = IslandType::ALL.iter().map(|kind| kind.label()).collect();
        let point_labels: Vec<_> = PointType::ALL.iter().map(|kind| kind.label()).collect();
        let view_labels: Vec<_> = ViewMode::ALL.iter().map(|mode| mode.label()).collect();

        assert_eq!(island_labels, ["Radial", "Perlin", "Simplex"]);
        assert_eq!(point_labels, ["Square"]);
        assert_eq!(view_labels, ["Biomes", "2D slopes"]);
        assert_eq!(&POINT_COUNTS[..], &[4000, 8000, 16000, 32000]);
    }

    #[test]
    fn island_button_positions_cover_all_island_shapes() {
        assert_eq!(island_button_x_positions().len(), IslandType::ALL.len());
    }

    #[test]
    fn removed_controls_are_not_accepted_from_debug_env() {
        assert_eq!(IslandType::from_debug_env("square"), None);
        assert_eq!(IslandType::from_debug_env("blob"), None);
        assert_eq!(PointType::from_debug_env("random"), None);
        assert_eq!(PointType::from_debug_env("relaxed"), None);
        assert_eq!(PointType::from_debug_env("hex"), None);
        assert_eq!(ViewMode::from_debug_env("smooth"), None);
        assert_eq!(ViewMode::from_debug_env("3d"), None);
        assert_eq!(ViewMode::from_debug_env("elevation"), None);
        assert_eq!(ViewMode::from_debug_env("moisture"), None);
        assert_eq!(ViewMode::from_debug_env("polygons"), None);
        assert_eq!(ViewMode::from_debug_env("watersheds"), None);
    }

    #[test]
    fn removed_point_counts_are_not_available() {
        for removed_count in [500, 1000, 2000] {
            assert!(!POINT_COUNTS.contains(&removed_count));
        }
    }

    #[test]
    fn zoom_source_rect_crops_around_pan() {
        let rect = map_source_rect(vec2(64.0, -32.0), 2.0);

        assert_eq!(rect.w, 300.0);
        assert_eq!(rect.h, 300.0);
        assert_eq!(rect.x, 214.0);
        assert_eq!(rect.y, 118.0);
    }

    #[test]
    fn pan_is_clamped_to_visible_map_bounds() {
        let pan = clamp_pan(vec2(999.0, -999.0), 4.0);

        assert_eq!(pan, vec2(225.0, -225.0));
    }

    #[test]
    fn square_4000_generation_completes() {
        let map = PolyMap::generate(
            DEFAULT_SEED_TEXT,
            IslandType::Perlin,
            PointType::Square,
            4000,
        );

        assert_eq!(map.centers.len(), 63 * 63);
        assert!(map.centers.iter().any(|center| !center.water));
    }

    #[test]
    fn map_generation_is_deterministic_for_same_seed() {
        let first = PolyMap::generate(
            DEFAULT_SEED_TEXT,
            IslandType::Perlin,
            PointType::Square,
            4000,
        );
        let second = PolyMap::generate(
            DEFAULT_SEED_TEXT,
            IslandType::Perlin,
            PointType::Square,
            4000,
        );

        assert_eq!(map_fingerprint(&first), map_fingerprint(&second));
    }

    #[test]
    fn changing_shape_seed_changes_generated_map() {
        let first = PolyMap::generate("85882-8", IslandType::Perlin, PointType::Square, 4000);
        let second = PolyMap::generate("85883-8", IslandType::Perlin, PointType::Square, 4000);

        assert_ne!(map_fingerprint(&first), map_fingerprint(&second));
    }

    #[test]
    fn changing_seed_variant_changes_generated_map() {
        let first = PolyMap::generate("85882-8", IslandType::Perlin, PointType::Square, 4000);
        let second = PolyMap::generate("85882-9", IslandType::Perlin, PointType::Square, 4000);

        assert_ne!(map_fingerprint(&first), map_fingerprint(&second));
    }

    #[test]
    fn radial_square_generation_builds_valid_island() {
        let map = PolyMap::generate(
            DEFAULT_SEED_TEXT,
            IslandType::Radial,
            PointType::Square,
            4000,
        );

        assert!(map.centers.iter().any(|center| center.ocean));
        assert!(map.centers.iter().any(|center| !center.water));
        assert!(map.centers.iter().any(|center| center.coast));
        assert!(map.edges.iter().any(|edge| edge.river > 0));
    }

    #[test]
    fn simplex_square_generation_builds_valid_island() {
        let map = PolyMap::generate(
            DEFAULT_SEED_TEXT,
            IslandType::Simplex,
            PointType::Square,
            4000,
        );

        assert!(map.centers.iter().any(|center| center.ocean));
        assert!(map.centers.iter().any(|center| !center.water));
        assert!(map.centers.iter().any(|center| center.coast));
        assert!(map.edges.iter().any(|edge| edge.river > 0));
    }

    #[test]
    fn simplex_is_deterministic_for_same_seed() {
        let first = PolyMap::generate(
            DEFAULT_SEED_TEXT,
            IslandType::Simplex,
            PointType::Square,
            4000,
        );
        let second = PolyMap::generate(
            DEFAULT_SEED_TEXT,
            IslandType::Simplex,
            PointType::Square,
            4000,
        );

        assert_eq!(map_fingerprint(&first), map_fingerprint(&second));
    }

    #[test]
    fn simplex_generates_different_map_from_perlin() {
        let simplex = PolyMap::generate(
            DEFAULT_SEED_TEXT,
            IslandType::Simplex,
            PointType::Square,
            4000,
        );
        let perlin = PolyMap::generate(
            DEFAULT_SEED_TEXT,
            IslandType::Perlin,
            PointType::Square,
            4000,
        );

        assert_ne!(map_fingerprint(&simplex), map_fingerprint(&perlin));
    }

    #[test]
    fn square_point_regions_remain_axis_aligned_cells() {
        let map = PolyMap::generate(
            DEFAULT_SEED_TEXT,
            IslandType::Perlin,
            PointType::Square,
            4000,
        );

        for center in &map.centers {
            assert_eq!(center.corners.len(), 4);
            for &edge_id in &center.borders {
                let edge = &map.edges[edge_id];
                let (Some(v0), Some(v1)) = (edge.v0, edge.v1) else {
                    panic!("square edge must have two corners");
                };
                let a = map.corners[v0].point;
                let b = map.corners[v1].point;
                assert!(
                    (a.x - b.x).abs() < 0.001 || (a.y - b.y).abs() < 0.001,
                    "square edge was not axis aligned: {a:?} -> {b:?}"
                );
            }
        }
    }

    #[test]
    fn noisy_edges_are_built_for_interior_edges() {
        let map = PolyMap::generate(
            DEFAULT_SEED_TEXT,
            IslandType::Perlin,
            PointType::Square,
            4000,
        );

        let interior_edge = map
            .edges
            .iter()
            .find(|edge| {
                edge.d0.is_some() && edge.d1.is_some() && edge.v0.is_some() && edge.v1.is_some()
            })
            .expect("square map should have interior edges");
        let noisy_edge = &map.noisy_edges[interior_edge.index];
        let path0 = noisy_edge.path0.as_ref().expect("path0 should exist");
        let path1 = noisy_edge.path1.as_ref().expect("path1 should exist");

        assert_eq!(
            path0.first().copied(),
            interior_edge.v0.map(|v| map.corners[v].point)
        );
        assert_eq!(
            path1.first().copied(),
            interior_edge.v1.map(|v| map.corners[v].point)
        );
        assert_eq!(path0.last().copied(), Some(interior_edge.midpoint));
        assert_eq!(path1.last().copied(), Some(interior_edge.midpoint));
    }

    #[test]
    fn graph_links_are_bidirectional() {
        let map = PolyMap::generate(
            DEFAULT_SEED_TEXT,
            IslandType::Perlin,
            PointType::Square,
            4000,
        );

        for center in &map.centers {
            for &neighbor in &center.neighbors {
                assert!(
                    map.centers[neighbor].neighbors.contains(&center.index),
                    "neighbor link was not reciprocal"
                );
            }
            for &corner_id in &center.corners {
                assert!(
                    map.corners[corner_id].touches.contains(&center.index),
                    "corner touch did not include center"
                );
            }
            for &edge_id in &center.borders {
                let edge = &map.edges[edge_id];
                assert!(
                    edge.d0 == Some(center.index) || edge.d1 == Some(center.index),
                    "border edge did not reference center"
                );
            }
        }

        for corner in &map.corners {
            for &adjacent in &corner.adjacent {
                assert!(
                    map.corners[adjacent].adjacent.contains(&corner.index),
                    "corner adjacency was not reciprocal"
                );
            }
            for &edge_id in &corner.protrudes {
                let edge = &map.edges[edge_id];
                assert!(
                    edge.v0 == Some(corner.index) || edge.v1 == Some(corner.index),
                    "protruding edge did not reference corner"
                );
            }
        }
    }

    #[test]
    fn generated_elevation_and_moisture_stay_normalized() {
        let map = PolyMap::generate(
            DEFAULT_SEED_TEXT,
            IslandType::Perlin,
            PointType::Square,
            4000,
        );

        for center in &map.centers {
            assert!((0.0..=1.0).contains(&center.elevation));
            assert!((0.0..=1.0).contains(&center.moisture));
        }
        for corner in &map.corners {
            assert!((0.0..=1.0).contains(&corner.elevation));
            assert!((0.0..=1.0).contains(&corner.moisture));
            if corner.border {
                assert_eq!(corner.elevation, 0.0);
            }
        }
    }

    #[test]
    fn biome_categories_match_water_state() {
        let map = PolyMap::generate(
            DEFAULT_SEED_TEXT,
            IslandType::Perlin,
            PointType::Square,
            4000,
        );

        for center in &map.centers {
            if center.ocean {
                assert_eq!(center.biome, "OCEAN");
            } else if center.water {
                assert!(matches!(center.biome, "MARSH" | "ICE" | "LAKE"));
            } else if center.coast {
                assert_eq!(center.biome, "BEACH");
            } else {
                assert!(!matches!(center.biome, "OCEAN" | "MARSH" | "ICE" | "LAKE"));
            }
        }
    }

    #[test]
    fn square_point_type_generates_without_drainage_loops() {
        let map = PolyMap::generate(
            DEFAULT_SEED_TEXT,
            IslandType::Perlin,
            PointType::Square,
            4000,
        );

        assert!(map.edges.iter().any(|edge| edge.river > 0));
        assert!(map.centers.iter().any(|center| center.coast));
    }
}

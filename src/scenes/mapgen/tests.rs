use super::generate::generate_square_points;
use super::*;
use std::collections::VecDeque;
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
        center.shallow_ocean.hash(&mut hasher);
        center.ocean_distance.hash(&mut hasher);
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

fn deep_ocean_components(map: &PolyMap) -> Vec<Vec<usize>> {
    let mut components = Vec::new();
    let mut visited = vec![false; map.centers.len()];
    for start in 0..map.centers.len() {
        if visited[start] || map.centers[start].biome != "DEEP_OCEAN" {
            continue;
        }

        let mut component = Vec::new();
        let mut queue = VecDeque::from([start]);
        visited[start] = true;
        while let Some(center_id) = queue.pop_front() {
            component.push(center_id);
            for &neighbor in &map.centers[center_id].neighbors {
                if !visited[neighbor] && map.centers[neighbor].biome == "DEEP_OCEAN" {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }
        components.push(component);
    }
    components
}

fn deep_component_is_fully_surrounded_by_shallow(map: &PolyMap, component: &[usize]) -> bool {
    let mut in_component = vec![false; map.centers.len()];
    for &center_id in component {
        in_component[center_id] = true;
    }

    !component
        .iter()
        .any(|&center_id| map.centers[center_id].border)
        && component.iter().all(|&center_id| {
            map.centers[center_id].neighbors.iter().all(|&neighbor| {
                in_component[neighbor] || map.centers[neighbor].biome == "SHALLOW_OCEAN"
            })
        })
}

fn passable_land_and_shallow_components(map: &PolyMap) -> Vec<Vec<usize>> {
    let mut components = Vec::new();
    let mut visited = vec![false; map.centers.len()];
    for start in 0..map.centers.len() {
        if visited[start] || !is_land_or_shallow_ocean(map, start) {
            continue;
        }

        let mut component = Vec::new();
        let mut queue = VecDeque::from([start]);
        visited[start] = true;
        while let Some(center_id) = queue.pop_front() {
            component.push(center_id);
            for &neighbor in &map.centers[center_id].neighbors {
                if !visited[neighbor] && is_land_or_shallow_ocean(map, neighbor) {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }
        components.push(component);
    }
    components
}

fn is_land_or_shallow_ocean(map: &PolyMap, center_id: usize) -> bool {
    let center = &map.centers[center_id];
    !center.water || center.biome == "SHALLOW_OCEAN"
}

fn land_count_reachable_without(map: &PolyMap, blocked: Option<usize>) -> usize {
    let Some(start) = map
        .centers
        .iter()
        .find(|center| !center.water && Some(center.index) != blocked)
        .map(|center| center.index)
    else {
        return 0;
    };

    let mut visited = vec![false; map.centers.len()];
    let mut queue = VecDeque::from([start]);
    visited[start] = true;
    let mut land_count = 0usize;

    while let Some(center_id) = queue.pop_front() {
        if !map.centers[center_id].water {
            land_count += 1;
        }

        for &neighbor in &map.centers[center_id].neighbors {
            if Some(neighbor) == blocked
                || visited[neighbor]
                || !is_land_or_shallow_ocean(map, neighbor)
            {
                continue;
            }
            visited[neighbor] = true;
            queue.push_back(neighbor);
        }
    }

    land_count
}

fn non_coastal_shallow_bridge_articulations(map: &PolyMap) -> Vec<usize> {
    let total_land = map.centers.iter().filter(|center| !center.water).count();

    map.centers
        .iter()
        .filter(|center| {
            center.biome == "SHALLOW_OCEAN"
                && !center.border
                && !center
                    .neighbors
                    .iter()
                    .any(|&neighbor| !map.centers[neighbor].water)
                && land_count_reachable_without(map, Some(center.index)) < total_land
        })
        .map(|center| center.index)
        .collect()
}

fn deep_ocean_finger_cells(map: &PolyMap) -> Vec<usize> {
    map.centers
        .iter()
        .filter(|center| {
            center.biome == "DEEP_OCEAN"
                && !center.border
                && center.ocean_distance <= 4
                && center
                    .neighbors
                    .iter()
                    .filter(|&&neighbor| map.centers[neighbor].biome == "SHALLOW_OCEAN")
                    .count()
                    >= 2
        })
        .map(|center| center.index)
        .collect()
}

fn deep_ocean_concavity_cells(map: &PolyMap, reach: usize, max_ocean_distance: i32) -> Vec<usize> {
    let grid_width = (map.centers.len() as f32).sqrt() as usize;
    let directions: [(isize, isize); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];

    map.centers
        .iter()
        .filter(|center| {
            center.biome == "DEEP_OCEAN"
                && !center.border
                && center.ocean_distance <= max_ocean_distance
                && directions.iter().any(|&(dx, dy)| {
                    sees_shallow_in_direction(map, grid_width, center.index, dx, dy, reach)
                        && sees_shallow_in_direction(map, grid_width, center.index, -dx, -dy, reach)
                })
        })
        .map(|center| center.index)
        .collect()
}

fn sees_shallow_in_direction(
    map: &PolyMap,
    grid_width: usize,
    center_id: usize,
    dx: isize,
    dy: isize,
    reach: usize,
) -> bool {
    let x = center_id / grid_width;
    let y = center_id % grid_width;

    for step in 1..=reach {
        let Some(next_x) = x.checked_add_signed(dx * step as isize) else {
            break;
        };
        let Some(next_y) = y.checked_add_signed(dy * step as isize) else {
            break;
        };
        if next_x >= grid_width || next_y >= grid_width {
            break;
        }

        let center = &map.centers[next_x * grid_width + next_y];
        if center.biome == "SHALLOW_OCEAN" {
            return true;
        }
        if !center.ocean {
            return false;
        }
    }

    false
}

fn deep_ocean_cells_left_by_mask_closing(
    map: &PolyMap,
    shallow_sea_size: ShallowSeaSize,
) -> Vec<usize> {
    let grid_width = (map.centers.len() as f32).sqrt() as usize;
    let radius = mask_closing_radius(shallow_sea_size, grid_width);
    let max_distance = shallow_sea_size.guaranteed_shallow_distance() + radius as i32 + 1;
    let inside_mask: Vec<_> = map
        .centers
        .iter()
        .map(|center| !center.ocean || center.shallow_ocean)
        .collect();
    let dilated = dilate_square_mask(&inside_mask, grid_width, radius);
    let closed = erode_square_mask(&dilated, grid_width, radius);

    map.centers
        .iter()
        .filter(|center| {
            center.biome == "DEEP_OCEAN"
                && !center.border
                && center.ocean_distance <= max_distance
                && closed[center.index]
        })
        .map(|center| center.index)
        .collect()
}

fn mask_closing_radius(shallow_sea_size: ShallowSeaSize, grid_width: usize) -> usize {
    let base = BayRounding::Strong.base_closing_radius(shallow_sea_size);
    ((base as f32 * grid_width as f32 / 63.0).round() as usize).max(base)
}

fn dilate_square_mask(mask: &[bool], grid_width: usize, radius: usize) -> Vec<bool> {
    let mut out = vec![false; mask.len()];
    for (center_id, value) in out.iter_mut().enumerate() {
        let x = center_id / grid_width;
        let y = center_id % grid_width;
        *value = each_square_neighbor(grid_width, x, y, radius).any(|neighbor| mask[neighbor]);
    }
    out
}

fn erode_square_mask(mask: &[bool], grid_width: usize, radius: usize) -> Vec<bool> {
    let mut out = vec![false; mask.len()];
    for (center_id, value) in out.iter_mut().enumerate() {
        let x = center_id / grid_width;
        let y = center_id % grid_width;
        *value = square_fits_grid(grid_width, x, y, radius)
            && each_square_neighbor(grid_width, x, y, radius).all(|neighbor| mask[neighbor]);
    }
    out
}

fn square_fits_grid(grid_width: usize, x: usize, y: usize, radius: usize) -> bool {
    x >= radius && y >= radius && x + radius < grid_width && y + radius < grid_width
}

fn each_square_neighbor(
    grid_width: usize,
    x: usize,
    y: usize,
    radius: usize,
) -> impl Iterator<Item = usize> {
    let min_x = x.saturating_sub(radius);
    let max_x = (x + radius).min(grid_width - 1);
    let min_y = y.saturating_sub(radius);
    let max_y = (y + radius).min(grid_width - 1);

    (min_x..=max_x)
        .flat_map(move |next_x| (min_y..=max_y).map(move |next_y| next_x * grid_width + next_y))
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
    assert_eq!(DEFAULT_SHALLOW_SEA_SIZE, ShallowSeaSize::Wide);
    assert_eq!(DEFAULT_BAY_ROUNDING, BayRounding::Light);
    assert_eq!(DEFAULT_VIEW_MODE, ViewMode::Biome);
}

#[test]
fn debug_env_accepts_only_remaining_controls() {
    assert_eq!(IslandType::from_debug_env("RADIAL"), None);
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
    assert_eq!(
        ShallowSeaSize::from_debug_env("narrow"),
        Some(ShallowSeaSize::Narrow)
    );
    assert_eq!(
        ShallowSeaSize::from_debug_env("wide"),
        Some(ShallowSeaSize::Wide)
    );
    assert_eq!(
        BayRounding::from_debug_env("light"),
        Some(BayRounding::Light)
    );
    assert_eq!(
        BayRounding::from_debug_env("normal"),
        Some(BayRounding::Normal)
    );
    assert_eq!(
        BayRounding::from_debug_env("strong"),
        Some(BayRounding::Strong)
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
    let left_panel = Rect::new(0.0, 0.0, 256.0, 768.0);
    let field = seed_field_rect(left_panel);

    assert!(field.contains(vec2(80.0, 29.0)));
    assert!(field.contains(vec2(166.0, 51.0)));
    assert!(!field.contains(vec2(78.0, 29.0)));
}

#[test]
fn random_seed_keeps_visible_seed_value() {
    let mut scene = MapgenScene {
        seed_text: DEFAULT_SEED_TEXT.to_string(),
        seed_edit_text: String::new(),
        seed_input_active: true,
        seed_replace_on_type: true,
        island_type: DEFAULT_ISLAND_TYPE,
        point_type: DEFAULT_POINT_TYPE,
        point_count: DEFAULT_POINT_COUNT,
        shallow_sea_size: DEFAULT_SHALLOW_SEA_SIZE,
        bay_rounding: DEFAULT_BAY_ROUNDING,
        view_mode: DEFAULT_VIEW_MODE,
        map: None,
        generation: None,
        pan: Vec2::ZERO,
        zoom: MIN_ZOOM,
    };

    scene.apply_random_seed_text("12345-6".to_string());

    assert_eq!(scene.seed_text, "12345-6");
    assert_eq!(scene.seed_edit_text, "12345-6");
    assert!(!scene.seed_input_active);
    assert!(!scene.seed_replace_on_type);
}

#[test]
fn wide_mapgen_layout_centers_square_map_in_map_area() {
    let layout = mapgen_layout(1536.0, 768.0);

    assert_eq!(layout.left_panel_rect, Rect::new(0.0, 0.0, 260.0, 768.0));
    assert_eq!(layout.map_area_rect, Rect::new(260.0, 0.0, 1016.0, 768.0));
    assert_eq!(
        layout.right_panel_rect,
        Rect::new(1276.0, 0.0, 260.0, 768.0)
    );
    assert_eq!(layout.map_rect.x, 384.0);
    assert_eq!(layout.map_rect.y, 0.0);
    assert_eq!(layout.map_rect.w, 768.0);
    assert_eq!(layout.map_rect.h, 768.0);
}

#[test]
fn biome_list_origin_uses_right_panel_top() {
    let right_panel = Rect::new(1276.0, 0.0, 260.0, 768.0);

    assert_eq!(render::biome_list_origin(right_panel), vec2(1294.0, 32.0));
}

#[test]
fn view_group_sits_below_bay_rounding_group() {
    let left_panel = Rect::new(0.0, 0.0, 260.0, 768.0);

    assert_eq!(
        render::view_group_rect(left_panel),
        Rect::new(16.0, 328.0, 232.0, 62.0)
    );
}

#[test]
fn wide_map_area_uses_neutral_backdrop_color() {
    assert_eq!(render::map_area_background_color(), 0x6f7074);
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
    let map = generate_map(
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
fn remaining_island_shape_buttons_change_inside_function() {
    let perlin = IslandProfile::new(IslandType::Perlin, 85882, DEFAULT_POINT_COUNT);
    let simplex = IslandProfile::new(IslandType::Simplex, 85882, DEFAULT_POINT_COUNT);

    let sample_points = [
        vec2(-0.75, -0.75),
        vec2(-0.55, -0.2),
        vec2(-0.25, 0.35),
        vec2(0.15, -0.65),
        vec2(0.45, 0.45),
        vec2(0.75, 0.1),
    ];

    assert!(
        sample_points
            .into_iter()
            .any(|point| simplex.inside(point) != perlin.inside(point))
    );
}

#[test]
fn demo_control_labels_match_reference() {
    let island_labels: Vec<_> = IslandType::ALL.iter().map(|kind| kind.label()).collect();
    let shallow_labels: Vec<_> = ShallowSeaSize::ALL
        .iter()
        .map(|size| size.label())
        .collect();
    let bay_rounding_labels: Vec<_> = BayRounding::ALL
        .iter()
        .map(|rounding| rounding.label())
        .collect();
    let view_labels: Vec<_> = ViewMode::ALL.iter().map(|mode| mode.label()).collect();

    assert_eq!(island_labels, ["Perlin", "Simplex"]);
    assert_eq!(shallow_labels, ["Narrow", "Normal", "Wide"]);
    assert_eq!(bay_rounding_labels, ["Light", "Normal", "Strong"]);
    assert_eq!(view_labels, ["Biomes", "2D slopes"]);
    assert_eq!(&POINT_COUNTS[..], &[4000, 8000, 16000, 32000]);
}

#[test]
fn island_button_positions_cover_all_island_shapes() {
    assert_eq!(island_button_x_positions().len(), IslandType::ALL.len());
}

#[test]
fn removed_controls_are_not_accepted_from_debug_env() {
    assert_eq!(IslandType::from_debug_env("radial"), None);
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
    assert_eq!(ShallowSeaSize::from_debug_env("huge"), None);
    assert_eq!(ShallowSeaSize::from_debug_env("verywide"), None);
    assert_eq!(ShallowSeaSize::from_debug_env("very wide"), None);
    assert_eq!(BayRounding::from_debug_env("huge"), None);
    assert_eq!(BayRounding::from_debug_env("none"), None);
}

#[test]
fn removed_point_counts_are_not_available() {
    for removed_count in [500, 1000, 2000] {
        assert!(!POINT_COUNTS.contains(&removed_count));
    }
}

#[test]
fn map_rng_wrappers_are_deterministic_and_ranged() {
    let mut first = map_rng(1234);
    let mut second = map_rng(1234);

    assert_eq!(map_random_u32(&mut first), map_random_u32(&mut second));
    assert_eq!(
        map_random_i32(&mut first, 1..=6),
        map_random_i32(&mut second, 1..=6)
    );

    for _ in 0..100 {
        let value = map_random_f32(&mut first, 0.2..0.8);
        assert!((0.2..0.8).contains(&value));
    }
}

#[test]
fn library_noise_wrappers_are_deterministic_and_seeded() {
    let a = fractal_noise_2d(12.5, 37.25, 42);
    let b = fractal_noise_2d(12.5, 37.25, 42);
    let c = fractal_noise_2d(12.5, 37.25, 43);

    assert_eq!(a, b);
    assert_ne!(a, c);
    assert!((0.0..=1.0).contains(&a));

    let simplex_a = simplex_fractal_noise_2d(0.2, -0.4, 42);
    let simplex_b = simplex_fractal_noise_2d(0.2, -0.4, 42);
    let simplex_c = simplex_fractal_noise_2d(0.2, -0.4, 43);

    assert_eq!(simplex_a, simplex_b);
    assert_ne!(simplex_a, simplex_c);
    assert!((0.0..=1.0).contains(&simplex_a));
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
fn zoomed_out_source_rect_matches_generated_map_bounds() {
    let rect = map_source_rect(Vec2::ZERO, MIN_ZOOM);

    assert_eq!(rect.x, 0.0);
    assert_eq!(rect.y, 0.0);
    assert_eq!(rect.w, MAP_SIZE);
    assert_eq!(rect.h, MAP_SIZE);
}

#[test]
fn pan_is_clamped_to_visible_map_bounds() {
    let pan = clamp_pan(vec2(999.0, -999.0), 4.0);

    assert_eq!(pan, vec2(225.0, -225.0));
}

#[test]
fn square_4000_generation_completes() {
    let map = generate_map(
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
    let first = generate_map(
        DEFAULT_SEED_TEXT,
        IslandType::Perlin,
        PointType::Square,
        4000,
    );
    let second = generate_map(
        DEFAULT_SEED_TEXT,
        IslandType::Perlin,
        PointType::Square,
        4000,
    );

    assert_eq!(map_fingerprint(&first), map_fingerprint(&second));
}

#[test]
fn changing_shape_seed_changes_generated_map() {
    let first = generate_map("85882-8", IslandType::Perlin, PointType::Square, 4000);
    let second = generate_map("85883-8", IslandType::Perlin, PointType::Square, 4000);

    assert_ne!(map_fingerprint(&first), map_fingerprint(&second));
}

#[test]
fn changing_seed_variant_changes_generated_map() {
    let first = generate_map("85882-8", IslandType::Perlin, PointType::Square, 4000);
    let second = generate_map("85882-9", IslandType::Perlin, PointType::Square, 4000);

    assert_ne!(map_fingerprint(&first), map_fingerprint(&second));
}

#[test]
fn simplex_square_generation_builds_valid_island() {
    let map = generate_map(
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
    let first = generate_map(
        DEFAULT_SEED_TEXT,
        IslandType::Simplex,
        PointType::Square,
        4000,
    );
    let second = generate_map(
        DEFAULT_SEED_TEXT,
        IslandType::Simplex,
        PointType::Square,
        4000,
    );

    assert_eq!(map_fingerprint(&first), map_fingerprint(&second));
}

#[test]
fn simplex_generates_different_map_from_perlin() {
    let simplex = generate_map(
        DEFAULT_SEED_TEXT,
        IslandType::Simplex,
        PointType::Square,
        4000,
    );
    let perlin = generate_map(
        DEFAULT_SEED_TEXT,
        IslandType::Perlin,
        PointType::Square,
        4000,
    );

    assert_ne!(map_fingerprint(&simplex), map_fingerprint(&perlin));
}

#[test]
fn square_point_regions_remain_axis_aligned_cells() {
    let map = generate_map(
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
    let map = generate_map(
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
    let map = generate_map(
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
    let map = generate_map(
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
    let map = generate_map(
        DEFAULT_SEED_TEXT,
        IslandType::Perlin,
        PointType::Square,
        4000,
    );

    for center in &map.centers {
        if center.ocean {
            assert!(matches!(center.biome, "SHALLOW_OCEAN" | "DEEP_OCEAN"));
        } else if center.water {
            assert!(matches!(center.biome, "MARSH" | "LAKE"));
        } else if center.coast {
            assert_eq!(center.biome, "BEACH");
        } else {
            assert!(!matches!(
                center.biome,
                "OCEAN" | "SHALLOW_OCEAN" | "DEEP_OCEAN" | "MARSH" | "LAKE"
            ));
        }
    }
}

#[test]
fn ice_is_not_a_generated_biome() {
    let high_water = Center {
        index: 0,
        point: vec2(0.0, 0.0),
        water: true,
        ocean: false,
        shallow_ocean: false,
        ocean_distance: -1,
        coast: false,
        border: false,
        biome: "",
        elevation: 0.95,
        moisture: 0.5,
        neighbors: Vec::new(),
        borders: Vec::new(),
        corners: Vec::new(),
    };

    assert_eq!(biome::get_biome(&high_water), "LAKE");
    assert_eq!(biome::biome_color("ICE"), 0x000000);
}

#[test]
fn renamed_land_biomes_match_ui_names_and_colors() {
    let mut highland = biome_probe_center(0.85, 0.2);
    let mut peak = biome_probe_center(0.85, 0.1);
    let mut rocky_plains = biome_probe_center(0.5, 0.1);
    let mut desert = biome_probe_center(0.1, 0.1);
    let mut wet_forest = biome_probe_center(0.45, 0.9);
    let mut forest = biome_probe_center(0.45, 0.6);
    let mut meadow = biome_probe_center(0.56, 0.6);
    let mut rainforest = biome_probe_center(0.1, 0.8);
    let mut woodland = biome_probe_center(0.1, 0.5);

    highland.biome = biome::get_biome(&highland);
    peak.biome = biome::get_biome(&peak);
    rocky_plains.biome = biome::get_biome(&rocky_plains);
    desert.biome = biome::get_biome(&desert);
    wet_forest.biome = biome::get_biome(&wet_forest);
    forest.biome = biome::get_biome(&forest);
    meadow.biome = biome::get_biome(&meadow);
    rainforest.biome = biome::get_biome(&rainforest);
    woodland.biome = biome::get_biome(&woodland);

    assert_eq!(highland.biome, "HIGHLANDS");
    assert_eq!(peak.biome, "PEAK");
    assert_eq!(rocky_plains.biome, "ROCKY_PLAINS");
    assert_eq!(desert.biome, "DESERT");
    assert_eq!(wet_forest.biome, "FOREST");
    assert_eq!(forest.biome, "FOREST");
    assert_eq!(meadow.biome, "MEADOW");
    assert_eq!(rainforest.biome, "RAINFOREST");
    assert_eq!(woodland.biome, "WOODLAND");
    assert_eq!(biome::biome_color("SNOW"), 0xe5ffff);
    assert_eq!(biome::biome_color("PEAK"), 0xffffff);
    assert_eq!(biome::biome_color("BARE"), 0x000000);
    assert_eq!(biome::biome_color("SCORCHED"), 0x000000);
    assert_eq!(biome::biome_color("TEMPERATE_DESERT"), 0x000000);
    assert_eq!(biome::biome_color("SUBTROPICAL_DESERT"), 0x000000);
    assert_eq!(biome::biome_color("TEMPERATE_RAIN_FOREST"), 0x000000);
    assert_eq!(biome::biome_color("TEMPERATE_DECIDUOUS_FOREST"), 0x000000);
    assert_eq!(biome::biome_color("TROPICAL_RAIN_FOREST"), 0x000000);
    assert_eq!(biome::biome_color("TROPICAL_SEASONAL_FOREST"), 0x000000);

    let mut map = generate_map(
        DEFAULT_SEED_TEXT,
        IslandType::Perlin,
        PointType::Square,
        4000,
    );
    for center in &mut map.centers {
        center.biome = "DEEP_OCEAN";
    }
    map.centers[0].biome = highland.biome;
    map.centers[1].biome = peak.biome;
    map.centers[2].biome = rocky_plains.biome;
    map.centers[3].biome = desert.biome;
    map.centers[4].biome = forest.biome;
    map.centers[5].biome = meadow.biome;
    map.centers[6].biome = rainforest.biome;
    map.centers[7].biome = woodland.biome;

    let counts = map.biome_counts();
    let visible_names: Vec<_> = counts.iter().map(|entry| entry.name).collect();
    assert!(visible_names.contains(&"Highlands"));
    assert!(visible_names.contains(&"Peak"));
    assert!(visible_names.contains(&"Rocky Plains"));
    assert!(visible_names.contains(&"Desert"));
    assert!(visible_names.contains(&"Forest"));
    assert!(visible_names.contains(&"Meadow"));
    assert!(visible_names.contains(&"Rainforest"));
    assert!(visible_names.contains(&"Woodland"));
}

fn biome_probe_center(elevation: f32, moisture: f32) -> Center {
    Center {
        index: 0,
        point: vec2(0.0, 0.0),
        water: false,
        ocean: false,
        shallow_ocean: false,
        ocean_distance: -1,
        coast: false,
        border: false,
        biome: "",
        elevation,
        moisture,
        neighbors: Vec::new(),
        borders: Vec::new(),
        corners: Vec::new(),
    }
}

#[test]
fn rivers_lakes_and_shallow_ocean_share_lake_color() {
    let lake_color = biome::biome_color("LAKE");

    assert_eq!(lake_color, 0x336699);
    assert_eq!(biome::biome_color("SHALLOW_OCEAN"), lake_color);
    assert_eq!(biome::biome_color("RIVER"), lake_color);
    assert_eq!(biome::biome_color("LAKESHORE"), lake_color);
}

#[test]
fn lake_shores_use_the_same_dark_outline_as_ocean_coasts() {
    let land = biome_probe_center(0.4, 0.4);
    let mut lake = biome_probe_center(0.4, 0.4);
    lake.water = true;
    lake.biome = "LAKE";
    let mut marsh = biome_probe_center(0.05, 0.4);
    marsh.water = true;
    marsh.biome = "MARSH";
    let mut ocean = biome_probe_center(0.0, 0.0);
    ocean.water = true;
    ocean.ocean = true;
    ocean.biome = "DEEP_OCEAN";

    assert_eq!(
        render::edge_stroke_style(&ocean, &land, 0),
        Some((2.0, 0x33335a))
    );
    assert_eq!(
        render::edge_stroke_style(&lake, &land, 0),
        Some((2.0, 0x33335a))
    );
    assert_eq!(
        render::edge_stroke_style(&marsh, &land, 0),
        Some((1.0, biome::LAKE_WATER_COLOR))
    );
    assert_eq!(
        render::edge_stroke_style(&land, &land, 4),
        Some((2.0, biome::LAKE_WATER_COLOR))
    );
}

#[test]
fn biome_counts_list_present_biomes_in_display_order() {
    let map = generate_map(
        DEFAULT_SEED_TEXT,
        IslandType::Perlin,
        PointType::Square,
        4000,
    );
    let counts = map.biome_counts();

    assert!(!counts.is_empty());
    assert!(counts.iter().all(|entry| entry.count > 0));
    assert_eq!(counts[0].name, "Deep Ocean");
    assert_eq!(
        counts.iter().map(|entry| entry.count).sum::<usize>(),
        map.centers.len()
    );
}

#[test]
fn ocean_biomes_split_into_shallow_and_deep() {
    let map = generate_map(
        DEFAULT_SEED_TEXT,
        IslandType::Perlin,
        PointType::Square,
        4000,
    );

    assert!(
        map.centers
            .iter()
            .any(|center| center.biome == "SHALLOW_OCEAN")
    );
    assert!(
        map.centers
            .iter()
            .any(|center| center.biome == "DEEP_OCEAN")
    );
    assert!(
        map.centers
            .iter()
            .filter(|center| center.ocean)
            .all(|center| matches!(center.biome, "SHALLOW_OCEAN" | "DEEP_OCEAN"))
    );
}

#[test]
fn shallow_sea_size_expands_shallow_ocean_without_removing_deep_ocean() {
    let narrow = generate_map_with_shallow_sea(
        DEFAULT_SEED_TEXT,
        IslandType::Perlin,
        PointType::Square,
        4000,
        ShallowSeaSize::Narrow,
        DEFAULT_BAY_ROUNDING,
    );
    let wide = generate_map_with_shallow_sea(
        DEFAULT_SEED_TEXT,
        IslandType::Perlin,
        PointType::Square,
        4000,
        ShallowSeaSize::Wide,
        DEFAULT_BAY_ROUNDING,
    );

    let narrow_shallow = narrow
        .centers
        .iter()
        .filter(|center| center.biome == "SHALLOW_OCEAN")
        .count();
    let wide_shallow = wide
        .centers
        .iter()
        .filter(|center| center.biome == "SHALLOW_OCEAN")
        .count();

    assert!(wide_shallow > narrow_shallow);
    assert!(
        wide.centers
            .iter()
            .any(|center| center.biome == "DEEP_OCEAN")
    );
}

#[test]
fn bay_rounding_expands_only_the_cleanup_strength() {
    let light = generate_map_with_shallow_sea(
        "85882-8",
        IslandType::Perlin,
        PointType::Square,
        4000,
        ShallowSeaSize::Narrow,
        BayRounding::Light,
    );
    let normal = generate_map_with_shallow_sea(
        "85882-8",
        IslandType::Perlin,
        PointType::Square,
        4000,
        ShallowSeaSize::Narrow,
        BayRounding::Normal,
    );
    let strong = generate_map_with_shallow_sea(
        "85882-8",
        IslandType::Perlin,
        PointType::Square,
        4000,
        ShallowSeaSize::Narrow,
        BayRounding::Strong,
    );

    let shallow_count = |map: &PolyMap| {
        map.centers
            .iter()
            .filter(|center| center.biome == "SHALLOW_OCEAN")
            .count()
    };

    assert!(shallow_count(&normal) >= shallow_count(&light));
    assert!(shallow_count(&strong) >= shallow_count(&normal));
    assert!(
        strong
            .centers
            .iter()
            .any(|center| center.biome == "DEEP_OCEAN")
    );
}

#[test]
fn shallow_bays_round_off_near_land_deep_ocean_fingers() {
    let seeds = ["85882-8", "85884-8", "85885-8"];

    for seed in seeds {
        let map = generate_map(seed, IslandType::Perlin, PointType::Square, 4000);

        assert!(
            deep_ocean_finger_cells(&map).is_empty(),
            "{seed} should not leave near-land deep ocean cells pinched between shallow ocean"
        );
    }
}

#[test]
fn wide_shallow_sea_closes_concave_deep_ocean_notches() {
    let map = generate_map_with_shallow_sea(
        "50088-9",
        IslandType::Perlin,
        PointType::Square,
        32000,
        ShallowSeaSize::Wide,
        BayRounding::Strong,
    );

    assert!(
        deep_ocean_concavity_cells(&map, 4, 6).is_empty(),
        "wide shallow sea should close deep-ocean notches that sit between nearby shallow water"
    );
}

#[test]
fn narrow_simplex_shallow_sea_closes_high_resolution_concave_notches() {
    let seeds = ["85882-8", "85882-9", "85884-8", "85885-8", "50088-9"];

    for seed in seeds {
        let map = generate_map_with_shallow_sea(
            seed,
            IslandType::Simplex,
            PointType::Square,
            32000,
            ShallowSeaSize::Narrow,
            BayRounding::Strong,
        );

        assert!(
            deep_ocean_concavity_cells(&map, 4, 4).is_empty(),
            "{seed} should close broad high-resolution Simplex/Narrow concave notches"
        );
    }
}

#[test]
fn narrow_perlin_shallow_sea_closes_high_resolution_inlets() {
    let seeds = ["85882-8", "85882-9", "85884-8", "85885-8", "50088-9"];

    for seed in seeds {
        let map = generate_map_with_shallow_sea(
            seed,
            IslandType::Perlin,
            PointType::Square,
            32000,
            ShallowSeaSize::Narrow,
            BayRounding::Strong,
        );

        assert!(
            deep_ocean_cells_left_by_mask_closing(&map, ShallowSeaSize::Narrow).is_empty(),
            "{seed} should close broad high-resolution Perlin/Narrow deep-ocean inlets"
        );
    }
}

#[test]
fn shallow_ocean_stays_closer_to_land_than_deep_ocean() {
    let map = generate_map(
        DEFAULT_SEED_TEXT,
        IslandType::Perlin,
        PointType::Square,
        4000,
    );

    let shallow_touching_land = map.centers.iter().any(|center| {
        center.biome == "SHALLOW_OCEAN"
            && center
                .neighbors
                .iter()
                .any(|&neighbor| !map.centers[neighbor].water)
    });
    let deep_touching_land = map.centers.iter().any(|center| {
        center.biome == "DEEP_OCEAN"
            && center
                .neighbors
                .iter()
                .any(|&neighbor| !map.centers[neighbor].water)
    });

    assert!(shallow_touching_land);
    assert!(!deep_touching_land);
}

#[test]
fn perlin_keeps_five_outer_cells_as_deep_ocean_buffer() {
    let map = generate_map(
        DEFAULT_SEED_TEXT,
        IslandType::Perlin,
        PointType::Square,
        4000,
    );
    let cell = MAP_SIZE / 63.0;
    let deep_buffer = cell * 5.0;
    let relaxed_band = cell * 8.0;
    let mut found_non_deep_ocean_after_buffer = false;

    for center in &map.centers {
        let distance_from_edge = center
            .point
            .x
            .min(center.point.y)
            .min(MAP_SIZE - center.point.x)
            .min(MAP_SIZE - center.point.y);
        if distance_from_edge <= deep_buffer {
            assert!(
                center.ocean && center.biome == "DEEP_OCEAN",
                "expected outer Perlin cell to be deep ocean at {:?}, got {}",
                center.point,
                center.biome
            );
        } else if distance_from_edge <= relaxed_band && center.biome != "DEEP_OCEAN" {
            found_non_deep_ocean_after_buffer = true;
        }
    }

    assert!(
        found_non_deep_ocean_after_buffer,
        "Perlin edge buffer should relax after roughly five cells"
    );
}

#[test]
fn simplex_is_not_forced_to_perlin_two_cell_edge_buffer() {
    let map = generate_map(
        DEFAULT_SEED_TEXT,
        IslandType::Simplex,
        PointType::Square,
        4000,
    );
    let cell = MAP_SIZE / 63.0;
    let buffer = cell * 2.0;

    assert!(map.centers.iter().any(|center| {
        let distance_from_edge = center
            .point
            .x
            .min(center.point.y)
            .min(MAP_SIZE - center.point.x)
            .min(MAP_SIZE - center.point.y);
        distance_from_edge <= buffer && center.biome != "DEEP_OCEAN"
    }));
}

#[test]
fn enclosed_deep_ocean_components_are_promoted_to_shallow() {
    let map = generate_map(
        DEFAULT_SEED_TEXT,
        IslandType::Perlin,
        PointType::Square,
        4000,
    );

    assert!(
        deep_ocean_components(&map)
            .iter()
            .all(|component| !deep_component_is_fully_surrounded_by_shallow(&map, component)),
        "deep ocean components fully surrounded by shallow ocean should be shallow"
    );
}

#[test]
fn open_deep_ocean_survives_shallow_cleanup() {
    let map = generate_map(
        DEFAULT_SEED_TEXT,
        IslandType::Perlin,
        PointType::Square,
        4000,
    );

    assert!(
        deep_ocean_components(&map).iter().any(|component| component
            .iter()
            .any(|&center_id| map.centers[center_id].border)),
        "cleanup must preserve the border-connected open deep ocean"
    );
}

#[test]
fn landmasses_are_connected_through_shallow_ocean() {
    let seeds = ["85882-8", "85882-9", "85883-8", "85884-8"];

    for seed in seeds {
        let map = generate_map(seed, IslandType::Perlin, PointType::Square, 4000);
        let land_components = passable_land_and_shallow_components(&map)
            .into_iter()
            .filter(|component| {
                component
                    .iter()
                    .any(|&center_id| !map.centers[center_id].water)
            })
            .count();

        assert_eq!(
            land_components, 1,
            "{seed} should connect every island to the mainland through shallow ocean"
        );
    }
}

#[test]
fn shallow_bridges_do_not_remove_open_deep_ocean() {
    let map = generate_map("85882-8", IslandType::Perlin, PointType::Square, 4000);

    assert!(
        deep_ocean_components(&map).iter().any(|component| component
            .iter()
            .any(|&center_id| map.centers[center_id].border)),
        "shallow bridge cleanup must preserve the border-connected deep ocean boundary"
    );
}

#[test]
fn island_bridges_do_not_depend_on_single_ocean_thread_cells() {
    let seeds = ["85882-8", "85884-8", "85885-8"];

    for seed in seeds {
        let map = generate_map(seed, IslandType::Perlin, PointType::Square, 4000);

        assert!(
            non_coastal_shallow_bridge_articulations(&map).is_empty(),
            "{seed} should not connect islands through one-cell shallow ocean threads"
        );
    }
}

#[test]
fn square_point_type_generates_without_drainage_loops() {
    let map = generate_map(
        DEFAULT_SEED_TEXT,
        IslandType::Perlin,
        PointType::Square,
        4000,
    );

    assert!(map.edges.iter().any(|edge| edge.river > 0));
    assert!(map.centers.iter().any(|center| center.coast));
}

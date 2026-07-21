use super::biome::LAKE_WATER_COLOR;
use super::*;
use macroquad::ui::widgets;
use macroquad::ui::{hash, root_ui};

pub(super) const COAST_OUTLINE_COLOR: u32 = 0x33335a;

pub(super) fn draw(scene: &mut MapgenScene, layout: MapgenLayout, source_rect: Rect) {
    draw_rectangle(
        layout.map_area_rect.x,
        layout.map_area_rect.y,
        layout.map_area_rect.w,
        layout.map_area_rect.h,
        color_from_u32(map_area_background_color()),
    );
    draw_map(
        scene.map.as_ref(),
        scene.view_mode,
        layout.map_rect,
        source_rect,
    );
    draw_rectangle(
        layout.left_panel_rect.x,
        layout.left_panel_rect.y,
        layout.left_panel_rect.w,
        layout.left_panel_rect.h,
        SIDEBAR_COLOR,
    );
    draw_rectangle(
        layout.right_panel_rect.x,
        layout.right_panel_rect.y,
        layout.right_panel_rect.w,
        layout.right_panel_rect.h,
        SIDEBAR_COLOR,
    );
    draw_controls(scene, layout.left_panel_rect);
    draw_seed_field(scene, layout.left_panel_rect);
    draw_biome_list(scene.map.as_ref(), layout.right_panel_rect);
}

pub(super) fn map_area_background_color() -> u32 {
    0x6f7074
}

fn draw_map(map: Option<&PolyMap>, view_mode: ViewMode, map_rect: Rect, source_rect: Rect) {
    draw_rectangle(
        map_rect.x,
        map_rect.y,
        map_rect.w,
        map_rect.h,
        color_from_u32(0x333866),
    );

    if let Some(map) = map {
        draw_polygons(map, view_mode, map_rect, source_rect);
        draw_edges(map, map_rect, source_rect);
    } else {
        draw_text(
            "Generating...",
            map_rect.x + 24.0,
            map_rect.y + 36.0,
            24.0,
            WHITE,
        );
    }
}

fn draw_polygons(map: &PolyMap, view_mode: ViewMode, map_rect: Rect, source_rect: Rect) {
    for center in &map.centers {
        if !center_visible(center, source_rect) {
            continue;
        }
        for &edge_id in &center.borders {
            let noisy_edge = map.noisy_edges.get(edge_id);
            let Some((path0, path1)) =
                noisy_edge.and_then(|edge| edge.path0.as_ref().zip(edge.path1.as_ref()))
            else {
                continue;
            };
            let color = color_from_u32(map.triangle_color(view_mode, center.index, edge_id));
            draw_path_wedge(center.point, path0, map_rect, source_rect, color);
            draw_path_wedge(center.point, path1, map_rect, source_rect, color);
        }
    }
}

fn draw_edges(map: &PolyMap, map_rect: Rect, source_rect: Rect) {
    for edge in &map.edges {
        let (Some(d0), Some(d1), Some(_), Some(_)) = (edge.d0, edge.d1, edge.v0, edge.v1) else {
            continue;
        };
        let a = &map.centers[d0];
        let b = &map.centers[d1];
        if !center_visible(a, source_rect) && !center_visible(b, source_rect) {
            continue;
        }

        let noisy_edge = map.noisy_edges.get(edge.index);
        let Some((path0, path1)) =
            noisy_edge.and_then(|edge| edge.path0.as_ref().zip(edge.path1.as_ref()))
        else {
            continue;
        };

        if let Some((width, color)) = edge_stroke_style(a, b, edge.river) {
            draw_noisy_edge_path(
                path0,
                path1,
                map_rect,
                source_rect,
                width,
                color_from_u32(color),
            );
        }
    }
}

pub(super) fn edge_stroke_style(a: &Center, b: &Center, river: i32) -> Option<(f32, u32)> {
    // Coasts and lake shores use the same dark line so enclosed water reads as a real boundary.
    if a.ocean != b.ocean || is_lake_shore(a, b) {
        Some((2.0, COAST_OUTLINE_COLOR))
    } else if a.water != b.water {
        Some((1.0, LAKE_WATER_COLOR))
    } else if !a.water && !b.water && river > 0 {
        Some(((river as f32).sqrt(), LAKE_WATER_COLOR))
    } else {
        None
    }
}

fn is_lake_shore(a: &Center, b: &Center) -> bool {
    a.water != b.water && (a.biome == "LAKE" || b.biome == "LAKE")
}

fn draw_biome_list(map: Option<&PolyMap>, panel: Rect) {
    let origin = biome_list_origin(panel);
    let x = origin.x;
    let mut y = origin.y;

    draw_text("Biomes:", x, y, 18.0, BLACK);
    let Some(map) = map else {
        return;
    };
    for (index, entry) in map.biome_counts().iter().enumerate() {
        y += 22.0;
        draw_biome_count_row(x, y, index + 1, entry);
    }
}

pub(super) fn biome_list_origin(panel: Rect) -> Vec2 {
    vec2(panel.x + 18.0, panel.y + 32.0)
}

fn draw_controls(scene: &mut MapgenScene, panel: Rect) {
    let x = panel.x + 16.0;
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
            scene.apply_random_seed_text(random_seed_text());
            needs_regenerate = true;
        }

        for (index, island_type) in IslandType::ALL.into_iter().enumerate() {
            let button_x = island_button_x_positions()[index];
            let label = selected_label(island_type.label(), island_type == scene.island_type);
            if ui.button(Some(vec2(button_x, 54.0)), label.as_str()) {
                scene.island_type = island_type;
                needs_regenerate = true;
            }
        }
    });

    widgets::Window::new(
        hash!("mapgen_island_size"),
        vec2(x, 104.0),
        vec2(232.0, 62.0),
    )
    .titlebar(false)
    .movable(false)
    .ui(&mut root_ui(), |ui| {
        ui.label(Some(vec2(72.0, 2.0)), "Island Size:");
        for (index, count) in POINT_COUNTS.into_iter().enumerate() {
            let button_x = [0.0, 56.0, 112.0, 168.0][index];
            let label = selected_label(&count.to_string(), count == scene.point_count);
            if ui.button(Some(vec2(button_x, 28.0)), label.as_str()) {
                scene.point_count = count;
                needs_regenerate = true;
            }
        }
    });

    widgets::Window::new(
        hash!("mapgen_shallow_sea"),
        vec2(x, 178.0),
        vec2(232.0, 62.0),
    )
    .titlebar(false)
    .movable(false)
    .ui(&mut root_ui(), |ui| {
        ui.label(Some(vec2(72.0, 2.0)), "Shallow Sea:");
        for (index, size) in ShallowSeaSize::ALL.into_iter().enumerate() {
            let button_x = [0.0, 60.0, 120.0][index];
            let label = selected_label(size.label(), size == scene.shallow_sea_size);
            if ui.button(Some(vec2(button_x, 28.0)), label.as_str()) {
                scene.shallow_sea_size = size;
                needs_regenerate = true;
            }
        }
    });

    widgets::Window::new(
        hash!("mapgen_bay_rounding"),
        vec2(x, 254.0),
        vec2(232.0, 62.0),
    )
    .titlebar(false)
    .movable(false)
    .ui(&mut root_ui(), |ui| {
        ui.label(Some(vec2(64.0, 2.0)), "Bay Rounding:");
        for (index, rounding) in BayRounding::ALL.into_iter().enumerate() {
            let button_x = [0.0, 56.0, 116.0][index];
            let label = selected_label(rounding.label(), rounding == scene.bay_rounding);
            if ui.button(Some(vec2(button_x, 28.0)), label.as_str()) {
                scene.bay_rounding = rounding;
                needs_regenerate = true;
            }
        }
    });

    let view_rect = view_group_rect(panel);
    widgets::Window::new(hash!("mapgen_view"), view_rect.point(), view_rect.size())
        .titlebar(false)
        .movable(false)
        .ui(&mut root_ui(), |ui| {
            ui.label(Some(vec2(82.0, 2.0)), "View:");
            for (index, mode) in ViewMode::ALL.into_iter().enumerate() {
                let col = index as f32;
                let label = selected_label(mode.label(), mode == scene.view_mode);
                if ui.button(Some(vec2(col * 95.0, 28.0)), label.as_str()) {
                    scene.view_mode = mode;
                }
            }
        });

    if needs_regenerate {
        scene.regenerate();
    }
}

pub(super) fn view_group_rect(panel: Rect) -> Rect {
    Rect::new(panel.x + 16.0, panel.y + 328.0, 232.0, 62.0)
}

fn draw_seed_field(scene: &MapgenScene, panel: Rect) {
    let field = seed_field_rect(panel);
    let border = if scene.seed_input_active { BLACK } else { GRAY };
    let value = if scene.seed_input_active {
        scene.seed_edit_text.as_str()
    } else {
        scene.seed_text.as_str()
    };
    let text = if scene.seed_input_active && (get_time() * 2.0) as i32 % 2 == 0 {
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

fn draw_biome_count_row(x: f32, y: f32, number: usize, entry: &BiomeCount) {
    draw_text(format!("{}.", number), x, y + 12.0, 14.0, BLACK);
    draw_rectangle(x + 22.0, y, 12.0, 12.0, color_from_u32(entry.color));
    draw_rectangle_lines(x + 22.0, y, 12.0, 12.0, 1.0, BLACK);
    draw_text(
        format!("{} - {}", entry.name, entry.count),
        x + 40.0,
        y + 12.0,
        14.0,
        BLACK,
    );
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

fn color_from_u32(color: u32) -> Color {
    Color::new(
        ((color >> 16) & 0xff) as f32 / 255.0,
        ((color >> 8) & 0xff) as f32 / 255.0,
        (color & 0xff) as f32 / 255.0,
        1.0,
    )
}

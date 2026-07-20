use super::*;
use macroquad::ui::widgets;
use macroquad::ui::{hash, root_ui};

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
        layout.sidebar_rect.x,
        layout.sidebar_rect.y,
        layout.sidebar_rect.w,
        layout.sidebar_rect.h,
        SIDEBAR_COLOR,
    );
    draw_histograms(scene.map.as_ref(), layout.sidebar_rect);
    draw_controls(scene, layout.sidebar_rect);
    draw_seed_field(scene, layout.sidebar_rect);
    draw_footer(scene, layout.sidebar_rect);
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
        } else if !a.water && !b.water && edge.river > 0 {
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

fn draw_histograms(map: Option<&PolyMap>, sidebar: Rect) {
    let x = sidebar.x + 25.0;
    let y = 230.0;
    let width = (sidebar.w - 50.0).max(120.0);

    draw_text("Distribution:", sidebar.x + 50.0, y, 18.0, BLACK);
    let Some(map) = map else {
        return;
    };
    draw_distribution(x, y + 12.0, width, 18.0, &map.land_histogram());
    draw_distribution(x, y + 36.0, width, 18.0, &map.biome_histogram());
    draw_histogram(x, y + 88.0, width, 28.0, &map.elevation_histogram());
    draw_histogram(x, y + 126.0, width, 18.0, &map.moisture_histogram());
}

fn draw_controls(scene: &mut MapgenScene, sidebar: Rect) {
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
            scene.seed_text = random_seed_text();
            scene.seed_edit_text = scene.seed_text.clone();
            scene.seed_input_active = false;
            scene.seed_replace_on_type = false;
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
            let label = selected_label(point_type.label(), point_type == scene.point_type);
            if ui.button(Some(vec2(button_x, 28.0)), label.as_str()) {
                scene.point_type = point_type;
                needs_regenerate = true;
            }
        }
        for (index, count) in POINT_COUNTS.into_iter().enumerate() {
            let button_x = [0.0, 56.0, 112.0, 168.0][index];
            let label = selected_label(&count.to_string(), count == scene.point_count);
            if ui.button(Some(vec2(button_x, 58.0)), label.as_str()) {
                scene.point_count = count;
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
            let label = selected_label(mode.label(), mode == scene.view_mode);
            if ui.button(Some(vec2(col * 95.0, 28.0)), label.as_str()) {
                scene.view_mode = mode;
                scene.status = format!("View: {}", mode.label());
            }
        }
    });

    if needs_regenerate {
        scene.regenerate();
    }
}

fn draw_seed_field(scene: &MapgenScene, sidebar: Rect) {
    let field = seed_field_rect(sidebar);
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

fn draw_footer(scene: &MapgenScene, sidebar: Rect) {
    draw_text(
        format!("Zoom {:.1}x", scene.zoom),
        sidebar.x + 16.0,
        724.0,
        16.0,
        BLACK,
    );
    draw_text(&scene.status, sidebar.x + 16.0, 744.0, 16.0, BLACK);
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

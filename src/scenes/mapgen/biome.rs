use super::model::Center;
use macroquad::prelude::{Vec2, vec3};
pub(super) fn get_biome(center: &Center) -> &'static str {
    if center.ocean {
        if center.shallow_ocean {
            "SHALLOW_OCEAN"
        } else {
            "DEEP_OCEAN"
        }
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

pub(super) fn biome_color(biome: &str) -> u32 {
    match biome {
        "OCEAN" | "DEEP_OCEAN" => 0x333866,
        "SHALLOW_OCEAN" => 0x4d6f93,
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

pub(super) fn calculate_lighting(
    center: Vec2,
    center_e: f32,
    a: Vec2,
    a_e: f32,
    b: Vec2,
    b_e: f32,
) -> f32 {
    let ab = vec3(a.x - center.x, a.y - center.y, a_e - center_e);
    let ac = vec3(b.x - center.x, b.y - center.y, b_e - center_e);
    let mut normal = ab.cross(ac);
    if normal.z < 0.0 {
        normal = -normal;
    }
    let normal = normal.normalize_or_zero();
    (0.5 + 35.0 * normal.dot(vec3(-1.0, -1.0, 0.0))).clamp(0.0, 1.0)
}

pub(super) fn interpolate_color(color0: u32, color1: u32, f: f32) -> u32 {
    let f = f.clamp(0.0, 1.0);
    let r = ((1.0 - f) * ((color0 >> 16) as f32) + f * ((color1 >> 16) as f32)).min(255.0) as u32;
    let g = ((1.0 - f) * (((color0 >> 8) & 0xff) as f32) + f * (((color1 >> 8) & 0xff) as f32))
        .min(255.0) as u32;
    let b = ((1.0 - f) * ((color0 & 0xff) as f32) + f * ((color1 & 0xff) as f32)).min(255.0) as u32;
    (r << 16) | (g << 8) | b
}

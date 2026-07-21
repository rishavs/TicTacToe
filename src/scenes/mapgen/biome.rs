use super::model::Center;
use macroquad::prelude::{Vec2, vec3};

pub(super) const LAKE_WATER_COLOR: u32 = 0x336699;

pub(super) fn get_biome(center: &Center) -> &'static str {
    if center.ocean {
        if center.shallow_ocean {
            "SHALLOW_OCEAN"
        } else {
            "DEEP_OCEAN"
        }
    } else if center.river {
        "RIVER"
    } else if center.water {
        if center.elevation < 0.1 {
            "MARSH"
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
            "HIGHLANDS"
        } else {
            "PEAK"
        }
    } else if center.elevation > 0.6 {
        if center.moisture > 0.66 {
            "TAIGA"
        } else if center.moisture > 0.33 {
            "SHRUBLAND"
        } else {
            "ROCKY_PLAINS"
        }
    } else if center.elevation > 0.3 {
        if center.elevation > 0.55 && center.moisture > 0.50 {
            "MEADOW"
        } else if center.moisture > 0.50 {
            "FOREST"
        } else if center.moisture > 0.16 {
            "GRASSLAND"
        } else {
            "ROCKY_PLAINS"
        }
    } else if center.moisture > 0.66 {
        "RAINFOREST"
    } else if center.moisture > 0.33 {
        "WOODLAND"
    } else if center.moisture > 0.16 {
        "GRASSLAND"
    } else {
        "DESERT"
    }
}

pub(super) fn biome_color(biome: &str) -> u32 {
    match biome {
        "OCEAN" | "DEEP_OCEAN" => 0x333866,
        "SHALLOW_OCEAN" => LAKE_WATER_COLOR,
        "COAST" => 0x33335a,
        "LAKESHORE" => LAKE_WATER_COLOR,
        "LAKE" => LAKE_WATER_COLOR,
        "RIVER" => LAKE_WATER_COLOR,
        "MARSH" => 0x2f6666,
        "BEACH" => 0xa09077,
        "SNOW" => 0xe5ffff,
        "TUNDRA" => 0xbbbbaa,
        "HIGHLANDS" => 0x888888,
        "PEAK" => 0xffffff,
        "TAIGA" => 0x99aa77,
        "SHRUBLAND" => 0x889977,
        "ROCKY_PLAINS" => 0xc9d29b,
        "FOREST" => 0x679459,
        "MEADOW" => 0xa8bf70,
        "GRASSLAND" => 0x88aa55,
        "DESERT" => 0xd2b98b,
        "RAINFOREST" => 0x337755,
        "WOODLAND" => 0x559944,
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

use super::{IslandProfile, MapRng};
use macroquad::prelude::Vec2;
use std::collections::HashMap;

#[derive(Clone)]
pub(super) struct Center {
    pub(super) index: usize,
    pub(super) point: Vec2,
    pub(super) water: bool,
    pub(super) ocean: bool,
    pub(super) shallow_ocean: bool,
    pub(super) ocean_distance: i32,
    pub(super) coast: bool,
    pub(super) border: bool,
    pub(super) biome: &'static str,
    pub(super) elevation: f32,
    pub(super) moisture: f32,
    pub(super) neighbors: Vec<usize>,
    pub(super) borders: Vec<usize>,
    pub(super) corners: Vec<usize>,
}

#[derive(Clone)]
pub(super) struct Corner {
    pub(super) index: usize,
    pub(super) point: Vec2,
    pub(super) ocean: bool,
    pub(super) water: bool,
    pub(super) coast: bool,
    pub(super) border: bool,
    pub(super) elevation: f32,
    pub(super) moisture: f32,
    pub(super) touches: Vec<usize>,
    pub(super) protrudes: Vec<usize>,
    pub(super) adjacent: Vec<usize>,
    pub(super) river: i32,
    pub(super) downslope: usize,
    pub(super) watershed: usize,
    pub(super) watershed_size: i32,
}

#[derive(Clone)]
pub(super) struct Edge {
    pub(super) index: usize,
    pub(super) d0: Option<usize>,
    pub(super) d1: Option<usize>,
    pub(super) v0: Option<usize>,
    pub(super) v1: Option<usize>,
    pub(super) midpoint: Vec2,
    pub(super) river: i32,
}

#[derive(Clone, Default)]
pub(super) struct NoisyEdge {
    pub(super) path0: Option<Vec<Vec2>>,
    pub(super) path1: Option<Vec<Vec2>>,
}

pub(super) struct PolyMap {
    pub(super) map_random: MapRng,
    pub(super) island_shape: IslandProfile,
    pub(super) centers: Vec<Center>,
    pub(super) corners: Vec<Corner>,
    pub(super) edges: Vec<Edge>,
    pub(super) noisy_edges: Vec<NoisyEdge>,
    pub(super) center_watersheds: Vec<Option<usize>>,
    pub(super) edge_by_corners: HashMap<(usize, usize), usize>,
}

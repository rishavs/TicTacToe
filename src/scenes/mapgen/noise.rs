use super::IslandType;
use ::noise::{Fbm, MultiFractal, NoiseFn, OpenSimplex, Perlin};
use macroquad::prelude::Vec2;

const PERLIN_DEEP_OCEAN_EDGE_BUFFER_CELLS: f32 = 2.0;
const PERLIN_SHALLOW_SHELF_CELLS: f32 = 3.0;
const PERLIN_EDGE_SOFTNESS_CELLS: f32 = 2.0;
const PERLIN_BASE_THRESHOLD: f32 = 0.3;
const PERLIN_RADIAL_FALLOFF: f32 = 0.3;
const PERLIN_EDGE_BIAS: f32 = 0.35;
const SIMPLEX_BASE_THRESHOLD: f32 = 0.34;
const SIMPLEX_RADIAL_FALLOFF: f32 = 0.34;

pub(super) struct IslandProfile {
    kind: IslandType,
    perlin_hard_edge_buffer: f32,
    perlin_soft_edge_buffer: f32,
    perlin_noise: Fbm<Perlin>,
    simplex_noise: OpenSimplex,
}

impl IslandProfile {
    pub(super) fn new(kind: IslandType, seed: u32, point_count: usize) -> Self {
        let grid_width = (point_count as f32).sqrt().floor().max(1.0);
        Self {
            kind,
            perlin_hard_edge_buffer: grid_cells_to_normalized_distance(
                PERLIN_DEEP_OCEAN_EDGE_BUFFER_CELLS + PERLIN_SHALLOW_SHELF_CELLS,
                grid_width,
            ),
            perlin_soft_edge_buffer: grid_cells_to_normalized_distance(
                PERLIN_EDGE_SOFTNESS_CELLS,
                grid_width,
            ),
            perlin_noise: make_fractal_noise(seed),
            simplex_noise: OpenSimplex::new(seed),
        }
    }

    pub(super) fn inside(&self, q: Vec2) -> bool {
        match self.kind {
            IslandType::Perlin => {
                let c = sample_fractal_noise(
                    &self.perlin_noise,
                    (q.x + 1.0) * 128.0,
                    (q.y + 1.0) * 128.0,
                );
                let edge_distance = 1.0 - q.x.abs().max(q.y.abs());
                if edge_distance <= self.perlin_hard_edge_buffer {
                    return false;
                }
                let edge_blend = ((edge_distance - self.perlin_hard_edge_buffer)
                    / self.perlin_soft_edge_buffer)
                    .clamp(0.0, 1.0);
                let edge_bias = (1.0 - edge_blend).powi(2) * PERLIN_EDGE_BIAS;
                c > PERLIN_BASE_THRESHOLD + PERLIN_RADIAL_FALLOFF * q.length_squared() + edge_bias
            }
            IslandType::Simplex => {
                let c = sample_simplex_fractal_noise(&self.simplex_noise, q.x * 2.2, q.y * 2.2);
                c > SIMPLEX_BASE_THRESHOLD + SIMPLEX_RADIAL_FALLOFF * q.length_squared()
            }
        }
    }
}

fn grid_cells_to_normalized_distance(cells: f32, grid_width: f32) -> f32 {
    cells * 2.0 / grid_width
}

#[cfg(test)]
pub(super) fn fractal_noise_2d(x: f32, y: f32, seed: u32) -> f32 {
    let noise = make_fractal_noise(seed);
    sample_fractal_noise(&noise, x, y)
}

fn make_fractal_noise(seed: u32) -> Fbm<Perlin> {
    Fbm::<Perlin>::new(seed)
        .set_octaves(8)
        .set_frequency(1.0 / 64.0)
        .set_lacunarity(2.0)
        .set_persistence(0.5)
}

fn sample_fractal_noise(noise: &Fbm<Perlin>, x: f32, y: f32) -> f32 {
    normalize_noise(noise.get([x as f64, y as f64]))
}

#[cfg(test)]
pub(super) fn simplex_fractal_noise_2d(x: f32, y: f32, seed: u32) -> f32 {
    let noise = OpenSimplex::new(seed);
    sample_simplex_fractal_noise(&noise, x, y)
}

fn sample_simplex_fractal_noise(noise: &OpenSimplex, x: f32, y: f32) -> f32 {
    let mut sum = 0.0;
    let mut amplitude = 1.0;
    let mut amplitude_sum = 0.0;
    let mut frequency = 1.0;
    for octave in 0..5 {
        let octave_offset = octave as f64 * 997.0;
        sum += noise.get([
            (x * frequency) as f64 + octave_offset,
            (y * frequency) as f64 - octave_offset,
        ]) as f32
            * amplitude;
        amplitude_sum += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    normalize_noise((sum / amplitude_sum) as f64)
}

fn normalize_noise(value: f64) -> f32 {
    (0.5 + 0.5 * value.clamp(-1.0, 1.0)) as f32
}

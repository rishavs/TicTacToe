use std::ops::{Range, RangeInclusive};

pub(super) type MapRng = macroquad::rand::RandGenerator;

pub(super) fn map_rng(seed: u64) -> MapRng {
    let rng = MapRng::new();
    rng.srand(seed);
    rng
}

#[cfg(test)]
pub(super) fn map_random_u32(rng: &mut MapRng) -> u32 {
    rng.rand()
}

pub(super) fn map_random_i32(rng: &mut MapRng, range: RangeInclusive<i32>) -> i32 {
    rng.gen_range(*range.start(), range.end().saturating_add(1))
}

pub(super) fn map_random_f32(rng: &mut MapRng, range: Range<f32>) -> f32 {
    rng.gen_range(range.start, range.end)
}

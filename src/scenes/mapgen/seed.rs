use super::random::{map_random_i32, map_rng};
use macroquad::prelude::get_time;

pub(super) fn is_seed_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'
}

pub(super) fn push_seed_char(seed_text: &mut String, replace_on_type: &mut bool, ch: char) -> bool {
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

pub(super) fn random_seed_text() -> String {
    let mut rng = map_rng(get_time().to_bits());
    format!(
        "{}-{}",
        map_random_i32(&mut rng, 0..=100_000),
        map_random_i32(&mut rng, 1..=9)
    )
}

pub(super) fn parse_seed(seed_text: &str) -> (u32, u32) {
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

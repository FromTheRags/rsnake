//! # Random Game Options Helper
//!
//! This module provides weighted ("ponderated alea") random generation for [`GameOptions`],
//! ensuring that randomized parameters span broad, exciting ranges while remaining
//! balanced and playable.

use crate::controls::speed::Speed;
use crate::game_logic::game_options::{DISPLAYABLE_EMOJI, GameOptions};
use crate::game_logic::logger::log_configuration::LogLevel;
use rand::{RngExt, rng};
use std::ops::RangeInclusive;

/// Picks a random value from a slice of weighted ranges.
///
/// Each tuple contains `(weight, range)`. A higher weight increases the probability
/// of choosing a value from that range.
#[must_use]
pub fn weighted_range(choices: &[(u32, RangeInclusive<u16>)]) -> u16 {
    let total_weight: u32 = choices.iter().map(|(w, _)| *w).sum();
    let roll: u32 = rng().random_range(1..=total_weight);
    let mut cumulative = 0;
    for (weight, range) in choices {
        cumulative += *weight;
        if roll <= cumulative {
            return rng().random_range(range.clone());
        }
    }
    *choices[0].1.start()
}

/// Generates a new [`GameOptions`] instance with randomized parameters using weighted distributions.
///
/// The resulting options span larger parameter ranges while favoring balanced, playable values.
/// The `random` field on the returned struct is set to `true`.
#[must_use]
pub fn generate_random_game_options() -> GameOptions {
    let speed = match rng().random_range(1..=100) {
        1..=20 => Speed::Slow,
        21..=60 => Speed::Normal,
        61..=90 => Speed::Fast,
        _ => Speed::Crazy,
    };

    let head_idx = rng().random_range(0..DISPLAYABLE_EMOJI.len());
    let mut body_idx = rng().random_range(0..DISPLAYABLE_EMOJI.len());
    while body_idx == head_idx {
        body_idx = rng().random_range(0..DISPLAYABLE_EMOJI.len());
    }
    let head_symbol = DISPLAYABLE_EMOJI[head_idx].to_string();
    let body_symbol = DISPLAYABLE_EMOJI[body_idx].to_string();

    // Weighted distributions spanning broader ranges across the parameter spectrum
    let snake_length = weighted_range(&[
        (45, 3..=12),   // Classic / short
        (30, 13..=30),  // Medium
        (15, 31..=75),  // Long
        (7, 76..=150),  // Very long
        (3, 151..=300), // Epic snake
    ]);

    let life = weighted_range(&[
        (40, 2..=5),   // Standard
        (30, 6..=12),  // Moderate
        (15, 1..=1),   // Hardcore 1-life
        (10, 13..=25), // Generous
        (5, 26..=50),  // Extra lives
    ]);

    let nb_of_fruits = weighted_range(&[
        (40, 3..=8),   // Standard
        (30, 9..=20),  // Bountiful
        (15, 1..=2),   // Scarce
        (10, 21..=50), // Fruit frenzy
        (5, 51..=100), // Mega harvest
    ]);

    let fruit_duration_seconds = weighted_range(&[
        (50, 15..=30), // Balanced
        (25, 5..=14),  // Fast expiry
        (20, 31..=50), // Relaxed
        (5, 51..=60),  // Max duration
    ]);

    let snake_growth_factor = weighted_range(&[
        (60, 1..=1), // Classic fruit effects
        (30, 2..=3), // Noticeably faster growth
        (10, 4..=6), // Chaotic games
    ]);

    let fruit_timer = rng().random_bool(0.8);
    let negative_size_fruits = rng().random_bool(0.5);
    let caps_fps = true;

    let mut options = GameOptions {
        speed,
        head_symbol,
        body_symbol,
        snake_length,
        life,
        nb_of_fruits,
        no_fruit_timer: !fruit_timer,
        fruit_timer,
        fruit_duration_seconds,
        snake_growth_factor,
        no_negative_size_fruits: !negative_size_fruits,
        negative_size_fruits,
        log_level: LogLevel::Off,
        no_caps_fps: !caps_fps,
        caps_fps,
        load: None,
        random: true,
    };
    options.validate();
    options
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_segmentation::UnicodeSegmentation;

    #[test]
    fn test_weighted_range_bounds() {
        let ranges = [(50, 5..=10), (50, 20..=30)];
        for _ in 0..100 {
            let val = weighted_range(&ranges);
            assert!((5..=10).contains(&val) || (20..=30).contains(&val));
        }
    }

    #[test]
    fn test_generate_random_game_options() {
        for _ in 0..100 {
            let opt = generate_random_game_options();
            assert!(opt.snake_length >= 2 && opt.snake_length <= 999);
            assert!(opt.life >= 1 && opt.life <= 99);
            assert!(opt.nb_of_fruits >= 1 && opt.nb_of_fruits <= 999);
            assert!(opt.fruit_duration_seconds >= 1 && opt.fruit_duration_seconds <= 60);
            assert!(opt.snake_growth_factor >= 1 && opt.snake_growth_factor <= 10);
            assert_eq!(opt.head_symbol.graphemes(true).count(), 1);
            assert_eq!(opt.body_symbol.graphemes(true).count(), 1);
            assert_ne!(opt.head_symbol, opt.body_symbol);
            assert!(opt.caps_fps);
            assert_eq!(opt.log_level, LogLevel::Off);
            assert!(
                opt.random,
                "Generated options must have random flag set to true"
            );
            assert_eq!(opt.no_fruit_timer, !opt.fruit_timer);
            assert_eq!(opt.no_negative_size_fruits, !opt.negative_size_fruits);
        }
    }
}

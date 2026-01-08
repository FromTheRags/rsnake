//! # Speed Enum with Static Configuration
//!
//! This module implements a Speed enum that uses static pre-configured values to avoid
//! repeated match expressions and improve performance. The configuration for each speed
//! level is defined once in static constants, which are accessed through accessor methods.
//!
//! # Design Choice
//! This implements the "enum with static data" pattern where:
//! - The enum is kept simple for Clap integration
//! - Associated data is stored in static constants (optional, can be plain after match, but it is less structured)
//! - Accessor methods avoid repeated match statements by using a `config()` method (optional)
//! - Provide an easy way to get all parameters at once, using `config()`
//!
//! A variant will look like: same enum, in `get_name` fn matches self and returns "fast" for fast variant etc.
//! => (no const or associated struct, but less structured (no named variant of parameters))
//!
//! # Example
//! ```rust
//! use rsnaker::controls::speed::Speed;
//!
//! let fast_speed = Speed::Fast;
//! println!("Speed value: {}", fast_speed.ms_value());
//! assert_eq!("Fast", fast_speed.name());
//! ```
//!

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

/// Contains all configuration data for a speed level
#[derive(Debug, Copy, Clone)]
pub struct SpeedConfig {
    /// Delay in milliseconds between moves
    pub ms_value: u64,
    /// Human-readable name of the speed level
    pub name: &'static str,
    /// Score multiplier for this speed level
    pub score_modifier: u16,
    /// Symbol representing this speed level
    pub symbol: &'static str,
}

// Static configuration for each speed level
const SLOW_CONFIG: SpeedConfig = SpeedConfig {
    ms_value: 150,
    name: "Slow",
    score_modifier: 1,
    symbol: "🐢",
};

const NORMAL_CONFIG: SpeedConfig = SpeedConfig {
    ms_value: 125,
    name: "Normal",
    score_modifier: 2,
    symbol: "🐍",
};

const FAST_CONFIG: SpeedConfig = SpeedConfig {
    ms_value: 100,
    name: "Fast",
    score_modifier: 3,
    symbol: "🐉",
};

const CRAZY_CONFIG: SpeedConfig = SpeedConfig {
    ms_value: 80,
    name: "Crazy",
    score_modifier: 4,
    symbol: "🦖",
};

/// Represents speed levels with embedded properties
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ValueEnum, Default)]
pub enum Speed {
    Slow,
    #[default]
    Normal,
    Fast,
    Crazy,
}
impl Display for Speed {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            format!("{}{}", self.name().to_string(), self.symbol())
        )
    }
}

impl Speed {
    /// Returns the configuration for this speed level
    #[must_use]
    pub fn config(&self) -> &'static SpeedConfig {
        match self {
            Speed::Slow => &SLOW_CONFIG,
            Speed::Normal => &NORMAL_CONFIG,
            Speed::Fast => &FAST_CONFIG,
            Speed::Crazy => &CRAZY_CONFIG,
        }
    }

    /// Returns the speed value in milliseconds
    #[must_use]
    pub fn ms_value(&self) -> u64 {
        self.config().ms_value
    }

    /// Returns the name of the speed level
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.config().name
    }

    /// Returns the symbol representing this speed level
    #[must_use]
    pub fn symbol(&self) -> &'static str {
        self.config().symbol
    }

    /// Returns the score modifier for this speed level
    #[must_use]
    pub fn score_modifier(&self) -> u16 {
        self.config().score_modifier
    }
}

/*Returns all speed variants for UI or configuration purposes
Useless as ValueEnum derive does the same (for Clap initially but can be reused elsewhere)
#[must_use]
pub fn iter_speed_variants() -> Vec<Speed> {
    vec![Speed::Slow, Speed::Normal, Speed::Fast, Speed::Crazy]
}
*/

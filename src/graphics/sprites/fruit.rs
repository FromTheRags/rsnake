//! # Fruit Management Module
//!
//! This module defines the `Fruit` struct, which represents different fruits in the game logic and provides the
//!  ability to create, position, and render them.
//!
//! The `FRUITS_SCORES_PROBABILITIES` constant defines various fruits with their respective scores and spawn probabilities.
//!
//! # Example
//! ```rust
//! use rsnaker::graphics::graphic_block::Position;
//! use rsnaker::graphics::sprites::fruit::Fruit;
//! use std::time::Duration;
//!
//! let position = Position { x: 5, y: 10 };
//! let apple = Fruit::new(40, 2, position, "🍎", 5f32, true);
//! assert_eq!(apple.get_score(), 40);
//! ```

use crate::graphics::graphic_block::{GraphicBlock, Position};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::Style;
use ratatui::widgets::Paragraph;
use ratatui::widgets::WidgetRef;
use std::time::{Duration, Instant};

/// Distribution statistics with weighted lottery / pie chart parts.
/// Image, score, probability, size effect
/// Yet pear, or strawberry ("🍓", 60, 5, 15),
/// In order: Symbol, score effect, probability, size effect
pub const FRUITS_SCORES_PROBABILITIES: &[(&str, i32, u16, i16)] = &[
    ("🦞", -50, 5, -150),
    ("🥥", -10, 5, -40),
    ("🍇", 5, 4, 0),
    ("🍐", 10, 10, 5),
    ("🥝", 20, 10, 8),
    ("🍋", 30, 15, 10),
    ("🍌", 40, 15, 15),
    ("🍉", 50, 15, 15),
    ("🍎", 75, 15, 15),
    ("🍓", 100, 5, 20),
    ("🍒", 200, 1, 25),
];

/// Represents a fruit on the map.
/// Fruits have a score value and are displayed as graphical blocks.
#[derive(PartialEq, Debug, Clone)]
pub struct Fruit<'a> {
    score: i32,
    grow_snake: i16,
    graphic_block: GraphicBlock<'a>,
    spawned_at: Instant,
    lifetime: Duration,
    timer_enabled: bool,
    timer_paused_since: Option<Instant>,
    accumulated_paused: Duration,
}

impl<'a> Fruit<'a> {
    /// Returns the lifetime multiplier applied on top of the base duration.
    #[must_use]
    pub fn duration_multiplier(size_effect: i16) -> f32 {
        if size_effect >= 0 {
            (1.0 + (f32::from(size_effect) / 50.0)).min(2.0)
        } else {
            (1.0 + (f32::from(size_effect) / 300.0)).max(0.5)
        }
    }

    /// Computes the lifetime from the base duration and the fruit bonus/malus.
    #[must_use]
    pub fn duration_from_base(base: Duration, size_effect: i16) -> Duration {
        Duration::from_secs_f32(base.as_secs_f32() * Self::duration_multiplier(size_effect))
    }

    /// Creates a new `Fruit` at a given position with an associated score and image.
    #[must_use]
    pub fn new(
        score: i32,
        grow_snake_by_relative_nb: i16,
        position: Position,
        image: &'a str,
        base_lifetime: f32,
        timer_enabled: bool,
    ) -> Fruit<'a> {
        Self {
            score,
            grow_snake: grow_snake_by_relative_nb,
            graphic_block: GraphicBlock::new(position, image, Style::default()),
            spawned_at: Instant::now(),
            lifetime: Duration::from_secs_f32(
                base_lifetime * Self::duration_multiplier(grow_snake_by_relative_nb),
            ),
            timer_enabled,
            timer_paused_since: None,
            accumulated_paused: Duration::ZERO,
        }
    }

    /// Checks if the fruit is at a specific position.
    #[must_use]
    pub fn is_at_position(&self, position: &Position) -> bool {
        self.graphic_block.get_position() == position
    }

    /// Checks if the fruit is at a specific position.
    pub fn set_position(&mut self, position: Position) {
        self.graphic_block.set_position(position);
    }

    /// Pauses or resumes the fruit timer.
    pub fn set_timer_paused(&mut self, paused: bool, now: Instant) {
        match (paused, self.timer_paused_since) {
            (true, None) => {
                // going to pause, keep the pause timing
                // NB timer_paused_since is also a boolean for pause or not with Some/None
                self.timer_paused_since = Some(now);
            }
            (false, Some(paused_since)) => {
                // going to play again, add the pause timing to the elapsed time
                self.accumulated_paused += now.duration_since(paused_since);
                self.timer_paused_since = None;
            }
            _ => {}
        }
    }

    /// Returns true if the fruit timer is enabled and the fruit lifetime is over.
    #[must_use]
    pub fn is_expired(&self, now: Instant) -> bool {
        self.timer_enabled
            && self.timer_paused_since.is_none()
            && self.time_elasped_with_breaks_sub(now) >= self.lifetime
    }

    /// Returns the remaining time before the fruit expires.
    #[must_use]
    pub fn remaining_time(&self, now: Instant) -> Option<Duration> {
        if self.timer_enabled {
            self.lifetime
                .checked_sub(self.time_elasped_with_breaks_sub(now))
        } else {
            None
        }
    }

    /// Returns the score of the fruit.
    #[must_use]
    pub fn get_score(&self) -> i32 {
        self.score
    }

    #[must_use]
    pub fn get_grow_snake(&self) -> i16 {
        self.grow_snake
    }

    fn time_elasped_with_breaks_sub(&self, now: Instant) -> Duration {
        now.duration_since(self.spawned_at).saturating_sub(
            self.accumulated_paused
                + if let Some(pause_time) = self.timer_paused_since {
                    // if we are currently paused, we need to add the pause time to the elapsed time
                    // because it is savec in accumulated time only after going to play again
                    now.duration_since(pause_time)
                } else {
                    Duration::ZERO
                },
        )
    }
}

/// Enables `Fruit` to be rendered as a widget.
impl Widget for Fruit<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.graphic_block.render(area, buf);
    }
}

/// Enables `Fruit` to be rendered as a reference widget.
impl WidgetRef for Fruit<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let now = Instant::now();
        if self.is_expired(now) {
            return;
        }

        self.graphic_block.render_ref(area, buf);
        if !self.timer_enabled {
            return;
        }

        if let Some(remaining) = self.remaining_time(now) {
            let timer_text = format!("{:.1}", remaining.as_secs_f32());
            let position = self.graphic_block.get_position();
            //The fruit manager let the right column free for displaying timing
            // (ugly/strange to change timer position)
            let timer_area = Rect::new(position.x.saturating_add(2), position.y, 4, 1);
            Paragraph::new(timer_text).render(timer_area, buf);
        }
    }
}

//! # Fruit Management Module
//!
//! This module defines the `Fruit` struct, which represents different fruits in the game logic and provides the
//!  ability to create, position, and render them.
//!
//! The `FRUITS_SCORES_PROBABILITIES` constant defines various fruits with their respective scores and spawn probabilities.
//!
//! # Example
//! ```rust
//! use rsnaker::graphics::sprites::fruit::Fruit;
//! use rsnaker::graphics::graphic_block::Position;
//!
//! let position = Position { x: 5, y: 10 };
//! let apple = Fruit::new(40, 2,position, "🍎");
//! assert_eq!(apple.get_score(), 40);
//! ```
//!

use crate::graphics::graphic_block::{GraphicBlock, Position};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::Style;
use ratatui::widgets::WidgetRef;

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
}

impl<'a> Fruit<'a> {
    /// Creates a new `Fruit` at a given position with an associated score and image.
    #[must_use]
    pub fn new(
        score: i32,
        grow_snake_by_relative_nb: i16,
        position: Position,
        image: &'a str,
    ) -> Fruit<'a> {
        Self {
            score,
            grow_snake: grow_snake_by_relative_nb,
            graphic_block: GraphicBlock::new(position, image, Style::default()),
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

    /// Returns the score of the fruit.
    #[must_use]
    pub fn get_score(&self) -> i32 {
        self.score
    }
    #[must_use]
    pub fn get_grow_snake(&self) -> i16 {
        self.grow_snake
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
        self.graphic_block.render_ref(area, buf);
    }
}

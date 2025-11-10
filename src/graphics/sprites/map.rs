use crate::graphics::graphic_block::Position;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, BorderType};

/// A struct representing the game logic map.
/// The map is resizable and represents the writeable area of the terminal, with a defined case size
/// for each cell and a viewport for rendering the game logic world.
///
/// # Fields
/// - `block`: A `Block` widget representing the map's visual border and title.
/// - `CASE_SIZE`: The size of each cell (case) in pixels on the terminal screen.
/// - `viewport`: The dimensions of the terminal viewport for rendering the map.
#[derive(Clone)]
pub struct Map<'a> {
    block: Block<'a>,
    case_size: u16,
    viewport: Rect,
}

impl Map<'_> {
    /// Creates a new `Map` instance with a given case size and viewport.
    ///
    /// # Parameters
    /// - `case_size_in_px`: The size of each case (cell) on the map in pixels.
    /// - `viewport`: The visible area of the terminal where the map will be rendered.
    ///
    /// # Returns
    /// A new `Map` instance with the specified `case_size_in_px` and `viewport`.
    ///
    /// # Note
    /// The map will have a double border and the title "Snake !".
    #[must_use]
    pub fn new<'a>(case_size_in_px: u16, viewport: Rect) -> Map<'a> {
        Map {
            block: Block::bordered().border_type(BorderType::Double),
            //.title("Snake !"),
            case_size: case_size_in_px,
            viewport,
        }
    }
    /// To resize the map viewport, not for in game logic use # unfair, but when after restarting it is OK
    pub fn resize_to_terminal(&mut self, viewport: Rect) {
        self.viewport = viewport;
    }

    /// Determines if a given position is outside the bounds of the map.
    ///
    /// # Parameters
    /// - `Position { x, y }`: The position to check.
    ///
    /// # Returns
    /// `true` if the position is outside the map's viewport, `false` otherwise.
    ///
    /// # Example
    /// ```rust
    /// use ratatui::layout::Rect;
    /// use rsnaker::graphics::graphic_block::Position;
    /// use rsnaker::graphics::sprites::map::Map;
    /// let map = Map::new(10, Rect::new(0, 0, 100, 40));
    /// let position = Position { x: 101, y: 20 };
    /// assert!(map.out_of_map(&position));
    /// ```
    #[must_use]
    pub fn out_of_map(&self, Position { x, y }: &Position) -> bool {
        //+/- 2 to put out the bordered map (last case being ==== )
        let x_max = self.viewport.width - (self.case_size + 2);
        let y_max = self.viewport.height - 2;
        let x_min = self.case_size;
        let y_min = 1;
        *x < x_min || *x > x_max || *y < y_min || *y > y_max
    }

    /// Reverses the position if it is outside the bounds of the map, effectively "wrapping" around
    /// to the opposite edge of the map.
    ///
    /// # Parameters
    /// - `Position { x, y }`: The position to check and possibly adjust.
    ///
    /// # Returns
    /// A new `Position` where out-of-bounds coordinates are wrapped to the opposite edge of the map.
    ///
    /// # Example
    /// ```rust
    /// use ratatui::layout::Rect;
    /// use rsnaker::graphics::graphic_block::Position;
    /// use rsnaker::graphics::sprites::map::Map;
    /// let map = Map::new(10, Rect::new(0, 0, 100, 40));
    /// let position = Position { x: 101, y: 20 };
    /// let new_position = map.out_of_map_reverse_position(&position);
    /// assert_eq!(new_position.x, 10);  // Wrapped to the opposite side
    /// ```
    #[must_use]
    pub fn out_of_map_reverse_position(&self, Position { x, y }: &Position) -> Position {
        //*2 to put out the bordered map (last case being ==== )
        let x_max = self.viewport.width - (self.case_size + 2);
        let y_max = self.viewport.height - 2;
        let x_min = self.case_size;
        let y_min = 1;

        //to keep the grid coherent
        let x_remainder = (x_max - x_min) % self.case_size;
        let x_max = x_max - x_remainder; // adjusted
        // y automatic as %1

        if *y > y_max {
            Position { x: *x, y: y_min }
        } else if *y < y_min {
            Position { x: *x, y: y_max }
        } else if *x > x_max {
            Position { x: x_min, y: *y }
        } else if *x < x_min {
            Position { x: x_max, y: *y }
        } else {
            Position { x: *x, y: *y }
        }
    }
    /// Returns the current viewport (the visible area) of the map.
    ///
    /// # Returns
    /// A reference to the `Rect` representing the map's viewport.
    #[must_use]
    pub fn area(&self) -> &Rect {
        &self.viewport
    }

    /// Returns the `Block` widget representing the map's border and title.
    ///
    /// # Returns
    /// A reference to the `Block` widget.
    #[must_use]
    pub fn get_widget(&self) -> &Block<'_> {
        &self.block
    }

    /// Returns the size of each case (cell) on the map.
    ///
    /// # Returns
    /// The size of each case in pixels.
    #[must_use]
    pub fn get_case_size(&self) -> u16 {
        self.case_size
    }
}

use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use std::time::{Duration, Instant};

pub const SPEED_MOVING_SNAKE_SLEEP_TIME_MS: u64 = 50;
/// Spacing between snake segments.
/// Horizontal spacing is double this value to account for a TUI cell aspect ratio.
const SEGMENT_GAP: i32 = 1;
/// Total number of snake segments (emojis) to display.
const TOTAL_SEGMENTS: usize = 5;

/// Manages a snake animation that moves along the terminal boundaries.
/// The snake follows the edges of the given area in a clockwise direction.
pub struct EdgeSnake {
    /// Current horizontal position of the head.
    pub x: u16,
    /// Current vertical position of the head.
    pub y: u16,
    /// Last update timestamp for frame rate control.
    last_update: Instant,
    /// Target duration between animation frames (speed).
    frame_duration: Duration,
}

impl Default for EdgeSnake {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgeSnake {
    /// Creates a new `EdgeSnake` instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            last_update: Instant::now(),
            frame_duration: Duration::from_millis(SPEED_MOVING_SNAKE_SLEEP_TIME_MS),
        }
    }

    /// Updates the snake's position along the edges.
    ///
    /// The movement is constrained by a fixed frame rate.
    pub fn update(&mut self, area: &Rect) {
        if self.last_update.elapsed() < self.frame_duration {
            return;
        }
        self.last_update = Instant::now();

        let (max_x, max_y) = Self::get_limits(area.width, area.height);
        if max_x == 0 && max_y == 0 {
            return;
        }
        (self.x, self.y) = Self::step_along_edge(self.x, self.y, max_x, max_y, 1);
    }
    pub fn render(&self, frame: &mut ratatui::Frame, area: &Rect) {
        for (x, y) in self.get_positions(area.width, area.height) {
            frame.render_widget(Paragraph::new("🐍"), Rect::new(x, y, 2, 1));
        }
    }

    /// Computes terminal-specific boundaries for the snake.
    ///
    /// Returns (`max_x`, `max_y`).
    fn get_limits(width: u16, height: u16) -> (u16, u16) {
        // Emojis are 2 cells wide, so we stop at width - 2.
        let max_x = width.saturating_sub(2);
        let max_y = height.saturating_sub(1);

        if width <= 2 || height <= 1 {
            return (0, 0);
        }

        (max_x, max_y)
    }
    /// Having a little fun with mathematics, as simple if condition can do the tricks also
    /// Unified helper that steps a given number of cells along the perimeter.
    /// A `delta` of `1` moves clockwise; `-1` moves counter-clockwise.
    #[must_use]
    pub fn step_along_edge(x: u16, y: u16, max_x: u16, max_y: u16, delta: i32) -> (u16, u16) {
        // sanity checks
        // Ensure the current position is within limits (handles resize)
        let x = x.min(max_x);
        let mut y = y.min(max_y);

        // If not on an edge (e.g., after resize), snap to the nearest edge;
        // To keep it simple, we snap to the top edge if it's internal.
        if x > 0 && x < max_x && y > 0 && y < max_y {
            y = 0;
        }
        //rectangle perimeter from maths
        let perimeter = 2 * (max_x + max_y);
        if perimeter == 0 {
            return (x, y);
        }

        // 1. Map the 2D coordinate to a 1D index (clockwise starting at 0,0)
        let index = if y == 0 {
            x
        } else if x == max_x {
            max_x + y
        } else if y == max_y {
            max_x + max_y + (max_x - x)
        } else if x == 0 {
            2 * max_x + max_y + (max_y - y)
        } else {
            return (x, y); // Safe fallback if the position is inside the grid
        };

        // 2. Step forward or backward along the 1D ring using Euclidean modulo to handle negatives safely
        let new_index = u16::try_from((i32::from(index) + delta).rem_euclid(i32::from(perimeter)))
            .expect("Maths error");

        // 3. Map the new 1D index back into 2D coordinates
        if new_index <= max_x {
            (new_index, 0)
        } else if new_index <= max_x + max_y {
            (max_x, new_index - max_x)
        } else if new_index <= 2 * max_x + max_y {
            (max_x - (new_index - max_x - max_y), max_y)
        } else {
            (0, max_y - (new_index - 2 * max_x - max_y))
        }
    }

    /// Checks if the segment at (x, y) is on a vertical edge.
    fn is_vertical(x: u16, y: u16, max_x: u16, max_y: u16) -> bool {
        // Right edge (excluding the top-right corner) or Left edge (excluding the bottom-left corner)
        (x == max_x && y > 0) || (x == 0 && y > 0 && y < max_y)
    }

    /// Returns the coordinates for all snake segments, starting from the head and going `TOTAL_SEGMENTS` time back
    /// Calculate each position as if size was 1, and compute an offset to do that the good amount of time (different between width and height)
    #[must_use]
    pub fn get_positions(&self, width: u16, height: u16) -> Vec<(u16, u16)> {
        let (max_x, max_y) = Self::get_limits(width, height);
        if max_x == 0 && max_y == 0 {
            return vec![(0, 0); TOTAL_SEGMENTS];
        }

        let mut positions = Vec::with_capacity(TOTAL_SEGMENTS);
        let (mut curr_x, mut curr_y) = (self.x, self.y);

        // Snap to limits for the current area
        curr_x = curr_x.min(max_x);
        curr_y = curr_y.min(max_y);

        for i in 0..TOTAL_SEGMENTS {
            positions.push((curr_x, curr_y));

            if i < TOTAL_SEGMENTS - 1 {
                // Determine how much to step back for the next segment.
                let offset: i32 = if Self::is_vertical(curr_x, curr_y, max_x, max_y) {
                    1 + SEGMENT_GAP
                } else {
                    2 + (SEGMENT_GAP * 2)
                };
                // Get backward offset nth position
                (curr_x, curr_y) = Self::step_along_edge(curr_x, curr_y, max_x, max_y, -offset);
            }
        }

        positions
    }
}

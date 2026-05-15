use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use std::time::{Duration, Instant};

pub const SPEED_MOVING_SNAKE_SLEEP_TIME_MS: u64 = 50;
/// Spacing between snake segments.
/// Horizontal spacing is double this value to account for TUI cell aspect ratio.
const SEGMENT_GAP: u32 = 1;
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

impl EdgeSnake {
    /// Creates a new `EdgeSnake` instance.
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

        // Ensure current position is within limits (handles resize)
        self.x = self.x.min(max_x);
        self.y = self.y.min(max_y);

        // If not on an edge (e.g. after resize), snap to the nearest edge
        // To keep it simple, we snap to top edge if it's internal.
        if self.x > 0 && self.x < max_x && self.y > 0 && self.y < max_y {
            self.y = 0;
        }

        (self.x, self.y) = Self::get_next_pos(self.x, self.y, max_x, max_y);
    }
    pub fn render(&self, frame: &mut ratatui::Frame, area: &Rect) {
        for (x, y) in self.get_positions(area.width, area.height) {
            frame.render_widget(Paragraph::new("🐍"), Rect::new(x, y, 2, 1));
        }
    }

    /// Computes terminal-specific boundaries for the snake.
    ///
    /// Returns (max_x, max_y).
    fn get_limits(width: u16, height: u16) -> (u16, u16) {
        // Emojis are 2 cells wide, so we stop at width - 2.
        let max_x = width.saturating_sub(2);
        let max_y = height.saturating_sub(1);

        if width <= 2 || height <= 1 {
            return (0, 0);
        }

        (max_x, max_y)
    }

    /// Moves one cell forward along the edges in a clockwise direction.
    fn get_next_pos(x: u16, y: u16, max_x: u16, max_y: u16) -> (u16, u16) {
        if y == 0 && x < max_x {
            // Top edge -> move right
            (x + 1, 0)
        } else if x == max_x && y < max_y {
            // Right edge -> move down
            (max_x, y + 1)
        } else if y == max_y && x > 0 {
            // Bottom edge -> move left
            (x - 1, max_y)
        } else if x == 0 && y > 0 {
            // Left edge -> move up
            (0, y - 1)
        } else {
            (x, y)
        }
    }

    /// Moves one cell backward along the edges in a counter-clockwise direction.
    fn get_prev_pos(x: u16, y: u16, max_x: u16, max_y: u16) -> (u16, u16) {
        if x == 0 && y < max_y {
            // Left edge (going back) -> move down
            (0, y + 1)
        } else if y == max_y && x < max_x {
            // Bottom edge (going back) -> move right
            (x + 1, max_y)
        } else if x == max_x && y > 0 {
            // Right edge (going back) -> move up
            (max_x, y - 1)
        } else if y == 0 && x > 0 {
            // Top edge (going back) -> move left
            (x - 1, 0)
        } else {
            (x, y)
        }
    }

    /// Checks if the segment at (x, y) is on a vertical edge.
    fn is_vertical(x: u16, y: u16, max_x: u16, max_y: u16) -> bool {
        // Right edge (excluding top-right corner) or Left edge (excluding bottom-left corner)
        (x == max_x && y > 0) || (x == 0 && y > 0 && y < max_y)
    }

    /// Returns the coordinates for all snake segments, starting from the head and going TOTAL_SEGMENTS time back
    /// Calculate each position as if size was 1, and compute an offset to do that the good number of time (different between width and height)
    pub fn get_positions(&self, width: u16, height: u16) -> Vec<(u16, u16)> {
        let (max_x, max_y) = Self::get_limits(width, height);
        if max_x == 0 && max_y == 0 {
            return vec![(0, 0); TOTAL_SEGMENTS];
        }

        let mut positions = Vec::with_capacity(TOTAL_SEGMENTS);
        let (mut curr_x, mut curr_y) = (self.x, self.y);

        // Snap to limits for current area
        curr_x = curr_x.min(max_x);
        curr_y = curr_y.min(max_y);

        for i in 0..TOTAL_SEGMENTS {
            positions.push((curr_x, curr_y));

            if i < TOTAL_SEGMENTS - 1 {
                // Determine how much to step back for the next segment.
                let offset = if Self::is_vertical(curr_x, curr_y, max_x, max_y) {
                    1 + SEGMENT_GAP
                } else {
                    2 + (SEGMENT_GAP * 2)
                };
                // we want to get the position of the previous segment as it was a 1 size
                // (so we come back offset time,because the rank of the element previous we want
                // is in reality: offset x 1
                for _ in 0..offset {
                    (curr_x, curr_y) = Self::get_prev_pos(curr_x, curr_y, max_x, max_y);
                }
            }
        }

        positions
    }
}

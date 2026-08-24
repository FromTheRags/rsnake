use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use std::time::{Duration, Instant};

pub const SPEED_MOVING_SNAKE_SLEEP_TIME_MS: u64 = 50;
/// Horizontal spacing between displayed snake segments.
const SEGMENT_SPACING: u16 = 4;
/// Total number of snake segments (emojis) to display.
const TOTAL_SEGMENTS: usize = 5;

/// Manages the welcome-screen snake moving across the top row.
pub struct EdgeSnake {
    /// Current horizontal position of the head.
    pub x: u16,
    /// Kept at the top row; exposed for compatibility with the menu renderer.
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

    /// Moves the snake one cell to the right on the top row, wrapping at the opposite edge.
    pub fn update(&mut self, area: &Rect) {
        if self.last_update.elapsed() < self.frame_duration {
            return;
        }
        self.last_update = Instant::now();

        self.x = Self::next_x(self.x, area.width);
        self.y = 0;
    }

    pub fn render(&self, frame: &mut ratatui::Frame, area: &Rect) {
        for (x, y) in self.get_positions(area.width) {
            frame.render_widget(Paragraph::new("🐍"), Rect::new(x, y, 2, 1));
        }
    }

    /// Returns the next head position, wrapping from the right edge to the left edge.
    fn next_x(x: u16, width: u16) -> u16 {
        let max_x = width.saturating_sub(2);
        if width <= 2 || x >= max_x { 0 } else { x + 1 }
    }

    /// Returns the top-row coordinates for the head and its body segments.
    /// Body segments wrap horizontally as well, so the animation remains continuous at the edge.
    #[must_use]
    pub fn get_positions(&self, width: u16) -> Vec<(u16, u16)> {
        if width <= 2 {
            return Vec::new();
        }

        let available_positions = width - 1;
        let head_x = self.x.min(width - 2);
        (0..TOTAL_SEGMENTS)
            .map(|segment| {
                let offset = (u16::try_from(segment).expect("segment count exceeds u16")
                    * SEGMENT_SPACING)
                    % available_positions;
                (
                    (head_x + available_positions - offset) % available_positions,
                    0,
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::EdgeSnake;

    #[test]
    fn head_wraps_from_the_right_edge_to_the_left_edge() {
        assert_eq!(EdgeSnake::next_x(7, 9), 0);
        assert_eq!(EdgeSnake::next_x(6, 9), 7);
    }

    #[test]
    fn body_stays_on_the_top_row_and_wraps_horizontally() {
        let mut snake = EdgeSnake::new();
        snake.x = 1;

        assert_eq!(
            snake.get_positions(10),
            vec![(1, 0), (6, 0), (2, 0), (7, 0), (3, 0)]
        );
    }
}

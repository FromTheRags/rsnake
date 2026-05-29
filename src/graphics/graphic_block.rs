/// This module contains definitions and implementations for graphical blocks used in the game logic.
///
/// It includes the following components:
///
/// - [`GraphicBlock`]: A struct representing a graphical block that can be placed in the game logic world.
/// - [`Position`]: A struct representing the position of a graphical block in a 2D space.
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Span, Style};
use ratatui::widgets::{Widget, WidgetRef};

/// A struct representing a graphical block that can be displayed in the game logic.
/// It holds the position and visual representation (image) of the block.
///
/// # Fields
/// - `position`: The position of the graphic block on the screen.
/// - `image`: The visual representation of the block, using a styled `Span`.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphicBlock<'a> {
    pub(crate) position: Position,
    image: Span<'a>,
    pub enabled: bool,
}

/// A struct representing the position of a graphical block in the 2D space.
/// It holds the x and y coordinates of the block.
///
/// # Fields
/// - `x`: The x-coordinate of the block's position.
/// - `y`: The y-coordinate of the block's position.
#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl<'a> GraphicBlock<'a> {
    /// Creates a new `GraphicBlock` with the specified position, image, and style.
    ///
    /// # Parameters
    /// - `position`: The position of the graphic block.
    /// - `image`: The string that represents the image of the block.
    /// - `style`: The style applied to the image of the block.
    ///
    /// # Returns
    /// A new `GraphicBlock` instance with the given position, image, and style.
    ///
    /// # Note
    /// If you want less lifetime propagation, this code can be enhanced to take to use a static string for the image using `'static` lifetime.
    /// But for a more general use, the lifetime parameter is better, allowing dynamic text (e.g., changing each time).
    /// If you want no lifetime and dynamic text, add an own `String`, to `GraphicBlock` and generate for each rendering a new `Span`,
    /// but this might affect performance.
    #[must_use]
    pub fn new(position: Position, image: &'a str, style: Style) -> GraphicBlock<'a> {
        GraphicBlock {
            position,
            image: Span::styled(image, style),
            enabled: true,
        }
    }

    /// Sets the position of the graphic block.
    ///
    /// # Parameters
    /// - `position`: The new position to set for the block.
    pub fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    /// Retrieves the current position of the graphic block.
    ///
    /// # Returns
    /// A reference to the current position of the graphic block.
    #[must_use]
    pub fn get_position(&self) -> &Position {
        &self.position
    }
    /// Retrieves the current position of the graphic block.
    ///
    /// # Returns
    /// A reference to the current position of the graphic block.
    #[must_use]
    pub fn get_content(&self) -> &str {
        &self.image.content
    }
}
///NB: Could also have used impl Widget for &T even if more limited
/// see: <https://github.com/ratatui/ratatui/discussions/1274>
impl WidgetRef for GraphicBlock<'_> {
    /// Renders the graphic block into the given buffer.
    ///
    /// # Parameters
    /// - `area`: The area in the terminal where the graphic block should be rendered.
    /// - `buf`: A mutable reference to the buffer where the block will be drawn.
    ///
    /// # Behavior
    /// If the block is outside the visible area, it will not be drawn, avoiding any panics.
    /// The game logic will not crash even if part of the block is outside the visible area due to window resizing.
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        if self.enabled
            && area.y + self.position.y < area.height
            && area.x + self.position.x < area.width
        {
            buf.set_span(
                area.x + self.position.x,
                area.y + self.position.y,
                &self.image,
                area.width,
            );
        }
    }
}

/// Backward compatibility implementation for the `Widget` trait.
impl Widget for GraphicBlock<'_> {
    /// Renders the graphic block into the given buffer, using the `render_ref` method.
    ///
    /// # Parameters
    /// - `area`: The area in the terminal where the graphic block should be rendered.
    /// - `buf`: A mutable reference to the buffer where the block will be drawn.
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}

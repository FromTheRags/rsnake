use crate::controls::direction::Direction;
use crate::graphics::graphic_block::{GraphicBlock, Position};
use crate::graphics::sprites::map::Map;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Style, Widget};
use ratatui::style::{Color, Modifier};
use ratatui::widgets::WidgetRef;

/// A struct representing the snake's body in the game.
/// It is composed of multiple `GraphicBlock` elements that make up the snake's segments.
/// The body can move, grow, and check for overlaps with itself.
///
/// # Fields
/// - `body`: A vector of `GraphicBlock` elements representing the segments of the snake's body.
/// - `CASE_SIZE`: The size of each segment of the snake's body in pixels.
/// - `position_ini`: The initial position of the snake's head.
/// - `size_ini`: The initial size of the snake (the number of body segments).
#[derive(Clone, Debug)]
pub struct SnakeBody<'a> {
    pub(crate) body: Vec<GraphicBlock<'a>>,
    case_size: u16,
    position_ini: Position,
    size_ini: u16,
}

impl<'a> SnakeBody<'a> {
    /// Creates a new `SnakeBody` instance with the specified body image, head image, number of segments,
    /// initial position, and case size.
    ///
    /// # Parameters
    /// - `body_image`: The image for the body segments of the snake.
    /// - `head_image`: The image for the snake's head.
    /// - `nb`: The number of body segments.
    /// - `position`: The initial position of the snake's head.
    /// - `CASE_SIZE`: The size of each body segment in pixels.
    ///
    /// # Returns
    /// A new `SnakeBody` instance with the specified parameters.
    #[must_use]
    pub fn new(
        body_image: &'a str,
        head_image: &'a str,
        nb: u16,
        position: Position,
        case_size: u16,
    ) -> SnakeBody<'a> {
        let snake_style = Style {
            fg: Some(Color::White),
            bg: Some(Color::Reset), // <- important if the global style has bg
            underline_color: Some(Color::White),
            add_modifier: Modifier::ITALIC,
            sub_modifier: Modifier::BOLD,
        };
        let Position { x, y } = position;
        let mut body = Vec::with_capacity(nb as usize);
        body.push(GraphicBlock::new(
            Position { x, y },
            head_image,
            snake_style,
        ));
        for i in 1..nb {
            body.push(GraphicBlock::new(
                Position {
                    //safer as will limit to 0
                    x: x.saturating_sub(case_size * i),
                    y,
                },
                body_image,
                snake_style,
            ));
        }
        SnakeBody {
            body,
            case_size,
            position_ini: position,
            size_ini: nb,
        }
    }

    /// Resets the snake's body to its initial position and size.
    /// The head is placed at the initial position, and the body segments are repositioned accordingly.
    pub fn reset(&mut self) {
        self.body.truncate(self.size_ini as usize);
        self.body[0].set_position(self.position_ini.clone());
        for i in 1..self.size_ini {
            self.body[i as usize].set_position(Position {
                x: self.position_ini.x.saturating_sub(self.case_size * i),
                y: self.position_ini.y,
            });
        }
    }

    /// Updates the positions of the body segments to simulate the movement of the snake.
    /// The body segments "follow" the previous segment.
    /// Add one previous not shown elements by enabling it (to avoid a big increase in tail as +10)
    ///
    /// # Parameters
    /// - `previous_head`: The position of the previous head of the snake.
    pub fn ramping_body(&mut self, previous_head: &Position) {
        let mut current = previous_head.clone();
        let mut previous = current;
        let has_grown_by_one = false;
        for i in 1..self.body.len() {
            current = self.body[i].get_position().clone();
            self.body[i].set_position(previous);
            previous = current;
            if !self.body[i].enabled && !has_grown_by_one {
                self.body[i].enable();
            }
        }
    }

    /// Checks if the snake's head overlaps with any part of its body.
    ///
    /// # Returns
    /// - `false` if the head does not overlap with the body.
    /// - `true` if the head overlaps with any part of the body.
    #[must_use]
    pub fn is_snake_eating_itself(&self) -> bool {
        let head = self.body[0].get_position();
        for b in self.body.iter().skip(1) {
            if head == b.get_position() {
                return true;
            }
        }
        false
    }

    /// Moves the snake's head left by one case and updates the body accordingly.
    pub fn left(&mut self) {
        let current = &self.body[0].get_position().clone();
        self.body[0].position.x -= self.case_size;
        self.ramping_body(current);
    }

    /// Moves the snake's head right by one case and updates the body accordingly.
    pub fn right(&mut self) {
        let current = &self.body[0].get_position().clone();
        self.body[0].position.x += self.case_size;
        self.ramping_body(current);
    }

    /// Moves the snake's head up by one case/line and updates the body accordingly.
    pub fn up(&mut self) {
        let current = &self.body[0].get_position().clone();
        self.body[0].position.y -= 1;
        self.ramping_body(current);
    }

    /// Moves the snake's head down by one case/line and updates the body accordingly.
    pub fn down(&mut self) {
        let current = &self.body[0].get_position().clone();
        self.body[0].position.y += 1;
        self.ramping_body(current);
    }

    /// Moves the snake in the specified direction and checks if the snake's head has moved outside the map
    /// or overlapped with its body. If the snake moves out of bounds, its position is reversed.
    ///
    /// # Parameters
    /// - `direction`: The direction in which to move the snake.
    /// - `carte`: The map used to check if the snake's head is out of bounds.
    ///
    /// # Returns
    /// - `&Position` the new snake's head position.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn ramp(&mut self, direction: &Direction, carte: &Map) -> &Position {
        match direction {
            Direction::Up => self.up(),
            Direction::Down => self.down(),
            Direction::Left => self.left(),
            Direction::Right => self.right(),
        }
        if carte.out_of_map(self.body[0].get_position()) {
            let new_position = carte.out_of_map_reverse_position(self.body[0].get_position());
            self.body[0].set_position(new_position);
        }
        self.body[0].get_position()
    }

    /// A backup plan in case the widget reference is unstable, by cloning the snake body.
    fn _get_widget(&self) -> impl Widget + 'a {
        self.clone()
    }

    /// Change the snake size by adding/removing a specified number of segments to its body.
    ///
    /// # Parameters
    /// - `nb`:The number of segments to add or to remove to the snake's body.
    /// # Panics
    /// If no element in Snake, as we keep a minimum size `size_ini`,
    /// when resizing down should not happen
    pub fn relative_size_change(&mut self, nb: i16) {
        if nb > 0 {
            for _ in 0..nb {
                let mut block_to_add = self
                    .body
                    .last()
                    .expect("Snake body has no elements ! Something went wrong")
                    .clone();
                //To show later, snake body one by one
                block_to_add.disable();
                self.body.push(block_to_add);
            }
        } else {
            //We must remove some element, but keeping a minimum length for the snake
            #[allow(clippy::cast_sign_loss)]
            let sub = self.body.len().saturating_sub((-nb) as usize);
            let to_keep = if sub < self.size_ini as usize {
                self.size_ini as usize
            } else {
                sub
            };
            self.body.truncate(to_keep);
        }
    }
}

/// Only needed for backwards compatibility
impl Widget for SnakeBody<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}
impl Widget for &SnakeBody<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_ref(area, buf);
    }
}
//In general, where you expect a widget to immutably work on its data,we recommended implementing
// Widget for a reference to the widget (impl Widget for &MyWidget).
// If you need to store state between draw calls, implement StatefulWidget if you want the Widget
// to be immutable,
// or implement Widget for a mutable reference to the widget (impl Widget for &mut MyWidget).
// If you want the widget to be mutable.
// The mutable widget pattern is used infrequently in apps
// but can be quite useful.
// A blanket implementation of Widget for &W where W implements WidgetRef is provided.
// The Widget trait is also implemented for &str and String types.

impl WidgetRef for SnakeBody<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        for body_element in &self.body {
            body_element.render_ref(area, buf);
        }
    }
}

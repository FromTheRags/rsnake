use crate::graphics::menus::generic_retro_parameter_logic::RowData;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Style};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

/// Creates a vertically centered rectangle within the given area with the specified number of lines.
#[must_use]
pub fn frame_vertically_centered_rect(area: Rect, lines: usize) -> Rect {
    // Auto center vertically, elsewhere could be manually calculated with y offset
    // by taking height from frame.area
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),                                          // top space
            Constraint::Length(u16::try_from(lines).unwrap_or(u16::MAX)), // content
            Constraint::Fill(1),                                          // bottom space
        ])
        // take second rect to render content
        .split(area)[1]
    // if we want to do manual center horizontally also
    /*Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Fill(1),                                            // top space
        Constraint::Length(lines.lines().last().unwrap().len() as u16), // content
        Constraint::Fill(1),                                            // bottom space
    ])
    // take second rect to render content
    .split(vertical_layout)[1]*/
}
/// Render a horizontally and vertically centered paragraph with text and color style
/// With the game external border set to the same color as the text
pub fn render_full_centered_paragraph(frame: &mut Frame, text: &'static str, color: Option<Color>) {
    let area = frame_vertically_centered_rect(frame.area(), text.lines().count());
    // ensure that all cells under the popup are cleared to avoid leaking content: no useful, and ugly
    //frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(text)
            //.style(Style::default().fg(color))
            .alignment(ratatui::layout::Alignment::Center),
        area,
    );
    // Set all screen items in the same color as the menu.
    // Applying style break position of emojis
    // even by previously setting a full style to item to avoid them to inherit the overall one
    // so keep global style to the final situations without the snake moving after
    if let Some(color) = color {
        frame.render_widget(
            Block::default().style(Style::default().fg(color)),
            frame.area(),
        );
    }
}
#[must_use]
pub fn constraint_length_from_widths(column_widths: &[u16]) -> Vec<Constraint> {
    let mut constraints = vec![];
    // A little fun with iterator, and peekable to foresee the future
    //peekable show the next element without advancing the iterator
    let mut iter = column_widths.iter().peekable();
    //for each element, add a constraint
    //  to have some fun with a destructuring pattern and while let
    while let Some(&size) = iter.next() {
        // if there is still more elements after
        if iter.peek().is_some() {
            constraints.push(Constraint::Length(size));
        } else {
            //for the last element, add all the remaining space
            constraints.push(Constraint::Length(size + 1));
        }
    }
    constraints
}
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub fn calculate_max_column_widths(rows: &[RowData], headers: &[String]) -> Vec<u16> {
    // Initialize with header widths
    let mut column_widths: Vec<u16> = headers.iter().map(|h| h.chars().count() as u16).collect();
    let column_len = column_widths.len();
    // Update with row data widths
    for row in rows {
        let cell_widths = row.get_cell_widths();
        for (i, &width) in cell_widths.iter().enumerate() {
            if width as u16 > column_widths[i] {
                column_widths[i % column_len] = width as u16;
            }
        }
    }

    column_widths
}

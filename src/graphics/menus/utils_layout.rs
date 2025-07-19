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

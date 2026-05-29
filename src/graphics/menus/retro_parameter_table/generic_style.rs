use crate::graphics::menus::retro_parameter_table::generic_logic::{
    CellValue, FooterData, RowData,
};
use ratatui::Frame;
use ratatui::layout::{Constraint, Margin, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use ratatui::widgets::{
    Block, BorderType, Borders, Cell, FrameExt, HighlightSpacing, Paragraph, Row, Scrollbar,
    ScrollbarOrientation, ScrollbarState, Table, TableState,
};
use std::fmt;

// Updated constants with emojis
pub const DEFAULT_ITEM_HEIGHT: usize = 1;

// Retro game colors - removed pink in favor of a more balanced palette
const RETRO_PURPLE: Color = Color::Rgb(50, 50, 150); //150,50,50
const RETRO_GREY: Color = Color::Rgb(128, 128, 128);
const RETRO_YELLOW: Color = Color::Rgb(255, 255, 0);
const RETRO_DARK_BLUE: Color = Color::Rgb(20, 20, 40);
const RETRO_ORANGE: Color = Color::Rgb(255, 165, 0);
const RETRO_BLUE: Color = Color::Rgb(0, 191, 255);
const RETRO_GOLD: Color = Color::Rgb(212, 175, 55);
// Create highlight symbols with more visual appeal
const HIGHLIGHT_SYMBOL_LEFT: &str = "► ";
const HIGHLIGHT_SYMBOL_RIGHT: &str = " ◄";
pub const DISPLAY_CELL_OUT_SPACE: usize = 6;

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellValue::Text(text) => write!(f, "{text}"),
            CellValue::Options {
                values,
                index,
                index_ini,
                ..
            } => {
                let fallback = "Bad index".to_string();
                let content = values.get(*index).unwrap_or(&fallback);
                if index == index_ini {
                    write!(f, "[ {content} ]")
                } else {
                    write!(f, "『 {content} 』")
                }
            }
        }
    }
}
pub(crate) struct ScrollBarCustomRetroStyle<'a> {
    pub scroll_state: ScrollbarState,
    pub margin: Margin,
    pub widget: Scrollbar<'a>,
}
impl ScrollBarCustomRetroStyle<'_> {
    pub fn new(row_sum_height: usize) -> Self {
        Self {
            scroll_state: ScrollbarState::new(row_sum_height),
            margin: Margin {
                vertical: 1,
                horizontal: 1,
            },
            widget: Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼"))
                .track_symbol(Some("│"))
                .thumb_symbol("█")
                .thumb_style(Style::default().fg(Color::DarkGray))
                .track_style(Style::default().fg(Color::Gray))
                .begin_style(Style::default().fg(Color::Red))
                .end_style(Style::default().fg(Color::Red)),
        }
    }
}
pub(crate) struct TableCustomRetroStyle<'a> {
    pub(crate) state: TableState,
    table: Table<'a>,
    pub(crate) rows: Vec<RowData>,
    pub(crate) headers: Vec<String>,
}
impl<'a> TableCustomRetroStyle<'a> {
    pub fn new(
        headers: &[String],
        rows: Vec<RowData>,
        selected_row: usize,
        constraints: Vec<Constraint>,
    ) -> Self {
        let headers_vec = headers.to_vec();
        // Create a header row using the custom headers with retro styling
        let header = Row::new(headers.iter().map(|h| Cell::from(h.clone())))
            .style(Style::default().fg(Color::Black).bg(RETRO_GREY))
            .height(1);

        //Create highlight symbols with more visual appeal
        let mut highlight_symbols = vec![HIGHLIGHT_SYMBOL_LEFT.into()];
        //To find the number of columns, we use header len,
        // We need the number of columns to display the right header one row below
        for _ in 1..headers.len() - 1 {
            highlight_symbols.push("".into());
        }
        highlight_symbols.push(HIGHLIGHT_SYMBOL_RIGHT.into());

        // Create a table with retro-style borders
        Self {
            state: TableState::default().with_selected(0),
            table: Table::new(
                TableCustomRetroStyle::get_rows_color(&rows, selected_row),
                constraints,
            )
            .header(header)
            .row_highlight_style(Style::default().bg(RETRO_PURPLE).fg(Color::White))
            .highlight_symbol(Text::from(highlight_symbols))
            .highlight_spacing(HighlightSpacing::Always)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(Color::DarkGray)),
            ),
            rows,
            headers: headers_vec,
        }
    }
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget_ref(&self.table, area, &mut self.state);
    }
    fn get_rows_color(rows: &[RowData], selected_row: usize) -> impl IntoIterator<Item = Row<'a>> {
        // Create rows with alternating background colors
        rows.iter()
            .enumerate()
            .map(|(index_row, row_data)| {
                let row_style = if index_row % 2 == 0 {
                    Style::default().bg(Color::Black)
                } else {
                    Style::default().bg(RETRO_DARK_BLUE)
                };
                let mut max_lines_for_this_row: u16 = 1;
                let cells = row_data.cells.iter().map(|cell| {
                    let content = cell.to_string();
                    let cell_style = if selected_row == index_row {
                        Style::default()
                            .fg(RETRO_YELLOW)
                            .add_modifier(Modifier::BOLD)
                    } else if let CellValue::Options {
                        index, index_ini, ..
                    } = cell
                    {
                        if index == index_ini {
                            Style::default().fg(RETRO_ORANGE)
                        } else {
                            Style::default().fg(RETRO_YELLOW)
                        }
                    } else {
                        Style::default().fg(RETRO_BLUE)
                    };
                    //useful for height calculation of the row
                    #[allow(clippy::cast_possible_truncation)]
                    let max = content.lines().count() as u16;
                    if max > max_lines_for_this_row {
                        max_lines_for_this_row = max;
                    }
                    Cell::from(Text::from(content)).style(cell_style)
                });

                Row::new(cells)
                    .style(row_style)
                    .height(max_lines_for_this_row)
            })
            .collect::<Vec<Row<'a>>>()
    }
    pub fn update_table_color_background(&mut self, selected_row: usize) {
        let new_rows = TableCustomRetroStyle::get_rows_color(&self.rows, selected_row);
        //Bad ratatui API design for widget reuse as the function consume self, without providing a setter with &mut
        self.table = self.table.clone().rows(new_rows);
    }
}
#[must_use]
pub fn get_formated_footer<'a>(data: &[FooterData]) -> Paragraph<'a> {
    // Create a multicolor, retro-styled footer
    let mut info_spans = Vec::with_capacity(data.len() + 1);

    //To act differently on the last element
    let mut iter = data.iter().peekable();
    while let Some(d) = iter.next() {
        info_spans.push(Span::styled(
            format!("({})", d.symbol),
            Style::default().fg(RETRO_GOLD).add_modifier(Modifier::BOLD),
        ));
        info_spans.push(Span::styled(
            format!(" {} ", d.text),
            Style::default().fg(Color::White),
        ));
        if let Some(value) = d.value {
            let red_style = Style::default().fg(Color::Red); //.add_modifier(Modifier::BOLD);
            info_spans.push(Span::styled("[".to_string(), red_style));
            info_spans.push(Span::styled(
                value.to_string(),
                Style::default().fg(Color::White), //.add_modifier(Modifier::BOLD),
            ));
            info_spans.push(Span::styled("]".to_string(), red_style));
        }
        //To have a closing pipe only if there are still others elements after
        if iter.peek().is_some() {
            info_spans.push(Span::styled(
                " | ".to_string(),
                Style::default().fg(Color::White),
            ));
        }
    }
    Paragraph::new(Line::from(info_spans)).centered()
}

use crate::game_logic::game_options::{GameOptions, ONLY_FOR_CLI_PARAMETERS};
use clap::CommandFactory;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::layout::Margin;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    BorderType, Cell, HighlightSpacing, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
    Table, TableState,
};
use ratatui::{
    layout::{Constraint, Layout, Rect}, style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal,
    Frame,
};
use std::fmt;

// Updated constants with emojis
const ITEM_HEIGHT: usize = 4;

// Retro game colors - removed pink in favor of a more balanced palette
const RETRO_PURPLE: Color = Color::Rgb(50, 50, 150); //150,50,50
const RETRO_GREY: Color = Color::Rgb(128, 128, 128);
const RETRO_YELLOW: Color = Color::Rgb(255, 255, 0);
const RETRO_DARK_BLUE: Color = Color::Rgb(20, 20, 40);
const RETRO_ORANGE: Color = Color::Rgb(255, 165, 0);
const RETRO_BLUE: Color = Color::Rgb(0, 191, 255);
const RETRO_GOLD: Color = Color::Rgb(212, 175, 55);

// Define a generic cell value type
enum CellValue {
    //either text only
    Text(String),
    //or a list of value
    Options {
        values: Vec<String>,
        index: usize,
        index_ini: usize,
    },
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellValue::Text(text) => write!(f, "{text}"),
            CellValue::Options {
                values,
                index,
                index_ini,
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

impl CellValue {
    pub fn new(text: String) -> Self {
        Self::Text(text)
    }
    pub fn new_with_options(values: Vec<String>, index: usize) -> Self {
        Self::Options {
            values,
            index,
            index_ini: index,
        }
    }
    fn next_value(&mut self) {
        if let CellValue::Options { values, index, .. } = self {
            *index = (*index + 1) % values.len();
        }
    }

    fn previous_value(&mut self) {
        if let CellValue::Options { values, index, .. } = self {
            let max = values.len();
            *index = (*index + max.saturating_sub(1)) % max;
        }
    }
}

// A row data type with only one option of changing parameter
// (no use case for a lateral switch, to only switch a cell)
// change all the row at all
// Easy to adapt by having a selected cell if need
struct RowData {
    // The column cells inside the row
    cells: Vec<CellValue>,
}

impl RowData {
    pub fn new(cells: Vec<CellValue>) -> Self {
        Self { cells }
    }
    fn get_cell_widths(&self) -> Vec<usize> {
        self.cells
            .iter()
            .map(|cell| {
                //We considers that table content never end with '\n' only used for wrapping
                let s = cell
                    .to_string()
                    .chars()
                    .fold(0, |acc, c| if c == '\n' { 0 } else { acc + 1 });
                match cell {
                    CellValue::Options { .. } => {
                        //Add 4 for the size of bracket added around value for option
                        s + 4
                    }
                    CellValue::Text(_) => s,
                }
            })
            .collect()
    }
    fn next_cell_value(&mut self) {
        for c in &mut self.cells {
            c.next_value();
        }
    }
    fn previous_cell_value(&mut self) {
        for c in &mut self.cells {
            c.previous_value();
        }
    }
}

// Updated Parameters Menu to work with the new data structure
// A table with no columns switch (no need)
pub(crate) struct ParametersMenu<'a> {
    state: TableState,
    rows: Vec<RowData>,
    column_widths: Vec<u16>,
    scroll_state: ScrollbarState,
    selected_row: usize,
    headers: Vec<String>,
    game_options: &'a GameOptions,
}
fn parse_interval(interval_str: &str) -> Option<(usize, usize)> {
    let cleaned = interval_str
        .split('[')
        .last()
        .unwrap_or("")
        .trim_end_matches(']');
    let mut parts = cleaned.split('-');

    let min_str = parts.next()?;
    let max_str = parts.next()?;

    let min = min_str.parse::<usize>().ok()?;
    let max = max_str.parse::<usize>().ok()?;

    Some((min, max))
}

impl<'a> ParametersMenu<'a> {
    pub(crate) fn new(options: &'a mut GameOptions) -> Self {
        let headers = vec![
            "🎯 Value".to_string(),
            "📋 Parameter".to_string(),
            "📝 Description / super power".to_string(),
        ];

        // Create rows with sample data
        //TODO: Load true data !
        let cmd = GameOptions::command();
        let mut rows = vec![];
        let mut option_value;
        for arg in cmd
            .get_arguments()
            .filter(|arg| !ONLY_FOR_CLI_PARAMETERS.contains(&arg.get_long().unwrap()))
        {
            let mut values = vec![
                arg.get_default_values()
                    .first()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            ];

            // if let Some((min, max)) = parse_interval(&arg.get_help().unwrap().to_string()) {
            //  values.extend((min..max).map(|i| i.to_string()));
            //}
            //For booleans and enums, use clap functionalities to get possible values
            let pv = arg.get_possible_values();
            if pv.is_empty() {
                /// Yet TODO 
                //for range
                // In Clap 4.x, ValueParser needs to be accessed and handled differently
                /* if let Ok(Some(vp)) = arg.get_value_parser().
                    .as_any()
                    .downcast_ref::<clap::builder::RangedU64ValueParser>()
                    .map(|parser| parser.get_range())
                {
                    let min = vp.start().unwrap_or(0);
                    let max = vp.end().unwrap_or(100);
                    // Convert u64 range to usize and add some sample values
                    for val in [min, max, (min + max) / 2].iter().map(|&v| v as usize) {
                        values.push(val.to_string());
                    }
                }*/
            } else {
                values.extend(pv.into_iter().map(|v| v.get_name().to_string()));
            }
            option_value = CellValue::new_with_options(values.clone(), 0);
            rows.push(RowData::new(vec![
                option_value,
                CellValue::new(arg.get_long().unwrap().to_string()),
                CellValue::new(arg.get_help().unwrap().to_string()),
            ]));
        }
        Self {
            state: TableState::default().with_selected(0),
            column_widths: calculate_column_widths(&rows, &headers),
            scroll_state: ScrollbarState::new((rows.len() - 1) * ITEM_HEIGHT),
            selected_row: 0,
            rows,
            headers,
            game_options: options,
        }
    }

    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.rows.len(),
            None => 0,
        };
        self.state.select(Some(i));
        self.selected_row = i;
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => (i + self.rows.len() - 1) % self.rows.len(),
            None => 0,
        };
        self.state.select(Some(i));
        self.selected_row = i;
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn next_parameter_value(&mut self) {
        if let Some(row) = self.rows.get_mut(self.selected_row) {
            row.next_cell_value();
        }
    }

    pub fn previous_parameter_value(&mut self) {
        if let Some(row) = self.rows.get_mut(self.selected_row) {
            row.previous_cell_value();
        }
    }

    pub(crate) fn run(&mut self, terminal: &mut DefaultTerminal) {
        // TODO: do not recreate the whole graphical element everytime, but just update them
        loop {
            terminal.draw(|frame| self.draw(frame)).unwrap();
            if let Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => return,
                        KeyCode::Down => self.next_row(),
                        KeyCode::Up => self.previous_row(),
                        KeyCode::Right => {
                            self.next_parameter_value();
                        }
                        KeyCode::Left => {
                            self.previous_parameter_value();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let vertical = &Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(
                u16::try_from(self.headers.len()).expect("too much headers to display"),
            ),
        ]);
        let rects = vertical.split(frame.area());

        self.render_table(frame, rects[0]);
        self.render_scrollbar(frame, rects[0]);
        render_footer(frame, rects[1]);
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        // Define styles - retro gaming colors!
        let header_style = Style::default().fg(Color::Black).bg(RETRO_GREY);
        let selected_style = Style::default().bg(RETRO_PURPLE).fg(Color::White);
        let even_style = Style::default().bg(Color::Black);
        let odd_style = Style::default().bg(RETRO_DARK_BLUE);
        let selected_cell_style = Style::default()
            .fg(RETRO_YELLOW)
            .add_modifier(Modifier::BOLD);

        // Create header row using the custom headers with retro styling
        let header = Row::new(self.headers.iter().map(|h| Cell::from(h.as_str())))
            .style(header_style)
            .height(1);

        // Create rows with alternating background colors and emojis
        let rows = self.rows.iter().enumerate().map(|(index_row, row_data)| {
            let row_style = if index_row % 2 == 0 {
                even_style
            } else {
                odd_style
            };

            let cells = row_data.cells.iter().map(|cell| {
                let content = cell.to_string();
                let cell_style = if self.selected_row == index_row {
                    selected_cell_style
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

                Cell::from(Text::from(format!("\n{content}\n"))).style(cell_style)
            });

            Row::new(cells).style(row_style).height(4)
        });

        // Calculate constraints
        let constraints = self.calculate_constraints();

        // Create highlight symbols with more visual appeal
        let highlight_symbol_left = "► ";
        let highlight_symbol_right = " ◄";
        let mut highlight_symbols = vec![highlight_symbol_left.into()];
        for _ in 1..self.headers.len() - 1 {
            highlight_symbols.push("".into());
        }
        highlight_symbols.push(highlight_symbol_right.into());

        // Create table with retro-style borders
        let t = Table::new(rows, constraints)
            .header(header)
            .row_highlight_style(selected_style)
            .highlight_symbol(Text::from(highlight_symbols))
            .highlight_spacing(HighlightSpacing::Always)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        //NB: could have used integrated table footer but less integrated (command related to row cell size...
        frame.render_stateful_widget(&t, area, &mut self.state);
        //to not consume (better), use :
        //frame.render_stateful_widget_ref(&t, area, &mut self.state);
    }

    fn render_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("▲"))
                .end_symbol(Some("▼"))
                .track_symbol(Some("│"))
                .thumb_symbol("█")
                .thumb_style(Style::default().fg(Color::DarkGray))
                .track_style(Style::default().fg(Color::Gray))
                .begin_style(Style::default().fg(Color::Red))
                .end_style(Style::default().fg(Color::Red)),
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scroll_state,
        );
    }

    fn calculate_constraints(&self) -> Vec<Constraint> {
        let mut constraints = vec![];
        // A little fun with iterator, and peekable to foresee the future
        //peekable show the next element without advancing the iterator
        let mut iter = self.column_widths.iter().peekable();
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
}

#[allow(clippy::cast_possible_truncation)]
fn calculate_column_widths(rows: &[RowData], headers: &[String]) -> Vec<u16> {
    // Initialize with header widths
    let mut column_widths: Vec<u16> = headers.iter().map(|h| h.chars().count() as u16).collect();

    // Update with row data widths
    for row in rows {
        let cell_widths = row.get_cell_widths();
        for (i, &width) in cell_widths.iter().enumerate() {
            if i < column_widths.len() && width as u16 > column_widths[i] {
                column_widths[i] = width as u16;
            }
        }
    }

    column_widths
}

fn render_footer(frame: &mut Frame, area: Rect) {
    // Create a multi-color, retro-styled footer
    let info_spans = vec![
        Span::styled(
            "(Esc)",
            Style::default().fg(RETRO_GOLD).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" quit | ", Style::default().fg(Color::White)),
        Span::styled(
            "(↕)",
            Style::default().fg(RETRO_GOLD).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" move | ", Style::default().fg(Color::White)),
        Span::styled(
            "(← →)",
            Style::default().fg(RETRO_GOLD).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" change value ", Style::default().fg(Color::White)),
    ];

    let info_footer = Paragraph::new(Line::from(info_spans)).centered();
    frame.render_widget(info_footer, area);
}

use crate::game_logic::game_options::GameOptions;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::layout::Margin;
use ratatui::text::Text;
use ratatui::widgets::{
    BorderType, Cell, HighlightSpacing, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
    Table, TableState,
};
use ratatui::{
    layout::{Constraint, Layout, Rect}, widgets::{Block, Paragraph},
    DefaultTerminal,
    Frame,
};

/// Inspired from # [Ratatui] Table example
/// |--------------------------------------------------|
/// |<`parameter_name`>|<value>| <`parameter_name`>|<value>|
/// |<`parameter_name`>|<value>| <`parameter_name`>|<value>|
/// |<`parameter_name`>|<value>| <`parameter_name`>|<value>|
/// |            |
/// |--------------------------------------------------|
/// \use dir arrow, enter to select,x, q, save to file /
/// Or easier but less cool for little screens:
/// |<`parameter_name`>|<value>|description|
/// |<`parameter_name`>|<value>|description|
const INFO_TEXT: &str = "(Esc) quit | (←↕→) move/change value | () select / unselect";

const ITEM_HEIGHT: usize = 4;

struct DataParam {
    name: String,
    values: Vec<String>,
    index: usize,
}

impl DataParam {
    pub fn new(name: String, values: Vec<String>, index: usize) -> Self {
        Self {
            name,
            values,
            index,
        }
    }
    fn ref_array(&self) -> [&String; 2] {
        [&self.name, self.values.get(self.index).unwrap()]
    }
    // For later use with an edit option!
    fn _add_value_and_select(&mut self, data: String) {
        self.values.push(data);
        self.index = self.values.len().saturating_sub(1);
    }
    fn get_selected_value(&self) -> &str {
        match self.values.get(self.index) {
            None => "_",
            Some(v) => v,
        }
    }
    fn next_value(&mut self) {
        self.index = (self.index + 1) % self.values.len();
    }
    fn previous_value(&mut self) {
        //maths trick
        let max = self.values.len();
        self.index = (self.index + max.saturating_sub(1)) % max;
    }
}

pub(crate) struct ParametersMenu {
    state: TableState,
    items: Vec<DataParam>,
    longest_item_by_column_lens: (u16, u16, u16, u16), // cl1, cl2, cl3, cl4
    scroll_state: ScrollbarState,
    index: usize,
    is_one_selected: bool,
}

impl ParametersMenu {
    pub(crate) fn new() -> Self {
        let data_vec = vec![
            DataParam::new(
                "speed".to_string(),
                vec!["slow".into(), "normal".into(), "fast".into()],
                1,
            ),
            DataParam::new(
                "Still Work in Progress for now ! Not usable :) ".to_string(),
                vec!["🐢".into(), "🍥".into(), "a".into()],
                1,
            ),
            DataParam::new(
                "speedy".to_string(),
                vec!["slow".into(), "normal".into(), "fast".into()],
                1,
            ),
            DataParam::new(
                "Another One ".to_string(),
                vec!["🐢".into(), "🍥".into(), "a".into()],
                0,
            ),
        ];
        Self {
            state: TableState::default().with_selected(0),
            longest_item_by_column_lens: constraint_len_calculator(&data_vec),
            scroll_state: ScrollbarState::new((data_vec.len() - 1) * ITEM_HEIGHT),
            index: 0,
            items: data_vec,
            is_one_selected: false,
        }
    }
    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }
    pub fn next_column(&mut self) {
        //by two, to always be
        self.state.select_next_column();
        self.state.select_next_column();
    }
    pub fn previous_column(&mut self) {
        self.state.select_previous_column();
        self.state.select_previous_column();
    }
    pub fn next_parameter_value(&mut self) {
        self.items.get_mut(self.index).unwrap().next_value();
    }

    pub fn previous_parameter_value(&mut self) {
        self.items.get_mut(self.index).unwrap().previous_value();
    }

    pub(crate) fn run(mut self, terminal: &mut DefaultTerminal, _options: &mut GameOptions) {
        loop {
            terminal.draw(|frame| self.draw(frame)).unwrap();
            if let Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => return,
                        KeyCode::Down => self.next_row(),
                        KeyCode::Up => self.previous_row(),
                        KeyCode::Enter => {
                            //toggle selection
                            self.is_one_selected = !self.is_one_selected;
                        }
                        KeyCode::Right if self.is_one_selected => self.next_parameter_value(),
                        KeyCode::Left if self.is_one_selected => self.previous_parameter_value(),
                        KeyCode::Right => self.next_column(),
                        KeyCode::Left => self.previous_column(),
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(4)]);
        let rects = vertical.split(frame.area());

        self.render_table(frame, rects[0]);
        self.render_scrollbar(frame, rects[0]);
        render_footer(frame, rects[1]);
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        /*let header_style = Style::default()
            .fg(self.colors.header_fg)
            .bg(self.colors.header_bg);
        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_row_style_fg);
        let selected_col_style = Style::default().fg(self.colors.selected_column_style_fg);
        let selected_cell_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_cell_style_fg);

         */

        let header = ["Name", "Value", "Name", "Value"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            //.style(header_style)
            .height(1);

        let rows = self.items.iter().enumerate().filter_map(|(i, data)| {
            // Vérifie qu'un élément suivant existe
            self.items.get(i + 1).map(|next| {
                let current_cells = data
                    .ref_array()
                    .into_iter()
                    .map(|content| Cell::from(Text::from(format!("\n{content}\n"))));

                let next_cells = next
                    .ref_array()
                    .into_iter()
                    .map(|content| Cell::from(Text::from(format!("\n{content}\n"))));

                // Concatène les cellules actuelles et suivantes
                let all_cells = current_cells.chain(next_cells);

                Row::new(all_cells).height(4)
            })
        });
        let bar = " █ ";
        let t = Table::new(
            rows,
            [
                // + 1 is for padding.
                Constraint::Length(self.longest_item_by_column_lens.0 + 1),
                Constraint::Min(self.longest_item_by_column_lens.1 + 1),
                Constraint::Min(self.longest_item_by_column_lens.2 + 1),
                Constraint::Min(self.longest_item_by_column_lens.3),
            ],
        )
        .header(header)
        //.row_highlight_style(selected_row_style)
        //.column_highlight_style(selected_col_style)
        //.cell_highlight_style(selected_cell_style)
        .highlight_symbol(Text::from(vec![
            "".into(),
            bar.into(),
            bar.into(),
            "".into(),
        ]))
        //.bg(self.colors.buffer_bg)
        .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(t, area, &mut self.state);
    }

    fn render_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scroll_state,
        );
    }
}

#[allow(clippy::cast_possible_truncation)]
fn constraint_len_calculator(items: &[DataParam]) -> (u16, u16, u16, u16) {
    let mut max: (u16, u16, u16, u16) = (0, 0, 0, 0);
    for (i, d) in items.iter().enumerate() {
        let current = (d.name.len(), d.get_selected_value().len());
        if i % 2 == 0 {
            if current.0 > max.0 as usize {
                max.0 = current.0 as u16;
            } else if current.1 > max.1 as usize {
                max.1 = current.1 as u16;
            }
        } else if current.0 > max.2 as usize {
            max.2 = current.0 as u16;
        } else if current.1 > max.3 as usize {
            max.3 = current.1 as u16;
        }
    }
    //UnicodeWidthStr::width
    max
}
fn render_footer(frame: &mut Frame, area: Rect) {
    let info_footer = Paragraph::new(INFO_TEXT)
        /*.style(
            Style::new()
                .fg(self.colors.row_fg)
                .bg(self.colors.buffer_bg),
        )*/
        .centered()
        .block(
            Block::bordered().border_type(BorderType::Double), //.border_style(Style::new().fg(self.colors.footer_border_color)),
        );
    frame.render_widget(info_footer, area);
}

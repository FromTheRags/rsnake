use crate::game_logic::game_options::GameOptions;
use crate::graphics::menus::customized_retro_parameter_with_cli::FooterData;
use crate::graphics::menus::generic_retro_parameter_style::{
    get_formated_footer, ScrollBarCustomRetroStyle, TableCustomRetroStyle, DISPLAY_CELL_OUT_SPACE,
    ITEM_HEIGHT,
};
use crate::graphics::menus::utils_layout::{
    calculate_max_column_widths, constraint_length_from_widths,
};
use clap::{CommandFactory, Parser};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::widgets::FrameExt;
use ratatui::{
    layout::{Constraint, Layout}, widgets::Paragraph,
    DefaultTerminal,
    Frame,
};

// Define a generic cell value type
pub enum CellValue {
    //either text only
    Text(String),
    //or a list of value
    Options {
        option_name: String,
        values: Vec<String>,
        index: usize,
        index_ini: usize,
    },
}
impl CellValue {
    #[must_use]
    pub fn new(text: String) -> Self {
        Self::Text(text)
    }
    #[must_use]
    pub fn new_with_options(option_name: String, values: Vec<String>, index: usize) -> Self {
        Self::Options {
            option_name,
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
    fn width(&self) -> usize {
        match self {
            CellValue::Options { values, .. } => {
                let max = values.iter().map(|v| v.chars().count()).max().unwrap_or(0);
                //Add 6 for the size of bracket added around value for option when displaying
                // (hardcoded for performance rather than using format and then count chars)
                max + DISPLAY_CELL_OUT_SPACE
            }
            //count max chars on the same line
            CellValue::Text(v) => v.split('\n').map(|s| s.chars().count()).max().unwrap_or(0),
        }
    }
}

// A row data type with only one option of changing parameter
// (no use case for a lateral switch, to only switch a cell).
// Changes all the row at all
// Easy to adapt by having a selected cell if you need
pub struct RowData {
    // The column cells inside the row
    pub cells: Vec<CellValue>,
}

impl RowData {
    #[must_use]
    pub fn new(cells: Vec<CellValue>) -> Self {
        Self { cells }
    }
    pub(crate) fn get_cell_widths(&self) -> Vec<usize> {
        self.cells.iter().map(CellValue::width).collect()
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
//The lain struct for the parameter
pub struct ParametersMenu<'a> {
    table_custom: TableCustomRetroStyle<'a>,
    scrollbar: ScrollBarCustomRetroStyle<'a>,
    selected_row: usize,
    info_footer: Paragraph<'a>,
    vertical_layout: Layout,
    //To be replaced by an Apply trait that GameOption will implement,
    // that take in the flow of parameter and out a Result
    game_options: &'a mut GameOptions,
}

impl<'a> ParametersMenu<'a> {
    pub fn new(
        options: &'a mut GameOptions,
        rows: Vec<RowData>,
        headers: &[String],
        info_footer: Vec<FooterData>,
    ) -> Self {
        // Calculate constraints
        let column_widths = calculate_max_column_widths(&rows, headers);
        let constraints = constraint_length_from_widths(&column_widths);
        let row_number = rows.len();
        let vertical_layout = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(
                u16::try_from(headers.len()).expect("too much headers to store :p "),
            ),
        ]);
        Self {
            table_custom: TableCustomRetroStyle::new(headers, rows, 0, constraints),
            scrollbar: ScrollBarCustomRetroStyle::new(row_number),
            selected_row: 0,
            info_footer: get_formated_footer(info_footer),
            vertical_layout,
            game_options: options,
        }
    }

    pub fn next_row(&mut self) {
        let i = match self.table_custom.state.selected() {
            Some(i) => (i + 1) % self.table_custom.rows.len(),
            None => 0,
        };
        self.table_custom.state.select(Some(i));
        self.selected_row = i;
        self.scrollbar.scroll_state = self.scrollbar.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn previous_row(&mut self) {
        let i = match self.table_custom.state.selected() {
            Some(i) => (i + self.table_custom.rows.len() - 1) % self.table_custom.rows.len(),
            None => 0,
        };
        self.table_custom.state.select(Some(i));
        self.selected_row = i;
        self.scrollbar.scroll_state = self.scrollbar.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn next_parameter_value(&mut self) {
        if let Some(row) = self.table_custom.rows.get_mut(self.selected_row) {
            row.next_cell_value();
        }
    }

    pub fn previous_parameter_value(&mut self) {
        if let Some(row) = self.table_custom.rows.get_mut(self.selected_row) {
            row.previous_cell_value();
        }
    }
    fn apply_parameters(&mut self) {
        let command = GameOptions::command();
        let prog_name = command.get_name().to_string();
        let mut new_args = vec![prog_name];
        for row in &self.table_custom.rows {
            for cell in &row.cells {
                if let CellValue::Options {
                    option_name,
                    values,
                    index,
                    ..
                } = cell
                {
                    let value = &values[*index];
                    match value.parse::<bool>() {
                        Ok(bv) => {
                            // Modern way to do CLI, two dedicated flag to set/unset the value, beginning with --no- (for false)
                            // UX better than --feature false / --feature true, better than default (no flag = false). If you want the possibility to set both values,
                            // as no clear default value or want to be able to easily programmatically change the value (as there)
                            // or to have a default at true
                            let bv_name: String = if bv {
                                option_name.clone()
                            } else {
                                option_name.replace("--", "--no-")
                            };
                            new_args.push(bv_name);
                        }
                        Err(_) => {
                            //not a boolean value
                            new_args.extend([option_name.clone(), value.clone()]);
                        }
                    }
                }
            }
        }
        // Update all the game options as a reparsing (only one way to update value to check).
        // Some debate over the utility of this feature for clap, but widely used to update from env / configuration
        //  Allows keeping the struct for cli parameter as a model object, feeding it with different stream of data.
        // The backup solution is to serialize the current value in TOML and load them in game_options as already done for the file saving
        // (safe as constraints in-game value)
        self.game_options.update_from(new_args);
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) {
        // TODO: do not recreate the whole graphical element everytime, but just update them
        // in draw, keep table as a parameter of ParameterMenu (alongside TableState, just change the style of the current selected row), same for footer)
        loop {
            terminal.draw(|frame| self.draw(frame)).unwrap();
            if let Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => {
                            self.apply_parameters();
                            return;
                        }
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
        let rects = self.vertical_layout.split(frame.area());
        //render the custom table (could have implemented the statefulWidget trait or Widget, but state is badly handled)
        // Bad API design in ratatui!
        self.table_custom
            .update_table_color_background(self.selected_row);
        self.table_custom.render(frame, rects[0]);
        //Unfortunately, scrollbar does not yet implement render_stateful_widget_ref,
        // so we have to use old way with clone
        //see : https://docs.rs/ratatui/latest/ratatui/widgets/trait.StatefulWidgetRef.html#implementors
        frame.render_stateful_widget(
            self.scrollbar.widget.clone(),
            rects[0].inner(self.scrollbar.margin),
            &mut self.scrollbar.scroll_state,
        );
        frame.render_widget_ref(&self.info_footer, rects[1]);
    }
}

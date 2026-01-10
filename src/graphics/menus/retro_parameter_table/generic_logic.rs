use crate::graphics::menus::retro_parameter_table::generic_style::{
    get_formated_footer, ScrollBarCustomRetroStyle, TableCustomRetroStyle, DISPLAY_CELL_OUT_SPACE,
};
use crate::graphics::menus::utils_layout::{
    calculate_max_column_widths, calculate_sum_inner_row_heights, constraint_length_from_widths,
};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::widgets::FrameExt;
use ratatui::{
    layout::{Constraint, Layout}, widgets::Paragraph,
    DefaultTerminal,
    Frame,
};
use unicode_segmentation::UnicodeSegmentation;

pub trait ActionParameter {
    fn apply_and_save(&mut self, rows: &[RowData], current_preset: Option<u16>);
}
#[derive(Clone)]
pub struct FooterData {
    pub symbol: String,
    pub text: String,
    pub value: Option<u16>,
}

pub struct ActionInputs<'a> {
    pub key: Vec<KeyCode>,
    pub action: Vec<TableParameterAction<'a>>,
}
#[allow(clippy::type_complexity)]
pub enum TableParameterAction<'a> {
    NextValue,
    PreviousValue,
    NextRow,
    PreviousRow,
    //logic options
    Quit,
    //Genericity using a trait to allow using any reference to a type implementing ActionParameter
    ApplyAndSave(&'a mut dyn ActionParameter),
    //Goal for loading: use the CLI fn to load from File and chain it
    //le u16 permet de keep track of the current preset loaded
    //Genericity using a closure to allow using any fn to load from File
    LoadPreset(
        u16,
        fn(u16) -> (Option<Vec<RowData>>, Option<Vec<FooterData>>),
    ),
}

// Define a generic cell value type
#[derive(Clone)]
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
                let max = values
                    .iter()
                    .map(|v| v.as_str().graphemes(true).count())
                    .max()
                    .unwrap_or(0);
                //Add 6 for the size of bracket added around value for option when displaying
                // (hardcoded for performance rather than using format and then count chars)
                max + DISPLAY_CELL_OUT_SPACE
            }
            //count max chars on the same line
            CellValue::Text(v) => v.split('\n').map(|s| s.chars().count()).max().unwrap_or(0),
        }
    }
    fn height(&self) -> usize {
        match self {
            CellValue::Options { values, .. } => values
                .iter()
                .map(|v| v.split('\n').count())
                .max()
                .unwrap_or(0),
            //number of lines
            CellValue::Text(v) => v.split('\n').count(),
        }
    }
}

// A row data type with only one option of changing the parameter
// (no use case for a lateral switch, to only switch a cell).
// Changes all the rows at once
// Easy to adapt by having a selected cell if you need
#[derive(Clone)]
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
    pub(crate) fn get_cell_heights(&self) -> Vec<usize> {
        self.cells.iter().map(CellValue::height).collect()
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
pub struct GenericMenu<'a> {
    table_custom: TableCustomRetroStyle<'a>,
    scrollbar: ScrollBarCustomRetroStyle<'a>,
    selected_row: usize,
    info_footer: Paragraph<'a>,
    info_footer_data: Vec<FooterData>,
    vertical_layout: Layout,
    current_preset: Option<u16>,
}

impl<'a> GenericMenu<'a> {
    #[must_use]
    pub fn new(
        rows: Vec<RowData>,
        headers: &[String],
        info_footer: Vec<FooterData>,
        current_preset: Option<u16>,
    ) -> Self {
        // Calculate constraints
        let column_widths = calculate_max_column_widths(&rows, headers);
        let constraints = constraint_length_from_widths(&column_widths);
        let row_sum_height = calculate_sum_inner_row_heights(&rows);
        let vertical_layout = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(
                u16::try_from(headers.len()).expect("too much headers to store :p "),
            ),
        ]);
        Self {
            table_custom: TableCustomRetroStyle::new(headers, rows, 0, constraints),
            scrollbar: ScrollBarCustomRetroStyle::new(row_sum_height),
            selected_row: 0,
            info_footer: get_formated_footer(&info_footer),
            info_footer_data: info_footer,
            vertical_layout,
            current_preset,
        }
    }

    pub fn next_row(&mut self) {
        let i = match self.table_custom.state.selected() {
            Some(i) => (i + 1) % self.table_custom.rows.len(),
            None => 0,
        };
        self.table_custom.state.select(Some(i));
        self.selected_row = i;
        self.scrollbar.scroll_state =
            self.scrollbar
                .scroll_state
                .position(calculate_sum_inner_row_heights(
                    &self.table_custom.rows[..i],
                ));
    }

    pub fn previous_row(&mut self) {
        let i = match self.table_custom.state.selected() {
            Some(i) => (i + self.table_custom.rows.len() - 1) % self.table_custom.rows.len(),
            None => 0,
        };
        self.table_custom.state.select(Some(i));
        self.selected_row = i;
        self.scrollbar.scroll_state =
            self.scrollbar
                .scroll_state
                .position(calculate_sum_inner_row_heights(
                    &self.table_custom.rows[..i],
                ));
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

    pub fn run(
        &mut self,
        mut actions_inputs: Vec<ActionInputs<'a>>,
        terminal: &mut DefaultTerminal,
    ) {
        loop {
            terminal.draw(|frame| self.draw(frame)).unwrap();
            if let Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    for action_input in &mut actions_inputs {
                        for key_code in action_input.key.clone() {
                            if key_code == key.code {
                                for unitary_tp_action in &mut action_input.action {
                                    match unitary_tp_action {
                                        TableParameterAction::NextValue => {
                                            self.next_parameter_value();
                                        }
                                        TableParameterAction::PreviousValue => {
                                            self.previous_parameter_value();
                                        }
                                        TableParameterAction::NextRow => {
                                            self.next_row();
                                        }
                                        TableParameterAction::PreviousRow => {
                                            self.previous_row();
                                        }
                                        TableParameterAction::ApplyAndSave(action) => {
                                            action.apply_and_save(
                                                &self.table_custom.rows,
                                                self.current_preset,
                                            );
                                        }
                                        TableParameterAction::Quit => {
                                            return;
                                        }
                                        TableParameterAction::LoadPreset(index, loader) => {
                                            let (new_rows, new_footer) = loader(*index);
                                            let new_rows =
                                                new_rows.unwrap_or(self.table_custom.rows.clone());
                                            let new_footer =
                                                new_footer.unwrap_or(self.info_footer_data.clone());
                                            //Refresh all the GUI, so recreate the table with the new data and recalculate constraint
                                            //Clean Rust way with self-overwriting
                                            *self = Self::new(
                                                new_rows,
                                                &self.table_custom.headers,
                                                new_footer,
                                                Some(*index),
                                            );
                                        }
                                    } //match
                                } // unitary action
                            } //key code
                        } //keyCodes
                    } //ActionInputs
                } //Press event
            } //event readable
        } //loop
    }

    fn draw(&mut self, frame: &mut Frame) {
        let rects = self.vertical_layout.split(frame.area());
        //render the custom table (could have implemented the statefulWidget trait or Widget, but state is badly handled)
        // Bad API design in ratatui!
        self.table_custom
            .update_table_color_background(self.selected_row);
        self.table_custom.render(frame, rects[0]);
        //Unfortunately, scrollbar does not yet implement render_stateful_widget_ref,
        // so we have to use the old way with clone
        //https://docs.rs/ratatui/latest/ratatui/widgets/trait.StatefulWidgetRef.html#implementors
        frame.render_stateful_widget(
            self.scrollbar.widget.clone(),
            rects[0].inner(self.scrollbar.margin),
            &mut self.scrollbar.scroll_state,
        );
        frame.render_widget_ref(&self.info_footer, rects[1]);
    }
}

#[must_use]
pub fn get_default_action_input<'a>() -> Vec<ActionInputs<'a>> {
    vec![
        ActionInputs {
            key: vec![KeyCode::Down],
            action: vec![TableParameterAction::NextRow],
        },
        ActionInputs {
            key: vec![KeyCode::Up],
            action: vec![TableParameterAction::PreviousRow],
        },
        ActionInputs {
            key: vec![KeyCode::Right],
            action: vec![TableParameterAction::NextValue],
        },
        ActionInputs {
            key: vec![KeyCode::Left],
            action: vec![TableParameterAction::PreviousValue],
        },
        ActionInputs {
            key: vec![KeyCode::Esc],
            action: vec![TableParameterAction::Quit],
        },
    ]
}

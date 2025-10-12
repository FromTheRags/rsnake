use crate::graphics::menus::retro_parameter_table::customized_with_cli::FooterData;
use crate::graphics::menus::retro_parameter_table::generic_style::{
    get_formated_footer, ScrollBarCustomRetroStyle, TableCustomRetroStyle, DISPLAY_CELL_OUT_SPACE,
    ITEM_HEIGHT,
};
use crate::graphics::menus::utils_layout::{
    calculate_max_column_widths, constraint_length_from_widths,
};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::widgets::FrameExt;
use ratatui::{
    layout::{Constraint, Layout}, widgets::Paragraph,
    DefaultTerminal,
    Frame,
};

pub trait ApplyParameter {
    fn apply(&mut self, rows: &[RowData]);
}

#[derive(Clone)]
pub struct ActionInputs {
    pub key: Vec<KeyCode>,
    pub action: TableParameterAction,
}
#[derive(Clone)]
pub enum TableParameterAction {
    NextValue,
    PreviousValue,
    NextRow,
    PreviousRow,
    //Shortcut for genericity could be a trait but no use there
    Apply,
}

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
    //table Action, including apply to change value to whatever need (in our case the CLI struct)
    actions: Vec<ActionInputs>,
    //To be generic, the table can be saved to any data structure
    saved_to: Option<&'a mut dyn ApplyParameter>,
}

impl<'a> ParametersMenu<'a> {
    #[must_use]
    pub fn new(
        rows: Vec<RowData>,
        headers: &[String],
        info_footer: Vec<FooterData>,
        actions: Vec<ActionInputs>,
        saved_to: Option<&'a mut dyn ApplyParameter>,
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
            actions,
            saved_to,
        }
    }
    #[must_use]
    pub fn new_with_default_action(
        rows: Vec<RowData>,
        headers: &[String],
        info_footer: Vec<FooterData>,
        saved_to: Option<&'a mut dyn ApplyParameter>,
    ) -> Self {
        ParametersMenu::new(
            rows,
            headers,
            info_footer,
            get_default_action_input(),
            saved_to,
        )
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

    pub fn run(&mut self, terminal: &mut DefaultTerminal) {
        // TODO: do not recreate the whole graphical element everytime, but just update them
        // in draw, keep table as a parameter of ParameterMenu (alongside TableState, just change the style of the current selected row), same for footer)
        let actions = self.actions.clone();
        loop {
            terminal.draw(|frame| self.draw(frame)).unwrap();
            if let Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    for action in &actions {
                        for key_code in &action.key {
                            if key_code == &key.code {
                                match action.action {
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
                                    TableParameterAction::Apply => {
                                        if let Some(s) = &mut self.saved_to {
                                            s.apply(&self.table_custom.rows);
                                        } else {
                                            panic!(
                                                "No data structure provided to save the table to ! "
                                            )
                                        }
                                        return;
                                    }
                                }
                            }
                        }
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

#[must_use]
pub fn get_default_action_input() -> Vec<ActionInputs> {
    vec![
        ActionInputs {
            key: vec![KeyCode::Down, KeyCode::Char('s')],
            action: TableParameterAction::NextRow,
        },
        ActionInputs {
            key: vec![KeyCode::Up, KeyCode::Char('z')],
            action: TableParameterAction::PreviousRow,
        },
        ActionInputs {
            key: vec![KeyCode::Right, KeyCode::Char('d')],
            action: TableParameterAction::NextValue,
        },
        ActionInputs {
            key: vec![KeyCode::Left, KeyCode::Char('q')],
            action: TableParameterAction::PreviousValue,
        },
    ]
}

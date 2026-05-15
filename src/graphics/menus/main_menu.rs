use crate::graphics::menus::edge_snake::EdgeSnake;
use crate::graphics::menus::messages::{CONTROLS_TABLE, SNAKE_LOGO};
use crate::graphics::menus::utils_layout::frame_vertically_centered_rect;
use clap::ValueEnum;
use ratatui::style::Stylize;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::cmp::PartialEq;
use std::sync::{Mutex, OnceLock};
use unicode_segmentation::UnicodeSegmentation;

//A bit overkill, but for the example of keeping data state outside of struct,
// initialized at first call
// and keeping track of the edge snake emoji position around call
static EDGE_SNAKE: OnceLock<Mutex<EdgeSnake>> = OnceLock::new();

/// Print the wanted welcome screen controls
/// Show Fruit and Speed menus alongside
/// Sadly `slow_blink` and `fast_blink` are not rendered anymore on modern terminal...
/// # Panics
/// Will panic if no suitable terminal for displaying ios provided
pub fn display_main_menu(
    terminal: &mut DefaultTerminal,
    selected_switch_menu_for_display_bracket: &SwitchMenu,
) {
    //terminal.clear().expect("Clearing terminal fail ");
    terminal
        .draw(|frame| {
            let area = frame.area();
            big_snake_menu(frame, selected_switch_menu_for_display_bracket);
            //set a border all around the terminal
            frame.render_widget(Block::bordered().border_type(BorderType::Double), area);

            // Render the edge snake emojis
            // Emojis often take 2 cells, we use width 2 to avoid clipping in some terminals
            let mut edge_snake = EDGE_SNAKE
                .get_or_init(|| Mutex::new(EdgeSnake::new()))
                .lock()
                .unwrap();
            edge_snake.update(&area);
            edge_snake.render(frame, &area);
        })
        .expect("Unusable terminal render");
}

fn big_snake_menu(frame: &mut Frame, to_display_switch: &SwitchMenu) {
    let mut lines = vec![];
    // Add logo lines
    for logo_line in SNAKE_LOGO.lines() {
        lines.push(Line::from(logo_line));
    }

    // Add navigation buttons
    lines.push(Line::from(get_button_span(to_display_switch)));

    // Add the control table
    for table_line in CONTROLS_TABLE.lines() {
        lines.push(Line::from(table_line));
    }

    // Add a greeting message
    lines.push(Line::from("Have a good 🐍 game ! 🎮".green()));
    let nb_lines = lines.len();

    frame.render_widget(
        Paragraph::new(Text::from(lines)).centered(),
        frame_vertically_centered_rect(frame.area(), nb_lines),
    );
}
/// Represents a button in the menu interface
pub struct Button {
    pub name: &'static str,
    pub selected: bool,
}

impl Button {
    /// Creates a new button with the given name and hotkey
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            selected: false,
        }
    }

    /// Sets the selected state of the button
    pub fn selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    /// Converts the button to a vector of spans for rendering
    #[must_use]
    pub fn to_spans(&self) -> Vec<Span<'static>> {
        if self.selected {
            vec![
                Span::styled(" [ ", Style::default().fg(Color::Red)).add_modifier(Modifier::BOLD),
                Span::raw(self.name),
                Span::styled(
                    " ] ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]
        } else {
            let (first, rest) = match self.name.graphemes(true).next() {
                Some(grapheme) => {
                    // A grapheme is a &str, so we can get its byte length with .len()
                    let rest = &self.name[grapheme.len()..];
                    (grapheme, rest) // Both are &str
                }
                None => ("", ""), // Handle empty string
            };
            vec![
                Span::raw("["),
                // Span::styled can take a &str directly
                Span::styled(first, Style::default().fg(Color::Yellow)),
                Span::raw(format!("{rest}]")),
            ]
        }
    }
}

/// Option that can be selected from the main menu, using a lateral switcher
#[derive(PartialEq, ValueEnum, Clone)]
pub enum SwitchMenu {
    Highs,
    Fruits,
    Speed,
    Run,
    Parameters,
    Doc,
    Main,
}
const BUTTONS: [(SwitchMenu, Button); 6] = [
    (SwitchMenu::Highs, Button::new("Highs💯")),
    (SwitchMenu::Fruits, Button::new("Fruit")),
    (SwitchMenu::Speed, Button::new("Speed")),
    (SwitchMenu::Run, Button::new("Run")),
    (SwitchMenu::Parameters, Button::new("Edit⚙️")),
    (SwitchMenu::Doc, Button::new("Docℹ️")),
];
/// Returns a vector of spans representing the button navigation menu
fn get_button_span(selected: &SwitchMenu) -> Vec<Span<'static>> {
    let mut vec_line_button = vec![Span::raw("↔")];
    // Add each button to the menu, marking the selected one
    for (menu, mut button) in BUTTONS {
        button.selected(selected == &menu);
        vec_line_button.extend(button.to_spans());
    }
    vec_line_button
}

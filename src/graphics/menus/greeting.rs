use crate::controls::input::GreetingMenuInput;
use crate::controls::speed;
use crate::controls::speed::SpeedConfig;
use crate::graphics::menus::utils_layout::frame_vertically_centered_rect;
use clap::ValueEnum;
use ratatui::style::Stylize;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::cmp::PartialEq;
use std::thread::sleep;
use std::time::Duration;

// Define the ASCII art as a constant
const SNAKE_LOGO: &str = "\
███████╗ ███╗   ██╗  █████╗  ██╗  ██╗ ███████╗
██╔════╝ ████╗  ██║ ██╔══██╗ ██║ ██╔╝ ██╔════╝
███████╗ ██╔██╗ ██║ ███████║ █████╔╝  █████╗  
╚════██║ ██║╚██╗██║ ██╔══██║ ██╔═██╗  ██╔══╝  
███████║ ██║ ╚████║ ██║  ██║ ██║  ██╗ ███████╗
╚══════╝ ╚═╝  ╚═══╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚══════╝";

// Define the control table as a constant
const CONTROLS_TABLE: &str = "\
+--------+------+------+-------+-----+-------+
|Controls| ←↕→  |  Q   | P / ⎵ |  M  |   R   | 
+--------+------+------+-------+-----+-------+
|Effects | Move | Quit | Pause | Menu| Start |
+--------+------+------+-------+-----+-------+";

// Usage example:
// let lines = create_greeting_lines(to_display_switch);

/// Print the wanted welcome screen controls
/// Show Fruit and Speed menus alongside
/// Sadly `slow_blink` and `fast_blink` are not rendered anymore on modern terminal...
/// # Panics
/// Will panic if no suitable terminal for displaying ios provided
pub fn main_greeting_menu(
    terminal: &mut DefaultTerminal,
    to_display: &GreetingSimpleDisplay,
    to_display_switch_menu: &SwitchMenu,
) {
    //terminal.clear().expect("Clearing terminal fail ");
    terminal
        .draw(|frame| {
            let area = frame.area();
            match to_display {
                GreetingSimpleDisplay::MainMenu => big_snake_menu(frame, to_display_switch_menu),
                GreetingSimpleDisplay::Velocity => {
                    speed_menu(frame, to_display_switch_menu);
                }
                GreetingSimpleDisplay::Help => {
                    help_menu(frame, to_display_switch_menu);
                }
            }
            //buttons_menu(frame, &app);
            //set a border all around the terminal
            frame.render_widget(Block::bordered().border_type(BorderType::Double), area);
        })
        .expect("Unusable terminal render");
    sleep(Duration::from_millis(100));
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
        //centered horizontally
        Paragraph::new(Text::from(lines)).centered(),
        frame_vertically_centered_rect(frame.area(), nb_lines),
    );
}
/// Display the speed menu center aligned, vertically centered
fn speed_menu(frame: &mut Frame, to_display_switch: &SwitchMenu) {
    // Speed effects
    let mut speed_lines = Vec::new();
    let speed_tab_jonction = Line::from("+------------+------------+----------------+---------+ ");
    speed_lines.push(speed_tab_jonction.clone());
    speed_lines.push(Line::from(
        "| Speed Name | Value (ms) | Score Modifier | Symbol  | ",
    ));
    speed_lines.push(speed_tab_jonction.clone());
    for s in speed::Speed::value_variants() {
        let SpeedConfig {
            name,
            ms_value,
            score_modifier,
            symbol,
        } = s.config();
        //:<10 and so on are formating options, e.g., saying aligning left with min 10 chars
        speed_lines.push(Line::from(format!(
            "| {name:<10} | {ms_value:>10} | {score_modifier:>14} | {symbol:<6} | "
        )));
    }
    speed_lines.push(speed_tab_jonction);
    speed_lines.push(Line::from(get_button_span(to_display_switch)));
    let nb_lines = speed_lines.len();
    frame.render_widget(
        Paragraph::new(Text::from(speed_lines)).centered(),
        frame_vertically_centered_rect(frame.area(), nb_lines),
    );
}
fn help_menu(frame: &mut Frame, to_display_switch: &SwitchMenu) {
    //formating reminder: Where:
    // - `<10` means left-aligned with width 10
    // - `>5` means right-aligned with width 5
    let lines = vec![
        Line::from("Snake Game Rules:".bold().yellow()),
        Line::from(format!("• {:<80}", "Eat fruits to score points")),
        Line::from(format!(
            "    • {:<80}",
            "Different fruits give various scores / effects, some even reduce size (and score)"
        )),
        Line::from(format!(
            "   • {:<80}",
            "Game speeds can be changed to increase difficulty and score multipliers"
        )),
        Line::from(format!(
            "   • {:<80}",
            "Check the Fruit and Speed menus for more details on game mechanics!"
        )),
        Line::from(format!(
            "• {:<80}",
            "Control the snake using the arrow keys (←↕→)"
        )),
        Line::from(format!(
            "   • {:<80}",
            "Avoid hitting yourself or your own tail"
        )),
        Line::from(format!(
            "   • {:<80}",
            "Walls are circulars so your head will appear on the other side of the screen"
        )),
        Line::from(format!("• {:<80}", "Press P or Space to pause the game")),
        Line::from(format!("• {:<80}", "Press Q to quit anytime")),
        Line::from(format!("• {:<80}", "Press R to start a new game")),
        Line::from(format!(
            "• {:<80}",
            "Press M to return to the menu, and maybe consider support this game on kofi🥤"
        )),
        Line::from(get_button_span(to_display_switch)),
    ];

    let nb_lines = lines.len();
    frame.render_widget(
        Paragraph::new(Text::from(lines)).centered(),
        frame_vertically_centered_rect(frame.area(), nb_lines),
    );
}
/// Represents a button in the menu interface
struct Button {
    name: &'static str,
    selected: bool,
}

impl Button {
    /// Creates a new button with the given name and hotkey
    const fn new(name: &'static str) -> Self {
        Self {
            name,
            selected: false,
        }
    }

    /// Sets the selected state of the button
    fn selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    /// Converts the button to a vector of spans for rendering
    fn to_spans(&self) -> Vec<Span<'static>> {
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
            vec![
                // Valid because only ASCII chars in the menu name, otherwise use chars().next etc
                Span::raw("["),
                Span::styled(&self.name[..1], Style::default().fg(Color::Yellow)),
                Span::raw(format!("{}]", &self.name[1..])),
            ]
        }
    }
}

/// Option that can be selected from the main menu, using a lateral switcher
#[derive(PartialEq, ValueEnum, Clone)]
pub enum SwitchMenu {
    Main,
    Fruits,
    Speed,
    Run,
    Parameters,
    Help,
}
const BUTTONS: [(SwitchMenu, Button); 6] = [
    (SwitchMenu::Main, Button::new("Main")),
    (SwitchMenu::Fruits, Button::new("Fruit")),
    (SwitchMenu::Speed, Button::new("Speed")),
    (SwitchMenu::Run, Button::new("Run")),
    (SwitchMenu::Parameters, Button::new("Edit⚙️")),
    (SwitchMenu::Help, Button::new("Help")),
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

#[derive(PartialEq, Clone, Debug)]
pub enum GreetingSimpleDisplay {
    Velocity,
    Help,
    MainMenu,
}
impl From<SwitchMenu> for GreetingSimpleDisplay {
    fn from(menu: SwitchMenu) -> Self {
        match menu {
            SwitchMenu::Speed => GreetingSimpleDisplay::Velocity,
            SwitchMenu::Help => GreetingSimpleDisplay::Help,
            // For run, we will default to MainMenu but should be treated differently
            // (to start the game) same for Parameters,
            // too complex to manage without a dedicated input manager/display
            SwitchMenu::Fruits | SwitchMenu::Parameters | SwitchMenu::Run | SwitchMenu::Main => {
                GreetingSimpleDisplay::MainMenu
            }
        }
    }
}
impl From<GreetingMenuInput> for GreetingSimpleDisplay {
    fn from(menu: GreetingMenuInput) -> Self {
        match menu {
            GreetingMenuInput::Speed => GreetingSimpleDisplay::Velocity,
            GreetingMenuInput::Help => GreetingSimpleDisplay::Help,
            _ => GreetingSimpleDisplay::MainMenu,
        }
    }
}

use crate::controls::playing_input::{QUIT_KEYS, START_KEYS};
use crate::game_logic::game_options::GameOptions;
use crate::graphics::menus::main_menu::SwitchMenu::{
    Doc, Fruits, Highs, Main, Parameters, Run, Speed,
};
use crate::graphics::menus::main_menu::{display_main_menu, SwitchMenu};
use crate::graphics::menus::retro_parameter_table::customized_with_doc::setup_and_run_doc_table_parameters;
use crate::graphics::menus::retro_parameter_table::customized_with_edit::setup_and_run_cli_table_parameters;
use crate::graphics::menus::retro_parameter_table::customized_with_fruits::setup_and_run_fruits_table_parameters;
use crate::graphics::menus::retro_parameter_table::customized_with_highscore::setup_and_run_highs_table_parameters;
use crate::graphics::menus::retro_parameter_table::customized_with_speed::setup_and_run_speed_table_parameters;
use crossterm::event;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

const SWITCH_MENUS_OPTION: [SwitchMenu; 6] = [Highs, Fruits, Speed, Run, Parameters, Doc];
const PARAMETERS_KEYS: [KeyCode; 2] = [KeyCode::Char('e'), KeyCode::Char('E')];
const FRUITS_KEYS: [KeyCode; 2] = [KeyCode::Char('f'), KeyCode::Char('F')];
const VELOCITY_KEYS: [KeyCode; 2] = [KeyCode::Char('s'), KeyCode::Char('P')];
const HELP_KEYS: [KeyCode; 2] = [KeyCode::Char('d'), KeyCode::Char('D')];
const HIGH_SCORE_KEYS: [KeyCode; 2] = [KeyCode::Char('h'), KeyCode::Char('H')];
pub const NEXT_KEYS: [KeyCode; 2] = [KeyCode::Right, KeyCode::Up];
pub const PREVIOUS_KEYS: [KeyCode; 3] = [KeyCode::Left, KeyCode::Backspace, KeyCode::Down];
pub const ENTER_KEYS: [KeyCode; 2] = [KeyCode::Enter, KeyCode::End];
/// The control part of the main menu
/// allows switching to a submenu (Fruits, Speed, Parameters, etc.)
/// Use `MainMenuInput` to known which keys have been used
/// and `GreetingSimpleDisplay` to display an easy menu, without input control (all except run and parameters)
/// Return true if the player wants to play, false otherwise
///
/// # Panics                                                                                              
/// if Terminal writing is not possible
enum MenuFlow {
    StayOnMainScreen,
    StartGame,
    QuitGame,
}

fn enter_menu_screen(
    input: &MainMenuInput,
    selected: &mut usize,
    terminal: &mut DefaultTerminal,
    options: &mut GameOptions,
) -> MenuFlow {
    //get the screen menu to display or do the action associated to the input
    let to_display = match input {
        MainMenuInput::Enter => SWITCH_MENUS_OPTION[*selected].clone(),
        MainMenuInput::Parameters => Parameters,
        MainMenuInput::Fruits => Fruits,
        MainMenuInput::Speed => Speed,
        MainMenuInput::Doc => Doc,
        MainMenuInput::Highs => Highs,
        MainMenuInput::Next => {
            *selected = (*selected + 1) % SWITCH_MENUS_OPTION.len();
            Main
        }
        MainMenuInput::Previous => {
            *selected = (*selected + SWITCH_MENUS_OPTION.len() - 1) % SWITCH_MENUS_OPTION.len();
            Main
        }
        MainMenuInput::QuitGame => {
            return MenuFlow::QuitGame;
        }
        MainMenuInput::Start => {
            return MenuFlow::StartGame;
        }
        MainMenuInput::Main => Main,
    };

    match to_display {
        Parameters => {
            run_submenu_and_reset(terminal, selected, |term| {
                setup_and_run_cli_table_parameters(term, options);
            });
        }
        Fruits => run_submenu_and_reset(terminal, selected, setup_and_run_fruits_table_parameters),
        Highs => run_submenu_and_reset(terminal, selected, setup_and_run_highs_table_parameters),
        Speed => run_submenu_and_reset(terminal, selected, setup_and_run_speed_table_parameters),
        Run => {
            return MenuFlow::StartGame;
        }
        Doc => run_submenu_and_reset(terminal, selected, setup_and_run_doc_table_parameters),
        Main => {
            display_main_menu(terminal, &SWITCH_MENUS_OPTION[*selected]);
        }
    }
    MenuFlow::StayOnMainScreen
}

pub fn controls_main_switch_menu(
    terminal: &mut DefaultTerminal,
    options: &mut GameOptions,
) -> bool {
    let mut selected = 3;
    display_main_menu(terminal, &SWITCH_MENUS_OPTION[selected]);

    loop {
        let input = main_menu_event();

        let flow = enter_menu_screen(&input, &mut selected, terminal, options);

        match flow {
            MenuFlow::StayOnMainScreen => {
                //for re-displaying after quiting a submenu without waiting for another input
                display_main_menu(terminal, &SWITCH_MENUS_OPTION[selected]);
            }
            MenuFlow::StartGame => return true,
            MenuFlow::QuitGame => return false,
        }
    }
}

fn run_submenu_and_reset<F>(terminal: &mut DefaultTerminal, selected: &mut usize, f: F)
where
    F: FnOnce(&mut DefaultTerminal),
{
    f(terminal);
    *selected = 3;
}

#[derive(PartialEq, Clone, Debug)]
pub enum MainMenuInput {
    Fruits,
    Speed,
    Start,
    Parameters,
    Doc,
    Highs,
    Main,
    QuitGame,
    Next,
    Previous,
    Enter,
}

/// Check input on the greeting screen
/// Return Some(GreetingOption) if input is valid, with the chosen Greeting Option, None otherwise
/// # Panics                                                                                              
/// if impossible to get key event, better crash as game will be unplayable  
#[must_use]
pub fn main_menu_event() -> MainMenuInput {
    // Read keyboard key event
    if let event::Event::Key(key) = event::read().expect("Error reading key event") {
        match key.kind {
            //If a key is pressed
            KeyEventKind::Press => {
                flush_input_buffer();
                // If it is a directional key
                if START_KEYS.contains(&key.code) {
                    MainMenuInput::Start
                    // if it is a quit key
                } else if QUIT_KEYS.contains(&key.code) {
                    MainMenuInput::QuitGame
                } else if PARAMETERS_KEYS.contains(&key.code) {
                    MainMenuInput::Parameters
                } else if FRUITS_KEYS.contains(&key.code) {
                    MainMenuInput::Fruits
                } else if VELOCITY_KEYS.contains(&key.code) {
                    MainMenuInput::Speed
                } else if HELP_KEYS.contains(&key.code) {
                    MainMenuInput::Doc
                } else if HIGH_SCORE_KEYS.contains(&key.code) {
                    MainMenuInput::Highs
                } else if NEXT_KEYS.contains(&key.code) {
                    MainMenuInput::Next
                } else if PREVIOUS_KEYS.contains(&key.code) {
                    MainMenuInput::Previous
                } else if ENTER_KEYS.contains(&key.code) {
                    MainMenuInput::Enter
                } else {
                    MainMenuInput::Main
                }
            }
            _ => MainMenuInput::Main,
        }
    } else {
        MainMenuInput::Main
    }
}
fn flush_input_buffer() {
    while event::poll(std::time::Duration::from_secs(0)).unwrap_or(false) {
        let _ = crossterm::event::read(); // Discard any buffered events
    }
}

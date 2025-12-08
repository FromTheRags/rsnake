use crate::controls::input::{QUIT_KEYS, START_KEYS};
use crate::game_logic::game_options::GameOptions;
use crate::graphics::menus::greeting::SwitchMenu::{Doc, Fruits, Highs, Parameters, Run, Speed};
use crate::graphics::menus::greeting::{main_greeting_menu, SwitchMenu};
use crate::graphics::menus::retro_parameter_table::customized_with_cli::setup_and_run_cli_table_parameters;
use crate::graphics::menus::retro_parameter_table::customized_with_doc::setup_and_run_doc_table_parameters;
use crate::graphics::menus::retro_parameter_table::customized_with_fruits::setup_and_run_fruits_table_parameters;
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
const NEXT_KEYS: [KeyCode; 2] = [KeyCode::Right, KeyCode::Up];
const PREVIOUS_KEYS: [KeyCode; 3] = [KeyCode::Left, KeyCode::Backspace, KeyCode::Down];
const ENTER_KEYS: [KeyCode; 2] = [KeyCode::Enter, KeyCode::End];
/// The control part of the main menu
/// allows switching to a submenu (Fruits, Speed, Parameters, etc.)
/// Use `GreetingMenuInput` to known which keys have been used
/// and `GreetingSimpleDisplay` to display an easy menu, without input control (all except run and parameters)
/// Return true if the player wants to play, false otherwise
///
/// # Panics                                                                                              
/// if Terminal writing is not possible
enum MenuFlow {
    Continue,
    StartGame,
    QuitGame,
}

fn handle_menu_action(
    input: Option<&GreetingMenuInput>,
    selected: &mut usize,
    terminal: &mut DefaultTerminal,
    options: &mut GameOptions,
) -> MenuFlow {
    let to_display = match input {
        Some(GreetingMenuInput::Enter) => SWITCH_MENUS_OPTION[*selected].clone(),
        Some(GreetingMenuInput::Parameters) => Parameters,
        Some(GreetingMenuInput::Fruits) => Fruits,
        Some(GreetingMenuInput::Speed) => Speed,
        Some(GreetingMenuInput::Doc) => Doc,
        _ => SwitchMenu::Highs,
    };

    if to_display == Run {
        return MenuFlow::StartGame;
    } else if to_display == Parameters {
        run_submenu_and_reset(terminal, selected, |term| {
            setup_and_run_cli_table_parameters(term, options);
        });
    } else if to_display == Fruits {
        run_submenu_and_reset(terminal, selected, setup_and_run_fruits_table_parameters);
    } else if to_display == Speed {
        run_submenu_and_reset(terminal, selected, setup_and_run_speed_table_parameters);
    } else if to_display == Doc {
        run_submenu_and_reset(terminal, selected, setup_and_run_doc_table_parameters);
    }
    MenuFlow::Continue
}

fn process_menu_input(
    input: Option<GreetingMenuInput>,
    selected: &mut usize,
    terminal: &mut DefaultTerminal,
    options: &mut GameOptions,
) -> MenuFlow {
    match input {
        Some(GreetingMenuInput::Next) => {
            *selected = (*selected + 1) % SWITCH_MENUS_OPTION.len();
        }
        Some(GreetingMenuInput::Previous) => {
            *selected = (*selected + SWITCH_MENUS_OPTION.len() - 1) % SWITCH_MENUS_OPTION.len();
        }
        Some(GreetingMenuInput::QuitGame) => {
            return MenuFlow::QuitGame;
        }
        Some(GreetingMenuInput::Start) => {
            return MenuFlow::StartGame;
        }
        Some(input) => {
            return handle_menu_action(Some(&input), selected, terminal, options);
        }
        _ => {}
    }

    MenuFlow::Continue
}

pub fn controls_main_switch_menu(
    terminal: &mut DefaultTerminal,
    options: &mut GameOptions,
) -> bool {
    let mut selected = 3;
    main_greeting_menu(terminal, &SWITCH_MENUS_OPTION[selected]);

    loop {
        let input = greeting_screen_manage_input();

        let flow = process_menu_input(input, &mut selected, terminal, options);

        match flow {
            MenuFlow::Continue => {
                main_greeting_menu(terminal, &SWITCH_MENUS_OPTION[selected]);
            }
            MenuFlow::StartGame => return true,
            MenuFlow::QuitGame => return false,
        }
    }
}

fn run_submenu_and_reset<F>(terminal: &mut DefaultTerminal, selected: &mut usize, submenu_logic: F)
where
    F: FnOnce(&mut DefaultTerminal),
{
    submenu_logic(terminal);
    *selected = 3;
}

#[derive(PartialEq, Clone, Debug)]
pub enum GreetingMenuInput {
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
pub fn greeting_screen_manage_input() -> Option<GreetingMenuInput> {
    // Read keyboard key event
    if let event::Event::Key(key) = event::read().expect("Error reading key event") {
        match key.kind {
            //If a key is pressed
            KeyEventKind::Press => {
                flush_input_buffer();
                // If it is a directional key
                if START_KEYS.contains(&key.code) {
                    Some(GreetingMenuInput::Start)
                    // if it is a quit key
                } else if QUIT_KEYS.contains(&key.code) {
                    Some(GreetingMenuInput::QuitGame)
                } else if PARAMETERS_KEYS.contains(&key.code) {
                    Some(GreetingMenuInput::Parameters)
                } else if FRUITS_KEYS.contains(&key.code) {
                    Some(GreetingMenuInput::Fruits)
                } else if VELOCITY_KEYS.contains(&key.code) {
                    Some(GreetingMenuInput::Speed)
                } else if HELP_KEYS.contains(&key.code) {
                    Some(GreetingMenuInput::Doc)
                } else if HIGH_SCORE_KEYS.contains(&key.code) {
                    Some(GreetingMenuInput::Highs)
                } else if NEXT_KEYS.contains(&key.code) {
                    Some(GreetingMenuInput::Next)
                } else if PREVIOUS_KEYS.contains(&key.code) {
                    Some(GreetingMenuInput::Previous)
                } else if ENTER_KEYS.contains(&key.code) {
                    Some(GreetingMenuInput::Enter)
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    }
}
fn flush_input_buffer() {
    while event::poll(std::time::Duration::from_secs(0)).unwrap_or(false) {
        let _ = crossterm::event::read(); // Discard any buffered events
    }
}

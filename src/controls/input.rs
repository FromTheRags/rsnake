use crate::controls::direction::Direction;
use crate::game_logic::state::{GameState, GameStatus};
use crossterm::event;
use crossterm::event::{KeyCode, KeyEventKind};
use std::sync::{Arc, RwLock};

const QUIT_KEYS: [KeyCode; 2] = [KeyCode::Char('q'), KeyCode::Char('Q')];
const PARAMETERS_KEYS: [KeyCode; 2] = [KeyCode::Char('e'), KeyCode::Char('E')];
const FRUITS_KEYS: [KeyCode; 2] = [KeyCode::Char('f'), KeyCode::Char('F')];
const VELOCITY_KEYS: [KeyCode; 2] = [KeyCode::Char('s'), KeyCode::Char('P')];
const HELP_KEYS: [KeyCode; 2] = [KeyCode::Char('h'), KeyCode::Char('H')];
const MAIN_MENU_KEYS: [KeyCode; 2] = [KeyCode::Char('m'), KeyCode::Char('M')];
//const DIRECTIONAL_KEYS: [KeyCode; 4] = [KeyCode::Down, KeyCode::Up, KeyCode::Left, KeyCode::Right];
const START_KEYS: [KeyCode; 2] = [KeyCode::Char('r'), KeyCode::Char('R')];
pub(crate) const PAUSE_KEYS: [KeyCode; 4] = [
    KeyCode::Char('p'),
    KeyCode::Char('P'),
    KeyCode::Char(' '),
    KeyCode::Home,
];
const RESET_KEYS: [KeyCode; 2] = [KeyCode::Char('r'), KeyCode::Char('R')];
const NEXT_KEYS: [KeyCode; 2] = [KeyCode::Right, KeyCode::Up];
const PREVIOUS_KEYS: [KeyCode; 3] = [KeyCode::Left, KeyCode::Backspace, KeyCode::Down];
const ENTER_KEYS: [KeyCode; 2] = [KeyCode::Enter, KeyCode::End];

/// You cannot block middle-click paste/scroll behavior from inside your Rust TUI app.
/// If you really want to disable it, you would have to modify user system settings or terminal emulator config
/// (e.g., in alacrity, kitty, gnome-terminal, etc.)
/// That is outside the app's control
/// # Panics
/// if Arc panics while holding the resources (poisoning), no recovery mechanism implemented better crash
pub fn playing_input_loop(direction: &Arc<RwLock<Direction>>, gs: &Arc<RwLock<GameState>>) {
    loop {
        if let Ok(event::Event::Key(key)) = event::read() {
            if key.kind == KeyEventKind::Press {
                //PAUSE
                if PAUSE_KEYS.contains(&key.code) {
                    let mut gs_guard = gs.write().unwrap();
                    //in others state does nothing to not break game_logic logic
                    if gs_guard.status == GameStatus::Playing {
                        gs_guard.status = GameStatus::Paused;
                    } else if gs_guard.status == GameStatus::Paused {
                        gs_guard.status = GameStatus::Playing;
                    }
                //QUIT
                } else if QUIT_KEYS.contains(&key.code) {
                    gs.write().unwrap().status = GameStatus::ByeBye;
                    break;
                //MENU
                } else if MAIN_MENU_KEYS.contains(&key.code) {
                    gs.write().unwrap().status = GameStatus::Menu;
                    break;
                //RESTART
                } else if RESET_KEYS.contains(&key.code) {
                    gs.write().unwrap().status = GameStatus::Restarting;
                //Arrow input
                } else {
                    let direction_input = match key.code {
                        KeyCode::Left => Some(Direction::Left),
                        KeyCode::Right => Some(Direction::Right),
                        KeyCode::Down => Some(Direction::Down),
                        KeyCode::Up => Some(Direction::Up),
                        _ => None,
                    };
                    if let Some(dir) = direction_input {
                        *direction.write().unwrap() = dir;
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum GreetingMenuInput {
    Fruits,
    Speed,
    Start,
    Parameters,
    Help,
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
                    Some(GreetingMenuInput::Help)
                } else if MAIN_MENU_KEYS.contains(&key.code) {
                    Some(GreetingMenuInput::Main)
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

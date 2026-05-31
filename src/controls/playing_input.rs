use crate::controls::direction::Direction;
use crate::controls::main_menu::{ENTER_KEYS, NEXT_KEYS, PREVIOUS_KEYS};
use crate::game_logic::state::{GameOverMenu, GameState, GameStatus};
use crossterm::event;
use crossterm::event::{KeyCode, KeyEventKind};
use std::sync::{Arc, RwLock};
use tracing::{error, info, warn};

pub const QUIT_KEYS: [KeyCode; 3] = [KeyCode::Char('q'), KeyCode::Char('Q'), KeyCode::Tab];
pub const START_KEYS: [KeyCode; 2] = [KeyCode::Char('r'), KeyCode::Char('R')];
pub const MAIN_MENU_KEYS: [KeyCode; 3] = [KeyCode::Char('m'), KeyCode::Char('M'), KeyCode::Home];
//const DIRECTIONAL_KEYS: [KeyCode; 4] = [KeyCode::Down, KeyCode::Up, KeyCode::Left, KeyCode::Right];
pub const PAUSE_KEYS: [KeyCode; 4] = [
    KeyCode::Char('p'),
    KeyCode::Char('P'),
    KeyCode::Char(' '),
    KeyCode::Char('-'),
];
pub const RESET_KEYS: [KeyCode; 2] = [KeyCode::Char('r'), KeyCode::Char('R')];

/// You cannot block middle-click paste/scroll behavior from inside your Rust TUI app.
/// If you really want to disable it, you would have to modify user system settings or terminal emulator config
/// (e.g., in alacrity, kitty, gnome-terminal, etc.)
/// That is outside the app's control
/// # Panics
/// if Arc panics while holding the resources (poisoning), no recovery mechanism implemented better crash
pub fn playing_input_loop(direction: &Arc<RwLock<Direction>>, gs: &Arc<RwLock<GameState>>) {
    loop {
        if let Ok(event::Event::Key(key)) = event::read()
            && key.kind == KeyEventKind::Press
        {
            let mut gs_guard = gs.write().unwrap();
            // Handle GameOver navigation separately
            if let GameStatus::GameOver(selection) = gs_guard.status {
                if NEXT_KEYS.contains(&key.code) {
                    gs_guard.status = GameStatus::GameOver(selection.next());
                } else if PREVIOUS_KEYS.contains(&key.code) {
                    gs_guard.status = GameStatus::GameOver(selection.previous());
                } else if ENTER_KEYS.contains(&key.code) {
                    match selection {
                        GameOverMenu::Restart => {
                            info!("Restarting the game ! ");
                            gs_guard.status = GameStatus::Restarting;
                        }
                        GameOverMenu::Menu => {
                            info!("Coming back to menu ! ");
                            gs_guard.status = GameStatus::Menu;
                            break;
                        }
                        GameOverMenu::Quit => {
                            error!("You choose to quit the game 😶‍🌫️  ");
                            gs_guard.status = GameStatus::ByeBye;
                            break;
                        }
                    }
                }
            }
            //We keep an if and not an else if, to keep keyboard shortcut working on game over screen
            //PAUSE
            if PAUSE_KEYS.contains(&key.code) {
                //in others state does nothing to not break game_logic logic
                if gs_guard.status == GameStatus::Playing {
                    info!("Game paused ! ⏸️ ");
                    gs_guard.status = GameStatus::Paused;
                } else if gs_guard.status == GameStatus::Paused {
                    info!("Game restart after pausing ! ⏯️ ");
                    gs_guard.status = GameStatus::Playing;
                }
            //QUIT
            } else if QUIT_KEYS.contains(&key.code) {
                error!("You choose to quit the game 😶‍🌫️ ");
                gs_guard.status = GameStatus::ByeBye;
                break;
            //MENU
            } else if MAIN_MENU_KEYS.contains(&key.code) {
                warn!("Going back to menu ! 🗺️");
                gs_guard.status = GameStatus::Menu;
                break;
            //RESTART
            } else if RESET_KEYS.contains(&key.code) {
                warn!("Game hot restart ! 🤖");
                gs_guard.status = GameStatus::Restarting;
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

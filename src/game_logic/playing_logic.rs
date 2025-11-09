use crate::controls::direction::Direction;
use crate::controls::input::{greeting_screen_manage_input, GreetingMenuInput};
use crate::game_logic::fruits_manager::FruitsManager;
use crate::game_logic::game_options::GameOptions;
use crate::game_logic::playing_logic::SwitchMenu::{Fruits, Help, Main, Parameters, Run, Speed};
use crate::game_logic::state::{GameState, GameStatus};
use crate::graphics::menus::greeting::{main_greeting_menu, GreetingSimpleDisplay, SwitchMenu};
use crate::graphics::menus::retro_parameter_table::customized_with_cli::setup_and_run_cli_table_parameters;
use crate::graphics::menus::retro_parameter_table::customized_with_fruits::setup_and_run_fruits_table_parameters;
use crate::graphics::sprites::fruit::Fruit;
use crate::graphics::sprites::map::Map;
use crate::graphics::sprites::snake_body::SnakeBody;
use ratatui::DefaultTerminal;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;

const SWITCH_MENUS_OPTION: [SwitchMenu; 6] = [Main, Fruits, Speed, Run, Parameters, Help];

/// # Panics
/// if Arc panics while holding the resources (poisoning),
/// no recovery mechanism implemented better crash  
pub fn playing_logic_loop(
    direction: &Arc<RwLock<Direction>>,
    snake: &Arc<RwLock<SnakeBody>>,
    gs: &Arc<RwLock<GameState>>,
    carte: &Arc<RwLock<Map>>,
    fruits_manager: &Arc<RwLock<FruitsManager>>,
    (game_speed, speed_score_modifier, classic_mode): (u64, u16, bool),
) {
    let mut gsc;
    loop {
        //do not want to keep the lock for too long + cannot hold in the same thread 2 times the same hold
        // so match a clone or use a let
        gsc = gs.read().unwrap().status.clone();
        //dead snakes tell no tales, nor move :p
        match gsc {
            GameStatus::Playing => {
                let mut write_guard = snake.write().unwrap();
                //Move the snake!
                let position = write_guard.ramp(&direction.read().unwrap(), &carte.read().unwrap());
                // Check if we eat a fruit
                let fruits = fruits_manager.read().unwrap().eat_some_fruits(position);
                // fruits effects
                if let Some(fruits) = fruits {
                    let score_fruits = fruits.iter().map(Fruit::get_score).sum::<i32>();
                    let size_effect = fruits.iter().map(Fruit::get_grow_snake).sum::<i16>();
                    // in all cases except classic mode with negative size, we always apply size modifiers
                    if !(classic_mode && size_effect <= 0) {
                        write_guard.relative_size_change(size_effect);
                    }
                    //NB:Converting an u16 to an i32 is always safe in Rust because the range of u16 (0 to 65,535)
                    // fits entirely within the range of i32 (−2,147,483,648 to 2,147,483,647).
                    //So no need to do: speed_score_modifier.try_into().expect("too much")/match for conversion
                    gs.write().unwrap().score += score_fruits * i32::from(speed_score_modifier);
                    fruits_manager.write().unwrap().replace_fruits(&fruits);
                }
                // Check if the gamer will lose one life
                if write_guard.is_snake_eating_itself() {
                    //Ouch. You bite yourself
                    let mut state_guard = gs.write().unwrap();
                    if (state_guard.life) >= 1 {
                        state_guard.life -= 1;
                    }
                    if state_guard.life == 0 {
                        state_guard.status = GameStatus::GameOver;
                    }
                }
            }
            GameStatus::Restarting => {
                //let some time for the restarting screen to appear
                sleep(Duration::from_millis(1000));
                gs.write().unwrap().reset();
                snake.write().unwrap().reset();
                *direction.write().unwrap() = Direction::Right;
                fruits_manager.write().unwrap().reset();
                //graphical resize on rendering part (not really a game_logic constant)
            }
            GameStatus::ByeBye | GameStatus::Menu => break,
            GameStatus::Paused | GameStatus::GameOver => {}
        }
        sleep(Duration::from_millis(game_speed));
    }
}
/// The control part of the main menu
/// allows switching to a submenu (Fruits, Speed, Parameters, etc.)
/// Use `GreetingMenuInput` to known which keys have been used
/// and `GreetingSimpleDisplay` to display an easy menu, without input control (all except run and parameters)
/// Return true if the player wants to play, false otherwise
///
/// # Panics                                                                                              
/// if Terminal writing is not possible
pub fn controls_main_switch_menu(
    terminal: &mut DefaultTerminal,
    options: &mut GameOptions,
) -> bool {
    let mut to_display_menu = GreetingSimpleDisplay::MainMenu;
    // To manage keys to switch the selected item
    let mut selected = 3;
    //first display
    main_greeting_menu(terminal, &to_display_menu, &Run);
    loop {
        match greeting_screen_manage_input() {
            Some(GreetingMenuInput::Parameters) => {
                setup_and_run_parameters_menu(
                    terminal,
                    options,
                    &mut selected,
                    &mut to_display_menu,
                );
            }
            Some(GreetingMenuInput::Fruits) => {
                setup_and_run_parameters_fruits(terminal, &mut selected, &mut to_display_menu);
            }
            Some(GreetingMenuInput::Next) => {
                selected = (selected + 1) % SWITCH_MENUS_OPTION.len();
            }
            Some(GreetingMenuInput::Previous) => {
                selected = (selected + SWITCH_MENUS_OPTION.len() - 1) % SWITCH_MENUS_OPTION.len();
            }
            //NB: selection can be done by selecting ENTER on a menu entry or using a key shortcut
            // that why there is GreetingMenuInput::<option> directly alongside an enter option
            Some(GreetingMenuInput::Enter) => {
                //for common options
                to_display_menu =
                    GreetingSimpleDisplay::from(SWITCH_MENUS_OPTION[selected].clone());
                if SWITCH_MENUS_OPTION[selected] == Run {
                    //start the game
                    return true;
                } else if SWITCH_MENUS_OPTION[selected] == Parameters {
                    setup_and_run_parameters_menu(
                        terminal,
                        options,
                        &mut selected,
                        &mut to_display_menu,
                    );
                } else if SWITCH_MENUS_OPTION[selected] == Fruits {
                    setup_and_run_parameters_fruits(terminal, &mut selected, &mut to_display_menu);
                }
            }
            Some(GreetingMenuInput::QuitGame) => {
                return false;
            }
            Some(GreetingMenuInput::Start) => {
                return true;
            }
            Some(x) => to_display_menu = GreetingSimpleDisplay::from(x),
            _ => {}
        }
        main_greeting_menu(terminal, &to_display_menu, &SWITCH_MENUS_OPTION[selected]);
    }
}

fn setup_and_run_parameters_menu(
    terminal: &mut DefaultTerminal,
    options: &mut GameOptions,
    selected: &mut usize,
    to_display_menu: &mut GreetingSimpleDisplay,
) {
    setup_and_run_cli_table_parameters(terminal, options);
    // Come back to the default menu display
    *selected = 3;
    *to_display_menu = GreetingSimpleDisplay::MainMenu;
}

fn setup_and_run_parameters_fruits(
    terminal: &mut DefaultTerminal,
    selected: &mut usize,
    to_display_menu: &mut GreetingSimpleDisplay,
) {
    setup_and_run_fruits_table_parameters(terminal);
    // Come back to the default menu display
    *selected = 3;
    *to_display_menu = GreetingSimpleDisplay::MainMenu;
}

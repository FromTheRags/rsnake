pub use crate::controls::direction::Direction;
use crate::controls::main_menu::controls_main_switch_menu;
use crate::controls::playing_input::playing_input_loop;
pub use crate::controls::speed::Speed;
pub use crate::game_logic::fruits_manager::FruitsManager;
use crate::game_logic::game_options::GameOptions;
use crate::game_logic::playing_logic::playing_logic_loop;
pub use crate::game_logic::state::{GameState, GameStatus};
use crate::graphics::playing_render::playing_render_loop;
use crate::graphics::sprites::map::Map;
use crate::graphics::sprites::snake_body::SnakeBody;
use ratatui::DefaultTerminal;
use ratatui::text::Span;
use std::cmp::max;
use std::sync::{Arc, RwLock};
use std::thread;
use tracing::{debug, info, trace};

/// our game engine
/// NB: 'c must outlive 'b as, 'c (fruits manager) uses in intern the map with lock on it.
/// NB: 't the terminal life must outlive all the other lifetimes
pub struct Game<'a, 'b, 'c: 'b, 't: 'a + 'b + 'c> {
    /// Game main parameters
    options: &'t GameOptions,
    /// The game logic speed, linked to the snake movements
    speed: Speed,
    /// Represents the snake moving around
    serpent: Arc<RwLock<SnakeBody<'a>>>,
    /// The direction chosen by the player for the snake
    direction: Arc<RwLock<Direction>>,
    /// The game logic map where items/snake are displayed
    /// NB: As we want a resizable map, `RwLock`, otherwise use only Arc<Map> (immuable)
    carte: Arc<RwLock<Map<'b>>>,
    /// Game states and metrics (life etc.)
    state: Arc<RwLock<GameState>>,
    /// Manage fruits (popping, eaten, etc.)
    fruits_manager: Arc<RwLock<FruitsManager<'c, 'b>>>,
    /// The current terminal
    terminal: &'t mut DefaultTerminal,
}
impl<'a, 'b, 'c, 't> Game<'a, 'b, 'c, 't> {
    #[must_use]
    fn new(
        options: &'t GameOptions,
        serpent: SnakeBody<'a>,
        carte: Map<'b>,
        terminal: &'t mut DefaultTerminal,
    ) -> Game<'a, 'b, 'c, 't> {
        let arc_carte = Arc::new(RwLock::new(carte));
        let life = options.life;
        let fruits_nb = options.nb_of_fruits;
        let speed = options.speed;
        Game {
            options,
            speed,
            serpent: Arc::new(RwLock::new(serpent)),
            direction: Arc::new(RwLock::new(Direction::Right)),
            carte: arc_carte.clone(),
            state: Arc::new(RwLock::new(GameState::new(life))),
            fruits_manager: Arc::new(RwLock::new(FruitsManager::new(
                fruits_nb,
                arc_carte.clone(),
                options.fruit_timer,
                f32::from(options.fruit_duration_seconds),
            ))),
            terminal,
        }
    }
    /// Displays the game menu and handles user navigation
    ///
    /// # Panics
    ///
    /// This function will panic if the internal `state` lock is poisoned
    /// and cannot be read.
    pub fn menu(mut options: GameOptions, mut terminal: DefaultTerminal) {
        info!("Welcome dear player ! Make your choice on Main menu !");
        //one loop means one game, hard reset of the game from the menu
        // (as parameters can change in the parameter menu)
        loop {
            //Display the menu and get the user choice: play or not
            // (as well as others menu options of course)
            if controls_main_switch_menu(&mut terminal, &mut options) {
                info!("Let's play! 🐍 (Run has been entered in the menu, starting the game...)");
                // if the player wants to play, we need to initiate some game values
                //  to get the correct case size for display
                let case_size = u16::try_from(max(
                    Span::raw(&options.body_symbol).width(),
                    Span::raw(&options.head_symbol).width(),
                    //ratatui using UnicodeWidthStr crates as dep
                ))
                .expect("Bad symbol size, use a real character");
                let carte: Map = Map::new(case_size, terminal.get_frame().area());
                let serpent: SnakeBody = SnakeBody::new(
                    &options.body_symbol,
                    &options.head_symbol,
                    options.snake_length,
                    GameOptions::initial_position(),
                    case_size,
                );
                info!(Snake_lenght = ?options.snake_length, Snake_head = ?options.head_symbol, Snake_body = ?options.body_symbol, "Snake info for starting");
                trace!(?serpent);
                debug!(?options);
                let mut game = Game::new(&options, serpent, carte, &mut terminal);
                game.start();
                if game
                    .state
                    .read()
                    .expect("Panic in a previous thread, check previous error")
                    .status
                    == GameStatus::ByeBye
                {
                    break;
                }
            } else {
                break;
            }
        }
        info!("Good bye, come back soon, snaker is waiting for you 🐍");
    }
    /// Start the main Game threads: input, rendering, logic
    pub fn start(&mut self) {
        debug!("Starting game threads");
        // Be careful: not all threads on the same structure and do not keep them too much
        // => performance issue otherwise
        // Prepare thread use of variable

        //For logical thread
        let logic_snake = Arc::clone(&self.serpent);
        let logic_gs = Arc::clone(&self.state);
        let logic_dir = Arc::clone(&self.direction);
        let carte = Arc::clone(&self.carte);
        let fruits_manager = Arc::clone(&self.fruits_manager);

        // For input management thread
        let input_gs = Arc::clone(&self.state);
        let input_dir = Arc::clone(&self.direction);

        //if we want to have a variable speed put it under an Arc<Rw>, constant can directly be put under an Arc
        // or share as a normal variable by copy
        //Game speed is a constant by game, so we can clone it normally
        let current_game_speed = self.speed;
        let negative_size_fruits = self.options.negative_size_fruits;
        let snake_growth_factor = self.options.snake_growth_factor;
        let snake_symbols = format!("{}{}", self.options.head_symbol, self.options.body_symbol);
        //In a scope to have auto cleaning by auto join at the end of the main thread
        thread::scope(|s| {
            // Game logic thread
            thread::Builder::new()
                //For better log name the thread, otherwise just s.spawn()
                .name("t_game_logic".to_string())
                .spawn_scoped(s, move || {
                    playing_logic_loop(
                        &logic_dir,
                        &logic_snake,
                        &logic_gs,
                        &carte,
                        &fruits_manager,
                        (
                            current_game_speed,
                            snake_symbols,
                            negative_size_fruits,
                            snake_growth_factor,
                        ),
                    );
                })
                .expect("Unable to create the thread");
            // input logic thread
            thread::Builder::new()
                //For better log name the thread, otherwise just s.spawn()
                .name("t_input".to_string())
                .spawn_scoped(s, move || {
                    playing_input_loop(&input_dir, &input_gs);
                })
                .expect("Unable to create the thread");

            // Graphical thread (last one, reusing the main thread)
            playing_render_loop(
                &Arc::clone(&self.carte),
                &Arc::clone(&self.fruits_manager),
                &Arc::clone(&self.state),
                &Arc::clone(&self.serpent),
                self.options.caps_fps,
                (self.speed.score_modifier(), self.speed.symbol()),
                self.terminal,
            );
        });
    }
}

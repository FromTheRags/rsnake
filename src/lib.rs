#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
// Documentation for all Clippy lints: https://github.com/rust-lang/rust-clippy/
//! # Snake Game using Ratatui
//!
//! This module implements a terminal-based snake game using the Ratatui crate for rendering.
//!
//! ## Features
//! - **Terminal UI**: Uses Ratatui for rendering a grid-based game.
//! - **Game Logic**: Manages snake movement, collisions, and scoring.
//! - **Multithreading**: Uses multiple threads for input handling, rendering at 60 FPS, and game logic execution.
//! - **Emoji-based graphics**: Supports rendering the snake using emojis instead of ASCII.
//! - **Configurable parameters**: With `clap` for command-line arguments.
//!
//! ## TODO
//! - [ ] Add a save score (local db) with a pseudo got from cmdline
//! - [ ] Add some performance log with tracing for example
//! - [ ] Fix too much life display outside of screen
//!
//!
//! ## References
//! - Clippy lints: <https://github.com/rust-lang/rust-clippy/>
//! - Ratatui tutorial: <https://ratatui.rs/tutorials/hello-world/>
//! - Example: <https://ratatui.rs/examples/widgets/canvas/>
//!
//! ## Architecture
//! - Uses `RwLock` for synchronization.
//! - Spawns separate threads for input handling, rendering (60Hz), and game logic execution.
//!
//! ## Documentation generation
//! - `cargo doc --document-private-items --no-deps --open`
//!
//! ## Tests
//!  - As usual run them with `cargo test` the project is set up with a lib containing all the code, and a main.rs just calling it
//!  - As this is a widespread pattern providing full compliance with the Rust test ecosystem, allowing doc comment to be automatically tested, for example.

pub mod controls;
pub mod game_logic;
pub mod graphics;

use crate::game_logic::playing_thread_manager::Game;
use clap::Parser;
use game_logic::game_options::GameOptions;

/// # Panics
/// If bad characters (invalid size) are provided for snake body or head
pub fn start_snake() {
    // get command line options and parsed them to check for errors (auto using value parser in clap)
    let mut args = GameOptions::parse();
    //println!("{}", args.to_structured_toml());
    //load or save as wished, for the easiest use hard-coded path,as people using it will not like cli
    if args.save {
        args.save_to_toml(game_logic::game_options::SAVE_FILE)
            .expect("Fail to save Snake configuration file");
    }
    if args.load {
        args = GameOptions::load_from_toml(game_logic::game_options::SAVE_FILE)
            .expect("Fail to load Snake configuration file");
    }
    // If everything is OK, inits terminal for rendering
    let terminal = ratatui::init();

    // init our own Game engine an display greeting screen
    Game::menu(args, terminal);

    //in all cases, restore
    ratatui::restore();
}

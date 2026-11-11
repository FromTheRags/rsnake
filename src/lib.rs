// Lint configuration (unsafe_code forbid and Clippy groups/lints) is centralized in Cargo.toml ([lints]).
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
//! ## References
//! - Clippy lints: <https://github.com/rust-lang/rust-clippy/>
//! - Ratatui tutorial: <https://ratatui.rs/tutorials/hello-ratatui/>
//! - Example: <https://ratatui.rs/examples/widgets/canvas/>
//!
//! ## Architecture
//! - Uses `RwLock` for synchronization.
//! - Spawns separate threads for input handling, rendering (60Hz), and game logic execution.
//! - For more details, see [ARCHITECTURE.md](https://github.com/FromTheRags/rsnake/blob/main/ARCHITECTURE.md).
//!
//! ## Documentation generation
//! - `cargo doc --document-private-items --no-deps --open`
//!
//! ## Tests
//!  - As usual, run them with `cargo test` the project is set up with a lib containing all the code, and a main.rs just calling it
//!  - As this is a widespread pattern providing full compliance with the Rust test ecosystem, allowing doc comment to be automatically tested, for example.
//!    For more details, see [readme.md](https://github.com/FromTheRags/rsnake/blob/main/README.md)

pub mod controls;
pub mod game_logic;
pub mod graphics;

use crate::game_logic::logger::log_configuration::{
    DEFAULT_LOG_CONFIG_PATH, LogLevel, init_logger, update_log_level,
};
use crate::game_logic::playing_thread_manager::Game;
use clap::Parser;
use game_logic::game_options::GameOptions;
use tracing::{debug, info};

/// # Panics
/// If bad characters (invalid size) are provided for snake body or head
pub fn start_snake() {
    // Initialize logger: configuration from a dedicated TOML file
    let (_guard, log_config) = init_logger(DEFAULT_LOG_CONFIG_PATH);
    // Get command line options and parsed them, add resolve to interpret game options
    let args = match GameOptions::parse().resolve() {
        Ok(args) => args,
        Err(e) => panic!("Fail to resolve command line arguments: {e}"),
    };
    // CLI log level (if explicitly set, i.e., not Off) overrides the log file configuration log level.
    if args.log_level != LogLevel::Off {
        update_log_level(args.log_level);
    }
    debug!("{}", args.to_structured_toml());
    info!("Snake game arguments parsed and configuration is successful!");
    debug!(log_config = ?log_config);
    // If everything is OK, inits terminal for rendering
    let terminal = ratatui::init();

    // init our own Game engine and a display greeting screen
    Game::menu(args, terminal);

    //in all cases, restore
    ratatui::restore();
}

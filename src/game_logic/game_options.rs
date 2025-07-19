use crate::controls::speed::Speed;
use crate::graphics::graphic_block::Position;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::Path;
/// Initial position of the snake's head at the start of the game
const INI_POSITION: Position = Position { x: 50, y: 5 };
pub const SAVE_FILE: &str = "snake_config.toml";
#[allow(clippy::needless_raw_string_hashes)]
const PARAMS_HEADER: &str = r#"
# Snake Game Configuration
# ---------------------------
# classic_mode:     true for classic rules (walls kill, no wrapping)
# uncaps_fps:       disables frame limiting (true = no limit) 
# life:             starting lives
# nb_of_fruit:      number of fruits available in the game at once
# body_symbol:      character for the snake's body  
# head_symbol:      character for the snake's head
# snake_length:     initial length of the snake
# speed:            speed of the snake (Slow, Normal, Fast, Tremendous)
# save/load:        save/load game parameters to/from file, not very useful from a file, but useful from the CLI
"#;
/// Structure holding all the configuration parameters for the game
#[derive(Parser, Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
#[command(
    author,
    version,
    long_version = concat!("v", env!("CARGO_PKG_VERSION"), " by ", env!("CARGO_PKG_AUTHORS"),
    env!("CARGO_PKG_DESCRIPTION"),
    "\nRepository: ", env!("CARGO_PKG_REPOSITORY"),
    "\nBuilt with Rust ", env!("CARGO_PKG_RUST_VERSION")),
    about = concat!("v", env!("CARGO_PKG_VERSION"), " by ", env!("CARGO_PKG_AUTHORS"),
    "\nSnake Game in terminal with CLI arguments.\nQuick custom run: cargo run -- -z 👾 -b 🪽 -l 10 "),
    long_about = concat!("v", env!("CARGO_PKG_VERSION"), " by ", env!("CARGO_PKG_AUTHORS"),"\n",
    env!("CARGO_PKG_DESCRIPTION"), " where you can configure the velocity, \
    snake appearance, and more using command-line arguments.\nExample for asian vibes: rsnake -z 🐼 -b 🍥")
)]
#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct GameOptions {
    /// Speed of the snake (Slow, Normal, Fast, Tremendous)
    /// Derives `ValueEnum` on the enum Speed and enforces the type
    /// `clap::ValueEnum`, which automatically handles possible values and displays them in the help message.
    /// Now, clap enforces valid inputs without requiring a manual `FromStr` implementation.
    #[arg(
        short,
        long,
        value_enum, default_value_t = Speed::Normal,
        help = "Sets the movement speed of the snake."
    )]
    pub speed: Speed,

    /// Snake symbol (emoji or character)
    /// Defines short value because doublon, as short and long,
    /// are by default based on the name of the variable
    #[arg(
        short = 'z',
        long,
        default_value = "🎄",
        help = "Symbol used to represent the snake's head. Cool symbol as 😁🤠🤡🥳🥸👺👹👽\
        👾🐼🐉🐍🦀🐳     .
        Do not use one emoji displaying on multiple chars as it will badly be rendered, but multiple characters and classic are allowed as:
        -z ZZ -b aa"
    )]
    pub head_symbol: String,

    /// Snake trail symbol (emoji or character)
    #[arg(
        short,
        long,
        default_value = "❄️",
        help = "Symbol used to represent the snake's body/trail. Cool symbol as 🍁😋🥑🐾🐢🦎🪽🐥\
        🐣♡🦠🦴👣🍥🥮🍪🍩🧊🏴🧨🦑🐟     . 
        Do not use one emoji displaying on multiple chars as it will badly be rendered, but multiple characters are allowed as: -z 🐳 -b 👽-🦴.
        Be careful, because of unicode width, not all combinaison are playable depending of your font/terminal"
    )]
    pub body_symbol: String,

    /// Initial length of the snake
    #[arg(
        short = 'n',
        long,
        default_value_t = 10,
        help = "Defines the initial length of the snake [1-999]"
    )]
    pub snake_length: u16,

    /// Number of lives
    #[arg(
        short,
        long,
        default_value_t = 3,
        help = "Defines the initial number of lives for the player.[1-9999]"
    )]
    pub life: u16,

    /// Number of fruits in the game
    #[arg(
        short = 'f',
        long,
        default_value_t = 5,
        help = "Defines the number of fruits available in the game_logic at once.[1-10000]"
    )]
    pub nb_of_fruit: u16,

    /// Caps to 60 FPS or not
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Set to uncaps default FPS limit (by default limit 60 FPS)"
    )]
    pub uncaps_fps: bool,
    /// Classic game with only growing snake
    #[arg(
        long,
        default_value_t = false,
        help = "Classic game_logic with only growing snake, so fruits with negative size effect will have no size effect"
    )]
    pub classic_mode: bool,
    /// Save game parameters
    /// NB: if one-day use with a user filename, add a clap argument for autocompletion: `value_hint` = `clap::ValueHint::FilePath`
    #[arg(
        long,
        default_value_t = false,
        help = format!("Save current game parameters to {SAVE_FILE} configuration file in the same folder as the executable.")
    )]
    pub save: bool,
    /// Load game parameters
    #[arg(
        long,
        default_value_t = false,
        help = format!("Load current game parameters from {SAVE_FILE} configuration file in the same folder as the executable. Override cli arguments.")
    )]
    pub load: bool,
}

impl GameOptions {
    /// Returns the initial snake position
    #[must_use]
    pub fn initial_position() -> Position {
        INI_POSITION
    }

    /// Gets the current speed setting
    #[must_use]
    pub fn get_speed(&self) -> Speed {
        self.speed
    }

    /// Save the current parameters to a TOML file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or written to.
    ///
    /// # Panics
    ///
    /// Panics if the game parameters cannot be serialized to TOML.
    pub fn save_to_toml<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        self.save = false;
        self.load = false;
        let toml_string =
            toml::to_string_pretty(self).expect("Failed to serialize GameParameters to TOML");
        let full_output = format!("{PARAMS_HEADER}\n{toml_string}");
        let mut file = File::create(path)?;
        file.write_all(full_output.as_bytes())?;
        Ok(())
    }

    /// Load parameters from a TOML file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or read.
    ///
    /// # Panics
    ///
    /// Panic if the file contents cannot be deserialized as valid TOML.
    pub fn load_from_toml<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let params =
            toml::from_str(&contents).expect("Failed to deserialize GameParameters from TOML");
        Ok(params)
    }
    // In real Life, we will validate data more thoroughly
    pub fn validate_and_adapt(&mut self) {
        self.nb_of_fruit = self.nb_of_fruit.clamp(1, 9999);
        self.life = self.life.clamp(1, 9999);
        self.snake_length = self.snake_length.clamp(1, 999);
    }
}

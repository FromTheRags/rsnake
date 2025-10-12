use crate::controls::speed::Speed;
use crate::graphics::graphic_block::Position;
use crate::graphics::menus::retro_parameter_table::generic_logic::{
    ApplyParameter, CellValue, RowData,
};
use clap::Parser;
use clap::{ArgAction, CommandFactory};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::iter::Iterator;
use std::ops::RangeInclusive;
use std::path::Path;
use toml::Table;
use unicode_segmentation::UnicodeSegmentation;
/// Initial position of the snake's head at the start of the game
pub const INI_POSITION: Position = Position { x: 50, y: 5 };
pub const SAVE_FILE: &str = "snake_config.toml";
//Options to not display in the table menu in-game parameters
pub const ONLY_FOR_CLI_PARAMETERS: [&str; 3] = ["save", "load", "no-"];
//Later auto generate the header based on help message as in-game table menu
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
# speed:            speed of the snake (Slow, Normal, Fast, Crazy)
# save/load:        save/load game parameters to/from file, not very useful from a file, but useful from the CLI
"#;
/// Define game parameters with their valid ranges
/// Format expected: `define_args_with_ranges`! {
///    x: 1 => 999,
///    y: 1 => 99,
/// }
/// To have some fun with basic macro and to facilitate parsing for in-game menu
/// (avoid help text parsing, avoid global variable (or lazy init as hashmap cannot be global), avoid eternal crate, add metadata like all flatten)
/// This macro creates:
/// 1. A function to get parameter ranges
/// 2. A parameter parser functions for use with clap
///    NB: to avoid the &str, a proc macro reading the parameter name of the cli struct is needed (too heavy for the benefits)
macro_rules! define_args_with_ranges {
    (
        $( $field_name:ident: $min:expr => $max:expr ),* $(,)?
    ) => {
        /// define a const to avoid str errors
        $(const $field_name: &str = stringify!($field_name);)*
        /// Returns the valid range for the parameter in O(1) or None
        #[must_use] pub fn get_parameter_range(param_name: &str) -> Option<std::ops::RangeInclusive<u16>> {
            let idiomatic =param_name.to_string().replace("-","_").to_uppercase();
            match idiomatic.as_str() {
                $(
                    stringify!($field_name) => Some($min..=$max)
                    ,
                )*
                _ => None,
            }
        }
        /// Get a clap value parser for a specific parameter or the default 1..99 range
        #[must_use] fn get_parameter_parser(param_name: &str) -> clap::builder::RangedI64ValueParser<u16> {
            match param_name {
                $(
                    stringify!($field_name) =>
                        clap::value_parser!(u16).range($min as i64..=$max as i64)
                    ,
                )*
                _ => clap::value_parser!(u16).range(1_i64..=99_i64),
            }
        }
    };
}
// Define all arguments and ranges in one place
define_args_with_ranges! {
    SNAKE_LENGTH: 1 => 999,
    LIFE: 1 => 99,
    NB_OF_FRUITS : 1 => 999,
}
const MAX_EMOJI_BY_LINE_COUNT: u16 = 19;
//split in 2 arrays representing max emoji on one line because easier to display
// (the main use case of the const)
pub const DISPLAYABLE_EMOJI: [&str; 38] = [
    "🍁", "😋", "🥑", "🐾", "🐢", "🦎", "🪽", "🐥", "🐣", "🦠", "🦴", "👣", "🍥", "🥮", "🍪", "🍩",
    "🧊", "🏴", "🧨", "🦑", "🐟", "😁", "🤠", "🤡", "🥳", "🥸", "👺", "👹", "👾", "🐼", "🐉", "🐍",
    "🦀", "🐳", "🎄", "❄️", "👽", "@",
];
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
    long_about = concat!("v", env!("CARGO_PKG_VERSION"), " by ", env!("CARGO_PKG_AUTHORS"), "\n",
    env!("CARGO_PKG_DESCRIPTION"), " where you can configure the velocity, \
    snake appearance, and more using command-line arguments.\nExample for asian vibes: rsnake -z 🐼 -b 🍥")
)]
#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct GameOptions {
    /// Speed of the snake (Slow, Normal, Fast, Crazy)
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
    /// Default is Christmas tree
    #[arg(
        short = 'z',
        long,
        default_value = DISPLAYABLE_EMOJI[34],
        help = format!("Symbol used to represent the snake's head.\nHint:{}"
        ,GameOptions::emojis_with_news_line()),
        long_help = format!("Symbol used to represent the snake's head.\nHint:{},\
        \n/!\\ emoji displaying on multiple chars could be badly rendered/unplayable",GameOptions::emojis_with_news_line()),
        value_parser = |s: &str| -> Result<String, String>{
            if s.graphemes(true).count() != 1 {
                return Err(String::from("Head symbol must be exactly one grapheme / character"));
            }
            Ok(s.to_string())
        }
    )]
    pub head_symbol: String,

    /// Snake trail symbol (emoji or character)
    /// need to operate over graphene not chars
    /// see <https://crates.io/crates/unicode-segmentation/> /
    /// Or deep explanation:<https://docs.rs/bstr/1.12.0/bstr/#when-should-i-use-byte-strings>
    /// Default is snow emoji
    #[arg(
        short,
        long,
        default_value = DISPLAYABLE_EMOJI[35],
        help = format!("Symbol used to represent the snake's body/trail.\
        \nHint:{}",GameOptions::emojis_with_news_line()),
        long_help = format!("Symbol used to represent the snake's body/trail.\
        \nHint:{}\n/!\\ emoji displaying on multiple chars could be badly rendered/unplayable",GameOptions::emojis_with_news_line()),
        value_parser = |s: &str| -> Result<String, String>{
            if s.graphemes(true).count() != 1 {
                return Err(String::from("Head symbol must be exactly one grapheme / character"));
            }
            Ok(s.to_string())
        }
    )]
    pub body_symbol: String,

    /// Initial length of the snake
    #[arg(
        short = 'n',
        long, // = SNAKE_LENGTH
        default_value_t = 10,
        value_parser = get_parameter_parser(SNAKE_LENGTH),
        help = format!("Defines the initial length of the snake {}",pretty_range(get_parameter_range(SNAKE_LENGTH).unwrap()))
    )]
    pub snake_length: u16,

    /// Number of lives
    #[arg(
        short,
        long,
        default_value_t = 3,
        value_parser = get_parameter_parser(LIFE),
        help = format!("Defines the initial number of lives for the player {}",pretty_range(get_parameter_range(LIFE).unwrap()))
    )]
    pub life: u16,

    /// Number of fruits in the game
    #[arg(
        short = 'f',
        long,
        default_value_t = 5,
        value_parser = get_parameter_parser(NB_OF_FRUITS),
        help = format!("Defines the number of fruits available at once {}",pretty_range(get_parameter_range(NB_OF_FRUITS).unwrap()))
    )]
    pub nb_of_fruits: u16,
    /// Modern way to do CLI, two dedicated flag to set/unset the value, beginning with --no- (for false)
    /// UX better than --feature false / --feature true, better than default (no flag = false). If you want possibility to set both values,
    /// as no clear default value or want to be able to easily programmatically change the value (as there)
    /// or to have a default at true <hr>
    /// See: <https://jwodder.github.io/kbits/posts/clap-bool-negate/>
    /// As default is true, no and value are swaped
    #[arg(
        long = "caps-fps",
        overrides_with = "caps_fps",
        help = "Set to caps FPS limit (max 60 FPS) [default] "
    )]
    #[serde(skip, default = "default_false")]
    no_caps_fps: bool,
    #[arg(
        long = "no-caps-fps",
        default_value_t = true,
        action = ArgAction::SetFalse,
    )]
    pub caps_fps: bool,
    /// As default is false, order is more logical
    #[arg(
        long,
        default_value_t = false,
        overrides_with = "no_classic_mode",
        help = "Classic mode: classic logic with only growing snake no cut-size-fruit \nNo-classic [default] with a more modern and challenging logic with cut-size-fruits "
    )]
    pub classic_mode: bool,
    #[arg(long)]
    #[serde(skip, default = "default_true")]
    no_classic_mode: bool,
    /// Save game parameters
    /// TODO: replace with preset 1..7
    /// `value_name("FILE`")
    /// NB: if one-day use with a user filename, add a clap argument for autocompletion: `value_hint` = `clap::ValueHint::FilePath`
    /// Better use preset for a game
    #[arg(
        long,
        default_value_t = false,
        help = format!("Save current game parameters to {SAVE_FILE} configuration file in the same folder as the executable.")
    )]
    #[serde(skip, default = "default_false")]
    pub save: bool,
    /// Load game parameters
    #[arg(
        long,
        default_value_t = false,
        help = format!("Load current game parameters from {SAVE_FILE} configuration file in the same folder as the executable. Override cli arguments.")
    )]
    #[serde(skip, default = "default_false")]
    pub load: bool,
}

impl GameOptions {
    /// Returns the initial snake position
    #[must_use]
    pub fn initial_position() -> Position {
        INI_POSITION
    }
    #[must_use]
    pub fn emojis_with_news_line() -> String {
        DISPLAYABLE_EMOJI
            .iter()
            .enumerate()
            .map(|(i, e)| {
                if i == MAX_EMOJI_BY_LINE_COUNT as usize {
                    "\n".to_string() + e
                } else {
                    (*e).to_string()
                }
            })
            .collect::<String>()
    }
    pub fn emojis_iterator() -> impl Iterator<Item = String> {
        DISPLAYABLE_EMOJI
            .iter()
            .map(std::string::ToString::to_string)
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
    /// To be editable easily
    /// # Panics
    /// if self cannot be parsed ( not possible)
    #[must_use]
    pub fn to_structured_toml(&self) -> Table {
        let toml_string =
            toml::to_string_pretty(self).expect("Failed to serialize GameParameters to TOML");
        toml_string.parse::<Table>().expect("invalid doc")
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
        // Later: apply same restrictions as CLI with a clamp
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let params =
            toml::from_str(&contents).expect("Failed to deserialize GameParameters from TOML");
        Ok(params)
    }
}

fn pretty_range(r: RangeInclusive<u16>) -> String {
    format!("[{}-{}]", r.start(), r.end()).to_string()
}
// Serde trick
fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}

impl ApplyParameter for GameOptions {
    fn apply(&mut self, rows: &[RowData]) {
        let command = GameOptions::command();
        let prog_name = command.get_name().to_string();
        let mut new_args = vec![prog_name];
        for row in rows {
            for cell in &row.cells {
                if let CellValue::Options {
                    option_name,
                    values,
                    index,
                    ..
                } = cell
                {
                    let value = &values[*index];
                    match value.parse::<bool>() {
                        Ok(bv) => {
                            // Modern way to do CLI, two dedicated flag to set/unset the value, beginning with --no- (for false)
                            // UX better than --feature false / --feature true, better than default (no flag = false). If you want the possibility to set both values,
                            // as no clear default value or want to be able to easily programmatically change the value (as there)
                            // or to have a default at true
                            let bv_name: String = if bv {
                                option_name.clone()
                            } else {
                                option_name.replace("--", "--no-")
                            };
                            new_args.push(bv_name);
                        }
                        Err(_) => {
                            //not a boolean value
                            new_args.extend([option_name.clone(), value.clone()]);
                        }
                    }
                }
            }
        }
        // Update all the game options as a reparsing (only one way to update value to check).
        // Some debate over the utility of this feature for clap, but widely used to update from env / configuration
        //  Allows keeping the struct for cli parameter as a model object, feeding it with different stream of data.
        // The backup solution is to serialize the current value in TOML and load them in game_options as already done for the file saving
        // (safe as constraints in-game value)
        self.update_from(new_args);
    }
}

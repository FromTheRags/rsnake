use crate::controls::speed::Speed;
use crate::game_logic::logger::log_configuration::LogLevel;
use crate::graphics::graphic_block::Position;
use crate::graphics::menus::retro_parameter_table::generic_logic::{
    ActionParameter, CellValue, RowData,
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
//Options to not display in the table menu in-game parameters
pub const ONLY_FOR_CLI_PARAMETERS: [&str; 2] = ["load", "no-"];
//Later auto generates the header based on the help message as the in-game table menu
#[allow(clippy::needless_raw_string_hashes)]
const PARAMS_HEADER: &str = r#"
# Snake Game Configuration
# ---------------------------
# negative_size_fruits: allow fruits that can shrink the snake
# fruit_timer:      enable fruit lifetime countdown and auto-replacement
# fruit_duration_seconds: base lifetime of a fruit before multipliers are applied
# caps_fps:         Enable frame limiting (default to true, false = no limit)
# life:             starting lives
# nb_of_fruits:     number of fruits available in the game at once
# body_symbol:      character for the snake's body  
# head_symbol:      character for the snake's head
# snake_length:     initial length of the snake
# speed:            speed of the snake (Slow, Normal, Fast, Crazy)
# log_level:        logging level (off, error, warn, info, debug, trace)
"#;

///To be able to iterate over range in a meta-way, see in table parameter very useful
macro_rules! define_args_withs {
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
    /// Get a clap value parser for a specific parameter or the default range of 1..99
    #[must_use] fn get_parameter_range_parser(param_name: &str) -> clap::builder::RangedI64ValueParser<u16> {
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
//Define all arguments and ranges in one place
//Snake length begins at 2 to have a head and a body different
define_args_withs! {
SNAKE_LENGTH: 2 => 999,
LIFE: 1 => 99,
NB_OF_FRUITS : 1 => 999,
FRUIT_DURATION_SECONDS: 1 => 60,
PRESETS: 1 => 7,
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
#[derive(Parser, Serialize, Deserialize, Debug, Clone, Default)]
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
        help = "Sets the movement speed of the snake.",
        ignore_case = true
    )]
    pub speed: Speed,

    /// Snake symbol (emoji or character)
    /// Defines short value because doublon, as short and long,
    /// are by default based on the name of the variable
    /// Default is a Christmas tree
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
        value_parser = get_parameter_range_parser(SNAKE_LENGTH),
        help = format!("Defines the initial length of the snake {}",pretty(get_parameter_range(SNAKE_LENGTH).unwrap()))
    )]
    #[serde(alias = "SNAKE_LENGTH")]
    pub snake_length: u16,

    /// Number of lives
    #[arg(
        short,
        long,
        default_value_t = 3,
        value_parser = get_parameter_range_parser(LIFE),
        help = format!("Defines the initial number of lives for the player {}",pretty(get_parameter_range(LIFE).unwrap()))
    )]
    #[serde(alias = "LIFE")]
    pub life: u16,

    /// Number of fruits in the game
    #[arg(
        short = 'f',
        long,
        default_value_t = 5,
        value_parser = get_parameter_range_parser(NB_OF_FRUITS),
        help = format!("Defines the number of fruits available at once {}",pretty(get_parameter_range(NB_OF_FRUITS).unwrap()))
    )]
    #[serde(alias = "nb_of_fruit", alias = "NB_OF_FRUITS")]
    pub nb_of_fruits: u16,

    /// Enables the fruit timer and automatic replacement when a fruit expires.
    #[arg(
        long = "fruit-timer",
        overrides_with = "fruit_timer",
        help = "Enable the fruit lifetime countdown and automatic replacement [default]"
    )]
    #[serde(skip, default = "default_false")]
    no_fruit_timer: bool,
    #[arg(
        long = "no-fruit-timer",
        default_value_t = true,
        action = ArgAction::SetFalse,
    )]
    pub fruit_timer: bool,

    /// Base fruit lifetime in seconds before per-fruit multipliers are applied.
    #[arg(
        long,
        default_value_t = 20,
        value_parser = get_parameter_range_parser(FRUIT_DURATION_SECONDS),
        help = format!(
            "Defines the base lifetime used to derive fruit durations {}",
            pretty(get_parameter_range(FRUIT_DURATION_SECONDS).unwrap())
        )
    )]
    pub fruit_duration_seconds: u16,

    /// Allow fruits with negative size effects.
    #[arg(
        long = "negative-size-fruits",
        overrides_with = "negative_size_fruits",
        help = "Allow fruits that can shrink the snake [default]"
    )]
    #[serde(skip, default = "default_false")]
    no_negative_size_fruits: bool,
    #[arg(
        long = "no-negative-size-fruits",
        default_value_t = true,
        action = ArgAction::SetFalse,
    )]
    pub negative_size_fruits: bool,

    /// Logging level
    #[arg(
        long,
        value_enum,
        default_value_t = LogLevel::Off,
        help = "Sets the logging level (off, error, warn, info, debug, trace).",
        ignore_case = true
    )]
    pub log_level: LogLevel,
    /// Modern way to do CLI, two dedicated flag to set/unset the value, beginning with --no- (for false)
    /// UX better than --feature false / --feature true, better than default (no flag = false).
    /// If you want the possibility to set both values,
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
    /// Load game parameters
    #[arg(
        long,
        default_missing_value = None,
        value_parser = get_parameter_range_parser(PRESETS),
        help = format!("Load game parameters PRESETS configuration from {}.\
        Files are searched in the same folder as the executable.\
        Save from edit menu to get the template or get one from best configuration example on the repository.\
        Override cli arguments.",pretty(get_parameter_range(PRESETS).unwrap()))
    )]
    #[serde(skip)]
    pub load: Option<u16>,
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
        DISPLAYABLE_EMOJI.iter().map(ToString::to_string)
    }

    /// To be editable easily
    /// # Panics
    /// if self cannot be parsed (not possible)
    #[must_use]
    pub fn to_structured_toml(&self) -> Table {
        let toml_string =
            toml::to_string_pretty(self).expect("Failed to serialize GameParameters to TOML");
        toml_string.parse::<Table>().expect("invalid doc")
    }

    /// Validates and clamps the parameters to their allowed ranges.
    /// Ensures that numeric values are within the bounds defined in the CLI macro
    /// and that symbols are exactly one grapheme long.
    /// Enums are already checked automatically during deserialization
    pub fn validate(&mut self) {
        clamp_to(&mut self.snake_length, get_parameter_range(SNAKE_LENGTH));
        clamp_to(&mut self.life, get_parameter_range(LIFE));
        clamp_to(&mut self.nb_of_fruits, get_parameter_range(NB_OF_FRUITS));
        clamp_to(
            &mut self.fruit_duration_seconds,
            get_parameter_range(FRUIT_DURATION_SECONDS),
        );
        if let Some(preset) = &mut self.load {
            clamp_to(preset, get_parameter_range(PRESETS));
            //self.load = Some(preset.clamp(*PRESETS.start(), *PRESETS.end()));
        }
        // Symbols check: Head and body must be exactly one grapheme.
        // If invalid, they are reset to default emojis.
        if self.head_symbol.graphemes(true).count() != 1 {
            self.head_symbol = DISPLAYABLE_EMOJI[34].to_string();
        }
        if self.body_symbol.graphemes(true).count() != 1 {
            self.body_symbol = DISPLAYABLE_EMOJI[35].to_string();
        }
    }

    /// Load parameters from a preset TOML configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the preset file cannot be opened or read, or if it contains invalid TOML.
    pub fn load_from_toml_preset(preset: u16) -> io::Result<Self> {
        let path = format!("snake_preset_{preset}.toml");
        let mut params = Self::load_from_toml(path)?;
        params.load = Some(preset);
        Ok(params)
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
    fn load_from_toml<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let mut params: Self =
            toml::from_str(&contents).expect("Failed to deserialize GameParameters from TOML");
        params.validate();
        Ok(params)
    }
    /// Save parameters to a preset TOML configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the preset file cannot be created or written to.
    pub fn save_to_toml_preset(&mut self, preset: u16) -> io::Result<()> {
        let path = format!("snake_preset_{preset}.toml");
        self.save_to_toml(path)
    }
    /// Save the current parameters to a TOML file
    /// NB: non-serializable parameters are marked skip with serde annotation and will not be included
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or written to.
    ///
    /// # Panics
    ///
    /// Panics if the game parameters cannot be serialized to TOML.
    fn save_to_toml<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let toml_string =
            toml::to_string_pretty(self).expect("Failed to serialize GameParameters to TOML");
        let full_output = format!("{PARAMS_HEADER}\n{toml_string}");
        let mut file = File::create(path)?;
        file.write_all(full_output.as_bytes())?;
        Ok(())
    }
}

fn clamp_to(value: &mut u16, range: Option<RangeInclusive<u16>>) {
    if let Some(range) = range {
        let (min, max) = range.into_inner();
        *value = (*value).clamp(min, max);
    }
}
fn pretty(r: RangeInclusive<u16>) -> String {
    format!("[{}-{}]", r.start(), r.end()).to_string()
}
// Serde trick
fn default_false() -> bool {
    false
}

impl ActionParameter for GameOptions {
    fn apply_and_save(&mut self, rows: &[RowData], current_preset: Option<u16>) {
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
                            // UX better than --feature false / --feature true, better than default (no flag = false). If you want the possibility to set both values
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
        // Allows keeping the struct for cli parameter as a model object, feeding it with different streams of data.
        // The backup solution is to serialize all the current values in TOML and load them in game_options as already done for the file saving
        // (safe as constraints in-game value)
        self.update_from(new_args);
        self.load = current_preset;
        //If we are on a custom preset, save it (before resetting values)
        if let Some(preset) = current_preset {
            let _ = self.save_to_toml_preset(preset);
        }
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn test_validates() {
        let mut options = GameOptions::default();
        // Values above max
        options.snake_length = 1000;
        options.life = 100;
        options.nb_of_fruits = 1000;
        options.load = Some(8);
        options.validate();
        assert_eq!(options.snake_length, 999);
        assert_eq!(options.life, 99);
        assert_eq!(options.nb_of_fruits, 999);
        assert_eq!(options.load, Some(7));

        // Values below or at min
        options.snake_length = 1;
        options.life = 0;
        options.nb_of_fruits = 0;
        options.load = Some(0);
        options.validate();
        assert_eq!(options.snake_length, 2);
        assert_eq!(options.life, 1);
        assert_eq!(options.nb_of_fruits, 1);
        assert_eq!(options.load, Some(1));
    }

    #[test]
    fn test_validate_symbols() {
        let mut options = GameOptions::default();
        options.head_symbol = "invalid".to_string();
        options.body_symbol = String::new();
        options.validate();
        assert_eq!(options.head_symbol.graphemes(true).count(), 1);
        assert_eq!(options.body_symbol.graphemes(true).count(), 1);
    }

    #[test]
    fn test_save_load_preset() {
        let mut options = GameOptions::default();
        options.snake_length = 42;
        let preset_idx = 7;
        let filename = format!("snake_preset_{preset_idx}.toml");
        // Save
        options.save_to_toml_preset(preset_idx).unwrap();

        // Load
        let loaded = GameOptions::load_from_toml_preset(preset_idx).unwrap();
        assert_eq!(loaded.snake_length, 42);
        assert_eq!(loaded.load, Some(preset_idx));

        // Cleanup
        let _ = std::fs::remove_file(filename);
    }

    #[test]
    fn test_load_invalid_toml_clamped() {
        let preset_idx = 6;
        let filename = format!("snake_preset_{preset_idx}.toml");
        let content = "SNAKE_LENGTH = 2000\nLIFE = 0\n";
        std::fs::write(&filename, content).unwrap();

        let loaded = GameOptions::load_from_toml_preset(preset_idx).unwrap();
        assert_eq!(loaded.snake_length, 999);
        assert_eq!(loaded.life, 1);

        let _ = std::fs::remove_file(filename);
    }
}

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use time::format_description;
use time::format_description::OwnedFormatItem;
use tracing_appender::non_blocking;
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, Registry, fmt, reload};

/// Default path to the logging configuration file (separate from game options).
pub const DEFAULT_LOG_CONFIG_PATH: &str = "snake_log_config.toml";

#[derive(Debug, Copy, Clone, Deserialize, Serialize, ValueEnum, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    #[default]
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Structure holding the logging configuration.
///
/// Every field is deserializable from a TOML file and has a sensible default
/// so missing keys do not break loading (`#[serde(default)]`).
/// Mirrors the boolean / time format options exposed by `tracing_subscriber::fmt::Layer`.
#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
#[allow(clippy::struct_excessive_bools)]
pub struct LogConfig {
    /// Logging level (off, error, warn, info, debug, trace)
    pub level: LogLevel,
    /// Name of the log file (written in current directory)
    pub file_name: String,
    /// Time format string (uses the `time` crate `format_description` syntax)
    pub time_format: String,
    /// Disable ANSI colors in the log file
    pub with_ansi: bool,
    /// Include the module path (target) in each log line
    pub with_target: bool,
    /// Include thread names in each log line
    pub with_thread_names: bool,
    /// Include thread ids in each log line
    pub with_thread_ids: bool,
    /// Include line numbers in each log line
    pub with_line_number: bool,
    /// Include source file name in each log line
    pub with_file: bool,
    /// Include the log level in each log line
    pub with_level: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Off,
            file_name: "snake.log".to_string(),
            time_format: "[hour]:[minute]:[second].[subsecond digits:6]".to_string(),
            with_ansi: false,
            with_target: false,
            with_thread_names: true,
            with_thread_ids: false,
            with_line_number: true,
            with_file: true,
            with_level: true,
        }
    }
}

impl LogConfig {
    /// Load logging configuration from a TOML file.
    /// Returns the default configuration if the file cannot be opened or parsed.
    /// Only deserialization is supported (no serialization back to the file).
    #[must_use]
    pub fn load_from_toml<P: AsRef<Path>>(path: P) -> Self {
        match fs::read_to_string(path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_else(|e| {
                eprintln!("Failed to parse log configuration file: {e}, using defaults");
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }
}
/*
[ App ]
│
▼ (by update_log_level)
[ RELOAD_HANDLE ] ───(modify function)───► [ EnvFilter (e.g.: "info") ]
│
▼ (apply the filter)
[ Log file ]
Work thanks to modify function: pub fn modify(&self, f: impl FnOnce(&mut L)) -> Result<(), Error>
Invokes a closure with a mutable reference to the current layer or filter, allowing it to be modified in place.
So the RELOAD_HANDLE itself is only written once, that the EnvFilter, which is updated as a &mut param
*/
static RELOAD_HANDLE: OnceLock<reload::Handle<EnvFilter, Registry>> = OnceLock::new();

/// Initializes the global logger.
///
/// Reads the logging configuration (level, file name, formatting booleans and
/// time format) directly from a TOML file located at `config_path`. If the
/// file does not exist or cannot be parsed, sensible defaults are used.
///
/// Writing a Lib? Only use the tracing crate and its macros (info!, span!). Do not initialize anything.
/// Writing a Binary? Use tracing-subscriber to build the registry and handlers.
/// Missing dependency logs? Check your `EnvFilter` string rules and ensure the "log" feature is enabled on tracing-subscriber to catch legacy logs
/// Returns a `WorkerGuard` that must be kept alive to ensure logs are flushed to the file.
/// If later wanna do rsyslog (libc): <https://docs.rs/syslog-tracing/0.3.1/syslog_tracing/struct.Syslog.html>
/// Or the newcomer full rust: <https://crates.io/crates/tracing-rfc-5424>
pub fn init_logger<P: AsRef<Path>>(
    config_path: Option<P>,
) -> (non_blocking::WorkerGuard, LogConfig) {
    // Read the entire configuration from the TOML file (deserialization only).
    let config = if let Some(conf) = config_path {
        LogConfig::load_from_toml(conf)
    } else {
        LogConfig::default()
    };

    let file_appender = tracing_appender::rolling::never(".", &config.file_name);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    //To only keep my logs, as sled logs are useless (and otherwise collected),
    // Each dependency log level can be configured with the EnvFilter, e.g.: "sled=info,rsnaker=debug"
    let filter = EnvFilter::new(format!(
        "rsnaker={},sled=off",
        log_level_to_str(config.level)
    ));
    let (filter_layer, reload_handle) = reload::Layer::new(filter);

    // Try to parse the user-supplied time format string at runtime; not as easy as static: LocalTime::new(format_description!("[hour]:[minute]:[second].[subsecond digits:6]" ));
    //LocalTime instead of UTC, to have the local time
    //The generic 2 is the version of this function from time crate (a bit strange but why not)
    let parsed_format: OwnedFormatItem = format_description::parse_owned::<2>(&config.time_format)
        .unwrap_or_else(|_| {
            //std error because logger is not yet initialized at that time
            eprintln!(
                "Failed to parse time format string: {}, using default",
                config.time_format
            );
            format_description::parse_owned::<2>(&LogConfig::default().time_format)
                .expect("Default time format is valid")
        });
    let layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(config.with_ansi)
        .with_target(config.with_target)
        .with_thread_names(config.with_thread_names)
        .with_thread_ids(config.with_thread_ids)
        .with_line_number(config.with_line_number)
        .with_file(config.with_file)
        .with_level(config.with_level);

    let subscriber = tracing_subscriber::registry().with(filter_layer);
    if subscriber
        .with(layer.with_timer(LocalTime::new(parsed_format)))
        .try_init()
        .is_err()
    {
        eprintln!("Failed to initialize logger: logger already initialized");
    } else {
        let _ = RELOAD_HANDLE.set(reload_handle);
    }
    (guard, config)
}

/// Updates the global log level dynamically.
pub fn update_log_level(level: LogLevel) {
    if let Some(handle) = RELOAD_HANDLE.get() {
        let _ = handle.modify(|filter| {
            if let Ok(new_filter) =
                EnvFilter::try_new(format!("rsnaker={},sled=off", log_level_to_str(level)))
            {
                *filter = new_filter;
            }
        });
    }
}

fn log_level_to_str(level: LogLevel) -> &'static str {
    //alt: use strum dependency, macro to get at compile time the code,
    //but it is not really necessary here, as this is the only use case in the whole code
    //// level.as_ref() could be auto-generated by strum with #[strum(serialize_all = "lowercase")], return &'static str
    match level {
        //See pub struct LevelFilter(Option<Level>); of tracing, defining the OFF level to disable logs
        LogLevel::Off => "off",
        LogLevel::Error => "error",
        LogLevel::Warn => "warn",
        LogLevel::Info => "info",
        LogLevel::Debug => "debug",
        LogLevel::Trace => "trace",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_log_level() {
        assert_eq!(LogLevel::default(), LogLevel::Off);
    }

    #[test]
    fn test_log_config_defaults() {
        let cfg = LogConfig::default();
        assert_eq!(cfg.level, LogLevel::Off);
        assert_eq!(cfg.file_name, "snake.log");
        assert!(cfg.with_line_number);
        assert!(cfg.with_file);
    }

    #[test]
    fn test_log_config_deserialization() {
        let path = "test_log_config_deser.toml";
        let content = r#"
level = "debug"
file_name = "test.log"
time_format = "[hour]:[minute]:[second]"
with_ansi = true
with_target = true
with_thread_names = false
with_thread_ids = true
with_line_number = false
with_file = false
with_level = false
"#;
        fs::write(path, content).unwrap();
        let cfg = LogConfig::load_from_toml(path);
        assert_eq!(cfg.level, LogLevel::Debug);
        assert_eq!(cfg.file_name, "test.log");
        assert!(cfg.with_ansi);
        assert!(!cfg.with_line_number);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_log_config_missing_file_uses_defaults() {
        let cfg = LogConfig::load_from_toml("non_existent_log_config_file.toml");
        assert_eq!(cfg.level, LogLevel::Off);
        assert_eq!(cfg.file_name, "snake.log");
    }
}

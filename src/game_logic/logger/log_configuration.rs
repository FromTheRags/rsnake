use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::backtrace::{Backtrace, BacktraceStatus};
use std::fs;
use std::panic::{self, PanicHookInfo};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use time::format_description;
use time::format_description::OwnedFormatItem;
use tracing::error;
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

/// The `tracing_appender` worker guard, shared between the [`LogGuard`] returned to the caller
/// and the panic hook, as both may have to flush the log file.
type SharedWorkerGuard = Arc<Mutex<Option<non_blocking::WorkerGuard>>>;

/// Keeps the background log writer alive: pending logs are flushed to the file when it is dropped.
///
/// The panic hook shares the same underlying worker guard, so the flush also happens
/// when the process aborts on panic (`panic = "abort"`), where nothing is dropped.
#[must_use = "logs are flushed and the log writer stops as soon as the guard is dropped"]
pub struct LogGuard(SharedWorkerGuard);

impl Drop for LogGuard {
    fn drop(&mut self) {
        flush_and_stop(&self.0);
    }
}

/// Flushes the pending logs to the file and stops the background writer (later logs are lost).
/// `tracing_appender` has no flush on demand: it only flushes when its worker guard is dropped.
/// Does nothing if already done.
fn flush_and_stop(worker_guard: &SharedWorkerGuard) {
    // The lock is released at the end of the statement: the (up to 1s) flush is done without holding it.
    let guard = worker_guard
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take();
    drop(guard);
}

/// Chains a panic hook writing the panic into the log file before running the previous hook
/// (the default one printing on stderr, or any hook installed before).
///
/// With `panic = "abort"` the process dies right after the hooks without unwinding, so the
/// `LogGuard` is never dropped: the hook flushes the log file itself, to keep the last logs
/// and the panic record. With `panic = "unwind"` the writer is kept running, as a panic in
/// a secondary thread does not stop the program (the flush is done by the `LogGuard` drop).
fn install_panic_hook(worker_guard: SharedWorkerGuard) {
    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        log_panic(panic_info);
        if cfg!(panic = "abort") {
            flush_and_stop(&worker_guard);
        }
        previous_hook(panic_info);
    }));
}

/// Logs the panic message and its location (the file/line of the log event itself would only
/// point here), plus the backtrace if enabled with `RUST_BACKTRACE`.
fn log_panic(panic_info: &PanicHookInfo<'_>) {
    let message = panic_info
        .payload_as_str()
        .unwrap_or("<non-string panic payload>");
    let location = panic_info
        .location()
        .map_or_else(|| "<unknown>".to_string(), ToString::to_string);
    let backtrace = Backtrace::capture();
    if backtrace.status() == BacktraceStatus::Captured {
        error!(%location, %backtrace, "Panic: {message}");
    } else {
        error!(%location, "Panic: {message}");
    }
}

/// Initializes the global logger.
///
/// Reads the logging configuration (level, file name, formatting booleans and
/// time format) directly from a TOML file located at `config_path`. If the
/// file does not exist or cannot be parsed, sensible defaults are used.
///
/// Writing a Lib? Only use the tracing crate and its macros (info!, span!). Do not initialize anything.
/// Writing a Binary? Use tracing-subscriber to build the registry and handlers.
/// Missing dependency logs? Check your `EnvFilter` string rules and ensure the "log" feature is enabled on tracing-subscriber to catch legacy logs
/// Returns a [`LogGuard`] that must be kept alive to ensure logs are flushed to the file.
/// Also installs a panic hook logging any panic, and flushing the log file when the release
/// profile uses `panic = "abort"` (see `install_panic_hook`).
/// If later wanna do rsyslog (libc): <https://docs.rs/syslog-tracing/0.3.1/syslog_tracing/struct.Syslog.html>
/// Or the newcomer full rust: <https://crates.io/crates/tracing-rfc-5424>
/// # Panics
/// Panics if the default fallback time format fails to parse.
pub fn init_logger<P: AsRef<Path>>(config_path: P) -> (LogGuard, LogConfig) {
    // Read the entire configuration from the TOML file (deserialization only).
    let config = LogConfig::load_from_toml(config_path);

    let file_appender = tracing_appender::rolling::never(".", &config.file_name);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let worker_guard: SharedWorkerGuard = Arc::new(Mutex::new(Some(guard)));
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
        // Only once, with the logger really in use: no hook stacking on a failed re-initialization
        install_panic_hook(Arc::clone(&worker_guard));
    }
    (LogGuard(worker_guard), config)
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

//! # Dynamic Logging Configuration System
//!
//! This module provides an asynchronous, non-blocking logging infrastructure
//! that can have its logging level dynamically updated at runtime. It is built
//! on top of the `tracing` ecosystem.
//!
//! ## Core Features
//! - **Non-blocking I/O:** Log events are pushed onto an in-memory ring buffer and written
//!   to disk by a dedicated background worker thread, ensuring the main application threads
//!   never stall due to disk writes.
//! - **Hot Reloading:** The active log level (`INFO`, `DEBUG`, etc.) can be modified on the fly
//!   without restarting the application, using a global `reload::Handle`.
//! - **File Rotation:** Uses `tracing_appender` to centralize all application logs into a local file.
//!
//! ## Example Usage
//! ```rust
//! use rsnaker::game_logic::logger::log_configuration::{init_logger, update_log_level, LogLevel};
//! use tracing::{info, span, Level};
//!     // 1. Initialize the global logger (keep the guard alive!)
//!     let _guard = init_logger(LogLevel::Info, "test_log.log");
//!
//!     info!("This log is visible at Info level.");
//!
//!     // 2. Change the log level dynamically later in execution
//!     update_log_level(LogLevel::Debug);
//!
//!     // 3. Using a Span to capture a specific context/timed window
//!     // Creates a new span named "game_session" with a dynamic property
//!     let session_span = span!(Level::INFO, "game_session", session_id = 12345);
//!
//!     // Enters the span. The context is active until `_enter` goes out of scope.
//!     let _enter = session_span.enter();
//!
//!     // This log occurs *inside* the span.
//!     // Because our config has `.with_file(true)` and `.with_line_number(true)`,
//!     // the output file will automatically include the filename, line number,
//!     // AND the context: `game_session{session_id=12345}`
//!     info!("Player spawned successfully into the game loop.");
//!
//!     // The span automatically closes here when `_enter` is dropped.
//! ```
//!
pub mod log_configuration;

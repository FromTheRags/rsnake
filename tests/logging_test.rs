use rsnaker::game_logic::logger::log_configuration::{init_logger, update_log_level, LogLevel};
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

#[test]
fn test_logger_writes_to_file_and_updates_level() {
    let log_file_name = "test_output.log";
    let log_path = Path::new(log_file_name);
    let config_path = "test_log_config_runtime.toml";

    // Ensure the file doesn't exist before we start
    if log_path.exists() {
        fs::remove_file(log_path).unwrap();
    }

    // Write a dedicated logging configuration TOML file (deserialized inside init_logger).
    let toml_content = format!(
        r#"level = "info"
file_name = "{log_file_name}"
time_format = "[hour]:[minute]:[second]"
with_ansi = false
with_target = false
with_thread_names = true
with_thread_ids = false
with_line_number = true
with_file = true
with_level = true
"#
    );
    fs::write(config_path, toml_content).unwrap();

    {
        // 1. Initialize logger from the TOML file (info level)
        let _guard = init_logger(Some(config_path));

        // 2. Log an info message (should be written)
        tracing::info!(target: "rsnaker", "This is an info message");

        // 3. Log a debug message (should NOT be written yet)
        tracing::debug!(target: "rsnaker", "This is a hidden debug message");

        // 4. Update log level to Debug
        update_log_level(LogLevel::Debug);

        // 5. Log another debug message (should be written now)
        tracing::debug!(target: "rsnaker", "This is a visible debug message");

        // 6. Update log level to Off
        update_log_level(LogLevel::Off);
        tracing::error!(target: "rsnaker", "This error should be hidden");

        // Give some time for the non-blocking appender to process
        thread::sleep(Duration::from_millis(200));
    }
    // _guard dropped here, should flush everything

    // Read the file and verify content
    let content = fs::read_to_string(log_path).expect("Failed to read log file");

    assert!(
        content.contains("This is an info message"),
        "Info message missing"
    );
    assert!(
        !content.contains("This is a hidden debug message"),
        "Hidden debug message found"
    );
    assert!(
        content.contains("This is a visible debug message"),
        "Visible debug message missing"
    );
    assert!(
        !content.contains("This error should be hidden"),
        "Hidden error message found"
    );

    // Clean up
    fs::remove_file(log_path).unwrap();
    let _ = fs::remove_file(config_path);
}

//! Error-chain formatting with optional source locations.

use std::fmt::Write;

/// Need a custom formatter for now as having localization automatically in a snafu report is yet unstable: <https://docs.rs/snafu/0.9.2/snafu/struct.Report.html#interaction-with-the-provider-api>
/// Alternative: use backtrace (heavy, and others calls), or `eyre::report`
/// Goal: add the line number where the error occurred alongside the error source, not only the line error where the error happen
/// without behind a heavy full Backtrace (which did not work well after stripping the binary by the way)
/// Result (cf. tests below):
/// ```text
/// Error: Operation save failed (at src\game_logic\logger\error_formatting.rs:173:14)
/// Caused by: Save failed (at src\game_logic\logger\error_formatting.rs:171:25)
/// Caused by: Read failed (at src\game_logic\logger\error_formatting.rs:170:54)
/// Caused by: disk error
/// ```
/// Macro to implement this trait are possible, but not very readable because of type inference needed.
/// Better to implement it manually, easier to read and maintain.
/// Custom wrappers must expose their custom source through `located_source`;
/// `Error::source` alone erases this trait and cannot expose location metadata.
pub trait LocatedError: std::error::Error {
    /// Some if we are a custom error with location. None if external error without a location.
    fn location(&self) -> Option<snafu::Location> {
        None
    }

    /// Some if current error has a source, which is a custom error with a location. None if the source is
    /// an external error without a source location.
    fn located_source(&self) -> Option<&dyn LocatedError> {
        None
    }
}

/// Formats each message and its optional location on one line.
#[must_use]
pub fn format_error_chain(mut error: &dyn LocatedError) -> String {
    let mut report = String::from("Error: ");
    // Follow the common trait; only custom errors supply a location.
    loop {
        report.push_str(&error.to_string().replace(['\r', '\n'], " "));
        if let Some(location) = error.location() {
            write!(report, " (at {location})").expect("writing to a String cannot fail");
        }
        // let else pattern (to change from match, good for early return)
        let Some(source) = error.located_source() else {
            break;
        };
        report.push_str("\nCaused by: ");
        error = source;
    }

    // An external error may itself wrap further causes (for example sled -> I/O).
    // Continue through standard source() links; these do not expose locations.
    let mut source = error.source();
    while let Some(cause) = source {
        report.push_str("\nCaused by: ");
        report.push_str(&cause.to_string().replace(['\r', '\n'], " "));
        source = cause.source();
    }
    report
}

#[cfg(test)]
mod tests {
    use super::{LocatedError, format_error_chain};
    use snafu::{Location, ResultExt, Snafu};

    #[derive(Debug, Snafu)]
    pub enum CustomError {
        #[snafu(display("Save failed"))]
        SaveError {
            source: ReadError,
            #[snafu(implicit)]
            location: Location,
        },
    }

    impl LocatedError for CustomError {
        fn location(&self) -> Option<Location> {
            match self {
                Self::SaveError { location, .. } => Some(*location),
            }
        }

        fn located_source(&self) -> Option<&dyn LocatedError> {
            match self {
                Self::SaveError { source, .. } => Some(source),
            }
        }
    }

    #[derive(Debug, Snafu)]
    pub enum ReadError {
        #[snafu(display("Read failed"))]
        Read {
            source: std::io::Error,
            #[snafu(implicit)]
            location: Location,
        },
    }

    impl LocatedError for ReadError {
        fn location(&self) -> Option<Location> {
            match self {
                Self::Read { location, .. } => Some(*location),
            }
        }
    }

    #[test]
    fn enum_reports_its_location_and_the_custom_source_location() {
        let result: Result<(), ReadError> =
            Err(std::io::Error::other("disk error")).context(ReadSnafu);
        let error = result.context(SaveSnafu).unwrap_err();
        let CustomError::SaveError { source, location } = &error;
        assert_eq!(error.to_string(), "Save failed");
        assert_eq!(source.to_string(), "Read failed");
        assert_eq!(
            format_error_chain(&error),
            format!(
                "Error: Save failed (at {location})\nCaused by: Read failed (at {})\nCaused by: disk error",
                source.location().unwrap(),
            )
        );
    }

    #[derive(Debug, Snafu)]
    enum OperationError {
        #[snafu(display("Operation {operation} failed"))]
        Operation {
            operation: &'static str,
            #[snafu(source(from(CustomError, Box::new)))]
            source: Box<CustomError>,
            #[snafu(implicit)]
            location: Location,
        },
        #[snafu(display("External failure"))]
        External {
            source: std::io::Error,
            #[snafu(implicit)]
            location: Location,
        },
        #[snafu(display("Missing value"))]
        Missing {
            #[snafu(implicit)]
            location: Location,
        },
    }

    impl LocatedError for OperationError {
        fn location(&self) -> Option<Location> {
            Some(match self {
                Self::Operation { location, .. }
                | Self::External { location, .. }
                | Self::Missing { location } => *location,
            })
        }

        fn located_source(&self) -> Option<&dyn LocatedError> {
            match self {
                Self::Operation { source, .. } => Some(source.as_ref()),
                Self::External { .. } | Self::Missing { .. } => None,
            }
        }
    }

    #[test]
    fn nested_enums_and_boxed_sources_keep_all_locations() {
        let read: Result<(), ReadError> =
            Err(std::io::Error::other("disk error")).context(ReadSnafu);
        let save = read.context(SaveSnafu);
        let error = save
            .context(OperationSnafu { operation: "save" })
            .unwrap_err();
        let OperationError::Operation {
            source, location, ..
        } = &error
        else {
            panic!("expected operation error");
        };
        let CustomError::SaveError {
            source: read,
            location: save_location,
        } = source.as_ref();
        println!("{}", format_error_chain(&error));
        assert_eq!(
            format_error_chain(&error),
            format!(
                "Error: Operation save failed (at {location})\nCaused by: Save failed (at {save_location})\nCaused by: Read failed (at {})\nCaused by: disk error",
                read.location().unwrap(),
            )
        );
    }

    #[test]
    fn enum_handles_external_sources_and_variants_without_sources() {
        let result: Result<(), OperationError> =
            Err(std::io::Error::other("disk error")).context(ExternalSnafu);
        let external = result.unwrap_err();
        assert!(external.located_source().is_none());
        assert_eq!(
            format_error_chain(&external),
            format!(
                "Error: External failure (at {})\nCaused by: disk error",
                external.location().unwrap(),
            )
        );
        let missing = MissingSnafu.build();
        assert!(missing.located_source().is_none());
        assert_eq!(
            format_error_chain(&missing),
            format!("Error: Missing value (at {})", missing.location().unwrap(), )
        );
    }

    #[test]
    fn external_multiline_cause_stays_on_one_line_without_a_location() {
        let result: Result<(), ReadError> =
            Err(std::io::Error::other("disk error\r\nmore details")).context(ReadSnafu);
        let error = result.unwrap_err();
        assert_eq!(
            format_error_chain(&error),
            format!(
                "Error: Read failed (at {})\nCaused by: disk error  more details",
                error.location().unwrap(),
            )
        );
    }
}

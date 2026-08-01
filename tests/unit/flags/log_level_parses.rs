//! Invariant: log levels parse only the documented spellings.

use std::str::FromStr;

use santh_cli::LogLevel;

#[test]
fn log_level_parses() {
    assert_eq!(LogLevel::from_str("trace"), Ok(LogLevel::Trace));
    assert_eq!(LogLevel::from_str("debug"), Ok(LogLevel::Debug));
    assert_eq!(LogLevel::from_str("info"), Ok(LogLevel::Info));
    assert_eq!(LogLevel::from_str("warn"), Ok(LogLevel::Warn));
    assert_eq!(LogLevel::from_str("error"), Ok(LogLevel::Error));
    assert!(LogLevel::from_str("verbose").is_err());
}

//! Invariant: an explicit missing config path is an actionable error.

use serde::Deserialize;

use santh_cli::{resolve_config, SanthError};

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
struct Config {
    value: String,
}

#[test]
fn invalid_config_path_errors() {
    let result = resolve_config::<Config>(Some(std::path::Path::new("missing/config.toml")), &[]);
    assert!(
        matches!(result, Err(SanthError::ConfigRead { .. })),
        "Fix: explicit config overrides must fail when unreadable so typoed paths do not use defaults."
    );
}

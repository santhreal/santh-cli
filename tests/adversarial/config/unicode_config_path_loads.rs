//! Invariant: valid Unicode config paths are loaded exactly like ASCII paths.

use serde::Deserialize;

use santh_cli::resolve_config;

#[derive(Debug, Default, Deserialize, PartialEq, Eq)]
struct Config {
    value: String,
}

#[test]
fn unicode_config_path_loads() {
    let tempdir = tempfile::tempdir().expect("Fix: test tempdir must be creatable.");
    let path = tempdir.path().join("設定.toml");
    std::fs::write(&path, "value = \"loaded\"\n").expect("Fix: test config must be writable.");

    let loaded = resolve_config::<Config>(Some(&path), &[])
        .expect("Fix: valid Unicode config path must load.");
    assert_eq!(
        loaded,
        Config {
            value: "loaded".to_string()
        }
    );
}

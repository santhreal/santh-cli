//! Invariant: repeated singleton flags are rejected instead of silently overriding.

use clap::Parser;

use santh_cli::GlobalFlags;

#[derive(Debug, Parser)]
struct Args {
    #[command(flatten)]
    globals: GlobalFlags,
}

#[test]
fn duplicate_config_rejected() {
    let error = Args::try_parse_from(["tool", "--config", "a.toml", "--config", "b.toml"]).err();
    assert!(
        matches!(
            error.map(|err| err.kind()),
            Some(clap::error::ErrorKind::ArgumentConflict)
        ),
        "Fix: singleton global flags must reject duplicates so config precedence is explicit."
    );
}

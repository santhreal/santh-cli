//! Invariant: unsupported log levels never fall back to a default.

use clap::Parser;

use santh_cli::GlobalFlags;

#[derive(Debug, Parser)]
struct Args {
    #[command(flatten)]
    globals: GlobalFlags,
}

#[test]
fn bogus_log_level_rejected() {
    let error = Args::try_parse_from(["tool", "--log-level", "chatty"]).err();
    assert!(
        matches!(
            error.map(|err| err.kind()),
            Some(clap::error::ErrorKind::InvalidValue)
        ),
        "Fix: invalid log levels must fail closed with an actionable clap diagnostic."
    );
}

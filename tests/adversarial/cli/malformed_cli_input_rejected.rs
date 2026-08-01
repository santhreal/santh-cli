//! Invariant: unknown flags are rejected as user errors by clap.

use clap::Parser;

use santh_cli::GlobalFlags;

#[derive(Debug, Parser)]
struct Args {
    #[command(flatten)]
    globals: GlobalFlags,
}

#[test]
fn malformed_cli_input_rejected() {
    let error = Args::try_parse_from(["tool", "--definitely-not-real"]).err();
    assert!(
        matches!(
            error.map(|err| err.kind()),
            Some(clap::error::ErrorKind::UnknownArgument)
        ),
        "Fix: unknown global flags must be rejected instead of ignored."
    );
}

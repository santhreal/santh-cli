//! Invariant: metrics port zero is rejected because it is not a stable endpoint.

use clap::Parser;

use santh_cli::GlobalFlags;

#[derive(Debug, Parser)]
struct Args {
    #[command(flatten)]
    globals: GlobalFlags,
}

#[test]
fn unreasonable_metrics_port_rejected() {
    let error = Args::try_parse_from(["tool", "--metrics-port", "0"]).err();
    assert!(
        matches!(
            error.map(|err| err.kind()),
            Some(clap::error::ErrorKind::ValueValidation)
        ),
        "Fix: metrics ports must be constrained to 1..=65535."
    );
}

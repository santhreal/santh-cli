//! Invariant: tools expose a config validation mode that does not scan.

use clap::Parser;

use santh_cli::GlobalFlags;

#[derive(Debug, Parser)]
struct Args {
    #[command(flatten)]
    globals: GlobalFlags,
}

#[test]
fn config_check_mode_exists() {
    let parsed = Args::try_parse_from(["tool", "--config-check"])
        .expect("Fix: --config-check must be a shared global flag.");
    assert!(
        parsed.globals.config_check,
        "Fix: --config-check must set the shared config validation flag."
    );
}

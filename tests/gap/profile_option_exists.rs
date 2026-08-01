//! Invariant: tools expose reusable execution profiles.

use clap::Parser;

use santh_cli::GlobalFlags;

#[derive(Debug, Parser)]
struct Args {
    #[command(flatten)]
    globals: GlobalFlags,
}

#[test]
fn profile_option_exists() {
    let parsed = Args::try_parse_from(["tool", "--profile", "ci.strict"])
        .expect("Fix: --profile must be a shared global flag.");
    assert_eq!(parsed.globals.profile.as_deref(), Some("ci.strict"));

    let rejected = Args::try_parse_from(["tool", "--profile", "../escape"]);
    assert!(
        rejected.is_err(),
        "Fix: profile names must be identifiers, not paths."
    );
}

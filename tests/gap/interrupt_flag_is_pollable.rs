//! Gap: a long-running `SanthCli::run` (e.g. a scanner iterating many targets)
//! needs to poll whether the user pressed Ctrl+C so it can stop early. Before,
//! the interrupt flag was a local `Arc<AtomicBool>` inside `run_inner` with no
//! public accessor, so implementations had no way to observe it.
//!
//! `santh_cli::is_interrupted()` now exposes the process-global interrupt flag.
//! In a normal test process no SIGINT is delivered, so it must report
//! not-interrupted; this locks the public symbol and its default.

#[test]
fn is_interrupted_is_publicly_pollable_and_defaults_false() {
    assert!(
        !santh_cli::is_interrupted(),
        "with no SIGINT delivered, is_interrupted() must report false"
    );
}

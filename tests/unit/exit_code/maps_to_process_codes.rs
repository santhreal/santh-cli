//! Invariant: each public exit code maps to the documented process code.

use santh_cli::SanthExitCode;

#[test]
fn maps_to_process_codes() {
    assert_eq!(
        std::process::ExitCode::from(SanthExitCode::Success),
        std::process::ExitCode::from(0)
    );
    assert_eq!(
        std::process::ExitCode::from(SanthExitCode::FindingsEmitted),
        std::process::ExitCode::from(1)
    );
    assert_eq!(
        std::process::ExitCode::from(SanthExitCode::UserError),
        std::process::ExitCode::from(2)
    );
    assert_eq!(
        std::process::ExitCode::from(SanthExitCode::SystemError),
        std::process::ExitCode::from(3)
    );
    assert_eq!(
        std::process::ExitCode::from(SanthExitCode::Interrupted),
        std::process::ExitCode::from(130)
    );
}

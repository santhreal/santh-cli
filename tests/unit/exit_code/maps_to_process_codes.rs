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
    assert_eq!(SanthExitCode::Success.as_u8(), 0);
    assert_eq!(SanthExitCode::FindingsEmitted.as_u8(), 1);
    assert_eq!(SanthExitCode::UserError.as_u8(), 2);
    assert_eq!(SanthExitCode::SystemError.as_u8(), 3);
    assert_eq!(SanthExitCode::Interrupted.as_u8(), 130);
}

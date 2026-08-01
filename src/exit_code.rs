use std::process::ExitCode;

/// Fixed process exit-code vocabulary for scanner tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
#[non_exhaustive]
pub enum SanthExitCode {
    /// Successful run with no findings.
    Success = 0,
    /// Successful run with one or more findings emitted.
    FindingsEmitted = 1,
    /// User-correctable failure such as invalid flags, config, or target.
    UserError = 2,
    /// System failure such as I/O, network, or permission errors.
    SystemError = 3,
    /// Interrupted by SIGINT.
    Interrupted = 130,
}

impl From<SanthExitCode> for ExitCode {
    fn from(value: SanthExitCode) -> Self {
        Self::from(value as u8)
    }
}

use std::ffi::OsString;
use std::io::Write;
use std::marker::PhantomData;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};

use clap::{CommandFactory, FromArgMatches, Parser};

use crate::{GlobalFlags, LogLevel, SanthExitCode};

/// Process-global interrupt flag, set by the Ctrl+C handler installed in
/// [`SanthCliBuilder::run`]. It is global (not per-run) because the underlying
/// signal handler is itself process-wide, so a single flag is the honest model.
/// Long-running [`SanthCli::run`] implementations poll it via [`is_interrupted`]
/// to abort cooperatively.
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Returns `true` once a Ctrl+C (SIGINT) has been received.
///
/// A long-running [`SanthCli::run`] (e.g. a scanner iterating a large target
/// set) should poll this and stop early when it returns `true`, so the tool
/// exits promptly with [`SanthExitCode::Interrupted`] instead of running to
/// completion after the user asked to cancel.
#[must_use]
pub fn is_interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}

/// Reset the process-global interrupt flag to `false`.
///
/// Multi-run execution harnesses or tests that run multiple commands
/// sequentially in the same process use this to clear signal state.
pub fn reset_interrupted() {
    INTERRUPTED.store(false, Ordering::SeqCst);
}

/// Contract implemented by command-line tools that use the shared CLI.
pub trait SanthCli {
    /// Tool subcommands.
    type Subcommand: clap::Subcommand;

    /// Binary name shown in help output.
    fn tool_name() -> &'static str;

    /// Tool version shown in help output.
    fn tool_version() -> &'static str;

    /// One-line tool description shown in help output.
    fn tool_description() -> &'static str;

    /// Execute the parsed command.
    fn run(globals: GlobalFlags, subcommand: Self::Subcommand) -> ExitCode;
}

/// Top-level entry point for scanner binaries.
pub fn santh_main<T>() -> ExitCode
where
    T: SanthCli,
{
    SanthCliBuilder::<T>::new().run()
}

/// Builder for executing a [`SanthCli`] implementation.
///
/// The default builder reads process arguments, installs Ctrl+C handling, and
/// initializes tracing. Tests and wrappers can provide explicit arguments or
/// disable process-global setup.
#[derive(Debug)]
pub struct SanthCliBuilder<T>
where
    T: SanthCli,
{
    args: Option<Vec<OsString>>,
    install_ctrlc: bool,
    init_logging: bool,
    _marker: PhantomData<T>,
}

impl<T> Default for SanthCliBuilder<T>
where
    T: SanthCli,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SanthCliBuilder<T>
where
    T: SanthCli,
{
    /// Build a runner that uses process arguments and standard setup.
    pub fn new() -> Self {
        Self {
            args: None,
            install_ctrlc: true,
            init_logging: true,
            _marker: PhantomData,
        }
    }

    /// Provide explicit arguments, including argv[0].
    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args = Some(args.into_iter().map(Into::into).collect());
        self
    }

    /// Disable Ctrl+C handler installation.
    #[must_use]
    pub fn without_ctrlc_handler(mut self) -> Self {
        self.install_ctrlc = false;
        self
    }

    /// Disable global tracing initialization.
    #[must_use]
    pub fn without_logging(mut self) -> Self {
        self.init_logging = false;
        self
    }

    /// Parse and execute the configured CLI.
    pub fn run(self) -> ExitCode {
        self.run_inner()
    }

    fn run_inner(self) -> ExitCode {
        INTERRUPTED.store(false, Ordering::SeqCst);
        if self.install_ctrlc {
            if let Err(source) = install_ctrlc_handler() {
                if !matches!(source, ctrlc::Error::MultipleHandlers) {
                    write_stderr_line(&format!(
                        "failed to install Ctrl+C handler: {source}. Fix: ensure no incompatible signal handler is already installed."
                    ));
                    return SanthExitCode::SystemError.into();
                }
            }
        }

        let parsed = match self.args {
            Some(args) => parse_cli_from_args::<T, _, _>(args),
            None => parse_process_cli::<T>(),
        };
        let (globals, log_level_overridden, subcommand) = match parsed {
            Ok(parsed) => parsed,
            Err(code) => return code.into(),
        };

        let effective_level = globals.effective_log_level(log_level_overridden);
        if self.init_logging {
            init_logging(T::tool_name(), effective_level);
        }

        if is_interrupted() {
            return SanthExitCode::Interrupted.into();
        }

        let result = T::run(globals, subcommand);
        if is_interrupted() {
            SanthExitCode::Interrupted.into()
        } else {
            result
        }
    }
}

/// Parse CLI arguments for a tool from an arbitrary iterator.
///
/// This is primarily useful for tests and wrapper binaries.
pub fn parse_santh_cli_from<T, I, S>(args: I) -> Result<(GlobalFlags, T::Subcommand), clap::Error>
where
    T: SanthCli,
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString> + Clone,
{
    let (globals, _, subcommand) = parse_cli_matches_from::<T, _, _>(args)?;
    Ok((globals, subcommand))
}

#[derive(Debug, Parser)]
struct SanthArgs<S>
where
    S: clap::Subcommand,
{
    #[command(flatten)]
    globals: GlobalFlags,
    #[command(subcommand)]
    subcommand: S,
    #[arg(skip)]
    _marker: PhantomData<S>,
}

fn parse_process_cli<T>() -> Result<(GlobalFlags, bool, T::Subcommand), SanthExitCode>
where
    T: SanthCli,
{
    parse_cli_from_args::<T, _, _>(std::env::args_os())
}

fn parse_cli_from_args<T, I, S>(
    args: I,
) -> Result<(GlobalFlags, bool, T::Subcommand), SanthExitCode>
where
    T: SanthCli,
    I: IntoIterator<Item = S>,
    S: Into<OsString> + Clone,
{
    match parse_cli_matches_from::<T, _, _>(args) {
        Ok(parsed) => Ok(parsed),
        Err(error) => {
            if let Err(print_error) = error.print() {
                write_stderr_line(&format!(
                    "failed to print CLI diagnostic: {print_error}. Fix: verify stderr is writable."
                ));
            }
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) {
                Err(SanthExitCode::Success)
            } else {
                Err(SanthExitCode::UserError)
            }
        }
    }
}

fn parse_cli_matches_from<T, I, S>(
    args: I,
) -> Result<(GlobalFlags, bool, T::Subcommand), clap::Error>
where
    T: SanthCli,
    I: IntoIterator<Item = S>,
    S: Into<OsString> + Clone,
{
    let command = SanthArgs::<T::Subcommand>::command()
        .name(T::tool_name())
        .version(T::tool_version())
        .about(T::tool_description());
    let matches = command.try_get_matches_from(args)?;

    let log_level_overridden = matches
        .value_source("log_level")
        .is_some_and(|source| source == clap::parser::ValueSource::CommandLine);
    let parsed = SanthArgs::<T::Subcommand>::from_arg_matches(&matches)?;
    parsed.globals.validate().map_err(|msg| {
        clap::Error::raw(clap::error::ErrorKind::InvalidValue, format!("{msg}\n"))
    })?;

    Ok((parsed.globals, log_level_overridden, parsed.subcommand))
}

fn install_ctrlc_handler() -> Result<(), ctrlc::Error> {
    ctrlc::set_handler(|| {
        INTERRUPTED.store(true, Ordering::SeqCst);
    })
}

fn init_logging(tool_name: &str, level: LogLevel) {
    let _guard = santh_tracing::init(tool_name, to_tracing_level(level));
}

fn to_tracing_level(level: LogLevel) -> santh_tracing::LogLevel {
    match level {
        LogLevel::Trace => santh_tracing::LogLevel::Trace,
        LogLevel::Debug => santh_tracing::LogLevel::Debug,
        LogLevel::Info => santh_tracing::LogLevel::Info,
        LogLevel::Warn => santh_tracing::LogLevel::Warn,
        LogLevel::Error => santh_tracing::LogLevel::Error,
    }
}

fn write_stderr_line(message: &str) {
    let mut stderr = std::io::stderr().lock();
    let _ = stderr.write_all(message.as_bytes());
    let _ = stderr.write_all(b"\n");
}

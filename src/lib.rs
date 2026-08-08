#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::todo,
        clippy::unimplemented,
        clippy::panic
    )
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::missing_errors_doc
)]

//! Shared CLI contract for Santh tools: global flags, exit codes, and parsing.
//!
//! # Safe defaults
//!
//! `santh-cli` is a parsing-and-dispatch scaffold that performs no I/O of its
//! own, so its defaults are conservative by construction:
//!
//! - **Input size**: only process command-line arguments and a single optional
//!   config file are read. Argument parsing is bounded by the OS `ARG_MAX`, and
//!   the config loader reads one small TOML file, so there is no unbounded
//!   input size.
//! - **Recursion depth**: argument and config parsing are flat (no recursive
//!   descent over caller data), so there is no recursion-depth exposure.
//! - **Outbound network**: none. The contract never opens sockets or performs
//!   network requests; all networking belongs to the tool that embeds it.
//! - **Process spawning**: none. `santh-cli` never spawns a child process.
//! - **Filesystem writes**: none by default. It only *reads* an optional config
//!   file; rendering output and writing files is the embedding tool's job.
//! - **Credential exposure**: none. It does not read, log, or persist
//!   credentials; secrets handled by an embedding tool are never emitted here.

mod config;
mod error;
mod exit_code;
mod finding;
mod flags;
mod runner;

pub use config::resolve_config;
pub use error::{SanthError, SanthResult};
pub use exit_code::SanthExitCode;
pub use finding::emit_finding;
pub use flags::{GlobalFlags, GlobalFlagsBuilder, LogLevel, OutputFormat};
pub use runner::{
    is_interrupted, parse_santh_cli_from, reset_interrupted, santh_main, SanthCli, SanthCliBuilder,
};

// Rung 7 (contract): the README quick-start is a doctest, so a README example
// that drifts from the real API fails `cargo test` instead of misleading users.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

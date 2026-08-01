use std::path::PathBuf;

use thiserror::Error;

/// Result type used by this crate.
pub type SanthResult<T> = Result<T, SanthError>;

/// Errors produced by shared CLI support.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SanthError {
    /// A configuration path could not be read.
    #[error("failed to read config file `{path}`: {source}. Fix: verify the path exists and is readable.")]
    ConfigRead {
        /// The config path that failed.
        path: PathBuf,
        /// The underlying filesystem error.
        #[source]
        source: std::io::Error,
    },

    /// A configuration file was not valid TOML for the requested type.
    #[error("failed to parse config file `{path}` as TOML: {source}. Fix: correct the TOML syntax and fields for this tool.")]
    ConfigParse {
        /// The config path that failed.
        path: PathBuf,
        /// The TOML parser error.
        #[source]
        source: toml::de::Error,
    },

    /// Finding output could not be serialized.
    #[error("failed to serialize finding output: {0}. Fix: ensure the finding contains serializable values.")]
    Serialize(#[from] serde_json::Error),

    /// Finding output could not be written.
    #[error("failed to write CLI output: {0}. Fix: check stdout, pipe consumers, and filesystem permissions.")]
    Write(#[from] std::io::Error),

    /// The Ctrl+C handler could not be installed.
    #[error("failed to install Ctrl+C handler: {0}. Fix: ensure no incompatible signal handler is already installed.")]
    CtrlC(#[from] ctrlc::Error),
}

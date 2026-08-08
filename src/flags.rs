use std::path::PathBuf;
use std::str::FromStr;

use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

/// Global flags shared by all scanner command-line tools.
#[derive(Debug, Clone, PartialEq, Eq, Args, Serialize, Deserialize)]
pub struct GlobalFlags {
    /// Tier A config file override.
    #[arg(
        long,
        value_name = "PATH",
        value_hint = clap::ValueHint::FilePath,
        allow_hyphen_values = true
    )]
    pub config: Option<PathBuf>,

    /// Finding output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
    pub output: OutputFormat,

    /// Logging verbosity.
    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,

    /// Suppress non-error output.
    #[arg(short, long)]
    pub quiet: bool,

    /// Optional Prometheus metrics endpoint port.
    #[arg(long, value_name = "PORT", value_parser = parse_metrics_port)]
    pub metrics_port: Option<u16>,

    /// Named execution profile loaded from the resolved Tier A config.
    #[arg(long, value_name = "NAME", value_parser = parse_profile_name)]
    pub profile: Option<String>,

    /// Validate config and profile selection without running a scan.
    #[arg(long)]
    pub config_check: bool,
}

impl Default for GlobalFlags {
    fn default() -> Self {
        Self {
            config: None,
            output: OutputFormat::Human,
            log_level: LogLevel::Info,
            quiet: false,
            metrics_port: None,
            profile: None,
            config_check: false,
        }
    }
}

impl GlobalFlags {
    /// Start a typed builder for global flags.
    pub fn builder() -> GlobalFlagsBuilder {
        GlobalFlagsBuilder::default()
    }

    /// Return the log level after applying `--quiet` semantics.
    pub fn effective_log_level(&self, log_level_overridden: bool) -> LogLevel {
        if self.quiet && !log_level_overridden {
            LogLevel::Error
        } else {
            self.log_level
        }
    }

    /// Validate that all flag values meet invariant constraints.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(port) = self.metrics_port {
            if port == 0 {
                return Err(
                    "invalid metrics port `0`. Fix: use an integer from 1 to 65535.".to_string(),
                );
            }
        }
        if let Some(profile) = &self.profile {
            parse_profile_name(profile)?;
        }
        Ok(())
    }
}

/// Builder for [`GlobalFlags`] used by wrapper tools and tests.
#[derive(Debug, Clone, Default)]
pub struct GlobalFlagsBuilder {
    flags: GlobalFlags,
}

impl GlobalFlagsBuilder {
    /// Set the config file override.
    #[must_use]
    pub fn config(mut self, path: impl Into<PathBuf>) -> Self {
        self.flags.config = Some(path.into());
        self
    }

    /// Set the finding output format.
    #[must_use]
    pub fn output(mut self, output: OutputFormat) -> Self {
        self.flags.output = output;
        self
    }

    /// Set the log level.
    #[must_use]
    pub fn log_level(mut self, log_level: LogLevel) -> Self {
        self.flags.log_level = log_level;
        self
    }

    /// Suppress non-error diagnostics unless the log level is explicit.
    #[must_use]
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.flags.quiet = quiet;
        self
    }

    /// Set the optional metrics port.
    pub fn metrics_port(mut self, port: u16) -> Result<Self, String> {
        if port == 0 {
            return Err(
                "invalid metrics port `0`. Fix: use an integer from 1 to 65535.".to_string(),
            );
        }
        self.flags.metrics_port = Some(port);
        Ok(self)
    }

    /// Select a named execution profile.
    pub fn profile(mut self, profile: impl Into<String>) -> Result<Self, String> {
        let profile = profile.into();
        parse_profile_name(&profile)?;
        self.flags.profile = Some(profile);
        Ok(self)
    }

    /// Enable or disable config-only validation mode.
    #[must_use]
    pub fn config_check(mut self, enabled: bool) -> Self {
        self.flags.config_check = enabled;
        self
    }

    /// Finish building global flags.
    #[must_use]
    pub fn build(self) -> GlobalFlags {
        self.flags
    }
}

/// Canonical finding output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum OutputFormat {
    /// Newline-delimited JSON, one finding per line.
    Json,
    /// SARIF 2.1.0 JSON.
    Sarif,
    /// Colorized terminal output for humans.
    Human,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Json => "json",
            Self::Sarif => "sarif",
            Self::Human => "human",
        })
    }
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "json" => Ok(Self::Json),
            "sarif" => Ok(Self::Sarif),
            "human" => Ok(Self::Human),
            _ => Err(format!(
                "invalid output format `{value}`. Fix: use `json`, `sarif`, or `human`."
            )),
        }
    }
}

/// Canonical log level for scanner tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum LogLevel {
    /// Trace-level diagnostics.
    Trace,
    /// Debug-level diagnostics.
    Debug,
    /// Informational diagnostics.
    Info,
    /// Warning diagnostics.
    Warn,
    /// Error-only diagnostics.
    Error,
}

impl LogLevel {
    /// Convert into the matching tracing level filter.
    pub fn as_tracing_filter(self) -> tracing::level_filters::LevelFilter {
        match self {
            Self::Trace => tracing::level_filters::LevelFilter::TRACE,
            Self::Debug => tracing::level_filters::LevelFilter::DEBUG,
            Self::Info => tracing::level_filters::LevelFilter::INFO,
            Self::Warn => tracing::level_filters::LevelFilter::WARN,
            Self::Error => tracing::level_filters::LevelFilter::ERROR,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        })
    }
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "trace" => Ok(Self::Trace),
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            _ => Err(format!(
                "invalid log level `{value}`. Fix: use `trace`, `debug`, `info`, `warn`, or `error`."
            )),
        }
    }
}

fn parse_metrics_port(raw: &str) -> Result<u16, String> {
    let port = raw.parse::<u16>().map_err(|error| {
        format!("invalid metrics port `{raw}`: {error}. Fix: use an integer from 1 to 65535.")
    })?;
    if port == 0 {
        return Err("invalid metrics port `0`. Fix: use an integer from 1 to 65535.".to_string());
    }
    Ok(port)
}

fn parse_profile_name(raw: &str) -> Result<String, String> {
    if raw.is_empty() {
        return Err("invalid profile name. Fix: pass a non-empty profile name.".to_string());
    }
    if raw.len() > 128 {
        return Err(
            "invalid profile name. Fix: keep profile names at 128 bytes or less.".to_string(),
        );
    }
    if raw
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        Ok(raw.to_string())
    } else {
        Err(
            "invalid profile name. Fix: use only ASCII letters, digits, `_`, `-`, and `.`."
                .to_string(),
        )
    }
}

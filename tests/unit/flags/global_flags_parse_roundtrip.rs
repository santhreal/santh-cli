//! Invariant: clap parses global flags into the same values a caller expects.

use clap::Parser;
use santh_cli::{GlobalFlags, LogLevel, OutputFormat};

#[derive(Debug, Parser, PartialEq, Eq)]
struct Args {
    #[command(flatten)]
    globals: GlobalFlags,
}

#[test]
fn global_flags_parse_roundtrip() {
    let parsed = Args::try_parse_from([
        "tool",
        "--config",
        "config/tool.toml",
        "--output",
        "sarif",
        "--log-level",
        "debug",
        "--quiet",
        "--metrics-port",
        "9090",
        "--profile",
        "ci.strict",
        "--config-check",
    ])
    .map(|args| args.globals)
    .expect("Fix: valid global flags must parse.");

    assert_eq!(
        parsed,
        GlobalFlags {
            config: Some("config/tool.toml".into()),
            output: OutputFormat::Sarif,
            log_level: LogLevel::Debug,
            quiet: true,
            metrics_port: Some(9090),
            profile: Some("ci.strict".to_string()),
            config_check: true,
        }
    );

    let built = GlobalFlags::builder()
        .config("config/tool.toml")
        .output(OutputFormat::Sarif)
        .log_level(LogLevel::Debug)
        .quiet(true)
        .metrics_port(9090)
        .expect("Fix: valid metrics port must build.")
        .profile("ci.strict")
        .expect("Fix: valid profile must build.")
        .config_check(true)
        .build();
    assert_eq!(built, parsed);
}
#[test]
fn global_flags_validate_rejects_zero_metrics_port() {
    let mut flags = GlobalFlags::default();
    flags.metrics_port = Some(0);
    assert!(flags.validate().is_err());
}

#[test]
fn global_flags_validate_rejects_invalid_profile_name() {
    let mut flags = GlobalFlags::default();
    flags.profile = Some("invalid profile name!".to_string());
    assert!(flags.validate().is_err());
}

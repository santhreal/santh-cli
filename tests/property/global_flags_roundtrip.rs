//! Invariant: any valid GlobalFlags value has an argv representation clap parses back.

use clap::Parser;
use proptest::prelude::*;
use santh_cli::{GlobalFlags, LogLevel, OutputFormat};

#[derive(Debug, Parser, PartialEq, Eq)]
struct Args {
    #[command(flatten)]
    globals: GlobalFlags,
}

proptest! {
    #[test]
    fn global_flags_roundtrip(
        output in output_format(),
        log_level in log_level(),
        quiet in any::<bool>(),
        config_check in any::<bool>(),
        metrics_port in prop::option::of(1u16..=u16::MAX),
        config in prop::option::of("[A-Za-z0-9_./-]{1,32}"),
        profile in prop::option::of("[A-Za-z0-9_.-]{1,32}"),
    ) {
        let flags = GlobalFlags {
            config: config.clone().map(Into::into),
            output,
            log_level,
            quiet,
            metrics_port,
            profile: profile.clone(),
            config_check,
        };
        // Use `--flag=value` syntax for the free-string flags: a config path or
        // profile name may legitimately start with `-`, which clap parses as a
        // flag under space-separated syntax. The `=` form roundtrips any value.
        let mut args = vec!["tool".to_string()];
        if let Some(path) = config {
            args.push(format!("--config={path}"));
        }
        args.push("--output".to_string());
        args.push(output.to_string());
        args.push("--log-level".to_string());
        args.push(log_level.to_string());
        if quiet {
            args.push("--quiet".to_string());
        }
        if let Some(port) = metrics_port {
            args.push("--metrics-port".to_string());
            args.push(port.to_string());
        }
        if let Some(profile) = profile {
            args.push(format!("--profile={profile}"));
        }
        if config_check {
            args.push("--config-check".to_string());
        }

        let parsed = Args::try_parse_from(args).map(|args| args.globals);
        prop_assert!(
            parsed.is_ok(),
            "Fix: generated valid GlobalFlags argv must parse: {parsed:?}"
        );
        prop_assert_eq!(parsed.ok(), Some(flags));
    }
}

fn output_format() -> impl Strategy<Value = OutputFormat> {
    prop_oneof![
        Just(OutputFormat::Json),
        Just(OutputFormat::Sarif),
        Just(OutputFormat::Human),
    ]
}

fn log_level() -> impl Strategy<Value = LogLevel> {
    prop_oneof![
        Just(LogLevel::Trace),
        Just(LogLevel::Debug),
        Just(LogLevel::Info),
        Just(LogLevel::Warn),
        Just(LogLevel::Error),
    ]
}

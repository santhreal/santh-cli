//! Original bug class: quiet modes often clobber explicit verbosity overrides.

use santh_cli::{GlobalFlags, LogLevel, OutputFormat};

#[test]
fn quiet_keeps_explicit_log_level() {
    let flags = GlobalFlags {
        config: None,
        output: OutputFormat::Human,
        log_level: LogLevel::Debug,
        quiet: true,
        metrics_port: None,
        profile: None,
        config_check: false,
    };

    assert_eq!(flags.effective_log_level(true), LogLevel::Debug);
    assert_eq!(flags.effective_log_level(false), LogLevel::Error);
}

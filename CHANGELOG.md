# Changelog

## [0.1.4] - 2026-08-07
### Fixed
- `emit_human` in `finding.rs` now sanitizes `finding.kind()` in addition to title, target, detail, and location to prevent control character / terminal escape sequence injection in finding types.
- `emit_finding` now inspects `serde_json::Error` on JSON and SARIF output streams and maps stdout I/O failures (such as `BrokenPipe`) to `SanthError::Write` instead of misclassifying them as data serialization errors (`SanthError::Serialize`).
- `SanthCliBuilder::run` now handles `ctrlc::Error::MultipleHandlers` gracefully so CLI runs in multi-command test suites or process hosts do not fail with `SanthExitCode::SystemError`.
- `SanthCliBuilder::run` resets the process-global interrupt flag on startup, and `reset_interrupted()` is now exposed so signal state does not leak across multiple runs in the same process.
- `GlobalFlags` now provides a `validate()` method to enforce flag invariant constraints (`metrics_port != 0`, valid profile names) on constructed or deserialized flag instances.

### Added
- `SanthExitCode` now provides an `as_u8()` accessor method returning the raw integer exit code.

## [0.1.3] - 2026-08-07

### Fixed
- Verified `authors` field in `Cargo.toml` set to `["Santh <64453045+santhreal@users.noreply.github.com>"]`.
- Audited SARIF output & config resolution silent fallbacks: confirmed region block omission for findings without line numbers, startLine without startColumn when line-only, and verified non-NotFound I/O errors surface as `ConfigRead`.

### Changed
- Confirmed honest `package.metadata.santh.status = "beta"` (no fuzz directory present).

## 0.1.1

### Security

- `emit_finding` human output now strips terminal control characters from
  finding fields. Finding fields are attacker-controlled (they describe
  scanned targets), so a malicious target could previously inject ANSI escape
  sequences (clear screen, OSC 52 clipboard write) or embed newlines that
  forge additional findings in the operator's terminal. JSON and SARIF output
  are escaped by `serde_json` and were not affected.

### Changed

- `GlobalFlagsBuilder` derives `Default`; builder methods on
  `GlobalFlagsBuilder` and `SanthCliBuilder` are marked `#[must_use]`.

## 0.1.0

- Initial alpha release with shared global flags, exit codes, config loading,
  finding emission, logging initialization, and top-level CLI runner.

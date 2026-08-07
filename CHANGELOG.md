# Changelog

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

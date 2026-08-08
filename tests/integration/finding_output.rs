//! Invariant: finding output supports every advertised format.

use santh_cli::{emit_finding, OutputFormat};
use secfinding::{Finding, FindingKind, Location, Severity};

#[test]
fn finding_output() {
    let finding = crate::support::sample_finding();
    for format in [OutputFormat::Json, OutputFormat::Sarif, OutputFormat::Human] {
        let mut output = Vec::new();
        let result = emit_finding(&finding, format, &mut output);
        assert!(
            result.is_ok(),
            "Fix: {format} finding output must be serializable and writable."
        );
        assert!(
            !output.is_empty(),
            "Fix: {format} finding output must write visible bytes."
        );
    }
}

/// The SARIF `properties` arrays (tags/cveIds/cweIds/references/matchedValues)
/// are serialized straight from the finding's `&[Arc<str>]` accessors (no
/// intermediate `Vec<String>`). Assert the actual serialized JSON so that
/// optimization is proven byte-equivalent to the old per-element `to_string`
/// collection, not merely non-empty.
#[test]
fn sarif_omits_region_when_finding_has_no_line() {
    // A file-level finding (Location with line/column = None) must NOT fabricate
    // a SARIF region at line 1 col 1 — the region block must be absent so
    // downstream consumers don't misattribute it to line 1.
    let finding = Finding::builder("unit-scanner", "src/main.rs", Severity::High)
        .title("File-level finding")
        .detail("No line information available.")
        .location(Location::new("src/main.rs").expect("valid location"))
        .build()
        .expect("Fix: finding builder inputs must stay valid.");

    let mut output = Vec::new();
    emit_finding(&finding, OutputFormat::Sarif, &mut output).expect("sarif emit must succeed");
    let doc: serde_json::Value =
        serde_json::from_slice(&output).expect("sarif output must be valid JSON");

    let physical = &doc["runs"][0]["results"][0]["locations"][0]["physicalLocation"];
    assert!(
        physical["region"].is_null(),
        "a finding with no line must emit no SARIF region, got {physical}"
    );
    assert_eq!(
        physical["artifactLocation"]["uri"],
        serde_json::json!("src/main.rs"),
        "the artifact URI must still be present"
    );
}

#[test]
fn sarif_emits_real_line_and_column_when_present() {
    // When the finding has a real line/column they must appear verbatim (not the
    // fabricated 1), proving the region is emitted only from real data.
    let loc = Location::new("src/main.rs")
        .expect("valid location")
        .line(42)
        .expect("nonzero line")
        .column(7)
        .expect("nonzero column");
    let finding = Finding::builder("unit-scanner", "src/main.rs", Severity::High)
        .title("Line finding")
        .detail("Has precise location.")
        .location(loc)
        .build()
        .expect("Fix: finding builder inputs must stay valid.");

    let mut output = Vec::new();
    emit_finding(&finding, OutputFormat::Sarif, &mut output).expect("sarif emit must succeed");
    let doc: serde_json::Value =
        serde_json::from_slice(&output).expect("sarif output must be valid JSON");

    let region = &doc["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["region"];
    assert_eq!(region["startLine"], serde_json::json!(42), "real startLine must appear");
    assert_eq!(region["startColumn"], serde_json::json!(7), "real startColumn must appear");
}

#[test]
fn sarif_emits_line_without_column_when_column_is_none() {
    let loc = Location::new("src/main.rs")
        .expect("valid location")
        .line(42)
        .expect("nonzero line");
    let finding = Finding::builder("unit-scanner", "src/main.rs", Severity::High)
        .title("Line-only finding")
        .detail("Has line but no column.")
        .location(loc)
        .build()
        .expect("Fix: finding builder inputs must stay valid.");

    let mut output = Vec::new();
    emit_finding(&finding, OutputFormat::Sarif, &mut output).expect("sarif emit must succeed");
    let doc: serde_json::Value =
        serde_json::from_slice(&output).expect("sarif output must be valid JSON");

    let region = &doc["runs"][0]["results"][0]["locations"][0]["physicalLocation"]["region"];
    assert_eq!(region["startLine"], serde_json::json!(42), "real startLine must appear");
    assert!(
        region["startColumn"].is_null(),
        "startColumn must be absent when column is None"
    );
}

#[test]
fn sarif_properties_arrays_serialize_string_values() {
    let finding = Finding::builder("unit-scanner", "src/main.rs", Severity::High)
        .title("Unsafe input reaches sink")
        .detail("User-controlled input reaches a command execution sink.")
        .kind(FindingKind::Vulnerability)
        .remediation("Validate arguments before command construction.")
        .tag("injection")
        .tag("cwe-top-25")
        .cve("CVE-2021-44228")
        .cwe("CWE-78")
        .reference("https://example.test/advisory/1")
        .matched_value("; rm -rf /")
        .build()
        .expect("Fix: finding builder inputs must stay valid.");

    let mut output = Vec::new();
    emit_finding(&finding, OutputFormat::Sarif, &mut output).expect("sarif emit must succeed");
    let doc: serde_json::Value =
        serde_json::from_slice(&output).expect("sarif output must be valid JSON");

    let props = &doc["runs"][0]["results"][0]["properties"];

    // The builder stores tags sorted, so the serialized array is in sorted
    // order ("cwe-top-25" < "injection"), not insertion order. What matters for
    // this test is that every tag round-trips as a JSON string.
    assert_eq!(
        props["tags"],
        serde_json::json!(["cwe-top-25", "injection"]),
        "tags must serialize as a JSON string array (builder keeps them sorted)"
    );
    assert_eq!(
        props["cveIds"],
        serde_json::json!(["CVE-2021-44228"]),
        "cveIds must serialize as a JSON string array"
    );
    assert_eq!(
        props["cweIds"],
        serde_json::json!(["CWE-78"]),
        "cweIds must serialize as a JSON string array"
    );
    assert_eq!(
        props["references"],
        serde_json::json!(["https://example.test/advisory/1"]),
        "references must serialize as a JSON string array"
    );
    assert_eq!(
        props["matchedValues"],
        serde_json::json!(["; rm -rf /"]),
        "matchedValues must serialize as a JSON string array, preserving exact bytes"
    );
}

#[test]
fn human_output_strips_terminal_control_characters() {
    // Regression lock: emit_human wrote finding fields verbatim to the
    // terminal. Finding fields are attacker-controlled (they describe scanned
    // targets), so a malicious target could inject ANSI escape sequences
    // (clear screen via ESC[2J, clipboard write via OSC 52) or embed newlines
    // that forge additional findings in the operator's terminal. Control
    // characters are now replaced with '?'.
    let finding = Finding::builder("unit-scanner", "src/main.rs", Severity::High)
        .title("forged\x1b[2J\x1b[H\x1b]52;c;aGVsbG8=\x07")
        .detail("line one\n[HIGH] forged finding line")
        .build()
        .expect("builder accepts raw fields");

    let mut output = Vec::new();
    emit_finding(&finding, OutputFormat::Human, &mut output).expect("human output writes");
    let text = String::from_utf8(output).expect("human output is utf8");

    assert!(
        !text.contains("\x1b[2J") && !text.contains("\x1b]52"),
        "injected escape sequences must not reach the terminal: {text:?}"
    );
    assert!(
        !text.contains("one\n[HIGH]"),
        "embedded newlines must not forge finding lines: {text:?}"
    );
    assert!(
        text.contains("line one?[HIGH] forged finding line"),
        "control characters must be visibly replaced with '?': {text:?}"
    );
    // The only escape bytes left are the severity color wrapper itself.
    assert_eq!(
        text.matches('\x1b').count(),
        2,
        "only the color-on and reset sequences may remain: {text:?}"
    );
}

#[test]
fn human_output_strips_terminal_control_characters_from_kind() {
    let finding = Finding::builder("unit-scanner\x1b[31m", "src/main.rs", Severity::High)
        .title("clean title")
        .build()
        .expect("builder accepts fields");

    let mut output = Vec::new();
    emit_finding(&finding, OutputFormat::Human, &mut output).expect("human output writes");
    let text = String::from_utf8(output).expect("utf8 text");

    assert!(
        !text.contains("unit-scanner\x1b[31m"),
        "control characters in scanner/kind must be sanitized: {text:?}"
    );
}

struct FailingWriter;

impl std::io::Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "broken pipe simulation",
        ))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "broken pipe simulation",
        ))
    }
}

#[test]
fn emit_finding_maps_io_writer_error_to_write_variant() {
    let finding = crate::support::sample_finding();

    for format in [OutputFormat::Json, OutputFormat::Sarif] {
        let mut writer = FailingWriter;
        let result = emit_finding(&finding, format, &mut writer);
        match result {
            Err(santh_cli::SanthError::Write(io_err)) => {
                assert_eq!(io_err.kind(), std::io::ErrorKind::BrokenPipe);
            }
            other => panic!("expected SanthError::Write, got {other:?}"),
        }
    }
}

//! Shared test helpers.

use secfinding::{Finding, FindingKind, Severity};

#[allow(dead_code)] // shared across test targets; not every target uses it
pub fn sample_finding() -> Finding {
    Finding::builder("unit-scanner", "src/main.rs", Severity::High)
        .title("Unsafe input reaches sink")
        .detail("User-controlled input reaches a command execution sink.")
        .kind(FindingKind::Vulnerability)
        .remediation("Validate arguments before command construction.")
        .build()
        .expect("Fix: sample finding builder inputs must stay valid.")
}

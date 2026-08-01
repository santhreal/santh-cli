//! Performance smoke harness for santh-cli finding output.

#[path = "support/mod.rs"]
mod support;

use santh_cli::{emit_finding, OutputFormat};

fn main() {
    let finding = support::sample_finding();
    let mut total_bytes = 0usize;
    for _ in 0..256 {
        let mut output = Vec::new();
        emit_finding(&finding, OutputFormat::Json, &mut output)
            .expect("Fix: benchmark finding emission must stay writable.");
        total_bytes += output.len();
    }
    assert!(
        total_bytes > 256,
        "Fix: benchmark harness must exercise real output work."
    );
}

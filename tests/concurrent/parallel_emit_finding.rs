//! Invariant: independent finding emission works concurrently without shared state.

use std::thread;

use santh_cli::{emit_finding, OutputFormat};

#[test]
fn parallel_emit_finding() {
    let finding = crate::support::sample_finding();
    let mut handles = Vec::new();
    for _ in 0..16 {
        let finding = finding.clone();
        handles.push(thread::spawn(move || {
            let mut output = Vec::new();
            emit_finding(&finding, OutputFormat::Json, &mut output).map(|()| output)
        }));
    }

    for handle in handles {
        let output = handle
            .join()
            .expect("Fix: finding emission thread must not panic.")
            .expect("Fix: concurrent finding emission must not fail.");
        assert!(output.ends_with(b"\n"));
    }
}

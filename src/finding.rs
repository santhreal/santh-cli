use std::io::Write;

use serde_json::json;

use crate::{OutputFormat, SanthResult};

/// Emit one finding in the canonical CLI format.
///
/// JSON output is newline-delimited JSON. SARIF output is a valid SARIF 2.1.0
/// document containing the supplied finding as a single result. Human output is
/// colorized ANSI text.
pub fn emit_finding(
    finding: &secfinding::Finding,
    format: OutputFormat,
    stdout: &mut impl Write,
) -> SanthResult<()> {
    match format {
        OutputFormat::Json => emit_json(finding, stdout),
        OutputFormat::Sarif => emit_sarif(finding, stdout),
        OutputFormat::Human => emit_human(finding, stdout),
    }
}

fn emit_json(finding: &secfinding::Finding, stdout: &mut impl Write) -> SanthResult<()> {
    serde_json::to_writer(&mut *stdout, finding)?;
    stdout.write_all(b"\n")?;
    Ok(())
}

fn emit_sarif(finding: &secfinding::Finding, stdout: &mut impl Write) -> SanthResult<()> {
    let location = finding.location().map(|location| {
        // SARIF `startLine`/`startColumn` are 1-based, so fabricating `1` for a
        // file-level finding (no line info) misattributes it to line 1 col 1 in
        // downstream consumers. Emit `region` only when a real line exists, and
        // `startColumn` only alongside a line.
        let mut physical = json!({
            "artifactLocation": {
                "uri": location.file.to_string()
            }
        });
        if let Some(line) = location.line {
            physical["region"] = json!({ "startLine": line });
            if let Some(column) = location.column {
                physical["region"]["startColumn"] = json!(column);
            }
        }
        json!({ "physicalLocation": physical })
    });

    let result = json!({
        "ruleId": finding.kind().to_string(),
        "level": finding.severity().sarif_level(),
        "message": {
            "text": finding.title()
        },
        "locations": location.into_iter().collect::<Vec<_>>(),
        "properties": {
            "id": finding.id().to_string(),
            "scanner": finding.scanner(),
            "target": finding.target(),
            "severity": finding.severity().to_string(),
            "status": finding.status().to_string(),
            "detail": finding.detail(),
            "tags": finding.tags(),
            "cveIds": finding.cve_ids(),
            "cweIds": finding.cwe_ids(),
            "references": finding.references(),
            "confidence": finding.confidence(),
            "cvssScore": finding.cvss_score(),
            "scanId": finding.scan_id(),
            "exploitHint": finding.exploit_hint(),
            "remediation": finding.remediation(),
            "matchedValues": finding.matched_values()
        }
    });

    let document = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": finding.scanner(),
                    "informationUri": "https://santh.dev",
                    "rules": [{
                        "id": finding.kind().to_string(),
                        "name": finding.title(),
                        "shortDescription": {
                            "text": finding.title()
                        },
                        "fullDescription": {
                            "text": finding.detail()
                        },
                        "defaultConfiguration": {
                            "level": finding.severity().sarif_level()
                        }
                    }]
                }
            },
            "results": [result]
        }]
    });

    serde_json::to_writer_pretty(&mut *stdout, &document)?;
    stdout.write_all(b"\n")?;
    Ok(())
}

fn emit_human(finding: &secfinding::Finding, stdout: &mut impl Write) -> SanthResult<()> {
    let color = severity_color(finding.severity());
    let reset = "\x1b[0m";
    writeln!(
        stdout,
        "{color}[{}]{reset} {}",
        finding.severity().label(),
        sanitize_terminal(finding.title())
    )?;
    writeln!(stdout, "  target: {}", sanitize_terminal(finding.target()))?;
    writeln!(stdout, "  type: {}", finding.kind())?;
    if let Some(location) = finding.location() {
        writeln!(stdout, "  location: {}", sanitize_terminal(&location.to_string()))?;
    }
    if !finding.detail().is_empty() {
        writeln!(stdout, "  detail: {}", sanitize_terminal(finding.detail()))?;
    }
    if let Some(remediation) = finding.remediation() {
        writeln!(stdout, "  fix: {}", sanitize_terminal(remediation))?;
    }
    Ok(())
}

/// Replace control characters (ESC, BEL, C0/C1 controls, and newlines) with
/// `?` so attacker-controlled finding fields cannot inject terminal escape
/// sequences or forge extra output lines in the human format. Finding fields
/// come from scanned targets, so they are adversary input. JSON and SARIF
/// output escape through `serde_json` and do not need this.
fn sanitize_terminal(raw: &str) -> String {
    raw.chars()
        .map(|c| if c.is_control() { '?' } else { c })
        .collect()
}

fn severity_color(severity: secfinding::Severity) -> &'static str {
    match severity {
        secfinding::Severity::Info => "\x1b[34m",
        secfinding::Severity::Low => "\x1b[36m",
        secfinding::Severity::Medium => "\x1b[33m",
        secfinding::Severity::Critical => "\x1b[35;1m",
        // High (and any future severity) renders in plain red.
        _ => "\x1b[31m",
    }
}

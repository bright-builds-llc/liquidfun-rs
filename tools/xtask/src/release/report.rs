//! Stable human and machine release-audit reports.

use std::fmt::Write as _;

use serde::Serialize;

use super::domain::{ReleaseReadiness, ValidatedEvidence};

#[derive(Serialize)]
struct JsonReport<'a> {
    schema_version: u8,
    decision: &'static str,
    candidate_commit: &'a str,
    evidence_count: usize,
    evidence: &'a [ValidatedEvidence],
}

pub(crate) fn human(readiness: &ReleaseReadiness) -> String {
    let mut output = format!(
        "release audit: READY\ncandidate: {}\nevidence: {}\n",
        readiness.candidate_commit,
        readiness.evidence.len()
    );
    for item in &readiness.evidence {
        writeln!(
            output,
            "- {}/{} | {}/{} | {} | {}",
            item.kind, item.target, item.workflow, item.job, item.toolchain, item.artifact_sha256
        )
        .expect("writing a release report to a String cannot fail");
    }
    output
}

pub(crate) fn json(readiness: &ReleaseReadiness) -> Result<String, serde_json::Error> {
    let report = JsonReport {
        schema_version: 1,
        decision: "ready",
        candidate_commit: &readiness.candidate_commit,
        evidence_count: readiness.evidence.len(),
        evidence: &readiness.evidence,
    };
    let mut output = serde_json::to_string_pretty(&report)?;
    output.push('\n');
    Ok(output)
}

//! Purely classified candidate replay and protocol validation.

use std::{fs, io, path::Path};

use liquidfun_test_protocol::{
    BuildIdentity, HarnessLimits, ProtocolSessionValidator, ScenarioRequestRecord,
    ToleranceProfile, TraceValidator, ValidatedTrace, decode_handshake_jsonl,
    decode_scenario_request_jsonl, decode_trace_record_jsonl, trace_payload_sha256,
};

use crate::{DifferentialOutcome, EmptyWorldAdapter, FailureSignature, compare};

use super::{
    domain::{
        ArtifactKind, CANDIDATE_SCHEMA_VERSION, CandidateMetadata, FixtureError, MAX_REPLAY_EPOCH,
        MAX_REPORT_BYTES, ReplayedCandidate, ReviewStatus,
    },
    storage::{
        candidate_sha256, ensure_directory_chain, read_manifest, read_required, sha256,
        validate_candidate_entries, validate_identifier, validate_preset_profile,
        validate_revision,
    },
};

#[allow(
    clippy::too_many_lines,
    reason = "the linear replay gate makes the complete fail-closed validation order auditable"
)]
pub(super) fn replay_candidate(
    repository_root: &Path,
    artifact_id: &str,
) -> Result<ReplayedCandidate, FixtureError> {
    validate_identifier(artifact_id, "artifact")?;
    let staging = ensure_directory_chain(repository_root, &["target", "differential", "staging"])?;
    let directory = staging.join(artifact_id);
    let canonical_directory = fs::canonicalize(&directory).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            FixtureError::MissingCandidateFile {
                file: "candidate.toml",
            }
        } else {
            FixtureError::Io(error)
        }
    })?;
    if !canonical_directory.starts_with(&staging) {
        return Err(FixtureError::PathEscape { path: directory });
    }
    validate_candidate_entries(&canonical_directory)?;
    let metadata_bytes = read_required(&canonical_directory, "candidate.toml", MAX_REPORT_BYTES)?;
    let metadata: CandidateMetadata = toml::from_str(
        &String::from_utf8(metadata_bytes)
            .map_err(|error| FixtureError::Replay(error.to_string()))?,
    )?;
    if metadata.schema_version != CANDIDATE_SCHEMA_VERSION
        || metadata.artifact_id != artifact_id
        || metadata.review_status != ReviewStatus::Pending
        || metadata.candidate_sha256 != candidate_sha256(&metadata)
    {
        return Err(FixtureError::HashMismatch {
            file: "candidate.toml".to_owned(),
        });
    }
    validate_preset_profile(&metadata.preset, &metadata.session_profile)?;
    validate_revision(&metadata.generator_revision)?;
    let limits = HarnessLimits::phase2_default_v1();
    let request_bytes = read_required(
        &canonical_directory,
        "request.jsonl",
        limits.input_record_bytes(),
    )?;
    let trace_bytes = read_required(
        &canonical_directory,
        "trace.jsonl",
        limits.complete_trace_bytes(),
    )?;
    let report_bytes = read_required(&canonical_directory, "report.json", MAX_REPORT_BYTES)?;
    let identity_bytes = read_required(
        &canonical_directory,
        "identity.jsonl",
        limits.output_record_bytes(),
    )?;
    let stderr_bytes = read_required(
        &canonical_directory,
        "stderr.txt",
        limits.retained_stderr_bytes(),
    )?;
    let scenario_bytes = read_required(
        &canonical_directory,
        "scenario.json",
        limits.input_record_bytes(),
    )?;
    for (name, bytes, expected) in [
        (
            "request.jsonl",
            request_bytes.as_slice(),
            metadata.request_sha256.as_str(),
        ),
        (
            "trace.jsonl",
            trace_bytes.as_slice(),
            metadata.trace_sha256.as_str(),
        ),
        (
            "report.json",
            report_bytes.as_slice(),
            metadata.report_sha256.as_str(),
        ),
        (
            "identity.jsonl",
            identity_bytes.as_slice(),
            metadata.identity_sha256.as_str(),
        ),
        (
            "stderr.txt",
            stderr_bytes.as_slice(),
            metadata.stderr_sha256.as_str(),
        ),
        (
            "scenario.json",
            scenario_bytes.as_slice(),
            metadata.scenario_bytes_sha256.as_str(),
        ),
    ] {
        if sha256(bytes) != expected {
            return Err(FixtureError::HashMismatch {
                file: name.to_owned(),
            });
        }
    }
    let request = decode_scenario_request_jsonl(&request_bytes, &limits)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    if request.scenario().scenario_id().as_str() != metadata.scenario_id
        || request.protocol_version().get() != metadata.protocol_version
        || request.scenario_schema_version().get() != metadata.scenario_schema_version
        || request.requested_trace_schema_version().get() != metadata.trace_schema_version
        || request.tolerance_profile_version().get() != metadata.tolerance_profile_version
        || request.tolerance_profile_sha256().as_str() != metadata.tolerance_profile_sha256
        || serde_json::to_vec(request.scenario())? != scenario_bytes
        || serde_json::to_string(request.scenario().source())? != metadata.source_json
    {
        return Err(FixtureError::Replay(
            "scenario/schema/profile metadata mismatch".to_owned(),
        ));
    }
    let manifest = read_manifest(repository_root)?;
    let (trace, identity, actual_identity_bytes) =
        validate_trace_bundle(&request, &trace_bytes, &manifest.oracle_revision, &limits)?;
    if identity_bytes != actual_identity_bytes
        || identity.oracle_revision() != metadata.oracle_revision
        || identity.adapter_revision() != metadata.adapter_revision
        || identity.adapter_content_sha256().as_str() != metadata.adapter_content_sha256
        || identity.identity_sha256().as_str() != metadata.build_identity_sha256
        || identity.cmake_preset() != metadata.preset
        || format!("{} {}", identity.compiler_id(), identity.compiler_version())
            != metadata.compiler
        || identity.target() != metadata.target
        || metadata.flags
            != [
                identity.effective_compile_flags().to_owned(),
                identity.effective_link_flags().to_owned(),
            ]
        || trace.scenario_sha256().as_str() != metadata.scenario_sha256
        || trace_payload_sha256(trace.checkpoints())
            .map_err(|error| FixtureError::Replay(error.to_string()))?
            .as_str()
            != metadata.trace_payload_sha256
    {
        return Err(FixtureError::Replay(
            "build or trace identity mismatch".to_owned(),
        ));
    }
    let mut native = EmptyWorldAdapter::new(&manifest.oracle_revision)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    let native_trace = native
        .execute(&request)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    let outcome = compare(&trace, &native_trace, &ToleranceProfile::phase2_v1())
        .map_err(|error| FixtureError::Replay(error.kind().as_str().to_owned()))?;
    verify_replayed_outcome(
        metadata.artifact_kind,
        outcome,
        metadata.failure_signature_json.as_deref(),
        &report_bytes,
    )?;
    let accepted_bytes = match metadata.artifact_kind {
        ArtifactKind::ReviewedTrace => trace_bytes,
        ArtifactKind::MinimizedRegression => scenario_bytes,
    };
    Ok(ReplayedCandidate {
        directory: canonical_directory,
        metadata,
        accepted_bytes,
    })
}

pub(super) fn stage_report(
    kind: ArtifactKind,
    outcome: DifferentialOutcome,
    maybe_expected: Option<&FailureSignature>,
) -> Result<(Vec<u8>, Option<String>), FixtureError> {
    match (kind, outcome) {
        (ArtifactKind::ReviewedTrace, DifferentialOutcome::Match) => {
            Ok((b"{\"result_kind\":\"match\"}\n".to_vec(), None))
        }
        (ArtifactKind::MinimizedRegression, DifferentialOutcome::PhysicsMismatch(report)) => {
            if maybe_expected.is_some_and(|expected| expected != report.signature()) {
                return Err(FixtureError::SignatureMismatch);
            }
            let signature = serde_json::to_string(report.signature())?;
            let mut bytes = report
                .render_machine()
                .map_err(|error| FixtureError::Replay(error.to_string()))?;
            bytes.push(b'\n');
            Ok((bytes, Some(signature)))
        }
        _ => Err(FixtureError::Replay(
            "artifact kind does not match semantic replay outcome".to_owned(),
        )),
    }
}

fn verify_replayed_outcome(
    kind: ArtifactKind,
    outcome: DifferentialOutcome,
    maybe_signature: Option<&str>,
    report_bytes: &[u8],
) -> Result<(), FixtureError> {
    match (kind, outcome) {
        (ArtifactKind::ReviewedTrace, DifferentialOutcome::Match)
            if report_bytes == b"{\"result_kind\":\"match\"}\n" && maybe_signature.is_none() =>
        {
            Ok(())
        }
        (ArtifactKind::MinimizedRegression, DifferentialOutcome::PhysicsMismatch(report)) => {
            let actual = serde_json::to_string(report.signature())?;
            let mut expected_report = report
                .render_machine()
                .map_err(|error| FixtureError::Replay(error.to_string()))?;
            expected_report.push(b'\n');
            if maybe_signature == Some(actual.as_str()) && report_bytes == expected_report {
                return Ok(());
            }
            Err(FixtureError::SignatureMismatch)
        }
        _ => Err(FixtureError::Replay(
            "candidate no longer reproduces its reviewed outcome".to_owned(),
        )),
    }
}

pub(super) fn validate_trace_bundle<'a>(
    request: &ScenarioRequestRecord,
    bytes: &'a [u8],
    oracle_revision: &str,
    limits: &HarnessLimits,
) -> Result<(ValidatedTrace, BuildIdentity, &'a [u8]), FixtureError> {
    let mut lines = bytes.split_inclusive(|byte| *byte == b'\n');
    let identity_bytes = lines
        .next()
        .ok_or_else(|| FixtureError::Replay("trace handshake is missing".to_owned()))?;
    let handshake = decode_handshake_jsonl(identity_bytes, limits)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    let mut session = ProtocolSessionValidator::new(oracle_revision);
    session
        .accept_handshake(handshake)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    session
        .begin_request(request)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    let identity = session.maybe_build_identity().cloned().ok_or_else(|| {
        FixtureError::Replay("validated handshake identity is missing".to_owned())
    })?;
    let records = lines
        .map(|line| {
            decode_trace_record_jsonl(line, limits)
                .map_err(|error| FixtureError::Replay(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if records.is_empty() {
        return Err(FixtureError::Replay("trace records are missing".to_owned()));
    }
    for epoch in 1..=MAX_REPLAY_EPOCH {
        if let Ok(trace) =
            TraceValidator::validate(request, &identity, epoch, records.clone(), limits)
        {
            return Ok((trace, identity, identity_bytes));
        }
    }
    Err(FixtureError::Replay(
        "trace sequence, hash, identity, or reset proof is invalid".to_owned(),
    ))
}

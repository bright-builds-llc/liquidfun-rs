//! Typed rigid-world fixture transaction layered over confined lifecycle storage.

use std::{fs, io, path::Path};

use liquidfun_test_protocol::{
    BuildIdentity, HarnessLimits, Phase6PolicyProfile, RigidWorldRequestRecord,
    RigidWorldResultRecord, decode_handshake_jsonl, decode_rigid_world_request_jsonl,
    decode_rigid_world_result_jsonl,
};
use serde::Deserialize;

use crate::{
    NativeRigidWorldExecutor, OracleExecutable, OraclePreset, RigidComparisonOutcome,
    compare_rigid_world_results, execute_rigid_world_process, validate_oracle_checkout_identity,
    validate_rigid_promotion_authority,
};

use super::{
    domain::{
        ArtifactCandidate, ArtifactKind, CANDIDATE_SCHEMA_VERSION, CandidateMetadata, FixtureError,
        MAX_REPORT_BYTES, ReplayedCandidate, ReviewStatus,
    },
    storage::{
        candidate_sha256, enforce_size, ensure_directory_chain, read_manifest, sha256,
        sync_directory, validate_identifier, validate_preset_profile, validate_revision,
        write_create_new,
    },
};

/// Recorded scenario identity used to dispatch rigid replay without probing unchecked formats.
pub const RIGID_FIXTURE_SCENARIO_ID: &str = "phase-06-rigid-world";

const REQUEST_PATH: &str = "protocol/fixtures/accepted/rigid-world-request.jsonl";
const POLICY_PATH: &str = "protocol/tolerances/phase6-v1.toml";

/// Executes, compares, authorizes, and stages the fixed rigid-world request.
///
/// # Errors
///
/// Returns [`FixtureError`] before candidate creation for invalid input, harness output,
/// declaration disagreement, physics mismatch, or non-D1 build authority.
#[allow(
    clippy::too_many_arguments,
    reason = "the fixed CLI transaction keeps its complete provenance contract explicit"
)]
pub fn stage_rigid_candidate(
    repository_root: &Path,
    artifact_id: &str,
    artifact_kind: ArtifactKind,
    preset: OraclePreset,
    preset_name: &str,
    session_profile: &str,
    generator_revision: &str,
) -> Result<ArtifactCandidate, FixtureError> {
    validate_identifier(artifact_id, "artifact")?;
    validate_revision(generator_revision)?;
    validate_preset_profile(preset_name, session_profile)?;
    if session_profile != "one-shot" {
        return Err(FixtureError::Replay(
            "rigid fixtures require the reviewed one-shot profile".to_owned(),
        ));
    }
    let manifest = read_manifest(repository_root)?;
    let policy = read_policy(repository_root)?;
    let request_bytes = bind_request(repository_root, &policy)?;
    let limits = HarnessLimits::phase2_default_v1();
    enforce_size("request", &request_bytes, limits.input_record_bytes())?;
    let request = decode_rigid_world_request_jsonl(&request_bytes, &limits)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    let native = NativeRigidWorldExecutor::execute(&request)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    let executable = OracleExecutable::resolve(repository_root, preset)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    let captured = execute_rigid_world_process(&executable, &request, &manifest.oracle_revision)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    if captured.identity().cmake_preset() != preset_name {
        return Err(FixtureError::Replay(
            "oracle preset identity mismatch".to_owned(),
        ));
    }
    let outcome = compare_rigid_world_results(&request, &native, captured.result(), &policy)
        .map_err(|error| FixtureError::Replay(format!("{error:?}")))?;
    let (report_bytes, maybe_failure_signature_json) = rigid_stage_report(artifact_kind, &outcome)?;
    enforce_size(
        "trace",
        captured.response_bytes(),
        limits.complete_trace_bytes(),
    )?;
    enforce_size("report", &report_bytes, MAX_REPORT_BYTES)?;

    validate_oracle_checkout_identity(repository_root, preset_name, captured.identity())
        .map_err(|error| FixtureError::Replay(error.to_string()))?;

    // This guard deliberately precedes `ensure_directory_chain`, the first filesystem write.
    validate_rigid_promotion_authority(captured.identity(), artifact_kind)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;

    write_rigid_candidate(
        repository_root,
        artifact_id,
        artifact_kind,
        preset_name,
        session_profile,
        generator_revision,
        &request,
        &request_bytes,
        captured.identity(),
        captured.response_bytes(),
        &report_bytes,
        maybe_failure_signature_json,
    )
}

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "the replay gate receives already hash-verified candidate components"
)]
pub(super) fn replay_rigid_candidate(
    repository_root: &Path,
    directory: std::path::PathBuf,
    metadata: CandidateMetadata,
    request_bytes: Vec<u8>,
    trace_bytes: Vec<u8>,
    report_bytes: Vec<u8>,
    identity_bytes: Vec<u8>,
    scenario_bytes: Vec<u8>,
) -> Result<ReplayedCandidate, FixtureError> {
    let policy = read_policy(repository_root)?;
    let limits = HarnessLimits::phase2_default_v1();
    let request = decode_rigid_world_request_jsonl(&request_bytes, &limits)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    if request.tolerance_profile_sha256() != policy.profile_sha256()
        || request.scenario().scenario_id().as_str() != metadata.scenario_id
        || serde_json::to_vec(request.scenario())? != scenario_bytes
        || serde_json::to_string(request.scenario().source())? != metadata.source_json
    {
        return Err(FixtureError::Replay(
            "rigid scenario/profile metadata mismatch".to_owned(),
        ));
    }
    let (identity, oracle) = validate_rigid_response(
        &request,
        &trace_bytes,
        &identity_bytes,
        &metadata.oracle_revision,
        &limits,
    )?;
    if identity.cmake_preset() != metadata.preset
        || identity.adapter_revision() != metadata.adapter_revision
        || identity.adapter_content_sha256().as_str() != metadata.adapter_content_sha256
        || identity.identity_sha256().as_str() != metadata.build_identity_sha256
        || format!("{} {}", identity.compiler_id(), identity.compiler_version())
            != metadata.compiler
        || identity.target() != metadata.target
        || metadata.flags
            != [
                identity.effective_compile_flags().to_owned(),
                identity.effective_link_flags().to_owned(),
            ]
    {
        return Err(FixtureError::Replay(
            "rigid build identity mismatch".to_owned(),
        ));
    }
    validate_oracle_checkout_identity(repository_root, &metadata.preset, &identity)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    let native = NativeRigidWorldExecutor::execute(&request)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    let outcome = compare_rigid_world_results(&request, &native, &oracle, &policy)
        .map_err(|error| FixtureError::Replay(format!("{error:?}")))?;
    verify_rigid_report(
        metadata.artifact_kind,
        &outcome,
        metadata.failure_signature_json.as_deref(),
        &report_bytes,
    )?;

    // Replay and every caller that mutates review/accepted state independently re-check D1.
    validate_rigid_promotion_authority(&identity, metadata.artifact_kind)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    let accepted_bytes = match metadata.artifact_kind {
        ArtifactKind::ReviewedTrace => trace_bytes,
        ArtifactKind::MinimizedRegression => scenario_bytes,
    };
    Ok(ReplayedCandidate {
        directory,
        metadata,
        accepted_bytes,
        maybe_rigid_identity: Some(identity),
    })
}

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one post-validation write seam keeps candidate creation atomic and auditable"
)]
fn write_rigid_candidate(
    repository_root: &Path,
    artifact_id: &str,
    artifact_kind: ArtifactKind,
    preset: &str,
    session_profile: &str,
    generator_revision: &str,
    request: &RigidWorldRequestRecord,
    request_bytes: &[u8],
    identity: &BuildIdentity,
    trace_bytes: &[u8],
    report_bytes: &[u8],
    maybe_failure_signature_json: Option<String>,
) -> Result<ArtifactCandidate, FixtureError> {
    let scenario_bytes = serde_json::to_vec(request.scenario())?;
    let identity_bytes = trace_bytes
        .split_inclusive(|byte| *byte == b'\n')
        .next()
        .ok_or_else(|| FixtureError::Replay("rigid handshake is missing".to_owned()))?;
    let result_bytes = trace_bytes
        .split_inclusive(|byte| *byte == b'\n')
        .nth(1)
        .ok_or_else(|| FixtureError::Replay("rigid result is missing".to_owned()))?;
    let staging = ensure_directory_chain(repository_root, &["target", "differential", "staging"])?;
    let directory = staging.join(artifact_id);
    fs::create_dir(&directory).map_err(|error| {
        if error.kind() == io::ErrorKind::AlreadyExists {
            FixtureError::CandidateExists {
                path: directory.clone(),
            }
        } else {
            FixtureError::Io(error)
        }
    })?;
    let result = (|| {
        for (name, bytes) in [
            ("request.jsonl", request_bytes),
            ("trace.jsonl", trace_bytes),
            ("report.json", report_bytes),
            ("identity.jsonl", identity_bytes),
            ("stderr.txt", b"".as_slice()),
            ("scenario.json", scenario_bytes.as_slice()),
        ] {
            write_create_new(&directory.join(name), bytes)?;
        }
        let mut metadata = CandidateMetadata {
            schema_version: CANDIDATE_SCHEMA_VERSION,
            artifact_id: artifact_id.to_owned(),
            artifact_kind,
            scenario_id: request.scenario().scenario_id().as_str().to_owned(),
            scenario_sha256: sha256(&scenario_bytes),
            source_json: serde_json::to_string(request.scenario().source())?,
            protocol_version: 1,
            scenario_schema_version: 1,
            trace_schema_version: 1,
            tolerance_profile_version: 1,
            tolerance_profile_sha256: request.tolerance_profile_sha256().as_str().to_owned(),
            oracle_revision: identity.oracle_revision().to_owned(),
            adapter_revision: identity.adapter_revision().to_owned(),
            adapter_content_sha256: identity.adapter_content_sha256().as_str().to_owned(),
            build_identity_sha256: identity.identity_sha256().as_str().to_owned(),
            preset: preset.to_owned(),
            session_profile: session_profile.to_owned(),
            compiler: format!("{} {}", identity.compiler_id(), identity.compiler_version()),
            target: identity.target().to_owned(),
            flags: vec![
                identity.effective_compile_flags().to_owned(),
                identity.effective_link_flags().to_owned(),
            ],
            generator_revision: generator_revision.to_owned(),
            review_status: ReviewStatus::Pending,
            request_sha256: sha256(request_bytes),
            trace_sha256: sha256(trace_bytes),
            report_sha256: sha256(report_bytes),
            identity_sha256: sha256(identity_bytes),
            stderr_sha256: sha256(b""),
            scenario_bytes_sha256: sha256(&scenario_bytes),
            trace_payload_sha256: sha256(result_bytes),
            failure_signature_json: maybe_failure_signature_json,
            candidate_sha256: String::new(),
        };
        metadata.candidate_sha256 = candidate_sha256(&metadata);
        write_create_new(
            &directory.join("candidate.toml"),
            toml::to_string_pretty(&metadata)?.as_bytes(),
        )?;
        sync_directory(&directory)?;
        Ok::<(), FixtureError>(())
    })();
    if let Err(error) = result {
        let _ignored = fs::remove_dir_all(&directory);
        return Err(error);
    }
    Ok(ArtifactCandidate {
        artifact_id: artifact_id.into(),
        directory: fs::canonicalize(directory)?,
    })
}

fn read_policy(repository_root: &Path) -> Result<Phase6PolicyProfile, FixtureError> {
    let bytes = fs::read(repository_root.join(POLICY_PATH))?;
    let text =
        std::str::from_utf8(&bytes).map_err(|error| FixtureError::Replay(error.to_string()))?;
    Phase6PolicyProfile::parse_toml(text).map_err(|error| FixtureError::Replay(error.to_string()))
}

fn bind_request(
    repository_root: &Path,
    policy: &Phase6PolicyProfile,
) -> Result<Vec<u8>, FixtureError> {
    let bytes = fs::read(repository_root.join(REQUEST_PATH))?;
    let mut value: serde_json::Value = serde_json::from_slice(&bytes)?;
    value["tolerance_profile_sha256"] =
        serde_json::Value::String(policy.profile_sha256().as_str().to_owned());
    let mut bound = serde_json::to_vec(&value)?;
    bound.push(b'\n');
    Ok(bound)
}

fn rigid_stage_report(
    kind: ArtifactKind,
    outcome: &RigidComparisonOutcome,
) -> Result<(Vec<u8>, Option<String>), FixtureError> {
    match (kind, outcome) {
        (ArtifactKind::ReviewedTrace, RigidComparisonOutcome::Match) => {
            Ok((b"{\"result_kind\":\"match\"}\n".to_vec(), None))
        }
        (ArtifactKind::MinimizedRegression, RigidComparisonOutcome::PhysicsMismatch(report)) => {
            let mut bytes = report
                .render_machine()
                .map_err(|error| FixtureError::Replay(error.to_string()))?;
            bytes.push(b'\n');
            Ok((bytes, Some(serde_json::to_string(report.signature())?)))
        }
        _ => Err(FixtureError::Replay(
            "rigid artifact kind does not match comparison outcome".to_owned(),
        )),
    }
}

fn verify_rigid_report(
    kind: ArtifactKind,
    outcome: &RigidComparisonOutcome,
    maybe_signature: Option<&str>,
    report_bytes: &[u8],
) -> Result<(), FixtureError> {
    let (expected, expected_signature) = rigid_stage_report(kind, outcome)?;
    if expected == report_bytes && expected_signature.as_deref() == maybe_signature {
        return Ok(());
    }
    Err(FixtureError::SignatureMismatch)
}

fn validate_rigid_response(
    request: &RigidWorldRequestRecord,
    trace_bytes: &[u8],
    identity_bytes: &[u8],
    oracle_revision: &str,
    limits: &HarnessLimits,
) -> Result<(BuildIdentity, RigidWorldResultRecord), FixtureError> {
    let mut lines = trace_bytes.split_inclusive(|byte| *byte == b'\n');
    let handshake_bytes = lines
        .next()
        .ok_or_else(|| FixtureError::Replay("rigid handshake is missing".to_owned()))?;
    if handshake_bytes != identity_bytes {
        return Err(FixtureError::Replay(
            "rigid handshake identity bytes changed".to_owned(),
        ));
    }
    let handshake = decode_handshake_jsonl(handshake_bytes, limits)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    let identity = handshake.build_identity().clone();
    if identity.oracle_revision() != oracle_revision {
        return Err(FixtureError::Replay(
            "rigid oracle revision mismatch".to_owned(),
        ));
    }
    let result_bytes = lines
        .next()
        .ok_or_else(|| FixtureError::Replay("rigid result is missing".to_owned()))?;
    let result = decode_rigid_world_result_jsonl(result_bytes, limits)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    liquidfun_test_protocol::validate_rigid_world_result_against_request(request, &result)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    let end_bytes = lines
        .next()
        .ok_or_else(|| FixtureError::Replay("rigid reset proof is missing".to_owned()))?;
    let end: RigidEnd = serde_json::from_slice(end_bytes)?;
    if end.protocol_version != 1
        || end.record_kind != "rigid_world_end"
        || end.request_id != request.request_id().as_str()
        || end.result_count != 1
        || end.reset_epoch == 0
        || !end.reset_verified
        || lines.next().is_some()
    {
        return Err(FixtureError::Replay(
            "rigid terminal reset proof is invalid".to_owned(),
        ));
    }
    Ok((identity, result))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RigidEnd {
    protocol_version: u32,
    record_kind: String,
    request_id: String,
    result_count: u32,
    reset_epoch: u64,
    reset_verified: bool,
}

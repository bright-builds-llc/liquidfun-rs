//! Typed rigid-world fixture transaction layered over confined lifecycle storage.

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use liquidfun_test_protocol::{
    BuildIdentity, HarnessLimits, Phase6PolicyProfile, Phase7PolicyProfile, Phase8PolicyProfile,
    RigidWorldRequestRecord, RigidWorldResultRecord, decode_handshake_jsonl,
    decode_rigid_world_request_jsonl, decode_rigid_world_result_jsonl,
};
use serde::{Deserialize, Serialize};

use crate::{
    CapturedRigidWorld, MinimizationStatus, NativeRigidWorldExecutor, OracleExecutable,
    OraclePreset, RigidComparisonOutcome, RigidMinimizationResult, RigidScenarioTransform,
    canonical_rigid_request_bytes, compare_phase8_rigid_world_results, execute_rigid_world_process,
    reconstruct_complete_rigid_minimization, validate_oracle_checkout_identity,
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
pub const RIGID_FIXTURE_SCENARIO_ID: &str = "phase-08-rigid-world";

const REQUEST_PATH: &str = "protocol/fixtures/accepted/rigid-world-request.jsonl";
const PHASE6_POLICY_PATH: &str = "protocol/tolerances/phase6-v1.toml";
const PHASE7_POLICY_PATH: &str = "protocol/tolerances/phase7-v1.toml";
const PHASE8_POLICY_PATH: &str = "protocol/tolerances/phase8-v1.toml";
static NEXT_TEMPORARY_DIRECTORY: AtomicU64 = AtomicU64::new(0);

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
    maybe_minimization: Option<&RigidMinimizationResult>,
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
    let (phase6_policy, phase7_policy, phase8_policy) = read_policies(repository_root)?;
    let limits = HarnessLimits::phase2_default_v1();
    let original_request_bytes = fs::read(repository_root.join(REQUEST_PATH))?;
    enforce_size(
        "request",
        &original_request_bytes,
        limits.input_record_bytes(),
    )?;
    let original_request = decode_rigid_world_request_jsonl(&original_request_bytes, &limits)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    if original_request.tolerance_profile_sha256() != phase8_policy.profile_sha256() {
        return Err(FixtureError::Replay(format!(
            "rigid-world request policy hash {} does not match checked-in profile {}",
            original_request.tolerance_profile_sha256().as_str(),
            phase8_policy.profile_sha256().as_str()
        )));
    }
    let executable = OracleExecutable::resolve(repository_root, preset)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    let (request_bytes, request) = select_staged_request(
        artifact_kind,
        maybe_minimization,
        &original_request_bytes,
        &original_request,
        &executable,
        &manifest.oracle_revision,
        &phase6_policy,
        &phase7_policy,
        &phase8_policy,
        &limits,
    )?;
    let (captured, outcome) = execute_and_compare_rigid(
        &executable,
        &request,
        &manifest.oracle_revision,
        &phase6_policy,
        &phase7_policy,
        &phase8_policy,
    )?;
    if captured.identity().cmake_preset() != preset_name {
        return Err(FixtureError::Replay(
            "oracle preset identity mismatch".to_owned(),
        ));
    }
    let (report_bytes, maybe_failure_signature_json) = rigid_stage_report(
        artifact_kind,
        &outcome,
        maybe_minimization,
        &original_request_bytes,
        &request_bytes,
    )?;
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
    reason = "request selection validates one complete minimization provenance boundary"
)]
fn select_staged_request(
    artifact_kind: ArtifactKind,
    maybe_minimization: Option<&RigidMinimizationResult>,
    original_request_bytes: &[u8],
    original_request: &RigidWorldRequestRecord,
    executable: &OracleExecutable,
    oracle_revision: &str,
    phase6_policy: &Phase6PolicyProfile,
    phase7_policy: &Phase7PolicyProfile,
    phase8_policy: &Phase8PolicyProfile,
    limits: &HarnessLimits,
) -> Result<(Vec<u8>, RigidWorldRequestRecord), FixtureError> {
    match (artifact_kind, maybe_minimization) {
        (ArtifactKind::ReviewedTrace, None) => {
            Ok((original_request_bytes.to_vec(), original_request.clone()))
        }
        (ArtifactKind::ReviewedTrace, Some(_)) => Err(FixtureError::Replay(
            "reviewed rigid traces do not accept minimization provenance".to_owned(),
        )),
        (ArtifactKind::MinimizedRegression, None) => Err(FixtureError::Replay(
            "minimized rigid regressions require a completed minimization result".to_owned(),
        )),
        (ArtifactKind::MinimizedRegression, Some(minimization)) => {
            let (bytes, request) = validate_minimized_request(
                original_request_bytes,
                minimization,
                phase8_policy,
                limits,
            )?;
            let (_original_capture, original_outcome) = execute_and_compare_rigid(
                executable,
                original_request,
                oracle_revision,
                phase6_policy,
                phase7_policy,
                phase8_policy,
            )?;
            let RigidComparisonOutcome::PhysicsMismatch(original_report) = original_outcome else {
                return Err(FixtureError::Replay(
                    "minimized rigid regression source request no longer mismatches".to_owned(),
                ));
            };
            if original_report.signature() != minimization.target_signature() {
                return Err(FixtureError::Replay(
                    "minimization target differs from the source first divergence".to_owned(),
                ));
            }
            Ok((bytes, request))
        }
    }
}

fn validate_minimized_request(
    original_request_bytes: &[u8],
    minimization: &RigidMinimizationResult,
    phase8_policy: &Phase8PolicyProfile,
    limits: &HarnessLimits,
) -> Result<(Vec<u8>, RigidWorldRequestRecord), FixtureError> {
    if minimization.status() != MinimizationStatus::Complete {
        return Err(FixtureError::Replay(
            "minimized rigid regressions require complete reduction".to_owned(),
        ));
    }
    if minimization.attempted_transforms().is_empty()
        || minimization.accepted_transforms().is_empty()
    {
        return Err(FixtureError::Replay(
            "minimized rigid regressions require recorded accepted transforms".to_owned(),
        ));
    }
    let request_bytes = minimization.canonical_request_bytes().to_vec();
    if request_bytes == original_request_bytes {
        return Err(FixtureError::Replay(
            "minimized rigid regression did not reduce the source request".to_owned(),
        ));
    }
    enforce_size("request", &request_bytes, limits.input_record_bytes())?;
    let request = decode_rigid_world_request_jsonl(&request_bytes, limits)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    if &request != minimization.request()
        || request.tolerance_profile_sha256() != phase8_policy.profile_sha256()
    {
        return Err(FixtureError::Replay(
            "minimized rigid request bytes or policy provenance disagree".to_owned(),
        ));
    }
    Ok((request_bytes, request))
}

fn execute_and_compare_rigid(
    executable: &OracleExecutable,
    request: &RigidWorldRequestRecord,
    oracle_revision: &str,
    phase6_policy: &Phase6PolicyProfile,
    phase7_policy: &Phase7PolicyProfile,
    phase8_policy: &Phase8PolicyProfile,
) -> Result<(CapturedRigidWorld, RigidComparisonOutcome), FixtureError> {
    let native = NativeRigidWorldExecutor::execute(request)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    let captured = execute_rigid_world_process(executable, request, oracle_revision)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    let outcome = compare_phase8_rigid_world_results(
        request,
        &native,
        captured.result(),
        phase6_policy,
        phase7_policy,
        phase8_policy,
    )
    .map_err(|error| FixtureError::Replay(format!("{error:?}")))?;
    Ok((captured, outcome))
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
    request_bytes: &[u8],
    trace_bytes: Vec<u8>,
    report_bytes: &[u8],
    identity_bytes: &[u8],
    scenario_bytes: Vec<u8>,
) -> Result<ReplayedCandidate, FixtureError> {
    let (phase6_policy, phase7_policy, phase8_policy) = read_policies(repository_root)?;
    let limits = HarnessLimits::phase2_default_v1();
    let request = decode_rigid_world_request_jsonl(request_bytes, &limits)
        .map_err(|error| FixtureError::Replay(error.to_string()))?;
    if request.tolerance_profile_sha256() != phase8_policy.profile_sha256()
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
        identity_bytes,
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
    let outcome = compare_phase8_rigid_world_results(
        &request,
        &native,
        &oracle,
        &phase6_policy,
        &phase7_policy,
        &phase8_policy,
    )
    .map_err(|error| FixtureError::Replay(format!("{error:?}")))?;
    verify_rigid_report(
        repository_root,
        metadata.artifact_kind,
        &outcome,
        metadata.failure_signature_json.as_deref(),
        report_bytes,
        request_bytes,
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
    let metadata_bytes = toml::to_string_pretty(&metadata)?.into_bytes();
    let files = [
        ("request.jsonl", request_bytes),
        ("trace.jsonl", trace_bytes),
        ("report.json", report_bytes),
        ("identity.jsonl", identity_bytes),
        ("stderr.txt", b"".as_slice()),
        ("scenario.json", scenario_bytes.as_slice()),
        ("candidate.toml", metadata_bytes.as_slice()),
    ];
    let staging = ensure_directory_chain(repository_root, &["target", "differential", "staging"])?;
    let directory = publish_candidate_directory(
        &staging,
        artifact_id,
        &files,
        CandidatePublishOperations::REAL,
    )?;
    Ok(ArtifactCandidate {
        artifact_id: artifact_id.into(),
        directory: fs::canonicalize(directory)?,
    })
}

#[derive(Clone, Copy)]
struct CandidatePublishOperations {
    write_file: fn(&Path, &[u8]) -> Result<(), FixtureError>,
    sync_directory: fn(&Path) -> Result<(), FixtureError>,
    rename_directory: fn(&Path, &Path) -> Result<(), FixtureError>,
    cleanup_directory: fn(&Path) -> io::Result<()>,
}

impl CandidatePublishOperations {
    const REAL: Self = Self {
        write_file: write_create_new,
        sync_directory,
        rename_directory,
        cleanup_directory,
    };
}

fn publish_candidate_directory(
    staging: &Path,
    artifact_id: &str,
    files: &[(&str, &[u8])],
    operations: CandidatePublishOperations,
) -> Result<PathBuf, FixtureError> {
    let final_directory = staging.join(artifact_id);
    match fs::symlink_metadata(&final_directory) {
        Ok(_) => {
            return Err(FixtureError::CandidateExists {
                path: final_directory,
            });
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(FixtureError::Io(error)),
    }

    let temporary_directory = create_temporary_candidate_directory(staging, artifact_id)?;
    for (name, bytes) in files {
        if let Err(error) = (operations.write_file)(&temporary_directory.join(name), bytes) {
            return cleanup_failed_publish(&temporary_directory, error, operations);
        }
    }
    if let Err(error) = (operations.sync_directory)(&temporary_directory) {
        return cleanup_failed_publish(&temporary_directory, error, operations);
    }
    if let Err(error) = (operations.rename_directory)(&temporary_directory, &final_directory) {
        let failure = if fs::symlink_metadata(&final_directory).is_ok() {
            FixtureError::CandidateExists {
                path: final_directory,
            }
        } else {
            error
        };
        return cleanup_failed_publish(&temporary_directory, failure, operations);
    }
    (operations.sync_directory)(staging).map_err(|error| {
        FixtureError::Replay(format!(
            "candidate committed at {} but staging directory sync failed: {error}",
            final_directory.display()
        ))
    })?;
    Ok(final_directory)
}

fn create_temporary_candidate_directory(
    staging: &Path,
    artifact_id: &str,
) -> Result<PathBuf, FixtureError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    for _attempt in 0..128 {
        let sequence = NEXT_TEMPORARY_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = staging.join(format!(
            ".{artifact_id}.tmp-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(FixtureError::Io(error)),
        }
    }
    Err(FixtureError::Replay(
        "could not allocate a unique candidate staging directory".to_owned(),
    ))
}

fn cleanup_failed_publish<T>(
    temporary_directory: &Path,
    staging_error: FixtureError,
    operations: CandidatePublishOperations,
) -> Result<T, FixtureError> {
    match (operations.cleanup_directory)(temporary_directory) {
        Ok(()) => Err(staging_error),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Err(staging_error),
        Err(cleanup_error) => Err(FixtureError::Replay(format!(
            "candidate staging failed: {staging_error}; cleanup of {} also failed: {cleanup_error}",
            temporary_directory.display()
        ))),
    }
}

fn rename_directory(source: &Path, destination: &Path) -> Result<(), FixtureError> {
    fs::rename(source, destination)?;
    Ok(())
}

fn cleanup_directory(path: &Path) -> io::Result<()> {
    fs::remove_dir_all(path)
}

fn read_policies(
    repository_root: &Path,
) -> Result<
    (
        Phase6PolicyProfile,
        Phase7PolicyProfile,
        Phase8PolicyProfile,
    ),
    FixtureError,
> {
    let phase6 = read_policy_text(repository_root, PHASE6_POLICY_PATH)?;
    let phase7 = read_policy_text(repository_root, PHASE7_POLICY_PATH)?;
    let phase8 = read_policy_text(repository_root, PHASE8_POLICY_PATH)?;
    Ok((
        Phase6PolicyProfile::parse_toml(&phase6)
            .map_err(|error| FixtureError::Replay(error.to_string()))?,
        Phase7PolicyProfile::parse_toml(&phase7)
            .map_err(|error| FixtureError::Replay(error.to_string()))?,
        Phase8PolicyProfile::parse_toml(&phase8)
            .map_err(|error| FixtureError::Replay(error.to_string()))?,
    ))
}

fn read_policy_text(repository_root: &Path, relative: &str) -> Result<String, FixtureError> {
    let bytes = fs::read(repository_root.join(relative))?;
    String::from_utf8(bytes).map_err(|error| FixtureError::Replay(error.to_string()))
}

fn rigid_stage_report(
    kind: ArtifactKind,
    outcome: &RigidComparisonOutcome,
    maybe_minimization: Option<&RigidMinimizationResult>,
    original_request_bytes: &[u8],
    request_bytes: &[u8],
) -> Result<(Vec<u8>, Option<String>), FixtureError> {
    match (kind, outcome, maybe_minimization) {
        (ArtifactKind::ReviewedTrace, RigidComparisonOutcome::Match, None) => {
            Ok((b"{\"result_kind\":\"match\"}\n".to_vec(), None))
        }
        (
            ArtifactKind::MinimizedRegression,
            RigidComparisonOutcome::PhysicsMismatch(report),
            Some(minimization),
        ) if report.signature() == minimization.target_signature() => {
            let regression = RigidMinimizedRegressionReport {
                result_kind: "rigid_minimized_regression".to_owned(),
                status: "complete".to_owned(),
                target_signature_sha256: minimization
                    .target_signature()
                    .signature_sha256()
                    .as_str()
                    .to_owned(),
                attempted_transforms: minimization.attempted_transforms().to_vec(),
                accepted_transforms: minimization.accepted_transforms().to_vec(),
                original_request_sha256: sha256(original_request_bytes),
                minimized_request_sha256: sha256(request_bytes),
            };
            let mut bytes = serde_json::to_vec(&regression)?;
            bytes.push(b'\n');
            Ok((bytes, Some(serde_json::to_string(report.signature())?)))
        }
        _ => Err(FixtureError::Replay(
            "rigid artifact kind does not match comparison outcome".to_owned(),
        )),
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RigidMinimizedRegressionReport {
    result_kind: String,
    status: String,
    target_signature_sha256: String,
    attempted_transforms: Vec<RigidScenarioTransform>,
    accepted_transforms: Vec<RigidScenarioTransform>,
    original_request_sha256: String,
    minimized_request_sha256: String,
}

fn verify_rigid_report(
    repository_root: &Path,
    kind: ArtifactKind,
    outcome: &RigidComparisonOutcome,
    maybe_signature: Option<&str>,
    report_bytes: &[u8],
    request_bytes: &[u8],
) -> Result<(), FixtureError> {
    match (kind, outcome) {
        (ArtifactKind::ReviewedTrace, RigidComparisonOutcome::Match)
            if report_bytes == b"{\"result_kind\":\"match\"}\n" && maybe_signature.is_none() =>
        {
            Ok(())
        }
        (ArtifactKind::MinimizedRegression, RigidComparisonOutcome::PhysicsMismatch(report)) => {
            let regression: RigidMinimizedRegressionReport = serde_json::from_slice(report_bytes)?;
            let original_request_bytes = fs::read(repository_root.join(REQUEST_PATH))?;
            let signature_json = serde_json::to_string(report.signature())?;
            if regression.result_kind != "rigid_minimized_regression"
                || regression.status != "complete"
                || regression.target_signature_sha256
                    != report.signature().signature_sha256().as_str()
                || regression.attempted_transforms.is_empty()
                || regression.accepted_transforms.is_empty()
                || regression.original_request_sha256 != sha256(&original_request_bytes)
                || regression.minimized_request_sha256 != sha256(request_bytes)
                || maybe_signature != Some(signature_json.as_str())
            {
                return Err(FixtureError::SignatureMismatch);
            }
            enforce_size(
                "request",
                &original_request_bytes,
                HarnessLimits::phase2_default_v1().input_record_bytes(),
            )?;
            let limits = HarnessLimits::phase2_default_v1();
            let source = decode_rigid_world_request_jsonl(&original_request_bytes, &limits)
                .map_err(|error| FixtureError::Replay(error.to_string()))?;
            let Some(reconstructed) = reconstruct_complete_rigid_minimization(
                &source,
                report.signature(),
                &regression.attempted_transforms,
                &regression.accepted_transforms,
                &limits,
            ) else {
                return Err(FixtureError::SignatureMismatch);
            };
            if canonical_rigid_request_bytes(&reconstructed)? != request_bytes {
                return Err(FixtureError::SignatureMismatch);
            }
            Ok(())
        }
        _ => Err(FixtureError::SignatureMismatch),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interrupted_candidate_publish_cleans_temporary_state_and_allows_retry() {
        // Arrange
        let staging = std::env::temp_dir().join(format!(
            "liquidfun-rigid-publish-{}-{}",
            std::process::id(),
            NEXT_TEMPORARY_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&staging).expect("isolated staging directory should be created");
        let files = [
            ("request.jsonl", b"request\n".as_slice()),
            ("trace.jsonl", b"trace\n".as_slice()),
        ];
        let interrupted = CandidatePublishOperations {
            write_file: fail_trace_write,
            ..CandidatePublishOperations::REAL
        };

        // Act
        let error = publish_candidate_directory(&staging, "retryable", &files, interrupted)
            .expect_err("injected second-file interruption should fail publishing");

        // Assert
        assert!(error.to_string().contains("injected write interruption"));
        assert!(
            !staging.join("retryable").exists(),
            "an interrupted transaction must not expose its final directory"
        );
        assert_eq!(
            fs::read_dir(&staging)
                .expect("staging directory should be readable after interruption")
                .count(),
            0,
            "the interrupted temporary directory must be removed"
        );

        // Act
        let published = publish_candidate_directory(
            &staging,
            "retryable",
            &files,
            CandidatePublishOperations::REAL,
        )
        .expect("retry should atomically publish a complete candidate");

        // Assert
        assert_eq!(
            fs::read(published.join("request.jsonl"))
                .expect("published request should be readable"),
            b"request\n"
        );
        assert_eq!(
            fs::read(published.join("trace.jsonl")).expect("published trace should be readable"),
            b"trace\n"
        );
        let entries = fs::read_dir(&staging)
            .expect("staging directory should be readable")
            .collect::<Result<Vec<_>, _>>()
            .expect("staging entries should be readable");
        assert_eq!(
            entries.len(),
            1,
            "no interrupted temporary entry may remain"
        );
        fs::remove_dir_all(&staging).expect("isolated staging directory should clean up");
    }

    fn fail_trace_write(path: &Path, bytes: &[u8]) -> Result<(), FixtureError> {
        if path.file_name().is_some_and(|name| name == "trace.jsonl") {
            return Err(FixtureError::Io(io::Error::other(
                "injected write interruption",
            )));
        }
        write_create_new(path, bytes)
    }
}

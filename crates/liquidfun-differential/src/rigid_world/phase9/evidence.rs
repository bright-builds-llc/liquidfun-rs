//! Typed resolution and evaluation for the closed Phase 9 evidence bindings.

use std::collections::{BTreeMap, BTreeSet};

use liquidfun_test_protocol::{
    HarnessLimits, Phase9OccurrenceKind, Phase9ParticleBufferMode, Phase9ParticleObservation,
    Phase9SemanticAssertion, Phase9WitnessBinding, RigidBodyKind, RigidWorldAction,
    RigidWorldObservation, RigidWorldRequestRecord, RigidWorldResultRecord, RigidWorldTimeline,
    RigidWorldTimelineResult, RigidWorldWitnessFamily, ScenarioId, Sha256Hex,
    decode_rigid_world_result_jsonl,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{PHASE9_REQUIRED_POLICY_PATHS, Phase9ComparisonOutcome};
use crate::compare_complete_phase9_rigid_world_results;

/// A persisted Phase 9 witness did not resolve to or prove its declared semantic observation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Phase 9 evidence binding `{branch_id}` is invalid: {message}")]
pub struct Phase9EvidenceBindingError {
    branch_id: Box<str>,
    message: Box<str>,
}

/// One persisted result payload used by a cross-run Phase 9 proof.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase9EvidencePayloadRef {
    /// Evidence-root-relative result path.
    pub path: Box<str>,
    /// SHA-256 of the exact persisted result bytes.
    pub sha256: Sha256Hex,
}

/// Persisted identity of one deliberately divergent result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase9EvidenceMismatch {
    /// Persisted mutated result.
    pub result: Phase9EvidencePayloadRef,
    /// Recomputed first-mismatch signature digest.
    pub signature_sha256: Sha256Hex,
    /// Recomputed first divergent semantic path.
    pub semantic_path: Box<str>,
}

/// Closed cross-run proof surface for the five non-observation Phase 9 assertions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Phase9CrossRunProof {
    /// Independently replayed native and oracle results match the case baselines.
    ReplayResultDigestEquality {
        /// Independent native replay.
        replay_native: Phase9EvidencePayloadRef,
        /// Independent oracle replay.
        replay_oracle: Phase9EvidencePayloadRef,
    },
    /// Two independently persisted reductions retain the same mismatch signature.
    MinimizedFailureSignaturePreservation {
        /// Minimized failing result.
        minimized: Phase9EvidenceMismatch,
        /// Independently copied failing result.
        copied: Phase9EvidenceMismatch,
    },
    /// Both persisted reductions first diverge at the reviewed semantic path.
    DeliberateFirstDivergence {
        /// Minimized failing result.
        minimized: Phase9EvidenceMismatch,
        /// Independently copied failing result.
        copied: Phase9EvidenceMismatch,
    },
    /// Independently repeated D0 results are byte-identical to the case baselines.
    D0RepeatedResultDigestEquality {
        /// Repeated native result.
        repeated_native: Phase9EvidencePayloadRef,
        /// Repeated oracle result.
        repeated_oracle: Phase9EvidencePayloadRef,
    },
    /// Independent debug and release oracle executions produce identical result bytes.
    DebugReleaseResultDigestEquality {
        /// Independently executed debug-oracle result.
        debug_oracle: Phase9EvidencePayloadRef,
        /// Independently executed release-oracle result.
        release_oracle: Phase9EvidencePayloadRef,
    },
}

impl Phase9CrossRunProof {
    fn semantic_assertion(&self) -> Phase9SemanticAssertion {
        match self {
            Self::ReplayResultDigestEquality { .. } => {
                Phase9SemanticAssertion::ReplayResultDigestEquality
            }
            Self::MinimizedFailureSignaturePreservation { .. } => {
                Phase9SemanticAssertion::MinimizedFailureSignaturePreservation
            }
            Self::DeliberateFirstDivergence { .. } => {
                Phase9SemanticAssertion::DeliberateFirstDivergence
            }
            Self::D0RepeatedResultDigestEquality { .. } => {
                Phase9SemanticAssertion::D0RepeatedResultDigestEquality
            }
            Self::DebugReleaseResultDigestEquality { .. } => {
                Phase9SemanticAssertion::DebugReleaseResultDigestEquality
            }
        }
    }
}

/// One branch-bound proof record persisted in a Phase 9 case manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase9CrossRunProofRecord {
    /// Closed corpus branch established by this proof.
    pub branch_id: ScenarioId,
    /// Digest of the exact case request bytes.
    pub request_sha256: Sha256Hex,
    /// Digest of the exact baseline native result bytes.
    pub native_result_sha256: Sha256Hex,
    /// Digest of the exact baseline oracle result bytes.
    pub oracle_result_sha256: Sha256Hex,
    /// Typed proof payload.
    pub proof: Phase9CrossRunProof,
}

impl Phase9CrossRunProofRecord {
    /// Validates the canonical case-local path topology for all cross-run proof roles.
    ///
    /// The only permitted path reuse maps replay payloads to their corresponding D0 roles and
    /// minimized or copied payloads to their corresponding first-divergence roles. Every other
    /// logical role has one distinct canonical filename below `cases/<case-id>/proofs/`.
    ///
    /// # Errors
    ///
    /// Returns [`Phase9CaseEvidenceError`] when a role is missing or duplicated, a path is
    /// absolute or traversing, a spelling is noncanonical, or a role does not use its exact
    /// case-local filename.
    pub fn validate_topology(
        case_id: &str,
        records: &[Self],
    ) -> Result<(), Phase9CaseEvidenceError> {
        validate_case_id(case_id)?;
        let mut families = BTreeSet::new();
        let mut role_paths = BTreeMap::new();
        for record in records {
            let family = proof_family(&record.proof);
            if !families.insert(family) {
                return Err(case_evidence_error(format!(
                    "duplicate `{family}` proof family"
                )));
            }
            match &record.proof {
                Phase9CrossRunProof::ReplayResultDigestEquality {
                    replay_native,
                    replay_oracle,
                } => {
                    register_role_path(
                        case_id,
                        ProofRole::ReplayNative,
                        replay_native,
                        &mut role_paths,
                    )?;
                    register_role_path(
                        case_id,
                        ProofRole::ReplayOracle,
                        replay_oracle,
                        &mut role_paths,
                    )?;
                }
                Phase9CrossRunProof::MinimizedFailureSignaturePreservation {
                    minimized,
                    copied,
                }
                | Phase9CrossRunProof::DeliberateFirstDivergence { minimized, copied } => {
                    register_role_path(
                        case_id,
                        ProofRole::Minimized,
                        &minimized.result,
                        &mut role_paths,
                    )?;
                    register_role_path(
                        case_id,
                        ProofRole::Copied,
                        &copied.result,
                        &mut role_paths,
                    )?;
                }
                Phase9CrossRunProof::D0RepeatedResultDigestEquality {
                    repeated_native,
                    repeated_oracle,
                } => {
                    register_role_path(
                        case_id,
                        ProofRole::ReplayNative,
                        repeated_native,
                        &mut role_paths,
                    )?;
                    register_role_path(
                        case_id,
                        ProofRole::ReplayOracle,
                        repeated_oracle,
                        &mut role_paths,
                    )?;
                }
                Phase9CrossRunProof::DebugReleaseResultDigestEquality {
                    debug_oracle,
                    release_oracle,
                } => {
                    register_role_path(case_id, ProofRole::Debug, debug_oracle, &mut role_paths)?;
                    register_role_path(
                        case_id,
                        ProofRole::Release,
                        release_oracle,
                        &mut role_paths,
                    )?;
                }
            }
        }
        if families.len() != 5 || role_paths.len() != ProofRole::ALL.len() {
            return Err(case_evidence_error(
                "proof topology must define all five families and six canonical roles",
            ));
        }
        let mut distinct_paths = BTreeSet::new();
        for (role, path) in &role_paths {
            if !distinct_paths.insert(*path) {
                return Err(case_evidence_error(format!(
                    "proof role `{}` aliases an independently persisted role",
                    role.label()
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ProofRole {
    ReplayNative,
    ReplayOracle,
    Debug,
    Release,
    Minimized,
    Copied,
}

impl ProofRole {
    const ALL: [Self; 6] = [
        Self::ReplayNative,
        Self::ReplayOracle,
        Self::Debug,
        Self::Release,
        Self::Minimized,
        Self::Copied,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::ReplayNative => "replay-native",
            Self::ReplayOracle => "replay-oracle",
            Self::Debug => "debug",
            Self::Release => "release",
            Self::Minimized => "minimized",
            Self::Copied => "copied",
        }
    }

    fn filename(self) -> String {
        format!("{}.json", self.label())
    }
}

fn proof_family(proof: &Phase9CrossRunProof) -> &'static str {
    match proof {
        Phase9CrossRunProof::ReplayResultDigestEquality { .. } => "replay",
        Phase9CrossRunProof::MinimizedFailureSignaturePreservation { .. } => "minimization",
        Phase9CrossRunProof::DeliberateFirstDivergence { .. } => "first-divergence",
        Phase9CrossRunProof::D0RepeatedResultDigestEquality { .. } => "d0",
        Phase9CrossRunProof::DebugReleaseResultDigestEquality { .. } => "debug-release",
    }
}

fn validate_case_id(case_id: &str) -> Result<(), Phase9CaseEvidenceError> {
    if case_id.is_empty()
        || case_id.len() > 128
        || !case_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(case_evidence_error(
            "proof topology case ID is empty, unbounded, or noncanonical",
        ));
    }
    Ok(())
}

fn register_role_path<'a>(
    case_id: &str,
    role: ProofRole,
    reference: &'a Phase9EvidencePayloadRef,
    role_paths: &mut BTreeMap<ProofRole, &'a str>,
) -> Result<(), Phase9CaseEvidenceError> {
    let normalized = normalize_logical_path(&reference.path)?;
    let expected = format!("cases/{case_id}/proofs/{}", role.filename());
    if reference.path.as_ref() != normalized || normalized != expected {
        return Err(case_evidence_error(format!(
            "proof role `{}` must use exact canonical path `{expected}`",
            role.label()
        )));
    }
    if let Some(existing) = role_paths.insert(role, reference.path.as_ref())
        && existing != reference.path.as_ref()
    {
        return Err(case_evidence_error(format!(
            "proof role `{}` resolves to inconsistent paths",
            role.label()
        )));
    }
    Ok(())
}

fn normalize_logical_path(path: &str) -> Result<String, Phase9CaseEvidenceError> {
    if path.is_empty() || path.len() > 512 {
        return Err(case_evidence_error("proof payload path is not bounded"));
    }
    let separator_normalized = path.replace('\\', "/");
    let drive_absolute = separator_normalized
        .as_bytes()
        .get(1)
        .is_some_and(|byte| *byte == b':');
    if separator_normalized.starts_with('/') || drive_absolute {
        return Err(case_evidence_error(
            "proof payload path must be evidence-root-relative",
        ));
    }
    let mut components = Vec::new();
    for component in separator_normalized.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                return Err(case_evidence_error(
                    "proof payload path must not contain parent traversal",
                ));
            }
            _ => components.push(component),
        }
    }
    Ok(components.join("/"))
}

/// A typed persisted case proof failed structural or semantic validation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Phase 9 cross-run evidence is invalid: {0}")]
pub struct Phase9CaseEvidenceError(Box<str>);

/// Recomputes every non-observation Phase 9 assertion from persisted result payloads.
///
/// # Errors
///
/// Returns [`Phase9CaseEvidenceError`] unless each case-level witness has exactly one matching
/// proof, every payload digest and result boundary validates, and all five cross-run predicates
/// are recomputed successfully.
#[allow(clippy::too_many_arguments)]
pub fn validate_phase9_cross_run_proofs(
    request: &RigidWorldRequestRecord,
    native: &RigidWorldResultRecord,
    oracle: &RigidWorldResultRecord,
    request_bytes: &[u8],
    native_bytes: &[u8],
    oracle_bytes: &[u8],
    bindings: &[Phase9WitnessBinding],
    records: &[Phase9CrossRunProofRecord],
    payloads: &BTreeMap<String, Vec<u8>>,
    limits: &HarnessLimits,
) -> Result<(), Phase9CaseEvidenceError> {
    let case_bindings = bindings
        .iter()
        .filter(|binding| binding.semantic_assertion.requires_case_evidence())
        .collect::<Vec<_>>();
    if case_bindings.len() != records.len() {
        return Err(case_evidence_error(format!(
            "expected {} proof records, found {}",
            case_bindings.len(),
            records.len()
        )));
    }
    let request_sha256 = digest(request_bytes);
    let native_sha256 = digest(native_bytes);
    let oracle_sha256 = digest(oracle_bytes);
    let mut seen = BTreeSet::new();
    for binding in case_bindings {
        let record = records
            .iter()
            .find(|record| record.branch_id == binding.branch_id)
            .ok_or_else(|| {
                case_evidence_error(format!("missing proof for `{}`", binding.branch_id))
            })?;
        if !seen.insert(record.branch_id.as_str()) {
            return Err(case_evidence_error(format!(
                "duplicate proof for `{}`",
                record.branch_id
            )));
        }
        if record.proof.semantic_assertion() != binding.semantic_assertion {
            return Err(case_evidence_error(format!(
                "proof kind does not match `{}`",
                binding.branch_id
            )));
        }
        if record.request_sha256 != request_sha256
            || record.native_result_sha256 != native_sha256
            || record.oracle_result_sha256 != oracle_sha256
        {
            return Err(case_evidence_error(format!(
                "baseline digest binding differs for `{}`",
                binding.branch_id
            )));
        }
        validate_cross_run_proof(
            request,
            native,
            oracle,
            native_bytes,
            oracle_bytes,
            &record.proof,
            payloads,
            limits,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_cross_run_proof(
    request: &RigidWorldRequestRecord,
    native: &RigidWorldResultRecord,
    oracle: &RigidWorldResultRecord,
    native_bytes: &[u8],
    oracle_bytes: &[u8],
    proof: &Phase9CrossRunProof,
    payloads: &BTreeMap<String, Vec<u8>>,
    limits: &HarnessLimits,
) -> Result<(), Phase9CaseEvidenceError> {
    match proof {
        Phase9CrossRunProof::ReplayResultDigestEquality {
            replay_native,
            replay_oracle,
        } => {
            let replay_native_bytes = payload(replay_native, payloads)?;
            let replay_oracle_bytes = payload(replay_oracle, payloads)?;
            decode_result(request, replay_native_bytes, limits)?;
            decode_result(request, replay_oracle_bytes, limits)?;
            require_digest_equality(replay_native_bytes, native_bytes, "native replay")?;
            require_digest_equality(replay_oracle_bytes, oracle_bytes, "oracle replay")
        }
        Phase9CrossRunProof::D0RepeatedResultDigestEquality {
            repeated_native,
            repeated_oracle,
        } => {
            let repeated_native_bytes = payload(repeated_native, payloads)?;
            let repeated_oracle_bytes = payload(repeated_oracle, payloads)?;
            decode_result(request, repeated_native_bytes, limits)?;
            decode_result(request, repeated_oracle_bytes, limits)?;
            if repeated_native_bytes != native_bytes || repeated_oracle_bytes != oracle_bytes {
                return Err(case_evidence_error(
                    "D0 repeated result bytes differ from the case baselines",
                ));
            }
            Ok(())
        }
        Phase9CrossRunProof::DebugReleaseResultDigestEquality {
            debug_oracle,
            release_oracle,
        } => {
            let debug_bytes = payload(debug_oracle, payloads)?;
            let release_bytes = payload(release_oracle, payloads)?;
            decode_result(request, debug_bytes, limits)?;
            decode_result(request, release_bytes, limits)?;
            if debug_bytes != release_bytes {
                return Err(case_evidence_error(
                    "debug and release oracle result bytes differ",
                ));
            }
            Ok(())
        }
        Phase9CrossRunProof::MinimizedFailureSignaturePreservation { minimized, copied } => {
            let minimized_report =
                validate_mismatch(request, native, oracle, minimized, payloads, limits)?;
            let copied_report =
                validate_mismatch(request, native, oracle, copied, payloads, limits)?;
            if minimized_report != copied_report {
                return Err(case_evidence_error(
                    "minimized and copied mismatch signatures differ",
                ));
            }
            Ok(())
        }
        Phase9CrossRunProof::DeliberateFirstDivergence { minimized, copied } => {
            let minimized_report =
                validate_mismatch(request, native, oracle, minimized, payloads, limits)?;
            let copied_report =
                validate_mismatch(request, native, oracle, copied, payloads, limits)?;
            if minimized_report != copied_report
                || minimized_report.1.as_ref() != "rigid_world.body.active"
            {
                return Err(case_evidence_error(
                    "deliberate mutation did not preserve the reviewed first divergence",
                ));
            }
            Ok(())
        }
    }
}

fn validate_mismatch(
    request: &RigidWorldRequestRecord,
    native: &RigidWorldResultRecord,
    _oracle: &RigidWorldResultRecord,
    mismatch: &Phase9EvidenceMismatch,
    payloads: &BTreeMap<String, Vec<u8>>,
    limits: &HarnessLimits,
) -> Result<(Sha256Hex, Box<str>), Phase9CaseEvidenceError> {
    let bytes = payload(&mismatch.result, payloads)?;
    let mutated = decode_result(request, bytes, limits)?;
    let outcome = compare_complete_phase9_rigid_world_results(request, native, &mutated)
        .map_err(|error| case_evidence_error(error.to_string()))?;
    let Phase9ComparisonOutcome::RetainedRigidMismatch(report) = outcome else {
        return Err(case_evidence_error(
            "persisted mutation did not produce a retained-rigid mismatch",
        ));
    };
    let actual = (
        report.signature().signature_sha256().clone(),
        report.semantic_path().into(),
    );
    if actual.0 != mismatch.signature_sha256 || actual.1 != mismatch.semantic_path {
        return Err(case_evidence_error(
            "persisted mismatch identity differs from recomputed comparison",
        ));
    }
    Ok(actual)
}

fn decode_result(
    request: &RigidWorldRequestRecord,
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<RigidWorldResultRecord, Phase9CaseEvidenceError> {
    let mut jsonl = bytes.to_vec();
    if !jsonl.ends_with(b"\n") {
        jsonl.push(b'\n');
    }
    let result = decode_rigid_world_result_jsonl(&jsonl, limits)
        .map_err(|error| case_evidence_error(error.to_string()))?;
    liquidfun_test_protocol::validate_rigid_world_result_against_request(request, &result)
        .map_err(|error| case_evidence_error(error.to_string()))?;
    Ok(result)
}

fn payload<'a>(
    reference: &Phase9EvidencePayloadRef,
    payloads: &'a BTreeMap<String, Vec<u8>>,
) -> Result<&'a [u8], Phase9CaseEvidenceError> {
    if reference.path.is_empty() || reference.path.len() > 512 {
        return Err(case_evidence_error("proof payload path is not bounded"));
    }
    let bytes = payloads.get(reference.path.as_ref()).ok_or_else(|| {
        case_evidence_error(format!("missing proof payload `{}`", reference.path))
    })?;
    if digest(bytes) != reference.sha256 {
        return Err(case_evidence_error(format!(
            "proof payload digest differs for `{}`",
            reference.path
        )));
    }
    Ok(bytes)
}

fn require_digest_equality(
    left: &[u8],
    right: &[u8],
    label: &str,
) -> Result<(), Phase9CaseEvidenceError> {
    if digest(left) != digest(right) {
        return Err(case_evidence_error(format!(
            "{label} result digest differs from its case baseline"
        )));
    }
    Ok(())
}

fn digest(bytes: &[u8]) -> Sha256Hex {
    Sha256Hex::from_digest(Sha256::digest(bytes).into())
}

fn case_evidence_error(message: impl Into<Box<str>>) -> Phase9CaseEvidenceError {
    Phase9CaseEvidenceError(message.into())
}

/// Resolves and evaluates every closed Phase 9 witness against one decoded result.
///
/// Each binding must name the reviewed action for its branch, place that action inside the
/// selected checkpoint interval, resolve to the corresponding particle observation ordinal,
/// and satisfy its semantic assertion against the decoded request and result.
///
/// # Errors
///
/// Returns [`Phase9EvidenceBindingError`] when any indexed binding, observation variant, action,
/// or semantic value differs from the reviewed Phase 9 corpus contract.
pub fn validate_phase9_evidence_bindings(
    request: &RigidWorldRequestRecord,
    result: &RigidWorldResultRecord,
    bindings: &[Phase9WitnessBinding],
) -> Result<(), Phase9EvidenceBindingError> {
    if bindings
        .iter()
        .any(|binding| binding.branch_id.as_str() == "retained_phase6_through_phase8")
        && request.scenario().timelines().len() != RigidWorldWitnessFamily::ALL.len()
    {
        return Err(unbound_error(
            "retained Phase 6 through Phase 8 timelines are incomplete",
        ));
    }
    let timeline = request
        .scenario()
        .timelines()
        .first()
        .ok_or_else(|| unbound_error("missing Phase 9 request timeline"))?;
    let result_timeline = result
        .timelines()
        .first()
        .ok_or_else(|| unbound_error("missing Phase 9 result timeline"))?;
    for binding in bindings {
        if binding.semantic_assertion.requires_case_evidence() {
            validate_action_checkpoint(timeline, binding)?;
            continue;
        }
        let observation = resolve_observation(timeline, result_timeline, binding)?;
        if observation.witness_kind() != binding.observation_kind {
            return Err(binding_error(
                binding,
                format!(
                    "expected {:?} observation, resolved {:?}",
                    binding.observation_kind,
                    observation.witness_kind()
                ),
            ));
        }
        evaluate_assertion(timeline, result_timeline, binding, observation)?;
    }
    Ok(())
}

fn resolve_observation<'a>(
    timeline: &'a RigidWorldTimeline,
    result: &'a RigidWorldTimelineResult,
    binding: &Phase9WitnessBinding,
) -> Result<&'a Phase9ParticleObservation, Phase9EvidenceBindingError> {
    let action_start = validate_action_checkpoint(timeline, binding)?;
    let particle_ordinal = timeline.actions()[action_start..binding.action_index]
        .iter()
        .filter(|candidate| matches!(candidate.action(), RigidWorldAction::Particle { .. }))
        .count();
    let result_checkpoint = result
        .checkpoints
        .get(binding.checkpoint_index)
        .ok_or_else(|| binding_error(binding, "result checkpoint is absent"))?;
    result_checkpoint
        .observations
        .iter()
        .filter_map(|candidate| match candidate {
            RigidWorldObservation::Particle { observation } => Some(observation),
            _ => None,
        })
        .nth(particle_ordinal)
        .ok_or_else(|| binding_error(binding, "bound particle observation is absent"))
}

fn validate_action_checkpoint(
    timeline: &RigidWorldTimeline,
    binding: &Phase9WitnessBinding,
) -> Result<usize, Phase9EvidenceBindingError> {
    let action = timeline
        .actions()
        .get(binding.action_index)
        .ok_or_else(|| binding_error(binding, "action index is out of range"))?;
    let expected_action_id = expected_action_id(binding.branch_id.as_str());
    if action.action_id().as_str() != expected_action_id {
        return Err(binding_error(
            binding,
            format!(
                "expected action `{expected_action_id}`, found `{}`",
                action.action_id()
            ),
        ));
    }
    if !matches!(action.action(), RigidWorldAction::Particle { .. }) {
        return Err(binding_error(
            binding,
            "bound action is not a Phase 9 particle action",
        ));
    }
    let checkpoint = timeline
        .checkpoints()
        .get(binding.checkpoint_index)
        .ok_or_else(|| binding_error(binding, "checkpoint index is out of range"))?;
    let action_end = timeline
        .actions()
        .iter()
        .position(|candidate| candidate.action_id() == checkpoint.after_action_id())
        .ok_or_else(|| binding_error(binding, "checkpoint terminator action is absent"))?;
    let action_start = if binding.checkpoint_index == 0 {
        0
    } else {
        let previous = &timeline.checkpoints()[binding.checkpoint_index - 1];
        timeline
            .actions()
            .iter()
            .position(|candidate| candidate.action_id() == previous.after_action_id())
            .ok_or_else(|| binding_error(binding, "previous checkpoint terminator is absent"))?
            + 1
    };
    if !(action_start..=action_end).contains(&binding.action_index) {
        return Err(binding_error(
            binding,
            "bound action does not belong to the selected checkpoint",
        ));
    }
    Ok(action_start)
}

fn evaluate_assertion(
    timeline: &RigidWorldTimeline,
    result: &RigidWorldTimelineResult,
    binding: &Phase9WitnessBinding,
    observation: &Phase9ParticleObservation,
) -> Result<(), Phase9EvidenceBindingError> {
    let satisfied = match &binding.semantic_assertion {
        Phase9SemanticAssertion::ObservedSemantic { branch_id } => {
            evaluate_observed_semantic(timeline, result, branch_id.as_str(), observation)
        }
        Phase9SemanticAssertion::FiniteLifetimeExpired { particle_id } => {
            matches!(
                observation,
                Phase9ParticleObservation::System { particle_ids, .. }
                    if !particle_ids.contains(particle_id)
            )
        }
        Phase9SemanticAssertion::InfiniteLifetimeSurvives { particle_id } => {
            matches!(
                observation,
                Phase9ParticleObservation::System { particle_ids, .. }
                    if particle_ids.contains(particle_id)
            )
        }
        Phase9SemanticAssertion::EqualExpirationOrder { particle_ids } => {
            equal_lifetime_is_declared(timeline, particle_ids)
                && matches!(
                    observation,
                    Phase9ParticleObservation::Lifecycle { occurrence }
                        if occurrence.kind == Phase9OccurrenceKind::ParticleDestroyed
                            && occurrence.maybe_particle_id.as_ref() == particle_ids.last()
                )
        }
        Phase9SemanticAssertion::StrictContactCardinality {
            enabled,
            contact_count,
        } => matches!(
            observation,
            Phase9ParticleObservation::Statistics { statistics }
                if statistics.maybe_system_id.as_ref().is_some_and(|system_id| {
                    system_declaration(timeline, system_id)
                        .is_some_and(|declaration| declaration.strict_contact_check == *enabled)
                }) && statistics.body_contact_count == *contact_count
        ),
        Phase9SemanticAssertion::ListenerEventEffect {
            enabled,
            event_count,
        } => listener_effect_matches(result, observation, *enabled, *event_count),
        Phase9SemanticAssertion::FilterContactEffect {
            enabled,
            contact_count,
        } => {
            let expected_system = if *enabled {
                "phase9-growable"
            } else {
                "phase9-fixed-paused"
            };
            matches!(
                observation,
                Phase9ParticleObservation::Statistics { statistics }
                    if statistics.maybe_system_id.as_ref().map(ScenarioId::as_str)
                        == Some(expected_system)
                        && statistics.particle_contact_count == *contact_count
            )
        }
        Phase9SemanticAssertion::CollisionEnergyPositiveFinite { minimum_bits } => {
            matches!(
                observation,
                Phase9ParticleObservation::Statistics { statistics }
                    if {
                        let energy = statistics.collision_energy_bits.to_f32();
                        energy.is_finite() && energy > 0.0 && energy >= minimum_bits.to_f32()
                    }
            )
        }
        Phase9SemanticAssertion::StuckCandidatesNonempty { particle_ids } => {
            matches!(
                observation,
                Phase9ParticleObservation::Statistics { statistics }
                    if !statistics.stuck_particle_ids.is_empty()
                        && particle_ids.iter().all(|particle_id| {
                            statistics.stuck_particle_ids.contains(particle_id)
                        })
            )
        }
        Phase9SemanticAssertion::ReplayResultDigestEquality
        | Phase9SemanticAssertion::MinimizedFailureSignaturePreservation
        | Phase9SemanticAssertion::DeliberateFirstDivergence
        | Phase9SemanticAssertion::D0RepeatedResultDigestEquality
        | Phase9SemanticAssertion::DebugReleaseResultDigestEquality => false,
    };
    if !satisfied {
        return Err(binding_error(
            binding,
            "resolved observation does not satisfy its semantic assertion",
        ));
    }
    Ok(())
}

fn evaluate_observed_semantic(
    timeline: &RigidWorldTimeline,
    result: &RigidWorldTimelineResult,
    branch: &str,
    observation: &Phase9ParticleObservation,
) -> bool {
    match branch {
        "multiple_systems" | "newest_first" => {
            timeline.particle_systems().len() == 2
                && matches!(
                    observation,
                    Phase9ParticleObservation::System { system_id, .. }
                        if system_id.as_str() == "phase9-growable"
                )
        }
        "paused_system" => {
            statistics_for_system(observation, "phase9-fixed-paused").is_some_and(|statistics| {
                system_declaration_by_name(timeline, "phase9-fixed-paused")
                    .is_some_and(|declaration| declaration.paused)
                    && statistics.particle_count == 2
            })
        }
        "stable_ids_sort" => matches!(
            observation,
            Phase9ParticleObservation::System { particle_ids, .. }
                if ids_equal(
                    particle_ids,
                    &["phase9-a", "phase9-b", "phase9-coupling", "phase9-capacity"],
                )
        ),
        "stable_ids_compact" => matches!(
            observation,
            Phase9ParticleObservation::MixedState { particle_ids, .. }
                if ids_equal(
                    particle_ids,
                    &["phase9-coupling", "phase9-evicting", "phase9-c", "phase9-e"],
                )
        ),
        "optional_lanes" => matches!(
            observation,
            Phase9ParticleObservation::Particle { snapshot }
                if snapshot.particle_id.as_str() == "phase9-a"
                    && snapshot.color == [0, 0, 255, 255]
                    && snapshot.weight_bits.to_f32() == 0.0
                    && snapshot.force.x_bits.to_f32() == 0.0
                    && snapshot.force.y_bits.to_f32() == 0.0
        ),
        "fixed_buffer" => {
            statistics_for_system(observation, "phase9-fixed-paused").is_some_and(|statistics| {
                system_declaration_by_name(timeline, "phase9-fixed-paused").is_some_and(
                    |declaration| {
                        declaration.buffer_mode == Phase9ParticleBufferMode::Fixed { capacity: 2 }
                            && statistics.declared_capacity == 2
                    },
                )
            })
        }
        "growable_buffer" => {
            statistics_for_system(observation, "phase9-growable").is_some_and(|statistics| {
                system_declaration_by_name(timeline, "phase9-growable").is_some_and(|declaration| {
                    declaration.buffer_mode
                        == Phase9ParticleBufferMode::Growable {
                            initial_capacity: 4,
                        }
                        && statistics.declared_capacity == 4
                })
            })
        }
        "fixed_full" => {
            statistics_for_system(observation, "phase9-fixed-paused").is_some_and(|statistics| {
                statistics.particle_count == 2 && statistics.effective_capacity == 2
            })
        }
        "teardown" => lifecycle_matches(
            observation,
            Phase9OccurrenceKind::SystemDestroyed,
            "phase9-fixed-paused",
            None,
        ),
        "oldest_lifetime" | "capacity_eviction" => lifecycle_matches(
            observation,
            Phase9OccurrenceKind::ParticleDestroyed,
            "phase9-growable",
            Some("phase9-a"),
        ),
        "maximum_lifetime" => {
            statistics_for_system(observation, "phase9-growable").is_some_and(|statistics| {
                system_declaration_by_name(timeline, "phase9-growable")
                    .is_some_and(|declaration| declaration.maximum_count == Some(4))
                    && statistics.effective_capacity == 4
            })
        }
        "requested_destruction_callback" => lifecycle_matches(
            observation,
            Phase9OccurrenceKind::ParticleDestroyed,
            "phase9-growable",
            Some("phase9-b"),
        ),
        "unrequested_destruction_callback" => {
            matches!(observation, Phase9ParticleObservation::MixedState { .. })
                && !checkpoint_has_particle_lifecycle(result, "phase9-capacity")
        }
        "zombie_pending" => matches!(
            observation,
            Phase9ParticleObservation::MixedState { particle_ids, .. }
                if particle_ids.iter().any(|id| id.as_str() == "phase9-b")
        ),
        "particle_contact" => matches!(
            observation,
            Phase9ParticleObservation::ParticleContact { contact }
                if contact.system_id.as_str() == "phase9-fixed-paused"
        ),
        "body_contact" => matches!(
            observation,
            Phase9ParticleObservation::BodyContact { contact }
                if contact.fixture_id.as_str() == "nc-kinematic-fixture"
        ),
        "contact_order" => matches!(
            observation,
            Phase9ParticleObservation::ParticleContact { contact }
                if contact.particle_a_id.as_str() == "phase9-c"
                    && contact.particle_b_id.as_str() == "phase9-d"
        ),
        "contact_multiplicity" => matches!(
            observation,
            Phase9ParticleObservation::ParticleContact { contact }
                if contact.particle_a_id != contact.particle_b_id
                    && contact.weight_bits.to_f32() > 0.0
        ),
        "coupling_fields" => matches!(
            observation,
            Phase9ParticleObservation::BodyContact { contact }
                if contact.particle_id.as_str() == "phase9-coupling"
                    && contact.mass_bits.to_f32() > 0.0
                    && contact.weight_bits.to_f32() > 0.0
        ),
        "dynamic_body_reaction" => {
            statistics_for_system(observation, "phase9-growable").is_some()
                && result.checkpoints.iter().any(|checkpoint| {
                    checkpoint.bodies.iter().any(|body| {
                        body.body_id.as_str() == "nc-dynamic"
                            && (body.linear_velocity.x_bits.to_f32() != 0.0
                                || body.linear_velocity.y_bits.to_f32() != 0.0)
                    })
                })
        }
        "static_body_no_reaction" => {
            statistics_for_system(observation, "phase9-growable").is_some()
                && result.checkpoints.iter().any(|checkpoint| {
                    checkpoint.bodies.iter().any(|body| {
                        body.body_id.as_str() == "nc-static"
                            && body.body_kind == RigidBodyKind::Static
                            && body.linear_velocity.x_bits.to_f32() == 0.0
                            && body.linear_velocity.y_bits.to_f32() == 0.0
                    })
                })
        }
        "force_range" => matches!(
            observation,
            Phase9ParticleObservation::Particle { snapshot }
                if snapshot.particle_id.as_str() == "phase9-a"
                    && snapshot.force.x_bits.to_f32() != 0.0
        ),
        "impulse_range" => matches!(
            observation,
            Phase9ParticleObservation::Particle { snapshot }
                if snapshot.particle_id.as_str() == "phase9-a"
                    && snapshot.velocity.y_bits.to_f32() != 0.0
        ),
        "statistics_counts" => {
            statistics_for_system(observation, "phase9-growable").is_some_and(|statistics| {
                statistics.system_count == 2 && statistics.particle_count == 4
            })
        }
        "system_aabb" | "system_culling" | "query_continue" => {
            query_matches(observation, false, &["phase9-a", "phase9-b"])
        }
        "world_aabb" => query_matches(
            observation,
            false,
            &["phase9-c", "phase9-d", "phase9-a", "phase9-b"],
        ),
        "query_terminate" => query_matches(observation, true, &["phase9-a"]),
        "system_ray" | "ray_culling" | "ray_start_inside_exclusion" | "ray_continue" => {
            ray_matches(observation, false, &["phase9-a", "phase9-b"], true)
        }
        "world_ray" => ray_matches(
            observation,
            false,
            &["phase9-c", "phase9-d", "phase9-a", "phase9-b"],
            true,
        ),
        "ray_ignore" => ray_matches(observation, false, &["phase9-a", "phase9-b"], false),
        "ray_clip" => ray_matches(observation, false, &["phase9-a"], true),
        "ray_terminate" => ray_matches(observation, true, &["phase9-a"], true),
        "retained_phase6_through_phase8" => {
            matches!(
                observation,
                Phase9ParticleObservation::Particle { snapshot }
                    if snapshot.particle_id.as_str() == "phase9-a"
            )
        }
        "phase10_rejection" => matches!(
            observation,
            Phase9ParticleObservation::Particle { snapshot }
                if snapshot.particle_id.as_str() == "phase9-a"
        ),
        "closed_policy_registry" => {
            PHASE9_REQUIRED_POLICY_PATHS.len() == 22
                && matches!(
                    observation,
                    Phase9ParticleObservation::Particle { snapshot }
                        if snapshot.particle_id.as_str() == "phase9-a"
                )
        }
        _ => false,
    }
}

fn expected_action_id(branch: &str) -> &'static str {
    match branch {
        "multiple_systems" | "newest_first" | "stable_ids_sort" => "inspect-system",
        "paused_system" | "fixed_buffer" | "fixed_full" => "statistics-fixed",
        "stable_ids_compact" => "compact-unrequested",
        "optional_lanes"
        | "retained_phase6_through_phase8"
        | "phase10_rejection"
        | "closed_policy_registry"
        | "replay_identity"
        | "minimization_identity"
        | "first_divergence_stability"
        | "d0_byte_identity"
        | "debug_release_agreement" => "inspect-particle",
        "growable_buffer" | "maximum_lifetime" | "statistics_counts" => "statistics",
        "teardown" => "destroy-fixed",
        "oldest_lifetime" | "capacity_eviction" => "create-evicting",
        "finite_lifetime" | "infinite_lifetime" => "inspect-system-after-step",
        "equal_lifetime" => "create-phase9-e",
        "requested_destruction_callback" => "compact",
        "unrequested_destruction_callback" => "compact-unrequested",
        "zombie_pending" => "mark",
        "particle_contact" | "contact_order" | "contact_multiplicity" => "inspect-particle-contact",
        "body_contact" | "coupling_fields" => "inspect-body-contact",
        "strict_contact_enabled" => "statistics-fixed",
        "strict_contact_disabled"
        | "dynamic_body_reaction"
        | "static_body_no_reaction"
        | "stuck_candidates" => "statistics",
        "listener_flag_enabled" => "inspect-occurrence-zero",
        "listener_flag_disabled" | "filter_flag_enabled" => "contact-statistics-growable",
        "filter_flag_disabled" => "contact-statistics-fixed",
        "force_range" => "inspect-after-force",
        "impulse_range" => "inspect-after-impulse",
        "collision_energy" => "statistics-fixed",
        "system_aabb" | "system_culling" | "query_continue" => "system-query",
        "world_aabb" => "world-query",
        "query_terminate" => "query-terminate",
        "system_ray" | "ray_culling" | "ray_start_inside_exclusion" | "ray_continue" => {
            "system-ray"
        }
        "world_ray" => "world-ray",
        "ray_ignore" => "ray-ignore",
        "ray_clip" => "ray-clip",
        "ray_terminate" => "ray-terminate",
        _ => "",
    }
}

fn listener_effect_matches(
    result: &RigidWorldTimelineResult,
    observation: &Phase9ParticleObservation,
    enabled: bool,
    expected_count: u32,
) -> bool {
    let occurrences = result
        .checkpoints
        .iter()
        .flat_map(|checkpoint| checkpoint.observations.iter())
        .filter_map(|candidate| match candidate {
            RigidWorldObservation::Particle {
                observation: Phase9ParticleObservation::Lifecycle { occurrence },
            } if occurrence.kind == Phase9OccurrenceKind::ContactCreated => Some(occurrence),
            _ => None,
        })
        .filter(|occurrence| {
            enabled
                || occurrence
                    .maybe_particle_id
                    .as_ref()
                    .map(ScenarioId::as_str)
                    == Some("phase9-capacity")
                || occurrence
                    .maybe_other_particle_id
                    .as_ref()
                    .map(ScenarioId::as_str)
                    == Some("phase9-capacity")
        })
        .count();
    u32::try_from(occurrences).ok() == Some(expected_count)
        && if enabled {
            matches!(
                observation,
                Phase9ParticleObservation::Lifecycle { occurrence }
                    if occurrence.kind == Phase9OccurrenceKind::ContactCreated
            )
        } else {
            statistics_for_system(observation, "phase9-growable").is_some()
        }
}

fn equal_lifetime_is_declared(timeline: &RigidWorldTimeline, ids: &[ScenarioId]) -> bool {
    let Some(first) = ids
        .first()
        .and_then(|id| particle_declaration(timeline, id))
    else {
        return false;
    };
    ids.len() >= 2
        && ids.iter().skip(1).all(|id| {
            particle_declaration(timeline, id)
                .is_some_and(|particle| particle.lifetime_bits == first.lifetime_bits)
        })
}

fn system_declaration<'a>(
    timeline: &'a RigidWorldTimeline,
    system_id: &ScenarioId,
) -> Option<&'a liquidfun_test_protocol::Phase9ParticleSystemDeclaration> {
    timeline
        .particle_systems()
        .iter()
        .find(|declaration| declaration.system_id == *system_id)
}

fn system_declaration_by_name<'a>(
    timeline: &'a RigidWorldTimeline,
    system_id: &str,
) -> Option<&'a liquidfun_test_protocol::Phase9ParticleSystemDeclaration> {
    timeline
        .particle_systems()
        .iter()
        .find(|declaration| declaration.system_id.as_str() == system_id)
}

fn particle_declaration<'a>(
    timeline: &'a RigidWorldTimeline,
    particle_id: &ScenarioId,
) -> Option<&'a liquidfun_test_protocol::Phase9ParticleDeclaration> {
    timeline
        .particles()
        .iter()
        .find(|declaration| declaration.particle_id == *particle_id)
}

fn statistics_for_system<'a>(
    observation: &'a Phase9ParticleObservation,
    system_id: &str,
) -> Option<&'a liquidfun_test_protocol::Phase9StatisticsObservation> {
    let Phase9ParticleObservation::Statistics { statistics } = observation else {
        return None;
    };
    (statistics.maybe_system_id.as_ref().map(ScenarioId::as_str) == Some(system_id))
        .then_some(statistics)
}

fn lifecycle_matches(
    observation: &Phase9ParticleObservation,
    kind: Phase9OccurrenceKind,
    system_id: &str,
    maybe_particle_id: Option<&str>,
) -> bool {
    matches!(
        observation,
        Phase9ParticleObservation::Lifecycle { occurrence }
            if occurrence.kind == kind
                && occurrence.system_id.as_str() == system_id
                && occurrence.maybe_particle_id.as_ref().map(ScenarioId::as_str)
                    == maybe_particle_id
    )
}

fn checkpoint_has_particle_lifecycle(result: &RigidWorldTimelineResult, particle_id: &str) -> bool {
    result
        .checkpoints
        .iter()
        .flat_map(|checkpoint| checkpoint.observations.iter())
        .any(|candidate| {
            matches!(
                candidate,
                RigidWorldObservation::Particle {
                    observation: Phase9ParticleObservation::Lifecycle { occurrence },
                } if occurrence.maybe_particle_id.as_ref().map(ScenarioId::as_str)
                    == Some(particle_id)
            )
        })
}

fn query_matches(
    observation: &Phase9ParticleObservation,
    terminated: bool,
    expected_ids: &[&str],
) -> bool {
    matches!(
        observation,
        Phase9ParticleObservation::Query {
            terminated: actual,
            particle_ids,
        } if *actual == terminated && ids_equal(particle_ids, expected_ids)
    )
}

fn ray_matches(
    observation: &Phase9ParticleObservation,
    terminated: bool,
    expected_ids: &[&str],
    require_nonzero_fractions: bool,
) -> bool {
    matches!(
        observation,
        Phase9ParticleObservation::RayCast {
            terminated: actual,
            particle_ids,
            fractions_bits,
        } if *actual == terminated
            && ids_equal(particle_ids, expected_ids)
            && particle_ids.len() == fractions_bits.len()
            && (!require_nonzero_fractions
                || fractions_bits.iter().all(|bits| bits.to_f32() > 0.0))
    )
}

fn ids_equal(actual: &[ScenarioId], expected: &[&str]) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected)
            .all(|(actual, expected)| actual.as_str() == *expected)
}

fn binding_error(
    binding: &Phase9WitnessBinding,
    message: impl Into<String>,
) -> Phase9EvidenceBindingError {
    Phase9EvidenceBindingError {
        branch_id: binding.branch_id.as_str().into(),
        message: message.into().into_boxed_str(),
    }
}

fn unbound_error(message: impl Into<String>) -> Phase9EvidenceBindingError {
    Phase9EvidenceBindingError {
        branch_id: "<registry>".into(),
        message: message.into().into_boxed_str(),
    }
}

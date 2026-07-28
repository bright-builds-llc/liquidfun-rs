//! Persisted cross-run Phase 9 proof validation.

use super::{
    BTreeMap, BTreeSet, Deserialize, Digest, HarnessLimits, Phase9ComparisonOutcome,
    Phase9SemanticAssertion, Phase9WitnessBinding, RigidWorldRequestRecord, RigidWorldResultRecord,
    ScenarioId, Serialize, Sha256, Sha256Hex, compare_complete_phase9_rigid_world_results,
    decode_rigid_world_result_jsonl,
};

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
        if records.is_empty() {
            return Ok(());
        }
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
    case_id: &str,
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
    Phase9CrossRunProofRecord::validate_topology(case_id, records)?;
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

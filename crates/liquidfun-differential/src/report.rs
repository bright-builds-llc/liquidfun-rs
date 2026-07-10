use std::fmt::Write;

use liquidfun_test_protocol::{
    CheckpointId, FloatBits, HarnessFailure, RequestId, Sha256Hex, ToleranceProfile, ValidatedTrace,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const MAXIMUM_PHASE_BYTES: usize = 4 * 1024;

/// Stable broad category of a semantic mismatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MismatchKind {
    /// An expected semantic value was absent.
    Missing,
    /// An unrequested semantic value was present.
    Unexpected,
    /// An exact discrete value differed.
    Exact,
    /// A floating value violated its field policy.
    Numeric,
    /// Solver-significant order differed.
    Order,
    /// An explicitly unordered multiset had different multiplicity.
    Multiplicity,
}

/// Error returned when a report phase name violates validated trace bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("phase name must be nonempty and no longer than 4096 bytes")]
pub struct PhaseNameError;

/// Validated stable phase name carried by a failure signature.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct PhaseName(Box<str>);

impl PhaseName {
    /// Validates an owned phase name.
    ///
    /// # Errors
    ///
    /// Returns [`PhaseNameError`] for empty or oversized input.
    pub fn new(value: impl Into<String>) -> Result<Self, PhaseNameError> {
        let value = value.into();
        if value.is_empty() || value.len() > MAXIMUM_PHASE_BYTES {
            return Err(PhaseNameError);
        }
        Ok(Self(value.into_boxed_str()))
    }

    /// Returns the validated phase spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Exact world-count field selected by deterministic comparison traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorldCountField {
    /// Rigid-body count.
    Bodies,
    /// Fixture count.
    Fixtures,
    /// Joint count.
    Joints,
    /// Contact count.
    Contacts,
    /// Particle-system count.
    ParticleSystems,
    /// Particle-group count.
    ParticleGroups,
    /// Particle count.
    Particles,
}

/// Closed typed semantic paths supported by the Phase-2 trace schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPath {
    /// Stable checkpoint identity.
    CheckpointId,
    /// Ordered checkpoint ordinal.
    CheckpointOrdinal,
    /// Named trace phase.
    Phase,
    /// One exact world-count field.
    WorldCount(WorldCountField),
    /// Exact-bit accumulated simulation time.
    SimulationTime,
    /// Presence of a whole checkpoint.
    CheckpointPresence,
}

impl SemanticPath {
    fn human_name(self) -> &'static str {
        match self {
            Self::CheckpointId => "checkpoint_id",
            Self::CheckpointOrdinal => "checkpoint_ordinal",
            Self::Phase => "phase",
            Self::WorldCount(WorldCountField::Bodies) => "world_counts.bodies",
            Self::WorldCount(WorldCountField::Fixtures) => "world_counts.fixtures",
            Self::WorldCount(WorldCountField::Joints) => "world_counts.joints",
            Self::WorldCount(WorldCountField::Contacts) => "world_counts.contacts",
            Self::WorldCount(WorldCountField::ParticleSystems) => "world_counts.particle_systems",
            Self::WorldCount(WorldCountField::ParticleGroups) => "world_counts.particle_groups",
            Self::WorldCount(WorldCountField::Particles) => "world_counts.particles",
            Self::SimulationTime => "simulation_time",
            Self::CheckpointPresence => "checkpoint_presence",
        }
    }
}

/// Stable first-divergence identity used by replay and reduction.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct FailureSignature {
    checkpoint_id: CheckpointId,
    phase: PhaseName,
    semantic_path: SemanticPath,
    kind: MismatchKind,
}

impl FailureSignature {
    /// Creates one complete typed failure identity.
    #[must_use]
    pub const fn new(
        checkpoint_id: CheckpointId,
        phase: PhaseName,
        semantic_path: SemanticPath,
        kind: MismatchKind,
    ) -> Self {
        Self {
            checkpoint_id,
            phase,
            semantic_path,
            kind,
        }
    }

    /// Returns the first divergent checkpoint identity.
    #[must_use]
    pub const fn checkpoint_id(&self) -> &CheckpointId {
        &self.checkpoint_id
    }

    /// Returns the first divergent named phase.
    #[must_use]
    pub const fn phase(&self) -> &PhaseName {
        &self.phase
    }

    /// Returns the deterministic typed semantic path.
    #[must_use]
    pub const fn semantic_path(&self) -> SemanticPath {
        self.semantic_path
    }

    /// Returns the mismatch category.
    #[must_use]
    pub const fn kind(&self) -> MismatchKind {
        self.kind
    }

    /// Returns an otherwise identical signature with a different mismatch kind.
    #[must_use]
    pub fn with_kind(mut self, kind: MismatchKind) -> Self {
        self.kind = kind;
        self
    }
}

/// Exact authoritative bits plus derived, non-authoritative decimal diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FloatMismatchEvidence {
    expected_bits: FloatBits,
    actual_bits: FloatBits,
    expected_decimal: Box<str>,
    actual_decimal: Box<str>,
}

impl FloatMismatchEvidence {
    fn new(expected_bits: FloatBits, actual_bits: FloatBits) -> Self {
        Self {
            expected_bits,
            actual_bits,
            expected_decimal: expected_bits.to_f32().to_string().into_boxed_str(),
            actual_decimal: actual_bits.to_f32().to_string().into_boxed_str(),
        }
    }

    /// Returns authoritative expected bits.
    #[must_use]
    pub const fn expected_bits(&self) -> FloatBits {
        self.expected_bits
    }

    /// Returns authoritative actual bits.
    #[must_use]
    pub const fn actual_bits(&self) -> FloatBits {
        self.actual_bits
    }

    /// Returns the derived expected decimal diagnostic.
    #[must_use]
    pub fn expected_decimal(&self) -> &str {
        &self.expected_decimal
    }

    /// Returns the derived actual decimal diagnostic.
    #[must_use]
    pub fn actual_decimal(&self) -> &str {
        &self.actual_decimal
    }
}

/// Machine-readable first divergence and bounded neighboring context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MismatchReport {
    signature: FailureSignature,
    checkpoint_ordinal: u32,
    maybe_previous_checkpoint_id: Option<CheckpointId>,
    maybe_next_checkpoint_id: Option<CheckpointId>,
    request_id: RequestId,
    request_sha256: Sha256Hex,
    scenario_sha256: Sha256Hex,
    policy_id: Box<str>,
    policy_sha256: Sha256Hex,
    #[serde(rename = "float_evidence")]
    maybe_float_evidence: Option<FloatMismatchEvidence>,
}

impl MismatchReport {
    pub(crate) fn discrete(
        trace: &ValidatedTrace,
        checkpoint_index: usize,
        semantic_path: SemanticPath,
        kind: MismatchKind,
        policy: &ToleranceProfile,
    ) -> Self {
        Self::new(trace, checkpoint_index, semantic_path, kind, policy, None)
    }

    pub(crate) fn numeric(
        trace: &ValidatedTrace,
        checkpoint_index: usize,
        expected_bits: FloatBits,
        actual_bits: FloatBits,
        policy: &ToleranceProfile,
    ) -> Self {
        Self::new(
            trace,
            checkpoint_index,
            SemanticPath::SimulationTime,
            MismatchKind::Numeric,
            policy,
            Some(FloatMismatchEvidence::new(expected_bits, actual_bits)),
        )
    }

    fn new(
        trace: &ValidatedTrace,
        checkpoint_index: usize,
        semantic_path: SemanticPath,
        kind: MismatchKind,
        policy: &ToleranceProfile,
        maybe_float_evidence: Option<FloatMismatchEvidence>,
    ) -> Self {
        let checkpoint = &trace.checkpoints()[checkpoint_index];
        let phase = PhaseName::new(checkpoint.phase())
            .expect("validated checkpoint phases satisfy report bounds");
        let signature = FailureSignature::new(
            checkpoint.checkpoint_id().clone(),
            phase,
            semantic_path,
            kind,
        );
        Self {
            signature,
            checkpoint_ordinal: checkpoint.ordinal(),
            maybe_previous_checkpoint_id: checkpoint_index
                .checked_sub(1)
                .and_then(|index| trace.checkpoints().get(index))
                .map(|checkpoint| checkpoint.checkpoint_id().clone()),
            maybe_next_checkpoint_id: trace
                .checkpoints()
                .get(checkpoint_index + 1)
                .map(|checkpoint| checkpoint.checkpoint_id().clone()),
            request_id: trace.request_id().clone(),
            request_sha256: request_contract_sha256(trace),
            scenario_sha256: trace.scenario_sha256().clone(),
            policy_id: policy.profile_id().into(),
            policy_sha256: policy.profile_sha256().clone(),
            maybe_float_evidence,
        }
    }

    /// Returns the stable reduction identity.
    #[must_use]
    pub const fn signature(&self) -> &FailureSignature {
        &self.signature
    }

    /// Returns the first divergent checkpoint ordinal.
    #[must_use]
    pub const fn checkpoint_ordinal(&self) -> u32 {
        self.checkpoint_ordinal
    }

    /// Returns the previous ordered checkpoint identity, when present.
    #[must_use]
    pub const fn maybe_previous_checkpoint_id(&self) -> Option<&CheckpointId> {
        self.maybe_previous_checkpoint_id.as_ref()
    }

    /// Returns the next ordered checkpoint identity, when present.
    #[must_use]
    pub const fn maybe_next_checkpoint_id(&self) -> Option<&CheckpointId> {
        self.maybe_next_checkpoint_id.as_ref()
    }

    /// Returns the stable request identity.
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    /// Returns a deterministic hash of the comparable request contract.
    #[must_use]
    pub const fn request_sha256(&self) -> &Sha256Hex {
        &self.request_sha256
    }

    /// Returns the canonical scenario hash.
    #[must_use]
    pub const fn scenario_sha256(&self) -> &Sha256Hex {
        &self.scenario_sha256
    }

    /// Returns the reviewed tolerance policy identity.
    #[must_use]
    pub const fn policy_sha256(&self) -> &Sha256Hex {
        &self.policy_sha256
    }

    /// Returns exact float evidence for numeric mismatches.
    #[must_use]
    pub const fn maybe_float_evidence(&self) -> Option<&FloatMismatchEvidence> {
        self.maybe_float_evidence.as_ref()
    }

    /// Renders deterministic parseable machine evidence.
    ///
    /// # Errors
    ///
    /// Returns [`ReportRenderError`] if typed report serialization fails.
    pub fn render_machine(&self) -> Result<Vec<u8>, ReportRenderError> {
        serde_json::to_vec(self).map_err(ReportRenderError)
    }

    /// Renders a concise human diagnostic from the same typed evidence.
    #[must_use]
    pub fn render_human(&self) -> String {
        let mut rendered = format!(
            "{:?} mismatch at checkpoint {} phase {} path {}",
            self.signature.kind,
            self.signature.checkpoint_id,
            self.signature.phase.as_str(),
            self.signature.semantic_path.human_name(),
        );
        if let Some(evidence) = &self.maybe_float_evidence {
            write!(
                rendered,
                ": expected 0x{:08x} ({}), actual 0x{:08x} ({})",
                evidence.expected_bits.bits(),
                evidence.expected_decimal,
                evidence.actual_bits.bits(),
                evidence.actual_decimal,
            )
            .expect("writing to an owned String cannot fail");
        }
        write!(rendered, "; policy {}", self.policy_id)
            .expect("writing to an owned String cannot fail");
        rendered
    }
}

/// Error produced while rendering deterministic machine evidence.
#[derive(Debug, thiserror::Error)]
#[error("mismatch report serialization failed: {0}")]
pub struct ReportRenderError(serde_json::Error);

/// Harness diagnostics remain a separate type from semantic mismatch reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessFailureReport {
    failure: HarnessFailure,
}

impl HarnessFailureReport {
    /// Wraps one classified non-physics harness failure.
    #[must_use]
    pub const fn new(failure: HarnessFailure) -> Self {
        Self { failure }
    }

    /// Returns the classified non-physics failure.
    #[must_use]
    pub const fn failure(&self) -> &HarnessFailure {
        &self.failure
    }
}

fn request_contract_sha256(trace: &ValidatedTrace) -> Sha256Hex {
    let mut hasher = Sha256::new();
    update_hash_field(&mut hasher, &trace.protocol_version().get().to_be_bytes());
    update_hash_field(&mut hasher, trace.request_id().as_str().as_bytes());
    update_hash_field(
        &mut hasher,
        &trace.trace_schema_version().get().to_be_bytes(),
    );
    update_hash_field(&mut hasher, trace.scenario_id().as_str().as_bytes());
    update_hash_field(&mut hasher, trace.scenario_sha256().as_str().as_bytes());
    update_hash_field(
        &mut hasher,
        &trace.tolerance_profile_version().get().to_be_bytes(),
    );
    update_hash_field(
        &mut hasher,
        trace.tolerance_profile_sha256().as_str().as_bytes(),
    );
    Sha256Hex::from_digest(hasher.finalize().into())
}

fn update_hash_field(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update(bytes.len().to_be_bytes());
    hasher.update(bytes);
}

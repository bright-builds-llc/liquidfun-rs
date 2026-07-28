//! Declaration-first comparison and stable rigid-world evidence.

mod base;
mod declaration;
mod phase7;
pub(crate) mod phase8;
mod signature;

use base::{compare_checkpoint_inherited, first_rigid_divergence};
use declaration::validate_rigid_declarations_with_identity;
use signature::declaration_signature;

use liquidfun_test_protocol::{
    BuildEvidenceTier, BuildIdentity, FieldPolicy, FloatBits, Phase6PolicyProfile,
    Phase7PolicyProfile, Phase8PolicyProfile, RigidStepOutcome, RigidWorldRequestRecord,
    RigidWorldResultRecord, RigidWorldWitnessFamily, Sha256Hex,
};
use serde::Serialize;

use crate::ArtifactKind;

/// Engine whose result first disagreed with the request declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidEngineSide {
    /// Native Rust result.
    Native,
    /// Pinned C++ oracle result.
    Oracle,
}

/// Stable broad category of a rigid physics mismatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidMismatchKind {
    /// Exact scalar or enum value differed.
    Exact,
    /// Float bits violated the named field policy.
    Numeric,
    /// Solver-significant sequence order or multiplicity differed.
    Order,
}

/// Exact action/checkpoint/field identity retained by replay and reduction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidFailureSignature {
    signature_sha256: Sha256Hex,
    witness_family: RigidWorldWitnessFamily,
    action_id: Box<str>,
    checkpoint_id: Box<str>,
    semantic_path: Box<str>,
    kind: RigidMismatchKind,
    stage: Box<str>,
    maybe_entity: Option<Box<str>>,
    expected: Box<str>,
    actual: Box<str>,
    maybe_expected_bits: Option<FloatBits>,
    maybe_actual_bits: Option<FloatBits>,
    profile_sha256: Sha256Hex,
    maybe_completion_context: Option<RigidCompletionContext>,
}

impl RigidFailureSignature {
    /// Returns the deterministic signature digest.
    #[must_use]
    pub const fn signature_sha256(&self) -> &Sha256Hex {
        &self.signature_sha256
    }

    /// Returns the action immediately preceding the divergent checkpoint.
    #[must_use]
    pub fn action_id(&self) -> &str {
        &self.action_id
    }

    /// Returns the divergent checkpoint identity.
    #[must_use]
    pub fn checkpoint_id(&self) -> &str {
        &self.checkpoint_id
    }

    /// Returns the exact reviewed policy path.
    #[must_use]
    pub fn semantic_path(&self) -> &str {
        &self.semantic_path
    }

    /// Returns the stable mismatch category.
    #[must_use]
    pub const fn kind(&self) -> RigidMismatchKind {
        self.kind
    }

    /// Returns the solver/action stage containing the first divergence.
    #[must_use]
    pub fn stage(&self) -> &str {
        &self.stage
    }

    /// Returns the stable semantic entity when the field is entity-scoped.
    #[must_use]
    pub fn maybe_entity(&self) -> Option<&str> {
        self.maybe_entity.as_deref()
    }
}

/// Expected and actual step state surrounding one first divergence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RigidCompletionContext {
    expected: RigidStepOutcome,
    actual: RigidStepOutcome,
}

impl RigidCompletionContext {
    pub(super) const fn new(expected: RigidStepOutcome, actual: RigidStepOutcome) -> Self {
        Self { expected, actual }
    }

    /// Returns the expected completion or partial-progress state.
    #[must_use]
    pub const fn expected(&self) -> RigidStepOutcome {
        self.expected
    }

    /// Returns the actual completion or partial-progress state.
    #[must_use]
    pub const fn actual(&self) -> RigidStepOutcome {
        self.actual
    }
}

/// Request-declaration disagreement found before cross-engine comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidDeclarationReport {
    signature: RigidFailureSignature,
    engine_side: RigidEngineSide,
    expected: Box<str>,
    actual: Box<str>,
}

impl RigidDeclarationReport {
    /// Returns the action identity associated with the first disagreement.
    #[must_use]
    pub fn action_id(&self) -> &str {
        self.signature.action_id()
    }

    /// Returns the first divergent checkpoint identity.
    #[must_use]
    pub fn checkpoint_id(&self) -> &str {
        self.signature.checkpoint_id()
    }

    /// Returns the exact semantic path.
    #[must_use]
    pub fn semantic_path(&self) -> &str {
        self.signature.semantic_path()
    }

    /// Returns the engine whose result contradicted the declaration.
    #[must_use]
    pub const fn engine_side(&self) -> RigidEngineSide {
        self.engine_side
    }
}

/// Comparator contract failure that is neither declaration nor physics evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidHarnessReport {
    reason: Box<str>,
    expected: Box<str>,
    actual: Box<str>,
}

/// Fail-closed boundary failures returned before a physics outcome exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "failure_kind", rename_all = "snake_case")]
pub enum RigidComparisonFailure {
    /// Request/result declarations disagree at an exact location.
    Declaration(Box<RigidDeclarationReport>),
    /// Policy or comparator identity is incompatible.
    Harness(RigidHarnessReport),
}

/// First cross-engine rigid mismatch evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidMismatchReport {
    signature: RigidFailureSignature,
    expected: Box<str>,
    actual: Box<str>,
    maybe_expected_bits: Option<FloatBits>,
    maybe_actual_bits: Option<FloatBits>,
    maybe_expected_decimal: Option<Box<str>>,
    maybe_actual_decimal: Option<Box<str>>,
    policy: FieldPolicy,
    profile_sha256: Sha256Hex,
}

impl RigidMismatchReport {
    /// Returns the exact stable replay/reduction identity.
    #[must_use]
    pub const fn signature(&self) -> &RigidFailureSignature {
        &self.signature
    }

    /// Returns the stable mismatch class.
    #[must_use]
    pub const fn kind(&self) -> RigidMismatchKind {
        self.signature.kind()
    }

    /// Returns the exact reviewed semantic path.
    #[must_use]
    pub fn semantic_path(&self) -> &str {
        self.signature.semantic_path()
    }

    /// Returns the exact action whose emitted state first diverged.
    #[must_use]
    pub fn action_id(&self) -> &str {
        self.signature.action_id()
    }

    /// Returns the declared action or solver stage.
    #[must_use]
    pub fn stage(&self) -> &str {
        self.signature.stage()
    }

    /// Returns the stable semantic entity when one owns the divergent field.
    #[must_use]
    pub fn maybe_entity(&self) -> Option<&str> {
        self.signature.maybe_entity()
    }

    /// Returns the exact expected diagnostic value.
    #[must_use]
    pub fn expected(&self) -> &str {
        &self.expected
    }

    /// Returns the exact actual diagnostic value.
    #[must_use]
    pub fn actual(&self) -> &str {
        &self.actual
    }

    /// Returns transported expected bits for a numeric divergence.
    #[must_use]
    pub const fn maybe_expected_bits(&self) -> Option<FloatBits> {
        self.maybe_expected_bits
    }

    /// Returns transported actual bits for a numeric divergence.
    #[must_use]
    pub const fn maybe_actual_bits(&self) -> Option<FloatBits> {
        self.maybe_actual_bits
    }

    /// Returns the deterministic decimal rendering of an expected numeric value.
    #[must_use]
    pub fn maybe_expected_decimal(&self) -> Option<&str> {
        self.maybe_expected_decimal.as_deref()
    }

    /// Returns the deterministic decimal rendering of an actual numeric value.
    #[must_use]
    pub fn maybe_actual_decimal(&self) -> Option<&str> {
        self.maybe_actual_decimal.as_deref()
    }

    /// Returns the registered closed field policy used for comparison.
    #[must_use]
    pub const fn policy(&self) -> &FieldPolicy {
        &self.policy
    }

    /// Returns completion or partial-progress state surrounding the divergence.
    #[must_use]
    pub const fn maybe_completion_context(&self) -> Option<&RigidCompletionContext> {
        self.signature.maybe_completion_context.as_ref()
    }

    /// Renders deterministic bounded machine evidence.
    ///
    /// # Errors
    ///
    /// Returns the serializer error if an invariant-breaking value cannot encode.
    pub fn render_machine(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

/// Complete comparison result after both engines pass declaration validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RigidComparisonOutcome {
    /// Every rigid observable matched its exact named policy.
    Match,
    /// The first aligned physics-visible observable differed.
    PhysicsMismatch(Box<RigidMismatchReport>),
}

/// Promotion-authority rejection for local or exploratory rigid evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RigidPromotionError {
    /// Only canonical D1 oracle output may enter accepted reference paths.
    #[error("{artifact_kind:?} requires D1 canonical authority, found {actual:?}")]
    NonCanonicalAuthority {
        /// Candidate artifact class.
        artifact_kind: ArtifactKind,
        /// Actual validated build tier.
        actual: BuildEvidenceTier,
    },
}

/// Proves a rigid candidate has canonical D1 authority before generic staging or promotion.
///
/// # Errors
///
/// Returns [`RigidPromotionError`] for D2 or D3 build identity.
pub fn validate_rigid_promotion_authority(
    identity: &BuildIdentity,
    artifact_kind: ArtifactKind,
) -> Result<(), RigidPromotionError> {
    if identity.can_promote_canonical_evidence() {
        return Ok(());
    }
    Err(RigidPromotionError::NonCanonicalAuthority {
        artifact_kind,
        actual: identity.evidence_tier(),
    })
}

/// Compares declaration-valid native and oracle rigid traces at the first aligned field.
///
/// Declaration checks run independently for both sides before any cross-engine value is
/// observed. Structural and collection paths compare exactly in their existing order; float
/// values use only the exact named `phase6-v1` field policy.
///
/// # Errors
///
/// Returns a declaration or harness boundary failure before producing physics evidence.
pub fn compare_rigid_world_results(
    request: &RigidWorldRequestRecord,
    native: &RigidWorldResultRecord,
    oracle: &RigidWorldResultRecord,
    profile: &Phase6PolicyProfile,
) -> Result<RigidComparisonOutcome, RigidComparisonFailure> {
    if request.tolerance_profile_sha256() != profile.profile_sha256() {
        return Err(RigidComparisonFailure::Harness(RigidHarnessReport {
            reason: "profile_identity".into(),
            expected: profile.profile_sha256().as_str().into(),
            actual: request.tolerance_profile_sha256().as_str().into(),
        }));
    }
    validate_rigid_declarations_with_identity(
        request,
        native,
        profile.profile_sha256(),
        RigidEngineSide::Native,
    )?;
    validate_rigid_declarations_with_identity(
        request,
        oracle,
        profile.profile_sha256(),
        RigidEngineSide::Oracle,
    )?;

    if let Some(report) = first_rigid_divergence(request, native, oracle, profile) {
        return Ok(RigidComparisonOutcome::PhysicsMismatch(Box::new(report)));
    }
    Ok(RigidComparisonOutcome::Match)
}

/// Compares declaration-valid Phase 7 output under the inherited Phase 6 and closed Phase 7
/// field registries.
///
/// # Errors
///
/// Returns a declaration or harness boundary failure before producing physics evidence.
pub fn compare_phase7_rigid_world_results(
    request: &RigidWorldRequestRecord,
    native: &RigidWorldResultRecord,
    oracle: &RigidWorldResultRecord,
    phase6_profile: &Phase6PolicyProfile,
    phase7_profile: &Phase7PolicyProfile,
) -> Result<RigidComparisonOutcome, RigidComparisonFailure> {
    if request.tolerance_profile_sha256() != phase7_profile.profile_sha256() {
        return Err(RigidComparisonFailure::Harness(RigidHarnessReport {
            reason: "profile_identity".into(),
            expected: phase7_profile.profile_sha256().as_str().into(),
            actual: request.tolerance_profile_sha256().as_str().into(),
        }));
    }
    validate_rigid_declarations_with_identity(
        request,
        native,
        phase7_profile.profile_sha256(),
        RigidEngineSide::Native,
    )?;
    validate_rigid_declarations_with_identity(
        request,
        oracle,
        phase7_profile.profile_sha256(),
        RigidEngineSide::Oracle,
    )?;
    if let Some(report) =
        phase7::first_divergence(request, native, oracle, phase6_profile, phase7_profile)?
    {
        return Ok(RigidComparisonOutcome::PhysicsMismatch(Box::new(report)));
    }
    Ok(RigidComparisonOutcome::Match)
}

/// Compares the complete Phase 8 rigid corpus under inherited Phase 6/7 and
/// closed Phase 8 policies.
///
/// # Errors
///
/// Returns a declaration or harness boundary failure before producing
/// physics evidence.
pub fn compare_phase8_rigid_world_results(
    request: &RigidWorldRequestRecord,
    native: &RigidWorldResultRecord,
    oracle: &RigidWorldResultRecord,
    phase6_profile: &Phase6PolicyProfile,
    phase7_profile: &Phase7PolicyProfile,
    phase8_profile: &Phase8PolicyProfile,
) -> Result<RigidComparisonOutcome, RigidComparisonFailure> {
    if request.tolerance_profile_sha256() != phase8_profile.profile_sha256() {
        return Err(RigidComparisonFailure::Harness(RigidHarnessReport {
            reason: "profile_identity".into(),
            expected: phase8_profile.profile_sha256().as_str().into(),
            actual: request.tolerance_profile_sha256().as_str().into(),
        }));
    }
    validate_rigid_declarations_with_identity(
        request,
        native,
        phase8_profile.profile_sha256(),
        RigidEngineSide::Native,
    )?;
    validate_rigid_declarations_with_identity(
        request,
        oracle,
        phase8_profile.profile_sha256(),
        RigidEngineSide::Oracle,
    )?;
    if let Some(report) =
        phase7::first_divergence(request, native, oracle, phase6_profile, phase7_profile)?
    {
        return Ok(RigidComparisonOutcome::PhysicsMismatch(Box::new(report)));
    }
    if let Some(report) =
        phase8::first_divergence(request, native, oracle, phase6_profile, phase8_profile)?
    {
        return Ok(RigidComparisonOutcome::PhysicsMismatch(Box::new(report)));
    }
    Ok(RigidComparisonOutcome::Match)
}

#[derive(Clone, Copy)]
pub(super) struct Location {
    pub(super) timeline_index: usize,
    pub(super) checkpoint_index: usize,
}

#[derive(Clone, Copy)]
pub(super) struct EvidenceContext<'a> {
    pub(super) location: Location,
    pub(super) maybe_action_id: Option<&'a str>,
    pub(super) maybe_stage: Option<&'a str>,
    pub(super) maybe_entity: Option<&'a str>,
    pub(super) maybe_completion_context: Option<RigidCompletionContext>,
}

impl EvidenceContext<'_> {
    const fn checkpoint(location: Location) -> Self {
        Self {
            location,
            maybe_action_id: None,
            maybe_stage: None,
            maybe_entity: None,
            maybe_completion_context: None,
        }
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "one builder binds the complete rigid signature"
)]
pub(super) fn mismatch(
    request: &RigidWorldRequestRecord,
    profile: &Phase6PolicyProfile,
    location: Location,
    path: &'static str,
    kind: RigidMismatchKind,
    expected: String,
    actual: String,
    maybe_bits: Option<(FloatBits, FloatBits)>,
) -> RigidMismatchReport {
    let policy = profile
        .field(path)
        .expect("validated Phase 6 profile contains every mismatch path");
    mismatch_with_context(
        request,
        profile.profile_sha256(),
        policy,
        EvidenceContext::checkpoint(location),
        path,
        kind,
        expected,
        actual,
        maybe_bits,
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "one builder binds the complete first-divergence evidence"
)]
pub(super) fn mismatch_with_context(
    request: &RigidWorldRequestRecord,
    profile_sha256: &Sha256Hex,
    policy: &FieldPolicy,
    context: EvidenceContext<'_>,
    path: &'static str,
    kind: RigidMismatchKind,
    expected: String,
    actual: String,
    maybe_bits: Option<(FloatBits, FloatBits)>,
) -> RigidMismatchReport {
    let signature = signature::build_signature(
        request,
        profile_sha256,
        context,
        path,
        kind,
        &expected,
        &actual,
        maybe_bits,
    );
    RigidMismatchReport {
        signature,
        expected: expected.into_boxed_str(),
        actual: actual.into_boxed_str(),
        maybe_expected_bits: maybe_bits.map(|bits| bits.0),
        maybe_actual_bits: maybe_bits.map(|bits| bits.1),
        maybe_expected_decimal: maybe_bits.map(|bits| bits.0.to_f32().to_string().into_boxed_str()),
        maybe_actual_decimal: maybe_bits.map(|bits| bits.1.to_f32().to_string().into_boxed_str()),
        policy: policy.clone(),
        profile_sha256: profile_sha256.clone(),
    }
}

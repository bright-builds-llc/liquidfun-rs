//! Declaration-first comparison and stable Phase 6 rigid-world evidence.

use std::fmt::Debug;

use liquidfun_test_protocol::{
    BuildEvidenceTier, BuildIdentity, FieldComparison, FloatBits, Phase6PolicyProfile,
    RigidContactIdentity, RigidDestructionRecord, RigidWorldCheckpointResult,
    RigidWorldRequestRecord, RigidWorldResultRecord, RigidWorldTimelineResult,
    RigidWorldWitnessFamily, Sha256Hex, validate_rigid_world_result_against_request,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{ArtifactKind, float_values_match_with_policy};

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct RigidFailureSignature {
    signature_sha256: Sha256Hex,
    witness_family: RigidWorldWitnessFamily,
    action_id: Box<str>,
    checkpoint_id: Box<str>,
    semantic_path: Box<str>,
    kind: RigidMismatchKind,
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
    Declaration(RigidDeclarationReport),
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
    PhysicsMismatch(RigidMismatchReport),
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
    validate_rigid_declarations(request, native, profile, RigidEngineSide::Native)?;
    validate_rigid_declarations(request, oracle, profile, RigidEngineSide::Oracle)?;

    if let Some(report) = first_rigid_divergence(request, native, oracle, profile) {
        return Ok(RigidComparisonOutcome::PhysicsMismatch(report));
    }
    Ok(RigidComparisonOutcome::Match)
}

fn validate_rigid_declarations(
    request: &RigidWorldRequestRecord,
    result: &RigidWorldResultRecord,
    profile: &Phase6PolicyProfile,
    side: RigidEngineSide,
) -> Result<(), RigidComparisonFailure> {
    if result.request_id() != request.request_id() {
        return declaration_root(
            request,
            profile,
            side,
            "rigid_world.result.request_id",
            request.request_id().as_str(),
            result.request_id().as_str(),
        );
    }
    if result.scenario_id() != request.scenario().scenario_id() {
        return declaration_root(
            request,
            profile,
            side,
            "rigid_world.result.scenario_id",
            request.scenario().scenario_id().as_str(),
            result.scenario_id().as_str(),
        );
    }
    if result.timelines().len() != request.scenario().timelines().len() {
        return declaration_root(
            request,
            profile,
            side,
            "rigid_world.timelines.order",
            request.scenario().timelines().len(),
            result.timelines().len(),
        );
    }

    for (timeline_index, (declared, actual)) in request
        .scenario()
        .timelines()
        .iter()
        .zip(result.timelines())
        .enumerate()
    {
        if declared.witness_family() != actual.witness_family {
            return declaration(
                request,
                profile,
                side,
                timeline_index,
                0,
                "rigid_world.timeline.witness_family",
                declared.witness_family(),
                actual.witness_family,
            );
        }
        if declared.checkpoints().len() != actual.checkpoints.len() {
            return declaration(
                request,
                profile,
                side,
                timeline_index,
                first_missing_index(declared.checkpoints().len(), actual.checkpoints.len()),
                "rigid_world.checkpoints.order",
                declared.checkpoints().len(),
                actual.checkpoints.len(),
            );
        }
        for (checkpoint_index, (expected, actual)) in declared
            .checkpoints()
            .iter()
            .zip(actual.checkpoints.iter())
            .enumerate()
        {
            if expected.checkpoint_id() != &actual.checkpoint_id {
                return declaration(
                    request,
                    profile,
                    side,
                    timeline_index,
                    checkpoint_index,
                    "rigid_world.checkpoint.id",
                    expected.checkpoint_id(),
                    &actual.checkpoint_id,
                );
            }
            if expected.phase() != actual.phase.as_ref() {
                return declaration(
                    request,
                    profile,
                    side,
                    timeline_index,
                    checkpoint_index,
                    "rigid_world.checkpoint.phase",
                    expected.phase(),
                    actual.phase.as_ref(),
                );
            }
            if expected.counts() != actual.counts {
                return declaration(
                    request,
                    profile,
                    side,
                    timeline_index,
                    checkpoint_index,
                    "rigid_world.checkpoint.counts",
                    expected.counts(),
                    actual.counts,
                );
            }
            let body_ids = actual
                .bodies
                .iter()
                .map(|body| &body.body_id)
                .collect::<Vec<_>>();
            let declared_body_ids = declared
                .bodies()
                .iter()
                .map(liquidfun_test_protocol::RigidBodyDeclaration::body_id)
                .collect::<Vec<_>>();
            if !is_subsequence(&declared_body_ids, &body_ids) {
                return declaration(
                    request,
                    profile,
                    side,
                    timeline_index,
                    checkpoint_index,
                    "rigid_world.checkpoint.bodies.declaration_order",
                    declared_body_ids,
                    body_ids,
                );
            }
            let fixture_ids = actual
                .fixtures
                .iter()
                .map(|fixture| &fixture.fixture_id)
                .collect::<Vec<_>>();
            let declared_fixture_ids = declared
                .fixtures()
                .iter()
                .map(liquidfun_test_protocol::RigidFixtureDeclaration::fixture_id)
                .collect::<Vec<_>>();
            if !is_subsequence(&declared_fixture_ids, &fixture_ids) {
                return declaration(
                    request,
                    profile,
                    side,
                    timeline_index,
                    checkpoint_index,
                    "rigid_world.checkpoint.fixtures.declaration_order",
                    declared_fixture_ids,
                    fixture_ids,
                );
            }
        }
    }

    validate_rigid_world_result_against_request(request, result).map_err(|error| {
        RigidComparisonFailure::Harness(RigidHarnessReport {
            reason: "validate_result_declaration".into(),
            expected: "request declarations".into(),
            actual: error.to_string().into_boxed_str(),
        })
    })
}

fn first_rigid_divergence(
    request: &RigidWorldRequestRecord,
    expected: &RigidWorldResultRecord,
    actual: &RigidWorldResultRecord,
    profile: &Phase6PolicyProfile,
) -> Option<RigidMismatchReport> {
    for (timeline_index, (expected_timeline, actual_timeline)) in expected
        .timelines()
        .iter()
        .zip(actual.timelines())
        .enumerate()
    {
        for (checkpoint_index, (expected_checkpoint, actual_checkpoint)) in expected_timeline
            .checkpoints
            .iter()
            .zip(actual_timeline.checkpoints.iter())
            .enumerate()
        {
            let location = Location {
                timeline_index,
                checkpoint_index,
            };
            if let Some(report) = compare_checkpoint(
                request,
                profile,
                location,
                expected_checkpoint,
                actual_checkpoint,
            ) {
                return Some(report);
            }
        }
    }
    None
}

#[derive(Clone, Copy)]
struct Location {
    timeline_index: usize,
    checkpoint_index: usize,
}

macro_rules! exact_field {
    ($request:expr, $profile:expr, $location:expr, $path:expr, $left:expr, $right:expr) => {
        if let Some(report) = exact(
            $request,
            $profile,
            $location,
            $path,
            RigidMismatchKind::Exact,
            &$left,
            &$right,
        ) {
            return Some(report);
        }
    };
    ($request:expr, $profile:expr, $location:expr, $path:expr, $left:expr, $right:expr, $kind:expr) => {
        if let Some(report) = exact($request, $profile, $location, $path, $kind, &$left, &$right) {
            return Some(report);
        }
    };
}

macro_rules! float_field {
    ($request:expr, $profile:expr, $location:expr, $path:expr, $left:expr, $right:expr) => {
        if let Some(report) = float($request, $profile, $location, $path, $left, $right) {
            return Some(report);
        }
    };
}

fn compare_checkpoint(
    request: &RigidWorldRequestRecord,
    profile: &Phase6PolicyProfile,
    location: Location,
    expected: &RigidWorldCheckpointResult,
    actual: &RigidWorldCheckpointResult,
) -> Option<RigidMismatchReport> {
    exact_field!(
        request,
        profile,
        location,
        "rigid_world.checkpoint.counts",
        expected.counts,
        actual.counts
    );
    let expected_body_ids = expected
        .bodies
        .iter()
        .map(|body| &body.body_id)
        .collect::<Vec<_>>();
    let actual_body_ids = actual
        .bodies
        .iter()
        .map(|body| &body.body_id)
        .collect::<Vec<_>>();
    exact_field!(
        request,
        profile,
        location,
        "rigid_world.checkpoint.bodies.declaration_order",
        expected_body_ids,
        actual_body_ids,
        RigidMismatchKind::Order
    );
    for (left, right) in expected.bodies.iter().zip(actual.bodies.iter()) {
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.body.id",
            left.body_id,
            right.body_id
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.body.kind",
            left.body_kind,
            right.body_kind
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.body.active",
            left.active,
            right.active
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.transform.position.x",
            left.transform.position.x_bits,
            right.transform.position.x_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.transform.position.y",
            left.transform.position.y_bits,
            right.transform.position.y_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.transform.angle",
            left.transform.angle_bits,
            right.transform.angle_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.linear_velocity.x",
            left.linear_velocity.x_bits,
            right.linear_velocity.x_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.linear_velocity.y",
            left.linear_velocity.y_bits,
            right.linear_velocity.y_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.angular_velocity",
            left.angular_velocity_bits,
            right.angular_velocity_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.mass",
            left.mass_bits,
            right.mass_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.local_center.x",
            left.local_center.x_bits,
            right.local_center.x_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.local_center.y",
            left.local_center.y_bits,
            right.local_center.y_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.body.inertia",
            left.inertia_bits,
            right.inertia_bits
        );
    }

    let expected_fixture_ids = expected
        .fixtures
        .iter()
        .map(|fixture| &fixture.fixture_id)
        .collect::<Vec<_>>();
    let actual_fixture_ids = actual
        .fixtures
        .iter()
        .map(|fixture| &fixture.fixture_id)
        .collect::<Vec<_>>();
    exact_field!(
        request,
        profile,
        location,
        "rigid_world.checkpoint.fixtures.declaration_order",
        expected_fixture_ids,
        actual_fixture_ids,
        RigidMismatchKind::Order
    );
    for (left, right) in expected.fixtures.iter().zip(actual.fixtures.iter()) {
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.id",
            left.fixture_id,
            right.fixture_id
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.owner_body_id",
            left.owner_body_id,
            right.owner_body_id
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.sensor",
            left.sensor,
            right.sensor
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.filter.category_bits",
            left.filter.category_bits(),
            right.filter.category_bits()
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.filter.mask_bits",
            left.filter.mask_bits(),
            right.filter.mask_bits()
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.filter.group_index",
            left.filter.group_index(),
            right.filter.group_index()
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.density",
            left.density_bits,
            right.density_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.friction",
            left.friction_bits,
            right.friction_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.fixture.restitution",
            left.restitution_bits,
            right.restitution_bits
        );
    }

    let expected_contacts = expected
        .contacts
        .iter()
        .map(|contact| &contact.identity)
        .collect::<Vec<_>>();
    let actual_contacts = actual
        .contacts
        .iter()
        .map(|contact| &contact.identity)
        .collect::<Vec<_>>();
    exact_field!(
        request,
        profile,
        location,
        "rigid_world.checkpoint.contacts.manager_order",
        expected_contacts,
        actual_contacts,
        RigidMismatchKind::Order
    );
    for (left, right) in expected.contacts.iter().zip(actual.contacts.iter()) {
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.contact.touching",
            left.touching,
            right.touching
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.contact.enabled",
            left.enabled,
            right.enabled
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.contact.sensor",
            left.sensor,
            right.sensor
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.contact.mixed_friction",
            left.mixed_friction_bits,
            right.mixed_friction_bits
        );
        float_field!(
            request,
            profile,
            location,
            "rigid_world.contact.mixed_restitution",
            left.mixed_restitution_bits,
            right.mixed_restitution_bits
        );
        exact_field!(
            request,
            profile,
            location,
            "rigid_world.contact.manifold.presence",
            left.maybe_manifold.is_some(),
            right.maybe_manifold.is_some()
        );
        if let (Some(left), Some(right)) = (&left.maybe_manifold, &right.maybe_manifold) {
            exact_field!(
                request,
                profile,
                location,
                "rigid_world.contact.manifold.kind",
                left.manifold_kind,
                right.manifold_kind
            );
            let expected_features = left
                .points
                .iter()
                .map(|point| point.feature)
                .collect::<Vec<_>>();
            let actual_features = right
                .points
                .iter()
                .map(|point| point.feature)
                .collect::<Vec<_>>();
            exact_field!(
                request,
                profile,
                location,
                "rigid_world.contact.manifold.points.order",
                expected_features,
                actual_features,
                RigidMismatchKind::Order
            );
            float_field!(
                request,
                profile,
                location,
                "rigid_world.contact.manifold.local_normal.x",
                left.local_normal.x_bits,
                right.local_normal.x_bits
            );
            float_field!(
                request,
                profile,
                location,
                "rigid_world.contact.manifold.local_normal.y",
                left.local_normal.y_bits,
                right.local_normal.y_bits
            );
            float_field!(
                request,
                profile,
                location,
                "rigid_world.contact.manifold.local_point.x",
                left.local_point.x_bits,
                right.local_point.x_bits
            );
            float_field!(
                request,
                profile,
                location,
                "rigid_world.contact.manifold.local_point.y",
                left.local_point.y_bits,
                right.local_point.y_bits
            );
            for (left, right) in left.points.iter().zip(right.points.iter()) {
                exact_field!(
                    request,
                    profile,
                    location,
                    "rigid_world.contact.manifold.point.feature",
                    left.feature,
                    right.feature
                );
                float_field!(
                    request,
                    profile,
                    location,
                    "rigid_world.contact.manifold.point.position.x",
                    left.point.x_bits,
                    right.point.x_bits
                );
                float_field!(
                    request,
                    profile,
                    location,
                    "rigid_world.contact.manifold.point.position.y",
                    left.point.y_bits,
                    right.point.y_bits
                );
                float_field!(
                    request,
                    profile,
                    location,
                    "rigid_world.contact.manifold.point.normal_impulse",
                    left.normal_impulse_bits,
                    right.normal_impulse_bits
                );
                float_field!(
                    request,
                    profile,
                    location,
                    "rigid_world.contact.manifold.point.tangent_impulse",
                    left.tangent_impulse_bits,
                    right.tangent_impulse_bits
                );
            }
        }
    }

    exact_field!(
        request,
        profile,
        location,
        "rigid_world.checkpoint.events.report_order",
        expected.events,
        actual.events,
        RigidMismatchKind::Order
    );
    exact_field!(
        request,
        profile,
        location,
        "rigid_world.checkpoint.destructions.report_order",
        expected.destructions,
        actual.destructions,
        RigidMismatchKind::Order
    );
    None
}

fn exact<T: Debug + PartialEq>(
    request: &RigidWorldRequestRecord,
    profile: &Phase6PolicyProfile,
    location: Location,
    path: &'static str,
    kind: RigidMismatchKind,
    expected: &T,
    actual: &T,
) -> Option<RigidMismatchReport> {
    let policy = profile
        .field(path)
        .expect("validated Phase 6 profile contains every exact path");
    debug_assert_eq!(policy.comparison(), FieldComparison::ExactDiscrete);
    (expected != actual).then(|| {
        mismatch(
            request,
            profile,
            location,
            path,
            kind,
            format!("{expected:?}"),
            format!("{actual:?}"),
            None,
        )
    })
}

fn float(
    request: &RigidWorldRequestRecord,
    profile: &Phase6PolicyProfile,
    location: Location,
    path: &'static str,
    expected: FloatBits,
    actual: FloatBits,
) -> Option<RigidMismatchReport> {
    let policy = profile
        .field(path)
        .expect("validated Phase 6 profile contains every float path");
    (!float_values_match_with_policy(expected, actual, policy)).then(|| {
        mismatch(
            request,
            profile,
            location,
            path,
            RigidMismatchKind::Numeric,
            format!("0x{:08x}", expected.bits()),
            format!("0x{:08x}", actual.bits()),
            Some((expected, actual)),
        )
    })
}

#[allow(
    clippy::too_many_arguments,
    reason = "one builder binds the complete rigid signature"
)]
fn mismatch(
    request: &RigidWorldRequestRecord,
    profile: &Phase6PolicyProfile,
    location: Location,
    path: &'static str,
    kind: RigidMismatchKind,
    expected: String,
    actual: String,
    maybe_bits: Option<(FloatBits, FloatBits)>,
) -> RigidMismatchReport {
    let signature = signature(request, profile, location, path, kind);
    RigidMismatchReport {
        signature,
        expected: expected.into_boxed_str(),
        actual: actual.into_boxed_str(),
        maybe_expected_bits: maybe_bits.map(|bits| bits.0),
        maybe_actual_bits: maybe_bits.map(|bits| bits.1),
        profile_sha256: profile.profile_sha256().clone(),
    }
}

fn signature(
    request: &RigidWorldRequestRecord,
    profile: &Phase6PolicyProfile,
    location: Location,
    path: &str,
    kind: RigidMismatchKind,
) -> RigidFailureSignature {
    let timeline = &request.scenario().timelines()[location.timeline_index];
    let checkpoint = &timeline.checkpoints()[location.checkpoint_index];
    let input = format!(
        "{}|{:?}|{}|{}|{}|{:?}|{}",
        request.request_id().as_str(),
        timeline.witness_family(),
        checkpoint.after_action_id().as_str(),
        checkpoint.checkpoint_id().as_str(),
        path,
        kind,
        profile.profile_sha256().as_str(),
    );
    RigidFailureSignature {
        signature_sha256: Sha256Hex::from_digest(Sha256::digest(input.as_bytes()).into()),
        witness_family: timeline.witness_family(),
        action_id: checkpoint.after_action_id().as_str().into(),
        checkpoint_id: checkpoint.checkpoint_id().as_str().into(),
        semantic_path: path.into(),
        kind,
    }
}

fn declaration<T: Debug>(
    request: &RigidWorldRequestRecord,
    profile: &Phase6PolicyProfile,
    side: RigidEngineSide,
    timeline_index: usize,
    checkpoint_index: usize,
    path: &'static str,
    expected: T,
    actual: T,
) -> Result<(), RigidComparisonFailure> {
    let timeline = &request.scenario().timelines()[timeline_index];
    let bounded_index = checkpoint_index.min(timeline.checkpoints().len().saturating_sub(1));
    let location = Location {
        timeline_index,
        checkpoint_index: bounded_index,
    };
    Err(RigidComparisonFailure::Declaration(
        RigidDeclarationReport {
            signature: signature(request, profile, location, path, RigidMismatchKind::Exact),
            engine_side: side,
            expected: format!("{expected:?}").into_boxed_str(),
            actual: format!("{actual:?}").into_boxed_str(),
        },
    ))
}

fn declaration_root<T: Debug>(
    request: &RigidWorldRequestRecord,
    profile: &Phase6PolicyProfile,
    side: RigidEngineSide,
    path: &'static str,
    expected: T,
    actual: T,
) -> Result<(), RigidComparisonFailure> {
    declaration(request, profile, side, 0, 0, path, expected, actual)
}

fn first_missing_index(expected: usize, actual: usize) -> usize {
    expected.min(actual)
}

fn is_subsequence<T: PartialEq>(declared: &[T], actual: &[T]) -> bool {
    let mut next = 0;
    for item in actual {
        let Some(offset) = declared[next..]
            .iter()
            .position(|declared_item| declared_item == item)
        else {
            return false;
        };
        next += offset + 1;
    }
    true
}

#[allow(
    dead_code,
    reason = "closed path types remain visible to rustdoc and future diagnostics"
)]
fn _order_types_are_semantic(
    _identity: &RigidContactIdentity,
    _destruction: &RigidDestructionRecord,
    _timeline: &RigidWorldTimelineResult,
) {
}

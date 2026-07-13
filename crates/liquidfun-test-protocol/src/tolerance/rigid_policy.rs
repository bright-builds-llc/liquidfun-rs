#![allow(
    missing_docs,
    reason = "closed private-harness policy errors and accessors are self-describing"
)]

use std::collections::HashSet;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::{
    CollectionPolicy, DivergenceHorizon, EvidenceTier, FieldComparison, FieldPolicy, FloatPolicy,
    NonFinitePolicy, ZeroPolicy,
};
use crate::{RigidWorldWitnessFamily, Sha256Hex, ToleranceProfileVersion};

const MAXIMUM_SEMANTIC_PATH_BYTES: usize = 256;
const MAXIMUM_JUSTIFICATION_BYTES: usize = 512;

const STRUCTURAL_PATHS: &[&str] = &[
    "rigid_world.result.request_id",
    "rigid_world.result.scenario_id",
    "rigid_world.timelines.order",
    "rigid_world.timeline.witness_family",
    "rigid_world.checkpoints.order",
    "rigid_world.checkpoint.id",
    "rigid_world.checkpoint.phase",
    "rigid_world.checkpoint.counts",
    "rigid_world.checkpoint.bodies.declaration_order",
    "rigid_world.body.id",
    "rigid_world.body.kind",
    "rigid_world.body.active",
    "rigid_world.checkpoint.fixtures.declaration_order",
    "rigid_world.fixture.id",
    "rigid_world.fixture.owner_body_id",
    "rigid_world.fixture.sensor",
    "rigid_world.fixture.filter.category_bits",
    "rigid_world.fixture.filter.mask_bits",
    "rigid_world.fixture.filter.group_index",
    "rigid_world.checkpoint.contacts.manager_order",
    "rigid_world.contact.identity",
    "rigid_world.contact.touching",
    "rigid_world.contact.enabled",
    "rigid_world.contact.sensor",
    "rigid_world.contact.manifold.presence",
    "rigid_world.contact.manifold.kind",
    "rigid_world.contact.manifold.points.order",
    "rigid_world.contact.manifold.point.feature",
    "rigid_world.checkpoint.events.report_order",
    "rigid_world.event.kind",
    "rigid_world.event.contact_identity",
    "rigid_world.checkpoint.destructions.report_order",
    "rigid_world.destruction.kind",
    "rigid_world.destruction.identity",
];

const FLOAT_PATHS: &[&str] = &[
    "rigid_world.body.transform.position.x",
    "rigid_world.body.transform.position.y",
    "rigid_world.body.transform.angle",
    "rigid_world.body.linear_velocity.x",
    "rigid_world.body.linear_velocity.y",
    "rigid_world.body.angular_velocity",
    "rigid_world.body.mass",
    "rigid_world.body.local_center.x",
    "rigid_world.body.local_center.y",
    "rigid_world.body.inertia",
    "rigid_world.fixture.density",
    "rigid_world.fixture.friction",
    "rigid_world.fixture.restitution",
    "rigid_world.contact.mixed_friction",
    "rigid_world.contact.mixed_restitution",
    "rigid_world.contact.manifold.local_normal.x",
    "rigid_world.contact.manifold.local_normal.y",
    "rigid_world.contact.manifold.local_point.x",
    "rigid_world.contact.manifold.local_point.y",
    "rigid_world.contact.manifold.point.position.x",
    "rigid_world.contact.manifold.point.position.y",
    "rigid_world.contact.manifold.point.normal_impulse",
    "rigid_world.contact.manifold.point.tangent_impulse",
];

const PHASE7_STRUCTURAL_PATHS: &[&str] = &[
    "rigid_world.phase7.observations.order",
    "rigid_world.phase7.body.id",
    "rigid_world.phase7.body.awake",
    "rigid_world.phase7.body.bullet",
    "rigid_world.phase7.body.sleeping_allowed",
    "rigid_world.phase7.body.fixed_rotation",
    "rigid_world.phase7.step.outcome.kind",
    "rigid_world.phase7.step.completion",
    "rigid_world.phase7.step.partial_classification",
    "rigid_world.phase7.contact.transitions.order",
    "rigid_world.phase7.contact.identity",
    "rigid_world.phase7.island.body_order",
    "rigid_world.phase7.island.contact_order",
    "rigid_world.phase7.query.completion",
    "rigid_world.phase7.query.occurrences.identity",
    "rigid_world.phase7.ray.completion",
    "rigid_world.phase7.ray.hit.identity",
    "rigid_world.phase7.ray.equal_minimum.identities",
];

const PHASE7_ABSOLUTE_RELATIVE_PATHS: &[&str] = &[
    "rigid_world.phase7.body.transform.position.x",
    "rigid_world.phase7.body.transform.position.y",
    "rigid_world.phase7.body.linear_velocity.x",
    "rigid_world.phase7.body.linear_velocity.y",
    "rigid_world.phase7.ray.point.x",
    "rigid_world.phase7.ray.point.y",
    "rigid_world.phase7.origin_shift.x",
    "rigid_world.phase7.origin_shift.y",
];

const PHASE7_ULP_PATHS: &[&str] = &[
    "rigid_world.phase7.body.transform.angle",
    "rigid_world.phase7.body.angular_velocity",
    "rigid_world.phase7.body.linear_damping",
    "rigid_world.phase7.body.angular_damping",
    "rigid_world.phase7.body.gravity_scale",
    "rigid_world.phase7.contact.normal_impulse",
    "rigid_world.phase7.contact.tangent_impulse",
    "rigid_world.phase7.ray.fraction",
    "rigid_world.phase7.ray.normal.x",
    "rigid_world.phase7.ray.normal.y",
];

/// Strict closed comparison profile for Phase 7 rigid evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase7PolicyProfile {
    profile_id: Box<str>,
    version: ToleranceProfileVersion,
    fields: Box<[FieldPolicy]>,
    profile_sha256: Sha256Hex,
}

impl Phase7PolicyProfile {
    /// Parses and validates the complete Phase 7 field registry.
    ///
    /// # Errors
    ///
    /// Returns [`Phase7PolicyError`] for any unknown, missing, duplicate, or
    /// incorrectly classified semantic path.
    pub fn parse_toml(input: &str) -> Result<Self, Phase7PolicyError> {
        let raw: RawProfile = toml::from_str(input).map_err(Phase7PolicyError::Toml)?;
        if raw.profile_id.as_ref() != "phase7-v1"
            || raw.version != ToleranceProfileVersion::CURRENT.get()
        {
            return Err(Phase7PolicyError::UnsupportedIdentity);
        }
        let expected_count = PHASE7_STRUCTURAL_PATHS.len()
            + PHASE7_ABSOLUTE_RELATIVE_PATHS.len()
            + PHASE7_ULP_PATHS.len();
        if raw.fields.len() != expected_count {
            return Err(Phase7PolicyError::IncompleteProfile);
        }

        let mut fields = raw.fields;
        let mut semantic_paths = HashSet::with_capacity(fields.len());
        for field in &fields {
            validate_phase7_field(field)?;
            if !semantic_paths.insert(field.semantic_path()) {
                return Err(Phase7PolicyError::DuplicateSemanticPath(
                    field.semantic_path().into(),
                ));
            }
        }
        if PHASE7_STRUCTURAL_PATHS
            .iter()
            .chain(PHASE7_ABSOLUTE_RELATIVE_PATHS)
            .chain(PHASE7_ULP_PATHS)
            .any(|path| !semantic_paths.contains(path))
        {
            return Err(Phase7PolicyError::IncompleteProfile);
        }
        if RigidWorldWitnessFamily::PHASE7_REQUIRED
            .into_iter()
            .flat_map(phase7_witness_policy_paths)
            .any(|path| !semantic_paths.contains(path))
        {
            return Err(Phase7PolicyError::UnregisteredWitnessPolicy);
        }
        fields.sort_unstable_by(|left, right| left.semantic_path().cmp(right.semantic_path()));
        let canonical = serde_json::to_vec(&fields)
            .map_err(|error| Phase7PolicyError::Canonicalization(error.to_string()))?;
        let mut hasher = Sha256::new();
        update_hash_field(&mut hasher, raw.profile_id.as_bytes());
        update_hash_field(&mut hasher, &raw.version.to_be_bytes());
        update_hash_field(&mut hasher, &canonical);

        Ok(Self {
            profile_id: raw.profile_id,
            version: ToleranceProfileVersion::CURRENT,
            fields: fields.into_boxed_slice(),
            profile_sha256: Sha256Hex::from_digest(hasher.finalize().into()),
        })
    }

    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    #[must_use]
    pub const fn version(&self) -> ToleranceProfileVersion {
        self.version
    }

    #[must_use]
    pub fn fields(&self) -> &[FieldPolicy] {
        &self.fields
    }

    #[must_use]
    pub const fn profile_sha256(&self) -> &Sha256Hex {
        &self.profile_sha256
    }

    /// Looks up an explicit policy. Unknown paths have no fallback.
    #[must_use]
    pub fn field(&self, semantic_path: &str) -> Option<&FieldPolicy> {
        self.fields
            .binary_search_by_key(&semantic_path, |field| field.semantic_path())
            .ok()
            .map(|index| &self.fields[index])
    }
}

fn phase7_witness_policy_paths(family: RigidWorldWitnessFamily) -> &'static [&'static str] {
    match family {
        RigidWorldWitnessFamily::BodyControlAndForcePolicy => &[
            "rigid_world.phase7.body.id",
            "rigid_world.phase7.body.awake",
            "rigid_world.phase7.body.linear_velocity.x",
        ],
        RigidWorldWitnessFamily::MultiContactIslandAndWarmStart => &[
            "rigid_world.phase7.island.body_order",
            "rigid_world.phase7.island.contact_order",
            "rigid_world.phase7.contact.normal_impulse",
        ],
        RigidWorldWitnessFamily::SleepingAndWaking => &[
            "rigid_world.phase7.body.awake",
            "rigid_world.phase7.step.completion",
        ],
        RigidWorldWitnessFamily::ContinuousCollisionAndSubStepping => &[
            "rigid_world.phase7.body.bullet",
            "rigid_world.phase7.body.transform.position.x",
            "rigid_world.phase7.step.completion",
        ],
        RigidWorldWitnessFamily::ContinuousBudgetResume => &[
            "rigid_world.phase7.step.outcome.kind",
            "rigid_world.phase7.step.partial_classification",
            "rigid_world.phase7.step.completion",
        ],
        RigidWorldWitnessFamily::WorldQueryAndRayCast => &[
            "rigid_world.phase7.query.occurrences.identity",
            "rigid_world.phase7.ray.equal_minimum.identities",
            "rigid_world.phase7.ray.completion",
        ],
        RigidWorldWitnessFamily::OriginShiftCovariance => &[
            "rigid_world.phase7.origin_shift.x",
            "rigid_world.phase7.origin_shift.y",
            "rigid_world.phase7.query.occurrences.identity",
            "rigid_world.phase7.ray.fraction",
        ],
        RigidWorldWitnessFamily::NonCollidingBodyFixtureLifecycle
        | RigidWorldWitnessFamily::SingleContactLifecycle => &[],
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Phase7PolicyError {
    #[error("phase7 policy TOML is invalid: {0}")]
    Toml(toml::de::Error),
    #[error("phase7 policy identity or version is unsupported")]
    UnsupportedIdentity,
    #[error("phase7 policy must classify every observable exactly once")]
    IncompleteProfile,
    #[error("semantic path is invalid or unregistered")]
    InvalidSemanticPath,
    #[error("duplicate semantic path: {0}")]
    DuplicateSemanticPath(Box<str>),
    #[error("phase7 field policy has incompatible comparison metadata")]
    IncompatibleMetadata,
    #[error("phase7 witness family has no complete policy registration")]
    UnregisteredWitnessPolicy,
    #[error("phase7 policy canonicalization failed: {0}")]
    Canonicalization(String),
}

fn validate_phase7_field(field: &FieldPolicy) -> Result<(), Phase7PolicyError> {
    validate_path(field.semantic_path()).map_err(|_| Phase7PolicyError::InvalidSemanticPath)?;
    if field.justification().is_empty() || field.justification().len() > MAXIMUM_JUSTIFICATION_BYTES
    {
        return Err(Phase7PolicyError::IncompatibleMetadata);
    }
    if field.horizon() != DivergenceHorizon::PhaseLocal
        || field.zero_policy() != ZeroPolicy::Distinct
        || field.non_finite_policy() != NonFinitePolicy::RejectArithmeticNaN
    {
        return Err(Phase7PolicyError::IncompatibleMetadata);
    }
    let path = field.semantic_path();
    if PHASE7_STRUCTURAL_PATHS.contains(&path) {
        let expected_collection = if matches!(
            path,
            "rigid_world.phase7.query.occurrences.identity" | "rigid_world.phase7.ray.hit.identity"
        ) {
            CollectionPolicy::Multiset
        } else if path == "rigid_world.phase7.ray.equal_minimum.identities" {
            CollectionPolicy::Set
        } else {
            CollectionPolicy::Ordered
        };
        if field.comparison() != FieldComparison::ExactDiscrete
            || field.evidence_tier() != EvidenceTier::D1Canonical
            || field.collection_policy() != expected_collection
        {
            return Err(Phase7PolicyError::IncompatibleMetadata);
        }
        return Ok(());
    }
    if field.collection_policy() != CollectionPolicy::Ordered
        || field.evidence_tier() != EvidenceTier::D2Supported
    {
        return Err(Phase7PolicyError::IncompatibleMetadata);
    }
    let FieldComparison::Float { policy } = field.comparison() else {
        return Err(Phase7PolicyError::IncompatibleMetadata);
    };
    validate_float_thresholds(policy).map_err(|_| Phase7PolicyError::IncompatibleMetadata)?;
    let expected = if PHASE7_ABSOLUTE_RELATIVE_PATHS.contains(&path) {
        matches!(policy, FloatPolicy::AbsoluteRelative { .. })
    } else if PHASE7_ULP_PATHS.contains(&path) {
        matches!(policy, FloatPolicy::Ulps { .. })
    } else {
        return Err(Phase7PolicyError::InvalidSemanticPath);
    };
    expected
        .then_some(())
        .ok_or(Phase7PolicyError::IncompatibleMetadata)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase6PolicyProfile {
    profile_id: Box<str>,
    version: ToleranceProfileVersion,
    fields: Box<[FieldPolicy]>,
    profile_sha256: Sha256Hex,
}

impl Phase6PolicyProfile {
    /// Parses, validates, sorts, and hashes one closed Phase 6 profile.
    ///
    /// # Errors
    ///
    /// Returns [`Phase6PolicyError`] when identity, completeness, comparison,
    /// horizon, tier, collection, or edge-case metadata is not exact.
    pub fn parse_toml(input: &str) -> Result<Self, Phase6PolicyError> {
        let raw: RawProfile = toml::from_str(input).map_err(Phase6PolicyError::Toml)?;
        if raw.profile_id.as_ref() != "phase6-v1"
            || raw.version != ToleranceProfileVersion::CURRENT.get()
        {
            return Err(Phase6PolicyError::UnsupportedIdentity);
        }
        if raw.fields.len() != STRUCTURAL_PATHS.len() + FLOAT_PATHS.len() {
            return Err(Phase6PolicyError::IncompleteProfile);
        }

        let mut fields = raw.fields;
        let mut semantic_paths = HashSet::with_capacity(fields.len());
        for field in &fields {
            validate_field(field)?;
            if !semantic_paths.insert(field.semantic_path()) {
                return Err(Phase6PolicyError::DuplicateSemanticPath(
                    field.semantic_path().into(),
                ));
            }
        }
        if STRUCTURAL_PATHS
            .iter()
            .chain(FLOAT_PATHS)
            .any(|path| !semantic_paths.contains(path))
        {
            return Err(Phase6PolicyError::IncompleteProfile);
        }
        fields.sort_unstable_by(|left, right| left.semantic_path().cmp(right.semantic_path()));

        let canonical = serde_json::to_vec(&fields)
            .map_err(|error| Phase6PolicyError::Canonicalization(error.to_string()))?;
        let mut hasher = Sha256::new();
        update_hash_field(&mut hasher, raw.profile_id.as_bytes());
        update_hash_field(&mut hasher, &raw.version.to_be_bytes());
        update_hash_field(&mut hasher, &canonical);

        Ok(Self {
            profile_id: raw.profile_id,
            version: ToleranceProfileVersion::CURRENT,
            fields: fields.into_boxed_slice(),
            profile_sha256: Sha256Hex::from_digest(hasher.finalize().into()),
        })
    }

    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    #[must_use]
    pub const fn version(&self) -> ToleranceProfileVersion {
        self.version
    }

    #[must_use]
    pub fn fields(&self) -> &[FieldPolicy] {
        &self.fields
    }

    #[must_use]
    pub const fn profile_sha256(&self) -> &Sha256Hex {
        &self.profile_sha256
    }

    #[must_use]
    pub fn field(&self, semantic_path: &str) -> Option<&FieldPolicy> {
        self.fields
            .binary_search_by_key(&semantic_path, |field| field.semantic_path())
            .ok()
            .map(|index| &self.fields[index])
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Phase6PolicyError {
    #[error("phase6 policy TOML is invalid: {0}")]
    Toml(toml::de::Error),
    #[error("phase6 policy identity or version is unsupported")]
    UnsupportedIdentity,
    #[error("phase6 policy must classify every reviewed observable exactly once")]
    IncompleteProfile,
    #[error("semantic path is invalid or default-like")]
    InvalidSemanticPath,
    #[error("duplicate semantic path: {0}")]
    DuplicateSemanticPath(Box<str>),
    #[error("floating threshold must be finite and nonnegative")]
    InvalidThreshold,
    #[error("phase6 observables begin with exact comparison")]
    NonExactComparison,
    #[error("phase6 policy carries incompatible horizon, tier, collection, or edge metadata")]
    IncompatibleMetadata,
    #[error("justification must be nonempty and at most 512 bytes")]
    InvalidJustification,
    #[error("phase6 policy canonicalization failed: {0}")]
    Canonicalization(String),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProfile {
    profile_id: Box<str>,
    version: u32,
    fields: Vec<FieldPolicy>,
}

fn validate_field(field: &FieldPolicy) -> Result<(), Phase6PolicyError> {
    validate_path(field.semantic_path())?;
    if field.justification().is_empty() || field.justification().len() > MAXIMUM_JUSTIFICATION_BYTES
    {
        return Err(Phase6PolicyError::InvalidJustification);
    }
    if field.collection_policy() != CollectionPolicy::Ordered
        || field.horizon() != DivergenceHorizon::PhaseLocal
        || field.zero_policy() != ZeroPolicy::Distinct
        || field.non_finite_policy() != NonFinitePolicy::RejectArithmeticNaN
    {
        return Err(Phase6PolicyError::IncompatibleMetadata);
    }

    let path = field.semantic_path();
    if STRUCTURAL_PATHS.contains(&path) {
        if field.comparison() != FieldComparison::ExactDiscrete
            || field.evidence_tier() != EvidenceTier::D1Canonical
        {
            return Err(Phase6PolicyError::IncompatibleMetadata);
        }
        return Ok(());
    }
    if !FLOAT_PATHS.contains(&path) {
        return Err(Phase6PolicyError::InvalidSemanticPath);
    }

    let FieldComparison::Float { policy } = field.comparison() else {
        return Err(Phase6PolicyError::IncompatibleMetadata);
    };
    validate_float_thresholds(policy)?;
    if policy != FloatPolicy::ExactBits {
        return Err(Phase6PolicyError::NonExactComparison);
    }
    if field.evidence_tier() != EvidenceTier::D2Supported {
        return Err(Phase6PolicyError::IncompatibleMetadata);
    }
    Ok(())
}

fn validate_path(path: &str) -> Result<(), Phase6PolicyError> {
    let default_like = path.split('.').any(|segment| {
        matches!(
            segment.to_ascii_lowercase().as_str(),
            "*" | "**" | "any" | "all" | "default" | "fallback"
        )
    });
    if path.is_empty()
        || path.len() > MAXIMUM_SEMANTIC_PATH_BYTES
        || path
            .chars()
            .any(|character| matches!(character, '*' | '?' | '[' | ']'))
        || default_like
    {
        return Err(Phase6PolicyError::InvalidSemanticPath);
    }
    Ok(())
}

fn validate_float_thresholds(policy: FloatPolicy) -> Result<(), Phase6PolicyError> {
    let valid = |bits: crate::FloatBits| {
        let value = bits.to_f32();
        value.is_finite() && value >= 0.0
    };
    match policy {
        FloatPolicy::Absolute { max_bits } => valid(max_bits),
        FloatPolicy::AbsoluteRelative {
            absolute_bits,
            relative_bits,
        } => valid(absolute_bits) && valid(relative_bits),
        FloatPolicy::ExactBits | FloatPolicy::Ulps { .. } => true,
    }
    .then_some(())
    .ok_or(Phase6PolicyError::InvalidThreshold)
}

fn update_hash_field(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}

#[cfg(test)]
fn render_phase6_policy_presentation(profile: &Phase6PolicyProfile) -> String {
    let mut rendered = format!(
        "profile_id = \"{}\"\nversion = {}\n\n",
        profile.profile_id(),
        profile.version().get()
    );
    for path in STRUCTURAL_PATHS.iter().chain(FLOAT_PATHS) {
        let field = profile
            .field(path)
            .expect("closed Phase 6 registry requires every rendered field");
        let comparison = match field.comparison() {
            FieldComparison::ExactDiscrete => "{ kind = \"exact_discrete\" }",
            FieldComparison::Float {
                policy: FloatPolicy::ExactBits,
            } => "{ kind = \"float\", policy = { kind = \"exact_bits\" } }",
            FieldComparison::Float { .. } => {
                unreachable!("validated Phase 6 policy begins with exact comparisons")
            }
        };
        let tier = match field.evidence_tier() {
            EvidenceTier::D1Canonical => "d1_canonical",
            EvidenceTier::D2Supported => "d2_supported",
            _ => unreachable!("validated Phase 6 policy carries only D1 or D2 authority"),
        };
        rendered.push_str(&format!(
            concat!(
                "[[fields]]\n",
                "semantic_path = \"{}\"\n",
                "comparison = {}\n",
                "zero_policy = \"distinct\"\n",
                "non_finite_policy = \"reject_arithmetic_nan\"\n",
                "collection_policy = \"ordered\"\n",
                "horizon = {{ kind = \"phase_local\" }}\n",
                "evidence_tier = \"{}\"\n",
                "justification = \"{}\"\n\n"
            ),
            path,
            comparison,
            tier,
            field.justification()
        ));
    }
    rendered.pop();
    rendered
}

#[cfg(test)]
mod tests {
    use super::{
        Phase6PolicyError, Phase6PolicyProfile, Phase7PolicyError, Phase7PolicyProfile,
        render_phase6_policy_presentation,
    };
    use crate::{CollectionPolicy, FieldComparison, FloatPolicy};

    const PROFILE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../protocol/tolerances/phase6-v1.toml"
    ));
    const PHASE7_PROFILE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../protocol/tolerances/phase7-v1.toml"
    ));

    #[test]
    fn rigid_policy_requires_one_explicit_rule_per_observable() {
        // Arrange and Act
        let profile = Phase6PolicyProfile::parse_toml(PROFILE)
            .expect("checked-in Phase 6 policy should validate");

        // Assert
        assert_eq!(profile.profile_id(), "phase6-v1");
        assert_eq!(profile.fields().len(), 57);
        assert!(
            profile
                .field("rigid_world.body.transform.position.x")
                .is_some()
        );
        assert!(profile.field("rigid_world.body.mass").is_some());
        assert!(
            profile
                .field("rigid_world.contact.manifold.point.normal_impulse")
                .is_some()
        );
        assert!(
            profile
                .field("rigid_world.contact.manifold.point.tangent_impulse")
                .is_some()
        );
        assert!(profile.field("rigid_world.unclassified").is_none());
        assert_eq!(profile.profile_sha256().as_str().len(), 64);
        assert_eq!(render_phase6_policy_presentation(&profile), PROFILE);
    }

    #[test]
    fn rigid_policy_rejects_missing_duplicate_and_default_like_paths() {
        // Arrange
        let first_field = PROFILE
            .split("[[fields]]")
            .nth(1)
            .expect("profile contains a field");
        let missing = PROFILE.replacen(&format!("[[fields]]{first_field}"), "", 1);
        let duplicate = format!("{PROFILE}\n[[fields]]{first_field}");
        let wildcard = PROFILE.replacen(
            "rigid_world.result.request_id",
            "rigid_world.*.request_id",
            1,
        );
        let fallback = PROFILE.replacen(
            "rigid_world.result.request_id",
            "rigid_world.fallback.request_id",
            1,
        );

        // Act
        let errors = [missing, duplicate, wildcard, fallback].map(|input| {
            Phase6PolicyProfile::parse_toml(&input).expect_err("incomplete policy must fail")
        });

        // Assert
        assert!(matches!(errors[0], Phase6PolicyError::IncompleteProfile));
        assert!(matches!(errors[1], Phase6PolicyError::IncompleteProfile));
        assert!(matches!(errors[2], Phase6PolicyError::InvalidSemanticPath));
        assert!(matches!(errors[3], Phase6PolicyError::InvalidSemanticPath));
    }

    #[test]
    fn rigid_policy_rejects_a_duplicate_without_hiding_it_behind_count_validation() {
        // Arrange
        let first_field = PROFILE
            .split("[[fields]]")
            .nth(1)
            .expect("profile contains a field");
        let second_field = PROFILE
            .split("[[fields]]")
            .nth(2)
            .expect("profile contains a second field");
        let duplicate = PROFILE.replacen(second_field, first_field, 1);

        // Act
        let error = Phase6PolicyProfile::parse_toml(&duplicate)
            .expect_err("duplicate semantic paths must fail");

        // Assert
        assert!(matches!(error, Phase6PolicyError::DuplicateSemanticPath(_)));
    }

    #[test]
    fn rigid_policy_rejects_threshold_horizon_tier_and_collection_changes() {
        // Arrange
        let threshold = PROFILE.replacen(
            "policy = { kind = \"exact_bits\" }",
            "policy = { kind = \"absolute\", max_bits = 2139095040 }",
            1,
        );
        let horizon = PROFILE.replacen(
            "horizon = { kind = \"phase_local\" }",
            "horizon = { kind = \"scenario_steps\", steps = 2 }",
            1,
        );
        let tier = PROFILE.replacen(
            "evidence_tier = \"d1_canonical\"",
            "evidence_tier = \"d2_supported\"",
            1,
        );
        let collection = PROFILE.replacen(
            "collection_policy = \"ordered\"",
            "collection_policy = \"set\"",
            1,
        );

        // Act
        let errors = [threshold, horizon, tier, collection].map(|input| {
            Phase6PolicyProfile::parse_toml(&input).expect_err("policy widening must fail")
        });

        // Assert
        assert!(matches!(errors[0], Phase6PolicyError::InvalidThreshold));
        assert!(matches!(errors[1], Phase6PolicyError::IncompatibleMetadata));
        assert!(matches!(errors[2], Phase6PolicyError::IncompatibleMetadata));
        assert!(matches!(errors[3], Phase6PolicyError::IncompatibleMetadata));
    }

    #[test]
    fn rigid_policy_phase7_closes_structural_collection_and_numeric_rules() {
        // Arrange and Act
        let profile = Phase7PolicyProfile::parse_toml(PHASE7_PROFILE)
            .expect("checked-in Phase 7 policy should validate");

        // Assert
        assert_eq!(profile.profile_id(), "phase7-v1");
        assert_eq!(profile.fields().len(), 36);
        assert_eq!(
            profile.profile_sha256().as_str(),
            "54ed48d847ad6e075e0d07e8d018ed65c131a89c2942e91c7dde631db7e85b9e"
        );
        assert_eq!(
            profile
                .field("rigid_world.phase7.query.occurrences.identity")
                .expect("query occurrence policy")
                .collection_policy(),
            CollectionPolicy::Multiset
        );
        assert_eq!(
            profile
                .field("rigid_world.phase7.ray.equal_minimum.identities")
                .expect("ray tie policy")
                .collection_policy(),
            CollectionPolicy::Set
        );
        assert_eq!(
            profile
                .field("rigid_world.phase7.ray.hit.identity")
                .expect("ray hit identity policy")
                .collection_policy(),
            CollectionPolicy::Multiset
        );
        assert!(matches!(
            profile
                .field("rigid_world.phase7.ray.fraction")
                .expect("ray fraction policy")
                .comparison(),
            FieldComparison::Float {
                policy: FloatPolicy::Ulps { max: 4 }
            }
        ));
        assert!(profile.field("rigid_world.phase7.unregistered").is_none());
        for unsupported in [
            "rigid_world.phase7.warm_start.enabled",
            "rigid_world.phase7.force_clearing.enabled",
            "rigid_world.phase7.query.directive_trace",
            "rigid_world.phase7.ray.directive_trace",
            "rigid_world.phase7.origin_shift.topology",
            "rigid_world.phase7.continuous.signed_separation",
        ] {
            assert!(profile.field(unsupported).is_none());
        }
    }

    #[test]
    fn rigid_policy_phase7_rejects_unknown_missing_and_widened_rules() {
        // Arrange
        let unknown = PHASE7_PROFILE.replacen(
            "rigid_world.phase7.body.id",
            "rigid_world.phase7.body.default",
            1,
        );
        let first_field = PHASE7_PROFILE
            .split("[[fields]]")
            .nth(1)
            .expect("profile contains a field");
        let missing = PHASE7_PROFILE.replacen(&format!("[[fields]]{first_field}"), "", 1);
        let widened = PHASE7_PROFILE.replacen(
            "collection_policy = \"multiset\"",
            "collection_policy = \"set\"",
            1,
        );

        // Act
        let errors = [unknown, missing, widened].map(|input| {
            Phase7PolicyProfile::parse_toml(&input).expect_err("open policy must fail closed")
        });

        // Assert
        assert!(matches!(errors[0], Phase7PolicyError::InvalidSemanticPath));
        assert!(matches!(errors[1], Phase7PolicyError::IncompleteProfile));
        assert!(matches!(errors[2], Phase7PolicyError::IncompatibleMetadata));
    }
}

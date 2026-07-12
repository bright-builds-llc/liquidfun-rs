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
use crate::{Sha256Hex, ToleranceProfileVersion};

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
    use super::{Phase6PolicyError, Phase6PolicyProfile, render_phase6_policy_presentation};

    const PROFILE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../protocol/tolerances/phase6-v1.toml"
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
}

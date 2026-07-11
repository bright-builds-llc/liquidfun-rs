use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{CollectionPolicy, FloatPolicy};
use crate::{Sha256Hex, ToleranceProfileVersion};

const MAXIMUM_SEMANTIC_PATH_BYTES: usize = 256;
const MAXIMUM_JUSTIFICATION_BYTES: usize = 512;
const MAXIMUM_SCENARIO_STEPS: u32 = 1_000_000;

/// Policy for the sign bit of numerically equal zero values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZeroPolicy {
    /// Keep positive and negative zero distinct.
    Distinct,
    /// Treat zero signs as semantically equivalent for this field.
    SignInsensitive,
}

/// Policy for non-finite floating-point results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NonFinitePolicy {
    /// Any arithmetic NaN or infinity is a mismatch.
    #[serde(rename = "reject_arithmetic_nan")]
    RejectArithmeticNaN,
    /// Compare exact transported IEEE-754 bits without arithmetic.
    ExactBitsTransport,
    /// Permit infinities only when their signs agree; NaN still mismatches.
    SameSignInfinity,
}

/// Bounded evidence horizon for one semantic field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DivergenceHorizon {
    /// One pure operation.
    Operation,
    /// One named algorithm or solver phase.
    PhaseLocal,
    /// A fixed number of repeated scenario steps.
    ScenarioSteps {
        /// Nonzero bounded step count.
        steps: u32,
    },
}

/// Authority tier attached to compatibility evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceTier {
    /// Byte-identical replay of one unchanged build.
    D0Replay,
    /// Pinned canonical Linux scalar comparison.
    D1Canonical,
    /// Reviewed supported-platform comparison.
    D2Supported,
    /// Diagnostic-only experimental evidence.
    D3Exploratory,
}

/// Closed comparison rule for one semantic field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FieldComparison {
    /// Apply an authoritative floating-point rule.
    Float {
        /// Exact, absolute, absolute-relative, or ULP policy.
        policy: FloatPolicy,
    },
    /// Require exact equality for a discrete field.
    ExactDiscrete,
}

/// Complete reviewed comparison policy for one semantic path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldPolicy {
    semantic_path: Box<str>,
    comparison: FieldComparison,
    zero_policy: ZeroPolicy,
    non_finite_policy: NonFinitePolicy,
    collection_policy: CollectionPolicy,
    horizon: DivergenceHorizon,
    evidence_tier: EvidenceTier,
    justification: Box<str>,
}

impl FieldPolicy {
    /// Returns the stable semantic path.
    #[must_use]
    pub fn semantic_path(&self) -> &str {
        &self.semantic_path
    }

    /// Returns the field comparison rule.
    #[must_use]
    pub const fn comparison(&self) -> FieldComparison {
        self.comparison
    }

    /// Returns the signed-zero policy.
    #[must_use]
    pub const fn zero_policy(&self) -> ZeroPolicy {
        self.zero_policy
    }

    /// Returns the non-finite policy.
    #[must_use]
    pub const fn non_finite_policy(&self) -> NonFinitePolicy {
        self.non_finite_policy
    }

    /// Returns the collection semantics.
    #[must_use]
    pub const fn collection_policy(&self) -> CollectionPolicy {
        self.collection_policy
    }

    /// Returns the fixed evidence horizon.
    #[must_use]
    pub const fn horizon(&self) -> DivergenceHorizon {
        self.horizon
    }

    /// Returns the evidence authority tier.
    #[must_use]
    pub const fn evidence_tier(&self) -> EvidenceTier {
        self.evidence_tier
    }

    /// Returns the source or probe justification.
    #[must_use]
    pub fn justification(&self) -> &str {
        &self.justification
    }
}

/// Strict validated Phase 4 comparison profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase4PolicyProfile {
    profile_id: Box<str>,
    version: ToleranceProfileVersion,
    fields: Box<[FieldPolicy]>,
    profile_sha256: Sha256Hex,
}

impl Phase4PolicyProfile {
    /// Parses, validates, sorts, and hashes one Phase 4 TOML profile.
    ///
    /// # Errors
    ///
    /// Returns [`Phase4PolicyError`] for malformed TOML or an invalid field policy.
    pub fn parse_toml(input: &str) -> Result<Self, Phase4PolicyError> {
        let raw: RawProfile = toml::from_str(input).map_err(Phase4PolicyError::Toml)?;
        if raw.profile_id.as_ref() != "phase4-v1"
            || raw.version != ToleranceProfileVersion::CURRENT.get()
        {
            return Err(Phase4PolicyError::UnsupportedIdentity);
        }
        if raw.fields.is_empty() {
            return Err(Phase4PolicyError::EmptyProfile);
        }

        let mut fields = raw.fields;
        let mut semantic_paths = HashSet::with_capacity(fields.len());
        for field in &fields {
            validate_field(field)?;
            if !semantic_paths.insert(field.semantic_path.as_ref()) {
                return Err(Phase4PolicyError::DuplicateSemanticPath(
                    field.semantic_path.clone(),
                ));
            }
        }
        fields.sort_unstable_by(|left, right| left.semantic_path.cmp(&right.semantic_path));

        let canonical = serde_json::to_vec(&fields)
            .map_err(|error| Phase4PolicyError::Canonicalization(error.to_string()))?;
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

    /// Returns the stable profile identifier.
    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    /// Returns the tolerance schema version.
    #[must_use]
    pub const fn version(&self) -> ToleranceProfileVersion {
        self.version
    }

    /// Returns policies in stable semantic-path order.
    #[must_use]
    pub fn fields(&self) -> &[FieldPolicy] {
        &self.fields
    }

    /// Returns the deterministic identity of every profile field.
    #[must_use]
    pub const fn profile_sha256(&self) -> &Sha256Hex {
        &self.profile_sha256
    }

    /// Finds one explicit field policy; there is no implicit fallback.
    #[must_use]
    pub fn field(&self, semantic_path: &str) -> Option<&FieldPolicy> {
        self.fields
            .binary_search_by_key(&semantic_path, |field| field.semantic_path())
            .ok()
            .map(|index| &self.fields[index])
    }
}

/// Validation failure for a Phase 4 policy profile.
#[derive(Debug, thiserror::Error)]
pub enum Phase4PolicyError {
    /// The TOML representation is malformed or contains unknown fields.
    #[error("phase4 policy TOML is invalid: {0}")]
    Toml(toml::de::Error),
    /// Profile identity or version is unsupported.
    #[error("phase4 policy identity or version is unsupported")]
    UnsupportedIdentity,
    /// No explicit semantic policies were supplied.
    #[error("phase4 policy must contain at least one field")]
    EmptyProfile,
    /// A semantic path is empty or oversized.
    #[error("semantic path must be nonempty and at most 256 bytes")]
    InvalidSemanticPath,
    /// A semantic path occurs more than once.
    #[error("duplicate semantic path: {0}")]
    DuplicateSemanticPath(Box<str>),
    /// A threshold is negative, NaN, or infinite.
    #[error("floating threshold must be finite and nonnegative")]
    InvalidThreshold,
    /// A repeated-step horizon is zero or exceeds the reviewed bound.
    #[error("scenario step horizon must be in 1..=1000000")]
    InvalidHorizon,
    /// A policy combination has contradictory semantics.
    #[error("field policy combines incompatible comparison and edge-case rules")]
    IncompatibleRules,
    /// A source/probe justification is empty or oversized.
    #[error("justification must be nonempty and at most 512 bytes")]
    InvalidJustification,
    /// Deterministic canonicalization failed unexpectedly.
    #[error("phase4 policy canonicalization failed: {0}")]
    Canonicalization(String),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProfile {
    profile_id: Box<str>,
    version: u32,
    fields: Vec<FieldPolicy>,
}

fn validate_field(field: &FieldPolicy) -> Result<(), Phase4PolicyError> {
    let path_is_default_like = field.semantic_path.split('.').any(|segment| {
        matches!(
            segment.to_ascii_lowercase().as_str(),
            "*" | "**" | "any" | "all" | "default" | "fallback"
        )
    });
    if field.semantic_path.is_empty()
        || field.semantic_path.len() > MAXIMUM_SEMANTIC_PATH_BYTES
        || field
            .semantic_path
            .chars()
            .any(|character| matches!(character, '*' | '?' | '[' | ']'))
        || path_is_default_like
    {
        return Err(Phase4PolicyError::InvalidSemanticPath);
    }
    if field.justification.is_empty() || field.justification.len() > MAXIMUM_JUSTIFICATION_BYTES {
        return Err(Phase4PolicyError::InvalidJustification);
    }
    if let DivergenceHorizon::ScenarioSteps { steps } = field.horizon
        && (steps == 0 || steps > MAXIMUM_SCENARIO_STEPS)
    {
        return Err(Phase4PolicyError::InvalidHorizon);
    }
    if let FieldComparison::Float { policy } = field.comparison {
        validate_float_policy(policy)?;
        if matches!(policy, FloatPolicy::ExactBits)
            && field.zero_policy == ZeroPolicy::SignInsensitive
        {
            return Err(Phase4PolicyError::IncompatibleRules);
        }
    } else if field.zero_policy != ZeroPolicy::Distinct
        || field.non_finite_policy != NonFinitePolicy::RejectArithmeticNaN
    {
        return Err(Phase4PolicyError::IncompatibleRules);
    }
    Ok(())
}

fn validate_float_policy(policy: FloatPolicy) -> Result<(), Phase4PolicyError> {
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
    .ok_or(Phase4PolicyError::InvalidThreshold)
}

fn update_hash_field(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update(bytes.len().to_be_bytes());
    hasher.update(bytes);
}

#[cfg(test)]
mod tests {
    use super::{EvidenceTier, Phase4PolicyError, Phase4PolicyProfile};

    const VALID: &str = r#"
profile_id = "phase4-v1"
version = 1

[[fields]]
semantic_path = "math.constants.pi"
comparison = { kind = "float", policy = { kind = "exact_bits" } }
zero_policy = "distinct"
non_finite_policy = "exact_bits_transport"
collection_policy = "ordered"
horizon = { kind = "operation" }
evidence_tier = "d1_canonical"
justification = "Pinned b2_pi token bit pattern."
"#;

    #[test]
    fn phase4_profile_parses_and_hashes_every_field() {
        // Arrange and Act
        let profile = Phase4PolicyProfile::parse_toml(VALID).expect("valid policy should parse");

        // Assert
        assert_eq!(profile.profile_id(), "phase4-v1");
        assert_eq!(profile.fields().len(), 1);
        assert_eq!(
            profile.fields()[0].evidence_tier(),
            EvidenceTier::D1Canonical
        );
        assert_eq!(profile.profile_sha256().as_str().len(), 64);
    }

    #[test]
    fn phase4_profile_hash_is_independent_of_input_field_order() {
        // Arrange
        let second = VALID.replace(
            "[[fields]]",
            "[[fields]]\nsemantic_path = \"math.branch.valid\"\ncomparison = { kind = \"exact_discrete\" }\nzero_policy = \"distinct\"\nnon_finite_policy = \"reject_arithmetic_nan\"\ncollection_policy = \"ordered\"\nhorizon = { kind = \"operation\" }\nevidence_tier = \"d1_canonical\"\njustification = \"Pinned validity branch.\"\n\n[[fields]]",
        );
        let reversed = second.split("\n\n[[fields]]").collect::<Vec<_>>();
        let reordered = format!(
            "{}\n\n[[fields]]{}\n\n[[fields]]{}",
            reversed[0], reversed[2], reversed[1]
        );

        // Act
        let first_profile =
            Phase4PolicyProfile::parse_toml(&second).expect("first policy should parse");
        let second_profile =
            Phase4PolicyProfile::parse_toml(&reordered).expect("reordered policy should parse");

        // Assert
        assert_eq!(
            first_profile.profile_sha256(),
            second_profile.profile_sha256()
        );
    }

    #[test]
    fn phase4_profile_rejects_duplicate_paths() {
        // Arrange
        let duplicated = format!(
            "{VALID}\n{}",
            VALID
                .split("[[fields]]")
                .nth(1)
                .expect("field exists")
                .prepend("[[fields]]")
        );

        // Act
        let error =
            Phase4PolicyProfile::parse_toml(&duplicated).expect_err("duplicate paths must fail");

        // Assert
        assert!(matches!(error, Phase4PolicyError::DuplicateSemanticPath(_)));
    }

    trait Prepend {
        fn prepend(self, prefix: &str) -> String;
    }

    impl Prepend for &str {
        fn prepend(self, prefix: &str) -> String {
            format!("{prefix}{self}")
        }
    }

    #[test]
    fn phase4_profile_rejects_zero_step_horizon() {
        // Arrange
        let invalid = VALID.replace(
            "horizon = { kind = \"operation\" }",
            "horizon = { kind = \"scenario_steps\", steps = 0 }",
        );

        // Act
        let error = Phase4PolicyProfile::parse_toml(&invalid).expect_err("zero steps must fail");

        // Assert
        assert!(matches!(error, Phase4PolicyError::InvalidHorizon));
    }

    #[test]
    fn phase4_profile_rejects_nonfinite_threshold() {
        // Arrange
        let invalid = VALID.replace(
            "policy = { kind = \"exact_bits\" }",
            "policy = { kind = \"absolute\", max_bits = 2139095040 }",
        );

        // Act
        let error =
            Phase4PolicyProfile::parse_toml(&invalid).expect_err("infinite threshold must fail");

        // Assert
        assert!(matches!(error, Phase4PolicyError::InvalidThreshold));
    }

    #[test]
    fn phase4_profile_has_no_implicit_fallback() {
        // Arrange
        let profile = Phase4PolicyProfile::parse_toml(VALID).expect("valid policy should parse");

        // Act
        let missing = profile.field("math.unreviewed.field");

        // Assert
        assert!(missing.is_none());
    }

    #[test]
    fn phase4_profile_rejects_wildcard_and_default_like_paths() {
        // Arrange
        let invalid_paths = ["math.*.value", "math.default.value", "math.fallback.value"];

        // Act
        let errors = invalid_paths.map(|path| {
            Phase4PolicyProfile::parse_toml(&VALID.replace("math.constants.pi", path))
                .expect_err("default-like policy paths must fail")
        });

        // Assert
        assert!(
            errors
                .iter()
                .all(|error| matches!(error, Phase4PolicyError::InvalidSemanticPath))
        );
    }
}

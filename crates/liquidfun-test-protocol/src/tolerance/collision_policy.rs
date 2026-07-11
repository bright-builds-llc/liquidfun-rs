#![allow(
    missing_docs,
    reason = "closed private-harness policy errors and accessors are self-describing"
)]

use std::collections::HashSet;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::{
    CollectionPolicy, DivergenceHorizon, EvidenceTier, FieldComparison, FieldPolicy,
    NonFinitePolicy,
};
use crate::{Sha256Hex, ToleranceProfileVersion};

const MAXIMUM_SEMANTIC_PATH_BYTES: usize = 256;
const MAXIMUM_JUSTIFICATION_BYTES: usize = 512;
const EXPECTED_FIELD_COUNT: usize = 17;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase5PolicyProfile {
    profile_id: Box<str>,
    version: ToleranceProfileVersion,
    fields: Box<[FieldPolicy]>,
    profile_sha256: Sha256Hex,
}

impl Phase5PolicyProfile {
    /// Parses, validates, sorts, and hashes one closed Phase 5 profile.
    ///
    /// # Errors
    ///
    /// Returns [`Phase5PolicyError`] for malformed TOML, missing operation
    /// policies, wildcards, unsupported thresholds, or contradictory metadata.
    pub fn parse_toml(input: &str) -> Result<Self, Phase5PolicyError> {
        let raw: RawProfile = toml::from_str(input).map_err(Phase5PolicyError::Toml)?;
        if raw.profile_id.as_ref() != "phase5-v1"
            || raw.version != ToleranceProfileVersion::CURRENT.get()
        {
            return Err(Phase5PolicyError::UnsupportedIdentity);
        }
        if raw.fields.len() != EXPECTED_FIELD_COUNT {
            return Err(Phase5PolicyError::IncompleteProfile);
        }

        let mut fields = raw.fields;
        let mut semantic_paths = HashSet::with_capacity(fields.len());
        for field in &fields {
            validate_field(field)?;
            if !semantic_paths.insert(field.semantic_path()) {
                return Err(Phase5PolicyError::DuplicateSemanticPath(
                    field.semantic_path().into(),
                ));
            }
        }
        for expected in
            crate::CollisionProbeOperation::ALL.map(crate::CollisionProbeOperation::policy_path)
        {
            if !semantic_paths.contains(expected) {
                return Err(Phase5PolicyError::IncompleteProfile);
            }
        }
        fields.sort_unstable_by(|left, right| left.semantic_path().cmp(right.semantic_path()));

        let canonical = serde_json::to_vec(&fields)
            .map_err(|error| Phase5PolicyError::Canonicalization(error.to_string()))?;
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
pub enum Phase5PolicyError {
    #[error("phase5 policy TOML is invalid: {0}")]
    Toml(toml::de::Error),
    #[error("phase5 policy identity or version is unsupported")]
    UnsupportedIdentity,
    #[error("phase5 policy must contain exactly one rule for every collision operation")]
    IncompleteProfile,
    #[error("semantic path is invalid or default-like")]
    InvalidSemanticPath,
    #[error("duplicate semantic path: {0}")]
    DuplicateSemanticPath(Box<str>),
    #[error("phase5 collision thresholds must begin at exact bits")]
    NonExactThreshold,
    #[error("collision policy metadata is inconsistent with the closed operation")]
    IncompatibleMetadata,
    #[error("justification must be nonempty and bounded")]
    InvalidJustification,
    #[error("phase5 policy canonicalization failed: {0}")]
    Canonicalization(String),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProfile {
    profile_id: Box<str>,
    version: u32,
    fields: Vec<FieldPolicy>,
}

fn validate_field(field: &FieldPolicy) -> Result<(), Phase5PolicyError> {
    let path = field.semantic_path();
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
        return Err(Phase5PolicyError::InvalidSemanticPath);
    }
    if field.justification().is_empty() || field.justification().len() > MAXIMUM_JUSTIFICATION_BYTES
    {
        return Err(Phase5PolicyError::InvalidJustification);
    }
    if !matches!(
        field.comparison(),
        FieldComparison::Float {
            policy: super::FloatPolicy::ExactBits
        }
    ) {
        return Err(Phase5PolicyError::NonExactThreshold);
    }
    if field.non_finite_policy() != NonFinitePolicy::RejectArithmeticNaN
        || field.evidence_tier() != EvidenceTier::D2Supported
    {
        return Err(Phase5PolicyError::IncompatibleMetadata);
    }
    let operation = crate::CollisionProbeOperation::ALL
        .into_iter()
        .find(|operation| operation.policy_path() == path)
        .ok_or(Phase5PolicyError::InvalidSemanticPath)?;
    let expected_horizon = match operation.expected_horizon() {
        crate::CollisionProbeHorizon::Operation => DivergenceHorizon::Operation,
        crate::CollisionProbeHorizon::PhaseLocal => DivergenceHorizon::PhaseLocal,
    };
    if field.horizon() != expected_horizon
        || field.collection_policy() != operation.expected_collection_policy()
        || !matches!(
            field.collection_policy(),
            CollectionPolicy::Ordered | CollectionPolicy::Set
        )
    {
        return Err(Phase5PolicyError::IncompatibleMetadata);
    }
    Ok(())
}

fn update_hash_field(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROFILE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../protocol/tolerances/phase5-v1.toml"
    ));

    #[test]
    fn collision_policy_requires_one_explicit_rule_per_operation() {
        // Arrange / Act
        let profile = Phase5PolicyProfile::parse_toml(PROFILE)
            .expect("checked-in Phase 5 policy should validate");

        // Assert
        assert_eq!(profile.fields().len(), 17);
        assert!(profile.field("collision.distance.result").is_some());
        assert!(profile.field("collision.time_of_impact.result").is_some());
        assert_eq!(
            profile.profile_sha256().as_str(),
            "78f237860a2cac803f0188fad909a0f0caf4bd7cce9c24eba7b8b6b9bb2f9c1f"
        );
    }

    #[test]
    fn collision_policy_rejects_wildcards_and_metadata_mismatch() {
        // Arrange
        let wildcard = PROFILE.replacen("collision.distance.result", "collision.*.result", 1);
        let collection = PROFILE.replacen(
            "collection_policy = \"set\"",
            "collection_policy = \"ordered\"",
            1,
        );

        // Act
        let wildcard_error =
            Phase5PolicyProfile::parse_toml(&wildcard).expect_err("wildcards must be rejected");
        let collection_error = Phase5PolicyProfile::parse_toml(&collection)
            .expect_err("collection mismatch must be rejected");

        // Assert
        assert!(matches!(
            wildcard_error,
            Phase5PolicyError::InvalidSemanticPath
        ));
        assert!(matches!(
            collection_error,
            Phase5PolicyError::IncompatibleMetadata
        ));
    }
}

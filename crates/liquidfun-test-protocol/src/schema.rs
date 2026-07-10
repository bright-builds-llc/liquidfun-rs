use std::collections::BTreeSet;

use serde::Deserialize;

use crate::{
    CollectionPolicy, DiscretePolicy, FloatBits, FloatPolicy, Sha256Hex, ToleranceProfile,
    ToleranceProfileVersion,
};

const PHASE2_DESCRIPTION: &str = "Phase 2 sets no broad rigid-body, joint, or particle tolerance values; synthetic numeric policies exist only for comparator coverage.";

#[derive(Debug, thiserror::Error)]
enum PresentationError {
    #[error("invalid tolerance profile TOML: {0}")]
    InvalidToml(#[from] toml::de::Error),
    #[error("tolerance profile presentation does not match typed authority: {0}")]
    AuthorityMismatch(&'static str),
    #[error("duplicate tolerance policy field `{0}`")]
    DuplicatePolicy(String),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ToleranceProfilePresentation {
    profile_id: String,
    version: ToleranceProfileVersion,
    profile_sha256: Sha256Hex,
    description: String,
    float_policies: Vec<FloatPolicyPresentation>,
    discrete_policies: Vec<DiscretePolicyPresentation>,
    collection_policies: Vec<CollectionPolicyPresentation>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FloatPolicyPresentation {
    field: String,
    scope: String,
    policy: PresentedFloatPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum PresentedFloatPolicy {
    ExactBits,
    Absolute {
        max_bits: FloatBits,
    },
    AbsoluteRelative {
        absolute_bits: FloatBits,
        relative_bits: FloatBits,
    },
    Ulps {
        max: u32,
    },
}

impl From<FloatPolicy> for PresentedFloatPolicy {
    fn from(policy: FloatPolicy) -> Self {
        match policy {
            FloatPolicy::ExactBits => Self::ExactBits,
            FloatPolicy::Absolute { max_bits } => Self::Absolute { max_bits },
            FloatPolicy::AbsoluteRelative {
                absolute_bits,
                relative_bits,
            } => Self::AbsoluteRelative {
                absolute_bits,
                relative_bits,
            },
            FloatPolicy::Ulps { max } => Self::Ulps { max },
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DiscretePolicyPresentation {
    field: String,
    kind: DiscretePolicy,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CollectionPolicyPresentation {
    field: String,
    kind: CollectionPolicy,
}

fn render_tolerance_profile_presentation() -> String {
    let profile = ToleranceProfile::phase2_v1();
    let [absolute, absolute_relative, ulps] = ToleranceProfile::synthetic_float_policies();
    let FloatPolicy::Absolute { max_bits } = absolute else {
        unreachable!("typed synthetic absolute policy must retain its variant");
    };
    let FloatPolicy::AbsoluteRelative {
        absolute_bits,
        relative_bits,
    } = absolute_relative
    else {
        unreachable!("typed synthetic absolute-relative policy must retain its variant");
    };
    let FloatPolicy::Ulps { max } = ulps else {
        unreachable!("typed synthetic ULP policy must retain its variant");
    };

    format!(
        concat!(
            "profile_id = \"{}\"\n",
            "version = {}\n",
            "profile_sha256 = \"{}\"\n",
            "description = \"{}\"\n",
            "\n",
            "[[float_policies]]\n",
            "field = \"simulation_time\"\n",
            "scope = \"phase2_trace\"\n",
            "policy = {{ kind = \"exact_bits\" }}\n",
            "\n",
            "[[float_policies]]\n",
            "field = \"synthetic_absolute\"\n",
            "scope = \"comparator_coverage\"\n",
            "policy = {{ kind = \"absolute\", max_bits = {} }}\n",
            "\n",
            "[[float_policies]]\n",
            "field = \"synthetic_absolute_relative\"\n",
            "scope = \"comparator_coverage\"\n",
            "policy = {{ kind = \"absolute_relative\", absolute_bits = {}, relative_bits = {} }}\n",
            "\n",
            "[[float_policies]]\n",
            "field = \"synthetic_ulps\"\n",
            "scope = \"comparator_coverage\"\n",
            "policy = {{ kind = \"ulps\", max = {} }}\n",
            "\n",
            "[[discrete_policies]]\n",
            "field = \"world_counts\"\n",
            "kind = \"exact\"\n",
            "\n",
            "[[collection_policies]]\n",
            "field = \"checkpoints\"\n",
            "kind = \"ordered\"\n"
        ),
        profile.profile_id(),
        profile.version().get(),
        profile.profile_sha256().as_str(),
        PHASE2_DESCRIPTION,
        max_bits.bits(),
        absolute_bits.bits(),
        relative_bits.bits(),
        max,
    )
}

fn check_tolerance_profile_presentation(input: &str) -> Result<(), PresentationError> {
    let presentation: ToleranceProfilePresentation = toml::from_str(input)?;
    let profile = ToleranceProfile::phase2_v1();

    validate_profile_header(&presentation, &profile)?;
    validate_float_policies(&presentation.float_policies, &profile)?;
    validate_discrete_policies(&presentation.discrete_policies, &profile)?;
    validate_collection_policies(&presentation.collection_policies, &profile)?;

    if render_tolerance_profile_presentation() != input {
        return Err(PresentationError::AuthorityMismatch(
            "tracked bytes are not the deterministic rendering",
        ));
    }
    Ok(())
}

fn validate_profile_header(
    presentation: &ToleranceProfilePresentation,
    profile: &ToleranceProfile,
) -> Result<(), PresentationError> {
    if presentation.profile_id != profile.profile_id() {
        return Err(PresentationError::AuthorityMismatch("profile ID"));
    }
    if presentation.version != profile.version() {
        return Err(PresentationError::AuthorityMismatch("profile version"));
    }
    if presentation.profile_sha256 != *profile.profile_sha256() {
        return Err(PresentationError::AuthorityMismatch("profile hash"));
    }
    if presentation.description != PHASE2_DESCRIPTION {
        return Err(PresentationError::AuthorityMismatch("profile description"));
    }
    Ok(())
}

fn validate_float_policies(
    policies: &[FloatPolicyPresentation],
    profile: &ToleranceProfile,
) -> Result<(), PresentationError> {
    reject_duplicate_fields(policies.iter().map(|policy| policy.field.as_str()))?;
    let synthetic = ToleranceProfile::synthetic_float_policies();
    let expected = [
        (
            "simulation_time",
            "phase2_trace",
            PresentedFloatPolicy::from(profile.simulation_time()),
        ),
        (
            "synthetic_absolute",
            "comparator_coverage",
            PresentedFloatPolicy::from(synthetic[0]),
        ),
        (
            "synthetic_absolute_relative",
            "comparator_coverage",
            PresentedFloatPolicy::from(synthetic[1]),
        ),
        (
            "synthetic_ulps",
            "comparator_coverage",
            PresentedFloatPolicy::from(synthetic[2]),
        ),
    ];
    let matches_authority = policies.len() == expected.len()
        && policies.iter().zip(expected).all(|(actual, expected)| {
            actual.field == expected.0 && actual.scope == expected.1 && actual.policy == expected.2
        });
    if !matches_authority {
        return Err(PresentationError::AuthorityMismatch("float policies"));
    }
    Ok(())
}

fn validate_discrete_policies(
    policies: &[DiscretePolicyPresentation],
    profile: &ToleranceProfile,
) -> Result<(), PresentationError> {
    reject_duplicate_fields(policies.iter().map(|policy| policy.field.as_str()))?;
    if policies.len() != 1
        || policies[0].field != "world_counts"
        || policies[0].kind != profile.world_counts()
    {
        return Err(PresentationError::AuthorityMismatch("discrete policies"));
    }
    Ok(())
}

fn validate_collection_policies(
    policies: &[CollectionPolicyPresentation],
    profile: &ToleranceProfile,
) -> Result<(), PresentationError> {
    reject_duplicate_fields(policies.iter().map(|policy| policy.field.as_str()))?;
    if policies.len() != 1
        || policies[0].field != "checkpoints"
        || policies[0].kind != profile.checkpoints()
    {
        return Err(PresentationError::AuthorityMismatch("collection policies"));
    }
    Ok(())
}

fn reject_duplicate_fields<'a>(
    fields: impl Iterator<Item = &'a str>,
) -> Result<(), PresentationError> {
    let mut unique = BTreeSet::new();
    for field in fields {
        if !unique.insert(field) {
            return Err(PresentationError::DuplicatePolicy(field.to_owned()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{check_tolerance_profile_presentation, render_tolerance_profile_presentation};

    const TRACKED_TOLERANCE_PROFILE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../protocol/tolerances/phase2-v1.toml"
    ));

    #[test]
    fn tolerance_profile_presentation_is_strict_and_byte_stable() {
        // Arrange
        let unsupported_version =
            TRACKED_TOLERANCE_PROFILE.replacen("version = 1", "version = 2", 1);
        let mismatched_hash = TRACKED_TOLERANCE_PROFILE.replacen(
            "177db8c2ff3011653fc27f74339fe144df5936bb078db85f28402d317e6622c3",
            "077db8c2ff3011653fc27f74339fe144df5936bb078db85f28402d317e6622c3",
            1,
        );
        let unknown_field = format!("unknown = true\n{TRACKED_TOLERANCE_PROFILE}");
        let decimal_threshold =
            TRACKED_TOLERANCE_PROFILE.replacen("max_bits = 1065353216", "max_bits = 1.0", 1);
        let duplicate_policy = format!(
            "{TRACKED_TOLERANCE_PROFILE}\n[[float_policies]]\nfield = \"simulation_time\"\nscope = \"phase2_trace\"\npolicy = {{ kind = \"exact_bits\" }}\n"
        );

        // Act
        let rendered = render_tolerance_profile_presentation();
        let strict_results = [
            check_tolerance_profile_presentation(&unsupported_version),
            check_tolerance_profile_presentation(&mismatched_hash),
            check_tolerance_profile_presentation(&unknown_field),
            check_tolerance_profile_presentation(&decimal_threshold),
            check_tolerance_profile_presentation(&duplicate_policy),
        ];

        // Assert
        assert_eq!(rendered, TRACKED_TOLERANCE_PROFILE);
        assert!(rendered.ends_with('\n'));
        assert!(rendered.contains("field = \"simulation_time\""));
        assert!(rendered.contains("kind = \"exact_bits\""));
        assert!(rendered.contains("max_bits = 1065353216"));
        assert!(rendered.contains("absolute_bits = 1065353216"));
        assert!(rendered.contains("relative_bits = 1048576000"));
        assert!(strict_results.into_iter().all(|result| result.is_err()));
    }

    #[test]
    fn tolerance_profile_presentation_matches_typed_authority() {
        // Arrange and Act
        let result = check_tolerance_profile_presentation(TRACKED_TOLERANCE_PROFILE);

        // Assert
        assert!(result.is_ok());
    }
}

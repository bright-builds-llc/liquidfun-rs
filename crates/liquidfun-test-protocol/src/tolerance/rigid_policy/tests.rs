use super::{
    Phase6PolicyError, Phase6PolicyProfile, Phase7PolicyError, Phase7PolicyProfile,
    Phase8PolicyError, Phase8PolicyProfile, render_phase6_policy_presentation,
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
const PHASE8_PROFILE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../protocol/tolerances/phase8-v1.toml"
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
        "59cf32e2564d857bbf56ec7e8423bd73046f4c7698f2e0e3eb83c5ea7ab2b86a"
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

#[test]
fn rigid_policy_phase8_closes_structural_configuration_and_numeric_rules() {
    // Arrange and Act
    let profile = Phase8PolicyProfile::parse_toml(PHASE8_PROFILE)
        .expect("checked-in Phase 8 policy should validate");

    // Assert
    assert_eq!(profile.profile_id(), "phase8-v1");
    assert_eq!(profile.fields().len(), 37);
    assert_eq!(
        profile.profile_sha256().as_str(),
        "e31c47660bb5cce5aeb502ad510448176b419e604ef5048d74403bdef2f3a493"
    );
    assert!(matches!(
        profile
            .field("rigid_world.phase8.joint.configuration.bits")
            .expect("configuration policy")
            .comparison(),
        FieldComparison::Float {
            policy: FloatPolicy::ExactBits
        }
    ));
    assert!(matches!(
        profile
            .field("rigid_world.phase8.diagnostics.tree_quality")
            .expect("tree quality policy")
            .comparison(),
        FieldComparison::Float {
            policy: FloatPolicy::Absolute { .. }
        }
    ));
    assert!(profile.field("rigid_world.phase8.unregistered").is_none());
}

#[test]
fn rigid_policy_phase8_rejects_unknown_missing_wildcard_and_widened_rules() {
    // Arrange
    let unknown = PHASE8_PROFILE.replacen(
        "rigid_world.phase8.joint.id",
        "rigid_world.phase8.joint.default",
        1,
    );
    let first_field = PHASE8_PROFILE
        .split("[[fields]]")
        .nth(1)
        .expect("profile contains a field");
    let missing = PHASE8_PROFILE.replacen(&format!("[[fields]]{first_field}"), "", 1);
    let wildcard = PHASE8_PROFILE.replacen(
        "rigid_world.phase8.joint.id",
        "rigid_world.phase8.joint.*",
        1,
    );
    let widened = PHASE8_PROFILE.replacen(
        "policy = { kind = \"exact_bits\" }",
        "policy = { kind = \"ulps\", max = 4 }",
        1,
    );

    // Act
    let errors = [unknown, missing, wildcard, widened].map(|input| {
        Phase8PolicyProfile::parse_toml(&input).expect_err("open policy must fail closed")
    });

    // Assert
    assert!(matches!(errors[0], Phase8PolicyError::InvalidSemanticPath));
    assert!(matches!(errors[1], Phase8PolicyError::IncompleteProfile));
    assert!(matches!(errors[2], Phase8PolicyError::InvalidSemanticPath));
    assert!(matches!(errors[3], Phase8PolicyError::IncompatibleMetadata));
}

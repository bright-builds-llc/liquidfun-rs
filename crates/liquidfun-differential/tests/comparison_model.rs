//! Behavioral coverage for the renderer-neutral checkpoint comparison model.

use liquidfun_differential::{
    ComparisonError, ComparisonLimits, ComparisonState, compare_canonical_checkpoints,
};
use liquidfun_test_protocol::{
    CanonicalCheckpoint, CheckpointId, CheckpointPosition, CheckpointProfileName, CheckpointSet,
    DebugColorBits, DebugLayerName, DebugOwnerId, DebugPrimitiveKey, DebugPrimitiveKindName,
    DebugPrimitiveOrder, DebugPrimitiveRecord, DebugStrokeBits, FloatBits, MathProbePolicyPath,
    NumericObservation, OccurrenceKind, OrderedOccurrence, Phase4PolicyProfile, PrimitivePoint,
    RequestId, ScenarioId, Sha256Hex, StructuralObservation, StructuralValue, Vec2Bits,
    WireDebugPrimitive,
};

const POLICY: &str = include_str!("../../../protocol/tolerances/phase4-v1.toml");

fn id(value: &str) -> ScenarioId {
    ScenarioId::new(value).expect("fixture semantic ID should validate")
}

fn policy() -> Phase4PolicyProfile {
    Phase4PolicyProfile::parse_toml(POLICY).expect("checked-in Phase 4 policy should validate")
}

fn checkpoint(
    observations: Vec<StructuralObservation>,
    numeric_observations: Vec<NumericObservation>,
    ordered_occurrences: Vec<OrderedOccurrence>,
    unordered_sets: Vec<CheckpointSet>,
    debug_primitives: Vec<DebugPrimitiveRecord>,
    profile_names: Vec<CheckpointProfileName>,
) -> CanonicalCheckpoint {
    CanonicalCheckpoint::new(
        RequestId::new("comparison-run").expect("fixture request ID should validate"),
        Sha256Hex::new("1".repeat(64)).expect("fixture hash should validate"),
        CheckpointId::new("checkpoint-0001").expect("fixture checkpoint ID should validate"),
        CheckpointPosition::LogicalStep { ordinal: 1 },
        FloatBits::from_f32(1.0 / 60.0),
        observations,
        numeric_observations,
        ordered_occurrences,
        unordered_sets,
        debug_primitives,
        profile_names,
    )
    .expect("fixture checkpoint should validate")
}

fn empty_checkpoint() -> CanonicalCheckpoint {
    checkpoint(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

fn numeric(value_bits: u32) -> NumericObservation {
    NumericObservation::new(
        id("world.contact-distance"),
        FloatBits::new(value_bits),
        MathProbePolicyPath::MathVectorLength,
    )
}

fn occurrence(id_value: &str, owner: &str) -> OrderedOccurrence {
    OrderedOccurrence::new(id(id_value), OccurrenceKind::ContactBegin, id(owner))
}

fn point(x_bits: u32) -> DebugPrimitiveRecord {
    let key = DebugPrimitiveKey::new(
        DebugOwnerId::Body(id("body-1")),
        DebugLayerName::Contacts,
        DebugPrimitiveKindName::Point,
        0,
        0,
    );
    let stroke = DebugStrokeBits::new(
        DebugColorBits::rgba(0x58, 0xa6, 0xff, 0xff),
        FloatBits::from_f32(0.125),
    )
    .expect("fixture stroke should validate");
    DebugPrimitiveRecord::new(
        DebugPrimitiveOrder::SourceSignificant,
        WireDebugPrimitive::Point(PrimitivePoint::new(
            key,
            stroke,
            None,
            Vec2Bits {
                x_bits: FloatBits::new(x_bits),
                y_bits: FloatBits::from_f32(2.0),
            },
            FloatBits::from_f32(0.05),
        )),
    )
}

fn compare(
    rust: &CanonicalCheckpoint,
    oracle: &CanonicalCheckpoint,
) -> liquidfun_differential::ComparisonModel {
    compare_canonical_checkpoints(rust, oracle, &policy(), ComparisonLimits::phase11_default())
        .expect("compatible checkpoints should compare")
}

#[test]
fn identical_checkpoints_produce_only_exact_entries() {
    // Arrange
    let rust = empty_checkpoint();
    let oracle = empty_checkpoint();

    // Act
    let model = compare(&rust, &oracle);

    // Assert
    assert!(!model.entries().is_empty());
    assert_eq!(model.state(), ComparisonState::ExactMatch);
    assert!(
        model
            .entries()
            .iter()
            .all(|entry| entry.state() == ComparisonState::ExactMatch)
    );
}

#[test]
fn numeric_difference_inside_named_policy_is_within_policy() {
    // Arrange
    let base = 1.0_f32.to_bits();
    let rust = checkpoint(
        Vec::new(),
        vec![numeric(base + 2)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let oracle = checkpoint(
        Vec::new(),
        vec![numeric(base)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );

    // Act
    let model = compare(&rust, &oracle);
    let entry = model
        .entries()
        .iter()
        .find(|entry| entry.semantic_path() == "numeric_observations.world.contact-distance.value")
        .expect("numeric entry should exist");

    // Assert
    assert_eq!(entry.state(), ComparisonState::WithinPolicy);
    assert_eq!(
        entry.maybe_policy_path(),
        Some(MathProbePolicyPath::MathVectorLength)
    );
}

#[test]
fn numeric_difference_one_over_named_policy_is_a_physics_mismatch() {
    // Arrange
    let base = 1.0_f32.to_bits();
    let rust = checkpoint(
        Vec::new(),
        vec![numeric(base + 3)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let oracle = checkpoint(
        Vec::new(),
        vec![numeric(base)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );

    // Act
    let model = compare(&rust, &oracle);
    let entry = model
        .entries()
        .iter()
        .find(|entry| entry.semantic_path().ends_with("contact-distance.value"))
        .expect("numeric entry should exist");

    // Assert
    assert_eq!(entry.state(), ComparisonState::PhysicsMismatch);
    assert!(entry.maybe_signature_sha256().is_some());
}

#[test]
fn exact_structural_kind_and_value_differences_are_mismatches() {
    // Arrange
    let rust = checkpoint(
        vec![StructuralObservation::new(
            id("world.body-count"),
            StructuralValue::Count(2),
        )],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let oracle = checkpoint(
        vec![StructuralObservation::new(
            id("world.body-count"),
            StructuralValue::FlagBits(2),
        )],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );

    // Act
    let model = compare(&rust, &oracle);

    // Assert
    assert!(model.entries().iter().any(|entry| {
        entry.semantic_path() == "observations.world.body-count.kind"
            && entry.state() == ComparisonState::PhysicsMismatch
    }));
}

#[test]
fn missing_observations_are_rust_only_or_oracle_only_never_matches() {
    // Arrange
    let observation = StructuralObservation::new(id("world.body-count"), StructuralValue::Count(1));
    let rust_present = checkpoint(
        vec![observation.clone()],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let oracle_present = checkpoint(
        vec![observation],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let absent = empty_checkpoint();

    // Act
    let rust_only = compare(&rust_present, &absent);
    let oracle_only = compare(&absent, &oracle_present);

    // Assert
    assert!(
        rust_only
            .entries()
            .iter()
            .any(|entry| entry.state() == ComparisonState::RustOnly)
    );
    assert!(
        oracle_only
            .entries()
            .iter()
            .any(|entry| entry.state() == ComparisonState::OracleOnly)
    );
}

#[test]
fn source_significant_occurrence_order_is_exact() {
    // Arrange
    let rust = checkpoint(
        Vec::new(),
        Vec::new(),
        vec![
            occurrence("contact-a", "body-a"),
            occurrence("contact-b", "body-b"),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let oracle = checkpoint(
        Vec::new(),
        Vec::new(),
        vec![
            occurrence("contact-b", "body-b"),
            occurrence("contact-a", "body-a"),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );

    // Act
    let model = compare(&rust, &oracle);

    // Assert
    assert!(model.entries().iter().any(|entry| {
        entry.semantic_path() == "ordered_occurrences.0.occurrence_id"
            && entry.state() == ComparisonState::PhysicsMismatch
    }));
}

#[test]
fn only_declared_unordered_sets_are_canonicalized() {
    // Arrange
    let rust_set = CheckpointSet::new(id("world.active-bodies"), vec![id("body-b"), id("body-a")])
        .expect("set should canonicalize");
    let oracle_set =
        CheckpointSet::new(id("world.active-bodies"), vec![id("body-a"), id("body-b")])
            .expect("set should canonicalize");
    let rust = checkpoint(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![rust_set],
        Vec::new(),
        Vec::new(),
    );
    let oracle = checkpoint(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![oracle_set],
        Vec::new(),
        Vec::new(),
    );

    // Act
    let model = compare(&rust, &oracle);

    // Assert
    assert!(model.entries().iter().all(|entry| {
        !entry.semantic_path().starts_with("unordered_sets.")
            || entry.state() == ComparisonState::ExactMatch
    }));
}

#[test]
fn duplicate_semantic_members_fail_before_comparison() {
    // Arrange / Act
    let maybe_set = CheckpointSet::new(id("world.active-bodies"), vec![id("body-a"), id("body-a")]);

    // Assert
    assert!(maybe_set.is_err());
}

#[test]
fn absent_phase4_policy_is_a_harness_error() {
    // Arrange
    let profile_without_path = Phase4PolicyProfile::parse_toml(&POLICY.replacen(
        "semantic_path = \"math.vector.length\"",
        "semantic_path = \"math.vector.length.unbound\"",
        1,
    ))
    .expect("modified closed profile should remain structurally valid");
    let value = 1.0_f32.to_bits();
    let rust = checkpoint(
        Vec::new(),
        vec![numeric(value)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let oracle = rust.clone();

    // Act
    let error = compare_canonical_checkpoints(
        &rust,
        &oracle,
        &profile_without_path,
        ComparisonLimits::phase11_default(),
    )
    .expect_err("unbound numeric paths must fail closed");

    // Assert
    assert_eq!(error, ComparisonError::InvalidPolicyBinding);
}

#[test]
fn private_semantic_path_is_rejected() {
    // Arrange
    let rust = checkpoint(
        vec![StructuralObservation::new(
            id("private"),
            StructuralValue::Count(1),
        )],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let oracle = rust.clone();

    // Act
    let error = compare_canonical_checkpoints(
        &rust,
        &oracle,
        &policy(),
        ComparisonLimits::phase11_default(),
    )
    .expect_err("private path segments must fail closed");

    // Assert
    assert_eq!(error, ComparisonError::InvalidSemanticPath);
}

#[test]
fn profiles_compare_names_but_expose_no_duration_values() {
    // Arrange
    let rust = checkpoint(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![CheckpointProfileName::RigidSolve],
    );
    let oracle = rust.clone();

    // Act
    let model = compare(&rust, &oracle);

    // Assert
    assert!(model.entries().iter().any(|entry| {
        entry.semantic_path() == "profile_names.0" && entry.state() == ComparisonState::ExactMatch
    }));
    assert!(
        model
            .entries()
            .iter()
            .all(|entry| !entry.semantic_path().contains("duration"))
    );
}

#[test]
fn primitive_numeric_entries_carry_stable_focus_keys() {
    // Arrange
    let base = 1.0_f32.to_bits();
    let rust = checkpoint(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![point(base + 1)],
        Vec::new(),
    );
    let oracle = checkpoint(
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![point(base)],
        Vec::new(),
    );

    // Act
    let model = compare(&rust, &oracle);
    let entry = model
        .entries()
        .iter()
        .find(|entry| entry.semantic_path() == "debug_primitives.0.position.x")
        .expect("point x entry should exist");

    // Assert
    assert_eq!(entry.state(), ComparisonState::WithinPolicy);
    assert_eq!(entry.maybe_primitive_key(), Some(point(base).key()));
}

#[test]
fn diagnostic_values_are_bounded_and_redacted_from_private_storage() {
    // Arrange
    let long_id = format!("observation-{}", "x".repeat(110));
    let long_value = format!("identity-{}", "y".repeat(113));
    let rust = checkpoint(
        vec![StructuralObservation::new(
            id(&long_id),
            StructuralValue::Identity {
                semantic_id: id(&long_value),
            },
        )],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let oracle = empty_checkpoint();

    // Act
    let model = compare(&rust, &oracle);

    // Assert
    assert!(model.entries().iter().all(|entry| {
        entry
            .maybe_rust_value()
            .is_none_or(|value| value.len() <= 256)
            && entry
                .maybe_oracle_value()
                .is_none_or(|value| value.len() <= 256)
            && !entry.context().contains("slot")
            && !entry.context().contains("pointer")
    }));
}

#[test]
fn entry_limit_accepts_exact_limit_and_rejects_one_over() {
    // Arrange
    let rust = empty_checkpoint();
    let oracle = empty_checkpoint();
    let complete = compare(&rust, &oracle);
    let exact_limit = ComparisonLimits::with_maximum_entries(complete.entries().len())
        .expect("exact fixture limit should validate");
    let one_under = ComparisonLimits::with_maximum_entries(complete.entries().len() - 1)
        .expect("smaller fixture limit should validate");

    // Act
    let at_limit = compare_canonical_checkpoints(&rust, &oracle, &policy(), exact_limit);
    let one_over = compare_canonical_checkpoints(&rust, &oracle, &policy(), one_under);

    // Assert
    assert!(at_limit.is_ok());
    assert_eq!(
        one_over.expect_err("one entry above the configured limit must fail"),
        ComparisonError::EntryLimitExceeded
    );
}

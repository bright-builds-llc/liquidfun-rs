//! Focused semantic comparison policy and compatibility tests.

use liquidfun_differential::{
    CanonicalValue, DifferentialOutcome, SemanticCollection, collections_match, compare,
    exact_values_match, float_values_match,
};
use liquidfun_test_protocol::{
    BuildIdentity, CheckpointRecord, EngineKind, FloatBits, FloatPolicy, HarnessLimits,
    ScenarioRequestRecord, ToleranceProfile, TraceBegin, TraceEnd, TraceRecord, TraceValidator,
    ValidatedTrace, WorldCounts, decode_handshake_jsonl, decode_scenario_request_jsonl,
    trace_payload_sha256,
};

const REQUEST_BYTES: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/empty-world-request.jsonl");
const TRACE_BYTES: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/empty-world-trace.jsonl");

fn fixture_request() -> ScenarioRequestRecord {
    decode_scenario_request_jsonl(REQUEST_BYTES, &HarnessLimits::phase2_default_v1())
        .expect("checked-in request should validate")
}

fn fixture_identity() -> BuildIdentity {
    let handshake = TRACE_BYTES
        .split_inclusive(|byte| *byte == b'\n')
        .next()
        .expect("checked-in trace should contain a handshake");
    decode_handshake_jsonl(handshake, &HarnessLimits::phase2_default_v1())
        .expect("checked-in handshake should validate")
        .build_identity()
        .clone()
}

fn validated_trace(
    request: &ScenarioRequestRecord,
    identity: &BuildIdentity,
    engine_kind: EngineKind,
    times: [FloatBits; 2],
) -> ValidatedTrace {
    let begin = TraceBegin::for_request(request, engine_kind, identity)
        .expect("validated request should create a trace begin");
    let checkpoints = request
        .scenario()
        .checkpoints()
        .iter()
        .zip(times)
        .enumerate()
        .map(|(ordinal, (requested, simulation_time_bits))| {
            CheckpointRecord::new(
                request.request_id().clone(),
                requested.checkpoint_id().clone(),
                u32::try_from(ordinal).expect("two checkpoints fit in u32"),
                requested.phase(),
                simulation_time_bits,
                WorldCounts::zero(),
                identity.identity_sha256().clone(),
            )
            .expect("fixture checkpoint should validate")
        })
        .collect::<Vec<_>>();
    let payload_hash = trace_payload_sha256(&checkpoints)
        .expect("typed checkpoints should hash deterministically");
    let end = TraceEnd::new(
        request.request_id().clone(),
        2,
        payload_hash,
        1,
        true,
        identity.identity_sha256().clone(),
    );
    let records = std::iter::once(TraceRecord::Begin(begin))
        .chain(checkpoints.into_iter().map(TraceRecord::Checkpoint))
        .chain(std::iter::once(TraceRecord::End(end)))
        .collect();
    TraceValidator::validate(
        request,
        identity,
        1,
        records,
        &HarnessLimits::phase2_default_v1(),
    )
    .expect("complete synthetic trace should validate")
}

#[test]
fn incompatible_scenario_returns_harness_failure_before_values() {
    // Arrange
    let request = fixture_request();
    let different_bytes = String::from_utf8(REQUEST_BYTES.to_vec())
        .expect("fixture is UTF-8")
        .replace(
            "\"scenario_id\":\"empty-world\"",
            "\"scenario_id\":\"other-world\"",
        );
    let different_request = decode_scenario_request_jsonl(
        different_bytes.as_bytes(),
        &HarnessLimits::phase2_default_v1(),
    )
    .expect("modified request should remain valid");
    let identity = fixture_identity();
    let expected = validated_trace(
        &request,
        &identity,
        EngineKind::CppOracle,
        [FloatBits::from_f32(0.5), FloatBits::from_f32(1.0)],
    );
    let actual = validated_trace(
        &different_request,
        &identity,
        EngineKind::NativeRust,
        [FloatBits::from_f32(9.0), FloatBits::from_f32(10.0)],
    );

    // Act
    let result = compare(&expected, &actual, &ToleranceProfile::phase2_v1());

    // Assert
    assert!(result.is_err());
}

#[test]
fn incompatible_engine_provenance_returns_harness_failure() {
    // Arrange
    let request = fixture_request();
    let identity = fixture_identity();
    let expected = validated_trace(
        &request,
        &identity,
        EngineKind::CppOracle,
        [FloatBits::from_f32(0.5), FloatBits::from_f32(1.0)],
    );
    let actual = validated_trace(
        &request,
        &identity,
        EngineKind::CppOracle,
        [FloatBits::from_f32(9.0), FloatBits::from_f32(10.0)],
    );

    // Act
    let result = compare(&expected, &actual, &ToleranceProfile::phase2_v1());

    // Assert
    assert!(result.is_err());
}

#[test]
fn compatible_value_difference_returns_physics_mismatch() {
    // Arrange
    let request = fixture_request();
    let identity = fixture_identity();
    let expected = validated_trace(
        &request,
        &identity,
        EngineKind::CppOracle,
        [FloatBits::from_f32(0.5), FloatBits::from_f32(1.0)],
    );
    let actual = validated_trace(
        &request,
        &identity,
        EngineKind::NativeRust,
        [FloatBits::from_f32(0.5), FloatBits::from_f32(1.25)],
    );

    // Act
    let result = compare(&expected, &actual, &ToleranceProfile::phase2_v1());

    // Assert
    assert!(matches!(
        result,
        Ok(DifferentialOutcome::PhysicsMismatch(_))
    ));
}

#[test]
fn exact_discrete_values_do_not_coerce_ids_or_counts() {
    // Arrange
    let left_id = liquidfun_test_protocol::CheckpointId::new("checkpoint-1")
        .expect("checkpoint ID should validate");
    let right_id = liquidfun_test_protocol::CheckpointId::new("checkpoint-2")
        .expect("checkpoint ID should validate");

    // Act
    let same_count = exact_values_match(&7_u32, &7_u32);
    let different_count = exact_values_match(&7_u32, &8_u32);
    let different_id = exact_values_match(&left_id, &right_id);

    // Assert
    assert!(same_count);
    assert!(!different_count);
    assert!(!different_id);
}

#[test]
fn float_policies_accept_values_at_their_thresholds() {
    // Arrange
    let [absolute, absolute_relative, ulps] = ToleranceProfile::synthetic_float_policies();
    let one = FloatBits::from_f32(1.0);

    // Act
    let absolute_match = float_values_match(one, FloatBits::from_f32(2.0), absolute);
    let relative_match = float_values_match(
        FloatBits::from_f32(6.0),
        FloatBits::from_f32(8.0),
        absolute_relative,
    );
    let ulp_match = float_values_match(one, FloatBits::new(one.bits() + 4), ulps);

    // Assert
    assert!(absolute_match);
    assert!(relative_match);
    assert!(ulp_match);
}

#[test]
fn float_policies_reject_values_past_their_thresholds() {
    // Arrange
    let [absolute, absolute_relative, ulps] = ToleranceProfile::synthetic_float_policies();
    let one = FloatBits::from_f32(1.0);

    // Act
    let absolute_match = float_values_match(one, FloatBits::from_f32(2.000_000_2), absolute);
    let relative_match = float_values_match(
        FloatBits::from_f32(6.0),
        FloatBits::from_f32(8.1),
        absolute_relative,
    );
    let ulp_match = float_values_match(one, FloatBits::new(one.bits() + 5), ulps);

    // Assert
    assert!(!absolute_match);
    assert!(!relative_match);
    assert!(!ulp_match);
}

#[test]
fn special_float_rules_preserve_nan_infinity_and_signed_zero() {
    // Arrange
    let exact = FloatPolicy::ExactBits;
    let tolerant = FloatPolicy::Absolute {
        max_bits: FloatBits::from_f32(1.0),
    };
    let nan = FloatBits::new(0x7fc0_0042);

    // Act
    let exact_nan = float_values_match(nan, nan, exact);
    let tolerant_nan = float_values_match(nan, nan, tolerant);
    let same_infinity = float_values_match(
        FloatBits::from_f32(f32::INFINITY),
        FloatBits::from_f32(f32::INFINITY),
        tolerant,
    );
    let opposite_infinity = float_values_match(
        FloatBits::from_f32(f32::INFINITY),
        FloatBits::from_f32(f32::NEG_INFINITY),
        tolerant,
    );
    let signed_zero = float_values_match(
        FloatBits::new(0.0_f32.to_bits()),
        FloatBits::new((-0.0_f32).to_bits()),
        tolerant,
    );

    // Assert
    assert!(exact_nan);
    assert!(!tolerant_nan);
    assert!(same_infinity);
    assert!(!opposite_infinity);
    assert!(!signed_zero);
}

#[test]
fn typed_collection_policies_preserve_order_and_multiplicity() {
    // Arrange
    let first = CanonicalValue::new("body-1", 0_u32);
    let second = CanonicalValue::new("body-2", 0_u32);
    let duplicate = CanonicalValue::new("body-1", 0_u32);

    // Act
    let ordered = collections_match(
        &SemanticCollection::Ordered(vec![first.clone(), second.clone()]),
        &SemanticCollection::Ordered(vec![second.clone(), first.clone()]),
    );
    let set = collections_match(
        &SemanticCollection::Set(vec![first.clone(), second.clone()]),
        &SemanticCollection::Set(vec![second.clone(), first.clone()]),
    );
    let multiset = collections_match(
        &SemanticCollection::Multiset(vec![first.clone(), duplicate]),
        &SemanticCollection::Multiset(vec![first]),
    );

    // Assert
    assert!(!ordered);
    assert!(set);
    assert!(!multiset);
}

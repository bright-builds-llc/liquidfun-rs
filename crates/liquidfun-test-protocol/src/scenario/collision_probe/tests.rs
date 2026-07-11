use std::collections::HashSet;

use serde_json::{Value, json};

use super::*;
use crate::{FloatBits, HarnessLimits, RecordLimit, encode_jsonl};

const REQUEST: &[u8] =
    include_bytes!("../../../../../protocol/fixtures/accepted/collision-probe-request.jsonl");

fn fixture_value() -> Value {
    serde_json::from_slice(REQUEST).expect("checked-in collision request should be JSON")
}

fn encode_value(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).expect("fixture mutation should serialize");
    bytes.push(b'\n');
    bytes
}

fn case_mut<'a>(value: &'a mut Value, family: &str) -> &'a mut Value {
    value["scenario"]["cases"]
        .as_array_mut()
        .expect("fixture cases should be an array")
        .iter_mut()
        .find(|case| case["witness_family"] == family)
        .expect("fixture should contain requested witness family")
}

#[test]
fn collision_probe_operation_registry_is_closed_and_complete() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();

    // Act
    let request = decode_collision_probe_request_jsonl(REQUEST, &limits)
        .expect("checked-in collision request should decode");
    let operations: HashSet<_> = request
        .scenario()
        .cases()
        .iter()
        .map(CollisionProbeCase::operation)
        .collect();

    // Assert
    assert_eq!(operations, HashSet::from(CollisionProbeOperation::ALL));
    assert_eq!(
        encode_jsonl(&request, &limits, RecordLimit::Input)
            .expect("validated request should encode"),
        REQUEST
    );
}

#[test]
fn collision_probe_required_families_are_complete() {
    // Arrange / Act
    let request =
        decode_collision_probe_request_jsonl(REQUEST, &HarnessLimits::phase2_default_v1())
            .expect("checked-in collision request should decode");
    let actual = request
        .scenario()
        .cases()
        .iter()
        .map(CollisionProbeCase::witness_family)
        .collect::<HashSet<_>>();

    // Assert
    assert_eq!(CollisionWitnessFamily::REQUIRED.len(), 78);
    assert_eq!(actual.len(), CollisionWitnessFamily::REQUIRED.len());
    assert!(
        CollisionWitnessFamily::REQUIRED
            .iter()
            .all(|family| actual.contains(family))
    );
}

#[test]
fn collision_probe_required_family_deletion_fails_closed() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();

    // Act / Assert
    for family in CollisionWitnessFamily::REQUIRED {
        let mut value = fixture_value();
        value["scenario"]["cases"]
            .as_array_mut()
            .expect("fixture cases should be an array")
            .retain(|case| {
                serde_json::from_value::<CollisionWitnessFamily>(case["witness_family"].clone())
                    .expect("fixture family should deserialize")
                    != family
            });
        let error = decode_collision_probe_request_jsonl(&encode_value(&value), &limits)
            .expect_err("deleting any required family must fail");
        assert!(matches!(
            error,
            CollisionProbeDecodeError::Validation(CollisionProbeErrorKind::MissingWitnessFamily)
        ));
    }
}

#[test]
fn collision_probe_expected_rejection_is_typed_and_fail_closed() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut wrong_kind = fixture_value();
    case_mut(&mut wrong_kind, "shape_rejected_circle")["expected_outcome"] =
        json!({ "kind": "accepted" });
    let mut wrong_reason = fixture_value();
    case_mut(&mut wrong_reason, "shape_rejected_circle")["expected_outcome"]["category"] =
        json!("non_finite_value");

    // Act
    let request = decode_collision_probe_request_jsonl(REQUEST, &limits)
        .expect("checked-in rejection witnesses should decode");
    let errors = [wrong_kind, wrong_reason].map(|value| {
        decode_collision_probe_request_jsonl(&encode_value(&value), &limits)
            .expect_err("a false rejection declaration must fail")
    });

    // Assert
    let rejected = request
        .scenario()
        .cases()
        .iter()
        .filter(|case| {
            matches!(
                case.expected_outcome(),
                CollisionExpectedOutcome::Rejected { .. }
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(rejected.len(), 7);
    assert!(
        rejected
            .iter()
            .all(|case| case.operation() == CollisionProbeOperation::ShapeConstruction)
    );
    assert!(matches!(
        errors[0],
        CollisionProbeDecodeError::Validation(CollisionProbeErrorKind::WitnessFamilyMismatch)
    ));
    assert!(matches!(
        errors[1],
        CollisionProbeDecodeError::Validation(CollisionProbeErrorKind::InvalidGeometry)
    ));
}

#[test]
fn collision_probe_accepted_cases_use_production_validation() {
    // Arrange
    let mut invalid_circle = fixture_value();
    case_mut(&mut invalid_circle, "shape_accepted_circle")["input"]["shape"]["radius_bits"] =
        json!(3_212_836_864_u32);
    let mut invalid_edge = fixture_value();
    let edge = &mut case_mut(&mut invalid_edge, "shape_accepted_edge_ghosts")["input"]["shape"];
    edge["maybe_previous"] = edge["start"].clone();
    let mut invalid_polygon = fixture_value();
    case_mut(&mut invalid_polygon, "shape_accepted_polygon_weld_hull")["input"]["shape"]["vertices"] = json!([
        { "x_bits": 0, "y_bits": 0 },
        { "x_bits": 0, "y_bits": 0 },
        { "x_bits": 0, "y_bits": 0 }
    ]);
    let mut invalid_chain = fixture_value();
    case_mut(&mut invalid_chain, "shape_accepted_chain_topology")["input"]["shape"]["vertices"] = json!([
        { "x_bits": 0, "y_bits": 0 },
        { "x_bits": 1_065_353_216_u32, "y_bits": 0 },
        { "x_bits": 0, "y_bits": 0 }
    ]);

    // Act
    let errors = [invalid_circle, invalid_edge, invalid_polygon, invalid_chain].map(|value| {
        decode_collision_probe_request_jsonl(
            &encode_value(&value),
            &HarnessLimits::phase2_default_v1(),
        )
        .expect_err("accepted invalid geometry must fail production validation")
    });

    // Assert
    assert!(errors.into_iter().all(|error| matches!(
        error,
        CollisionProbeDecodeError::Validation(CollisionProbeErrorKind::InvalidGeometry)
    )));
}

#[test]
fn collision_probe_rejects_collection_and_horizon_mismatch() {
    // Arrange
    let mut horizon = fixture_value();
    case_mut(&mut horizon, "tree_query_continue_stop")["horizon"] = json!({ "kind": "operation" });
    let mut collection = fixture_value();
    case_mut(&mut collection, "tree_query_continue_stop")["collection_policy"] = json!("ordered");

    // Act
    let errors = [horizon, collection].map(|value| {
        decode_collision_probe_request_jsonl(
            &encode_value(&value),
            &HarnessLimits::phase2_default_v1(),
        )
        .expect_err("mismatched closed metadata should fail")
    });

    // Assert
    assert!(matches!(
        errors[0],
        CollisionProbeDecodeError::Validation(CollisionProbeErrorKind::HorizonMismatch)
    ));
    assert!(matches!(
        errors[1],
        CollisionProbeDecodeError::Validation(CollisionProbeErrorKind::CollectionPolicyMismatch)
    ));
}

#[test]
fn collision_probe_rejects_unknown_duplicate_missing_policy_and_invalid_child() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let text = std::str::from_utf8(REQUEST).expect("fixture should be UTF-8");
    let unknown = text.replacen(
        "\"request_id\":\"phase-05-collision-probe-request\"",
        "\"request_id\":\"phase-05-collision-probe-request\",\"unknown\":true",
        1,
    );
    let duplicate = text.replacen(
        "\"request_id\":\"phase-05-collision-probe-request\"",
        "\"request_id\":\"phase-05-collision-probe-request\",\"request_id\":\"duplicate\"",
        1,
    );
    let missing_policy = text.replacen(
        "\"policy_path\":\"collision.shape_unary_query.result\",",
        "",
        1,
    );
    let invalid_child = text.replacen("\"child_index\":0", "\"child_index\":1", 1);
    let unknown_family = text.replacen(
        "\"witness_family\":\"shape_unary_query\"",
        "\"witness_family\":\"future_family\"",
        1,
    );

    // Act
    let errors = [
        unknown,
        duplicate,
        missing_policy,
        invalid_child,
        unknown_family,
    ]
    .map(|record| {
        decode_collision_probe_request_jsonl(record.as_bytes(), &limits)
            .expect_err("invalid collision request should fail")
    });

    // Assert
    assert!(matches!(errors[0], CollisionProbeDecodeError::Codec(_)));
    assert!(matches!(errors[1], CollisionProbeDecodeError::Codec(_)));
    assert!(matches!(errors[2], CollisionProbeDecodeError::Codec(_)));
    assert!(matches!(
        errors[3],
        CollisionProbeDecodeError::Validation(CollisionProbeErrorKind::InvalidChildIndex)
    ));
    assert!(matches!(errors[4], CollisionProbeDecodeError::Codec(_)));
}

#[test]
fn collision_probe_result_outcome_is_closed() {
    // Arrange
    let accepted = CollisionProbeResult::new(
        "accepted",
        CollisionProbeOperation::Distance,
        vec![CollisionProbeNumericValue::new(
            "distance_bits",
            FloatBits::new(0),
        )],
        vec![],
        vec![],
    )
    .expect("bounded accepted result should construct");
    let rejected = CollisionProbeResult::rejected(
        "rejected",
        CollisionProbeOperation::ShapeConstruction,
        CollisionRejectionCategory::InvalidGeometry,
        CollisionRejectionField::CircleRadius,
    );

    // Act
    let accepted_json = serde_json::to_value(accepted).expect("accepted result should serialize");
    let rejected_json = serde_json::to_value(rejected).expect("rejected result should serialize");

    // Assert
    assert_eq!(accepted_json["outcome"]["kind"], "accepted");
    assert!(accepted_json.get("numeric").is_none());
    assert_eq!(rejected_json["outcome"]["kind"], "rejected");
    assert!(rejected_json["outcome"].get("numeric").is_none());
}

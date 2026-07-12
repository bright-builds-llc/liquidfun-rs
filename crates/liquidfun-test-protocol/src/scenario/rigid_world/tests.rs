use std::collections::HashSet;

use serde_json::{Value, json};

use super::*;
use crate::{HarnessLimits, RecordLimit, RequestId, ScenarioId, encode_jsonl};

const REQUEST: &[u8] =
    include_bytes!("../../../../../protocol/fixtures/accepted/rigid-world-request.jsonl");

fn fixture_value() -> Value {
    serde_json::from_slice(REQUEST).expect("checked-in rigid-world request should be JSON")
}

fn encode_value(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).expect("fixture mutation should serialize");
    bytes.push(b'\n');
    bytes
}

fn timeline_mut<'a>(value: &'a mut Value, family: &str) -> &'a mut Value {
    value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .iter_mut()
        .find(|timeline| timeline["witness_family"] == family)
        .expect("fixture should contain requested witness family")
}

#[test]
fn rigid_world_fixture_decodes_into_two_required_timelines() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();

    // Act
    let request = decode_rigid_world_request_jsonl(REQUEST, &limits)
        .expect("checked-in rigid-world request should decode");
    let actual = request
        .scenario()
        .timelines()
        .iter()
        .map(RigidWorldTimeline::witness_family)
        .collect::<HashSet<_>>();

    // Assert
    assert_eq!(actual, HashSet::from(RigidWorldWitnessFamily::REQUIRED));
    assert_eq!(
        encode_jsonl(&request, &limits, RecordLimit::Input)
            .expect("validated rigid-world request should encode"),
        REQUEST
    );
}

#[test]
fn rigid_world_required_family_deletion_fails_closed() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();

    // Act / Assert
    for family in RigidWorldWitnessFamily::REQUIRED {
        let mut value = fixture_value();
        value["scenario"]["timelines"]
            .as_array_mut()
            .expect("fixture timelines should be an array")
            .retain(|timeline| {
                serde_json::from_value::<RigidWorldWitnessFamily>(
                    timeline["witness_family"].clone(),
                )
                .expect("fixture family should deserialize")
                    != family
            });
        let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
            .expect_err("deleting either required family must fail");
        assert_eq!(
            error.rigid_world_kind(),
            Some(RigidWorldErrorKind::MissingWitnessFamily)
        );
    }
}

#[test]
fn rigid_world_required_witness_deletion_fails_closed() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();

    // Act / Assert
    for family in RigidWorldWitnessFamily::REQUIRED {
        for witness in family.required_witnesses() {
            let mut value = fixture_value();
            let family_name = serde_json::to_value(family)
                .expect("family should serialize")
                .as_str()
                .expect("family should serialize as a string")
                .to_owned();
            for checkpoint in timeline_mut(&mut value, &family_name)["checkpoints"]
                .as_array_mut()
                .expect("checkpoints should be an array")
            {
                checkpoint["transitions"]
                    .as_array_mut()
                    .expect("transitions should be an array")
                    .retain(|transition| {
                        serde_json::from_value::<RigidWorldWitness>(transition["witness"].clone())
                            .expect("fixture witness should deserialize")
                            != *witness
                    });
            }
            let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
                .expect_err("deleting any required witness must fail");
            assert_eq!(
                error.rigid_world_kind(),
                Some(RigidWorldErrorKind::MissingWitness)
            );
        }
    }
}

#[test]
fn rigid_world_rejects_duplicate_unknown_owner_and_out_of_order_actions() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut duplicate = fixture_value();
    let bodies = timeline_mut(&mut duplicate, "non_colliding_body_fixture_lifecycle")["bodies"]
        .as_array_mut()
        .expect("bodies should be an array");
    bodies.push(bodies[0].clone());
    let mut owner = fixture_value();
    timeline_mut(&mut owner, "non_colliding_body_fixture_lifecycle")["fixtures"][0]["owner_body_id"] =
        json!("missing-body");
    let mut ordering = fixture_value();
    let actions = timeline_mut(&mut ordering, "non_colliding_body_fixture_lifecycle")["actions"]
        .as_array_mut()
        .expect("actions should be an array");
    actions.swap(0, 3);

    // Act
    let errors = [duplicate, owner, ordering].map(|value| {
        decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
            .expect_err("invalid declaration or lifecycle ordering must fail")
    });

    // Assert
    assert_eq!(
        errors[0].rigid_world_kind(),
        Some(RigidWorldErrorKind::DuplicateBodyId)
    );
    assert_eq!(
        errors[1].rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidOwner)
    );
    assert_eq!(
        errors[2].rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidActionOrder)
    );
}

#[test]
fn rigid_world_rejects_missing_counts_and_invalid_contact_occurrence() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut missing_counts = fixture_value();
    timeline_mut(&mut missing_counts, "non_colliding_body_fixture_lifecycle")["checkpoints"][0]
        .as_object_mut()
        .expect("checkpoint should be an object")
        .remove("counts");
    let mut occurrence = fixture_value();
    timeline_mut(&mut occurrence, "single_contact_lifecycle")["checkpoints"][1]["transitions"][0]
        ["maybe_contact"]["occurrence"] = json!(0);

    // Act
    let missing_error = decode_rigid_world_request_jsonl(&encode_value(&missing_counts), &limits)
        .expect_err("omitting expected counts must fail");
    let occurrence_error = decode_rigid_world_request_jsonl(&encode_value(&occurrence), &limits)
        .expect_err("zero contact occurrence must fail");

    // Assert
    assert!(matches!(missing_error, RigidWorldDecodeError::Codec(_)));
    assert_eq!(
        occurrence_error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidContactIdentity)
    );
}

#[test]
fn rigid_world_rejects_n_plus_one_collections_before_execution() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut bodies = fixture_value();
    let declarations = timeline_mut(&mut bodies, "non_colliding_body_fixture_lifecycle")["bodies"]
        .as_array_mut()
        .expect("bodies should be an array");
    while declarations.len() < 65 {
        let mut declaration = declarations[0].clone();
        declaration["body_id"] = json!(format!("extra-body-{}", declarations.len()));
        declarations.push(declaration);
    }
    let mut actions = fixture_value();
    let action_values =
        timeline_mut(&mut actions, "non_colliding_body_fixture_lifecycle")["actions"]
            .as_array_mut()
            .expect("actions should be an array");
    while action_values.len() < 129 {
        let mut action = action_values[6].clone();
        action["action_id"] = json!(format!("extra-action-{}", action_values.len()));
        action_values.push(action);
    }

    // Act
    let errors = [bodies, actions].map(|value| {
        decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
            .expect_err("N+1 collection must fail")
    });

    // Assert
    assert!(
        errors
            .iter()
            .all(|error| matches!(error, RigidWorldDecodeError::Codec(_)))
    );
}

#[test]
fn rigid_world_rejects_unknown_and_deferred_operations() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let text = std::str::from_utf8(REQUEST).expect("fixture should be UTF-8");
    let unknown = text.replacen(
        "\"request_id\":\"phase-06-rigid-world-request\"",
        "\"request_id\":\"phase-06-rigid-world-request\",\"unknown\":true",
        1,
    );
    let deferred = text.replacen("\"kind\":\"inspect_body\"", "\"kind\":\"apply_force\"", 1);

    // Act
    let errors = [unknown, deferred].map(|record| {
        decode_rigid_world_request_jsonl(record.as_bytes(), &limits)
            .expect_err("unknown fields and deferred actions must fail")
    });

    // Assert
    assert!(
        errors
            .iter()
            .all(|error| matches!(error, RigidWorldDecodeError::Codec(_)))
    );
}

#[test]
fn rigid_world_result_round_trip_contains_only_semantic_identity() {
    // Arrange
    let checkpoint = |id: &str, family: RigidWorldWitnessFamily| RigidWorldTimelineResult {
        witness_family: family,
        checkpoints: vec![RigidWorldCheckpointResult {
            checkpoint_id: ScenarioId::new(id).expect("test checkpoint ID should validate"),
            phase: "empty".into(),
            counts: RigidExpectedCounts {
                bodies: 0,
                fixtures: 0,
                contacts: 0,
                manifold_points: 0,
                events: 0,
                destructions: 0,
            },
            bodies: Box::new([]),
            fixtures: Box::new([]),
            contacts: Box::new([]),
            events: Box::new([]),
            destructions: Box::new([]),
        }]
        .into_boxed_slice(),
    };
    let record = RigidWorldResultRecord::new(
        RequestId::new("result-request").expect("test request ID should validate"),
        ScenarioId::new("result-scenario").expect("test scenario ID should validate"),
        vec![
            checkpoint(
                "result-non-contact",
                RigidWorldWitnessFamily::NonCollidingBodyFixtureLifecycle,
            ),
            checkpoint(
                "result-contact",
                RigidWorldWitnessFamily::SingleContactLifecycle,
            ),
        ],
    )
    .expect("bounded result should construct");
    let limits = HarnessLimits::phase2_default_v1();

    // Act
    let bytes =
        encode_jsonl(&record, &limits, RecordLimit::Output).expect("bounded result should encode");
    let decoded =
        decode_rigid_world_result_jsonl(&bytes, &limits).expect("encoded result should decode");
    let text = std::str::from_utf8(&bytes).expect("result should be UTF-8");

    // Assert
    assert_eq!(decoded.timelines().len(), 2);
    assert!(!text.contains("handle"));
    assert!(!text.contains("pointer"));
}

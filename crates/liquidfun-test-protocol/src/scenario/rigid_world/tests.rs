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

fn action_mut<'a>(value: &'a mut Value, action_id: &str) -> &'a mut Value {
    value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .iter_mut()
        .flat_map(|timeline| {
            timeline["actions"]
                .as_array_mut()
                .expect("fixture actions should be an array")
        })
        .find(|action| action["action_id"] == action_id)
        .expect("fixture should contain requested action")
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "test call sites construct owned JSON action values inline"
)]
fn insert_non_colliding_action(value: &mut Value, action_id: &str, action: Value) {
    let actions = timeline_mut(value, "non_colliding_body_fixture_lifecycle")["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    let insert_at = actions
        .iter()
        .position(|record| record["action"]["kind"] == "destroy_fixture")
        .expect("fixture should contain destruction actions");
    actions.insert(
        insert_at,
        json!({ "action_id": action_id, "phase": "phase7-contract", "action": action }),
    );
}

#[test]
fn rigid_world_fixture_decodes_into_all_required_timelines() {
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
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(actual, RigidWorldWitnessFamily::ALL);
    assert_eq!(
        encode_jsonl(&request, &limits, RecordLimit::Input)
            .expect("validated rigid-world request should encode"),
        REQUEST
    );
}

#[test]
fn rigid_world_non_dynamic_admission_witnesses_follow_separate_fixed_steps() {
    // Arrange
    let mut value = fixture_value();
    let timeline = timeline_mut(&mut value, "non_colliding_body_fixture_lifecycle").clone();
    let actions = timeline["actions"]
        .as_array()
        .expect("fixture actions should be an array");
    let checkpoints = timeline["checkpoints"]
        .as_array()
        .expect("fixture checkpoints should be an array");

    // Act / Assert
    let bodies = timeline["bodies"]
        .as_array()
        .expect("fixture bodies should be an array");
    let static_body = bodies
        .iter()
        .find(|body| body["body_id"] == "nc-static")
        .expect("static admission body should exist");
    let kinematic_body = bodies
        .iter()
        .find(|body| body["body_id"] == "nc-kinematic")
        .expect("kinematic admission body should exist");
    assert_eq!(static_body["body_kind"], "static");
    assert_eq!(static_body["transform"]["position"]["x_bits"], 0);
    assert_eq!(kinematic_body["body_kind"], "kinematic");
    assert_eq!(
        kinematic_body["transform"]["position"]["x_bits"],
        1.5_f32.to_bits()
    );
    let type_change = actions
        .iter()
        .find(|action| action["action_id"] == "nc-type")
        .expect("kinematic/kinematic configuration action should exist");
    assert_eq!(type_change["action"]["body_id"], "nc-static");
    assert_eq!(type_change["action"]["body_kind"], "kinematic");

    for (action_id, checkpoint_id, witness) in [
        (
            "nc-step-static-kinematic",
            "nc-static-kinematic-rejected",
            "static_kinematic_overlap_rejected",
        ),
        (
            "nc-step-kinematic-kinematic",
            "nc-kinematic-kinematic-rejected",
            "kinematic_kinematic_overlap_rejected",
        ),
    ] {
        let action = actions
            .iter()
            .find(|action| action["action_id"] == action_id)
            .expect("admission witness step should exist");
        assert_eq!(action["action"]["kind"], "step");
        assert_eq!(action["action"]["timestep_bits"], 0x3c88_8889_u32);
        assert_eq!(action["action"]["velocity_iterations"], 8);
        assert_eq!(action["action"]["position_iterations"], 3);

        let checkpoint = checkpoints
            .iter()
            .find(|checkpoint| checkpoint["checkpoint_id"] == checkpoint_id)
            .expect("admission witness checkpoint should exist");
        assert_eq!(checkpoint["after_action_id"], action_id);
        assert_eq!(checkpoint["counts"]["contacts"], 0);
        assert_eq!(checkpoint["counts"]["manifold_points"], 0);
        assert_eq!(checkpoint["counts"]["events"], 0);
        assert_eq!(
            checkpoint["transitions"],
            json!([{
                "witness": witness,
                "maybe_contact": null
            }])
        );
    }
}

#[test]
fn rigid_world_non_dynamic_admission_step_deletion_fails_closed() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();

    // Act / Assert
    for action_id in ["nc-step-static-kinematic", "nc-step-kinematic-kinematic"] {
        let mut value = fixture_value();
        timeline_mut(&mut value, "non_colliding_body_fixture_lifecycle")["actions"]
            .as_array_mut()
            .expect("fixture actions should be an array")
            .retain(|action| action["action_id"] != action_id);
        let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
            .expect_err("deleting either admission witness step must fail");
        assert_eq!(
            error.rigid_world_kind(),
            Some(RigidWorldErrorKind::InvalidCheckpointOrder)
        );
    }
}

#[test]
fn rigid_world_non_dynamic_admission_contact_expectation_fails_closed() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    let checkpoints =
        timeline_mut(&mut value, "non_colliding_body_fixture_lifecycle")["checkpoints"]
            .as_array_mut()
            .expect("fixture checkpoints should be an array");
    let checkpoint = checkpoints
        .iter_mut()
        .find(|checkpoint| checkpoint["checkpoint_id"] == "nc-kinematic-kinematic-rejected")
        .expect("admission witness checkpoint should exist");
    checkpoint["counts"]["contacts"] = json!(1);
    checkpoint["counts"]["manifold_points"] = json!(1);

    // Act
    let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
        .expect_err("permitting an admission contact must fail");

    // Assert
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::ExpectedCountMismatch)
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
fn rigid_world_accepts_exact_action_maximum() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    let actions = timeline_mut(&mut value, "non_colliding_body_fixture_lifecycle")["actions"]
        .as_array_mut()
        .expect("actions should be an array");
    let template = actions[9].clone();
    while actions.len() < RIGID_WORLD_MAXIMUM_ACTIONS {
        let mut action = template.clone();
        action["action_id"] = json!(format!("maximum-action-{}", actions.len()));
        actions.insert(actions.len() - 6, action);
    }

    // Act
    let result = decode_rigid_world_request_jsonl(&encode_value(&value), &limits);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn rigid_world_rejects_alternate_timestep_bits() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    action_mut(&mut value, "nc-step-zero")["action"]["timestep_bits"] =
        json!(RIGID_WORLD_TIMESTEP_BITS + 1);

    // Act
    let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
        .expect_err("alternate timestep must fail before execution");

    // Assert
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidActionOrder)
    );
}

#[test]
fn rigid_world_rejects_alternate_velocity_iterations() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    action_mut(&mut value, "nc-step-zero")["action"]["velocity_iterations"] =
        json!(RIGID_WORLD_VELOCITY_ITERATIONS + 1);

    // Act
    let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
        .expect_err("alternate velocity iterations must fail before execution");

    // Assert
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidActionOrder)
    );
}

#[test]
fn rigid_world_rejects_alternate_position_iterations() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    action_mut(&mut value, "nc-step-zero")["action"]["position_iterations"] =
        json!(RIGID_WORLD_POSITION_ITERATIONS + 1);

    // Act
    let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
        .expect_err("alternate position iterations must fail before execution");

    // Assert
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidActionOrder)
    );
}

#[test]
fn rigid_world_rejects_negative_centered_inertia() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    let action = &mut action_mut(&mut value, "nc-custom-mass")["action"];
    action["mass_bits"] = json!(1.0_f32.to_bits());
    action["center"]["x_bits"] = json!(2.0_f32.to_bits());
    action["center"]["y_bits"] = json!(0.0_f32.to_bits());
    action["inertia_bits"] = json!(1.0_f32.to_bits());

    // Act
    let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
        .expect_err("negative centered inertia must fail before execution");

    // Assert
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidGeometry)
    );
}

#[test]
fn rigid_world_rejects_zero_centered_inertia() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    let action = &mut action_mut(&mut value, "nc-custom-mass")["action"];
    action["mass_bits"] = json!(1.0_f32.to_bits());
    action["center"]["x_bits"] = json!(1.0_f32.to_bits());
    action["center"]["y_bits"] = json!(0.0_f32.to_bits());
    action["inertia_bits"] = json!(1.0_f32.to_bits());

    // Act
    let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
        .expect_err("zero centered inertia must fail before execution");

    // Assert
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidGeometry)
    );
}

#[test]
fn rigid_world_accepts_zero_origin_inertia_with_nonzero_center() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    let action = &mut action_mut(&mut value, "nc-custom-mass")["action"];
    action["mass_bits"] = json!(1.0_f32.to_bits());
    action["center"]["x_bits"] = json!(1.0_f32.to_bits());
    action["center"]["y_bits"] = json!(0.0_f32.to_bits());
    action["inertia_bits"] = json!(0.0_f32.to_bits());

    // Act
    let result = decode_rigid_world_request_jsonl(&encode_value(&value), &limits);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn rigid_world_rejects_non_finite_center_dot_product() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    let action = &mut action_mut(&mut value, "nc-custom-mass")["action"];
    action["mass_bits"] = json!(1.0_f32.to_bits());
    action["center"]["x_bits"] = json!(f32::MAX.to_bits());
    action["center"]["y_bits"] = json!(0.0_f32.to_bits());
    action["inertia_bits"] = json!(f32::MAX.to_bits());

    // Act
    let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
        .expect_err("non-finite center dot product must fail before execution");

    // Assert
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidGeometry)
    );
}

#[test]
fn rigid_world_rejects_non_finite_parallel_axis_product() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    let action = &mut action_mut(&mut value, "nc-custom-mass")["action"];
    action["mass_bits"] = json!(f32::MAX.to_bits());
    action["center"]["x_bits"] = json!(2.0_f32.to_bits());
    action["center"]["y_bits"] = json!(0.0_f32.to_bits());
    action["inertia_bits"] = json!(f32::MAX.to_bits());

    // Act
    let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
        .expect_err("non-finite parallel-axis product must fail before execution");

    // Assert
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidGeometry)
    );
}

#[test]
fn rigid_world_rejects_unknown_and_deferred_operations() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut unknown = fixture_value();
    unknown
        .as_object_mut()
        .expect("fixture request should be an object")
        .insert("unknown".to_owned(), json!(true));
    let mut deferred = fixture_value();
    action_mut(&mut deferred, "nc-create-static")["action"]["kind"] = json!("create_joint");

    // Act
    let errors = [unknown, deferred].map(|record| {
        decode_rigid_world_request_jsonl(&encode_value(&record), &limits)
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
fn rigid_world_accepts_closed_phase7_actions() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    let vector = json!({ "x_bits": 1.0_f32.to_bits(), "y_bits": 2.0_f32.to_bits() });
    let body_id = "nc-dynamic";
    let fixture_id = "nc-dynamic-fixture";
    let actions = vec![
        json!({ "kind": "set_linear_velocity", "body_id": body_id, "velocity": vector }),
        json!({ "kind": "set_angular_velocity", "body_id": body_id, "angular_velocity_bits": 1.0_f32.to_bits() }),
        json!({ "kind": "apply_force", "body_id": body_id, "force": vector, "point": vector, "wake_policy": "wake" }),
        json!({ "kind": "apply_torque", "body_id": body_id, "torque_bits": 1.0_f32.to_bits(), "wake_policy": "preserve_sleep" }),
        json!({ "kind": "apply_linear_impulse", "body_id": body_id, "impulse": vector, "point": vector, "wake_policy": "wake" }),
        json!({ "kind": "apply_angular_impulse", "body_id": body_id, "impulse_bits": 1.0_f32.to_bits(), "wake_policy": "preserve_sleep" }),
        json!({ "kind": "set_body_damping", "body_id": body_id, "linear_damping_bits": 0.1_f32.to_bits(), "angular_damping_bits": 0.2_f32.to_bits() }),
        json!({ "kind": "set_gravity_scale", "body_id": body_id, "gravity_scale_bits": 1.0_f32.to_bits() }),
        json!({ "kind": "set_fixed_rotation", "body_id": body_id, "fixed_rotation": true }),
        json!({ "kind": "set_sleeping_allowed", "body_id": body_id, "sleeping_allowed": true }),
        json!({ "kind": "set_awake", "body_id": body_id, "awake": true }),
        json!({ "kind": "set_bullet", "body_id": body_id, "bullet": true }),
        json!({ "kind": "set_world_gravity", "gravity": vector }),
        json!({ "kind": "set_automatic_force_clearing", "enabled": true }),
        json!({ "kind": "set_warm_starting", "enabled": true }),
        json!({ "kind": "set_continuous_physics", "enabled": true }),
        json!({ "kind": "set_sub_stepping", "enabled": true }),
        json!({ "kind": "clear_forces" }),
        json!({ "kind": "configured_step", "timestep_bits": (1.0_f32 / 60.0).to_bits(), "velocity_iterations": 8, "position_iterations": 3, "continuous_work_budget": 4096 }),
        json!({ "kind": "query_aabb", "aabb": { "lower": { "x_bits": (-1.0_f32).to_bits(), "y_bits": (-1.0_f32).to_bits() }, "upper": vector }, "directive_rules": [{ "target": { "fixture_id": fixture_id, "child_index": 0 }, "directive": "terminate" }] }),
        json!({ "kind": "ray_cast", "start": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() }, "end": vector, "directive_rules": [{ "target": { "fixture_id": fixture_id, "child_index": 0 }, "directive": { "kind": "clip", "fraction_bits": 0.5_f32.to_bits() } }] }),
        json!({ "kind": "shift_origin", "shift": vector }),
    ];
    for (index, action) in actions.into_iter().enumerate() {
        insert_non_colliding_action(&mut value, &format!("phase7-action-{index}"), action);
    }

    // Act
    let result = decode_rigid_world_request_jsonl(&encode_value(&value), &limits);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn rigid_world_rejects_invalid_phase7_step_and_directive_bounds() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut invalid_step = fixture_value();
    insert_non_colliding_action(
        &mut invalid_step,
        "invalid-configured-step",
        json!({ "kind": "configured_step", "timestep_bits": 0.0_f32.to_bits(), "velocity_iterations": 0, "position_iterations": 3, "continuous_work_budget": 1 }),
    );
    let mut invalid_ray = fixture_value();
    insert_non_colliding_action(
        &mut invalid_ray,
        "invalid-ray-clip",
        json!({
            "kind": "ray_cast",
            "start": { "x_bits": 0.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "end": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "directive_rules": [{
                "target": { "fixture_id": "nc-dynamic-fixture", "child_index": 0 },
                "directive": { "kind": "clip", "fraction_bits": 2.0_f32.to_bits() }
            }]
        }),
    );
    let mut invalid_query_child = fixture_value();
    insert_non_colliding_action(
        &mut invalid_query_child,
        "invalid-query-child",
        json!({
            "kind": "query_aabb",
            "aabb": {
                "lower": { "x_bits": (-1.0_f32).to_bits(), "y_bits": (-1.0_f32).to_bits() },
                "upper": { "x_bits": 1.0_f32.to_bits(), "y_bits": 1.0_f32.to_bits() }
            },
            "directive_rules": [{
                "target": { "fixture_id": "nc-dynamic-fixture", "child_index": 1 },
                "directive": "terminate"
            }]
        }),
    );
    let mut invalid_ray_child = fixture_value();
    insert_non_colliding_action(
        &mut invalid_ray_child,
        "invalid-ray-child",
        json!({
            "kind": "ray_cast",
            "start": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "end": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "directive_rules": [{
                "target": { "fixture_id": "nc-dynamic-fixture", "child_index": 1 },
                "directive": { "kind": "terminate" }
            }]
        }),
    );

    // Act
    let step_error = decode_rigid_world_request_jsonl(&encode_value(&invalid_step), &limits)
        .expect_err("zero velocity iterations must fail");
    let ray_error = decode_rigid_world_request_jsonl(&encode_value(&invalid_ray), &limits)
        .expect_err("clip fractions above one must fail");
    let query_child_error =
        decode_rigid_world_request_jsonl(&encode_value(&invalid_query_child), &limits)
            .expect_err("a query selector outside fixture topology must fail");
    let ray_child_error =
        decode_rigid_world_request_jsonl(&encode_value(&invalid_ray_child), &limits)
            .expect_err("a ray selector outside fixture topology must fail");

    // Assert
    assert_eq!(
        step_error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidStepConfiguration)
    );
    assert_eq!(
        ray_error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidRayDirective)
    );
    assert_eq!(
        query_child_error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidQueryDirective)
    );
    assert_eq!(
        ray_child_error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidRayDirective)
    );
}

#[test]
fn rigid_world_rejects_both_zero_ray_clip_bit_patterns_before_execution() {
    // Arrange and Act
    let limits = HarnessLimits::phase2_default_v1();
    let errors = [0.0_f32.to_bits(), (-0.0_f32).to_bits()].map(|fraction_bits| {
        let mut value = fixture_value();
        let timeline = timeline_mut(&mut value, "world_query_and_ray_cast");
        for body_id in ["query-left", "query-center"] {
            let body = timeline["bodies"]
                .as_array_mut()
                .expect("query bodies should be an array")
                .iter_mut()
                .find(|body| body["body_id"] == body_id)
                .expect("fraction-zero witness body should exist");
            body["transform"]["position"]["x_bits"] = json!((-3.0_f32).to_bits());
        }
        let action = timeline["actions"]
            .as_array_mut()
            .expect("query actions should be an array")
            .iter_mut()
            .find(|action| action["action_id"] == "query-10")
            .expect("clip action should exist");
        action["action"]["directive_rules"][0]["directive"]["fraction_bits"] = json!(fraction_bits);
        decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
            .expect_err("zero clips must fail before multiple fraction-zero hits execute")
    });

    // Assert
    for error in errors {
        assert_eq!(
            error.rigid_world_kind(),
            Some(RigidWorldErrorKind::InvalidRayDirective)
        );
    }
}

#[test]
fn rigid_world_rejects_derived_degenerate_and_overflowing_rays_before_execution() {
    // Arrange and Act
    let limits = HarnessLimits::phase2_default_v1();
    let errors = [
        (0.0_f32.to_bits(), 0_u32, (-0.0_f32).to_bits(), 0_u32),
        (0.0_f32.to_bits(), 0_u32, 1_u32, 0_u32),
        ((-f32::MAX).to_bits(), 0_u32, f32::MAX.to_bits(), 0_u32),
        (0.0_f32.to_bits(), 0_u32, f32::MAX.to_bits(), 0_u32),
    ]
    .map(|(start_x_bits, start_y_bits, end_x_bits, end_y_bits)| {
        let mut value = fixture_value();
        let timeline = timeline_mut(&mut value, "world_query_and_ray_cast");
        let action = timeline["actions"]
            .as_array_mut()
            .expect("query actions should be an array")
            .iter_mut()
            .find(|action| action["action_id"] == "query-08")
            .expect("ray action should exist");
        action["action"]["start"] = json!({
            "x_bits": start_x_bits,
            "y_bits": start_y_bits
        });
        action["action"]["end"] = json!({
            "x_bits": end_x_bits,
            "y_bits": end_y_bits
        });
        decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
            .expect_err("invalid derived ray geometry must fail before execution")
    });

    // Assert
    for error in errors {
        assert_eq!(
            error.rigid_world_kind(),
            Some(RigidWorldErrorKind::InvalidRayDirective)
        );
    }
}

#[test]
fn rigid_world_phase7_results_expose_semantics_without_ccd_storage() {
    // Arrange
    let observations = vec![
        RigidWorldObservation::Step {
            outcome: RigidStepOutcome::Completed {
                completion: RigidStepCompletion::ContinuousPending,
            },
        },
        RigidWorldObservation::Step {
            outcome: RigidStepOutcome::Partial {
                classification: RigidPartialProgressClassification::ContinuousWorkBudgetExhausted,
            },
        },
    ]
    .into_boxed_slice();
    let timeline = RigidWorldTimelineResult {
        witness_family: RigidWorldWitnessFamily::NonCollidingBodyFixtureLifecycle,
        checkpoints: vec![RigidWorldCheckpointResult {
            checkpoint_id: ScenarioId::new("phase7-result").expect("ID should validate"),
            phase: "phase7".into(),
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
            observations,
        }]
        .into_boxed_slice(),
    };
    let record = RigidWorldResultRecord::new(
        RequestId::new("phase7-result-request").expect("ID should validate"),
        ScenarioId::new("phase7-result-scenario").expect("ID should validate"),
        vec![timeline],
    )
    .expect("bounded result should construct");
    let limits = HarnessLimits::phase2_default_v1();

    // Act
    let bytes =
        encode_jsonl(&record, &limits, RecordLimit::Output).expect("semantic result should encode");
    let decoded =
        decode_rigid_world_result_jsonl(&bytes, &limits).expect("semantic result should decode");
    let text = std::str::from_utf8(&bytes).expect("result should be UTF-8");

    // Assert
    assert_eq!(decoded.timelines()[0].checkpoints[0].observations.len(), 2);
    assert!(!text.contains("toi_count"));
    assert!(!text.contains("cache"));
    assert!(!text.contains("candidate"));
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
            observations: Box::new([]),
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

#[test]
fn rigid_world_phase8_fixture_covers_all_new_families_and_joint_kinds() {
    // Arrange
    let value = fixture_value();
    let timelines = value["scenario"]["timelines"]
        .as_array()
        .expect("fixture timelines should be an array");
    let expected_families = RigidWorldWitnessFamily::PHASE8_REQUIRED
        .map(|family| serde_json::to_value(family).expect("family should serialize"));
    let joint_timeline = timelines
        .iter()
        .find(|timeline| timeline["witness_family"] == "joint_definitions_and_mutations")
        .expect("joint definition timeline should exist");

    // Act
    let actual_families = timelines
        .iter()
        .skip(RigidWorldWitnessFamily::ALL.len() - RigidWorldWitnessFamily::PHASE8_REQUIRED.len())
        .map(|timeline| timeline["witness_family"].clone())
        .collect::<Vec<_>>();
    let actual_kinds = joint_timeline["joints"]
        .as_array()
        .expect("joint declarations should be an array")
        .iter()
        .map(|joint| joint["definition"]["kind"].clone())
        .collect::<Vec<_>>();
    let expected_kinds = [
        RigidJointKind::Revolute,
        RigidJointKind::Prismatic,
        RigidJointKind::Distance,
        RigidJointKind::Pulley,
        RigidJointKind::Mouse,
        RigidJointKind::Wheel,
        RigidJointKind::Weld,
        RigidJointKind::Friction,
        RigidJointKind::Rope,
        RigidJointKind::Motor,
        RigidJointKind::Gear,
    ]
    .map(|kind| serde_json::to_value(kind).expect("joint kind should serialize"));

    // Assert
    assert_eq!(actual_families, expected_families);
    assert_eq!(actual_kinds, expected_kinds);
}

#[test]
fn rigid_world_phase8_family_deletion_fails_closed() {
    // Arrange and Act
    let limits = HarnessLimits::phase2_default_v1();
    let errors = RigidWorldWitnessFamily::PHASE8_REQUIRED.map(|family| {
        let mut value = fixture_value();
        let family = serde_json::to_value(family).expect("family should serialize");
        value["scenario"]["timelines"]
            .as_array_mut()
            .expect("fixture timelines should be an array")
            .retain(|timeline| timeline["witness_family"] != family);
        decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
            .expect_err("missing Phase 8 family must fail")
    });

    // Assert
    assert!(errors.iter().all(|error| {
        error.rigid_world_kind() == Some(RigidWorldErrorKind::MissingWitnessFamily)
    }));
}

#[test]
fn rigid_world_phase8_accepts_every_closed_joint_mutation() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let vector = json!({ "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() });
    let mutations = [
        (
            "joint-def-revolute",
            json!({ "kind": "limit_enabled", "enabled": true }),
        ),
        (
            "joint-def-prismatic",
            json!({ "kind": "limits", "lower_bits": 0.0_f32.to_bits(), "upper_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-wheel",
            json!({ "kind": "motor_enabled", "enabled": true }),
        ),
        (
            "joint-def-revolute",
            json!({ "kind": "motor_speed", "speed_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-prismatic",
            json!({ "kind": "max_motor_force", "force_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-wheel",
            json!({ "kind": "max_motor_torque", "torque_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-distance",
            json!({ "kind": "length", "length_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-weld",
            json!({ "kind": "frequency", "frequency_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-mouse",
            json!({ "kind": "damping_ratio", "damping_ratio_bits": 0.5_f32.to_bits() }),
        ),
        (
            "joint-def-mouse",
            json!({ "kind": "mouse_target", "target": vector }),
        ),
        (
            "joint-def-friction",
            json!({ "kind": "max_force", "force_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-motor",
            json!({ "kind": "max_torque", "torque_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-gear",
            json!({ "kind": "gear_ratio", "ratio_bits": (-1.0_f32).to_bits() }),
        ),
        (
            "joint-def-rope-joint",
            json!({ "kind": "rope_max_length", "max_length_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-motor",
            json!({ "kind": "linear_offset", "offset": vector }),
        ),
        (
            "joint-def-motor",
            json!({ "kind": "angular_offset", "offset_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-motor",
            json!({ "kind": "correction_factor", "factor_bits": 0.5_f32.to_bits() }),
        ),
    ];

    // Act
    let results = mutations.map(|(joint_id, mutation)| {
        let mut value = fixture_value();
        let action = action_mut(&mut value, "joint-def-mutate");
        action["action"]["joint_id"] = json!(joint_id);
        action["action"]["mutation"] = mutation;
        decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
    });

    // Assert
    assert!(results.iter().all(Result::is_ok));
}

#[test]
fn rigid_world_phase8_rejects_mutation_for_wrong_joint_kind() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    let action = action_mut(&mut value, "joint-def-mutate");
    action["action"]["joint_id"] = json!("joint-def-pulley");
    action["action"]["mutation"] =
        json!({ "kind": "motor_speed", "speed_bits": 1.0_f32.to_bits() });

    // Act
    let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
        .expect_err("unsupported joint mutations must fail closed");

    // Assert
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidJointDefinition)
    );
}

#[test]
fn rigid_world_phase8_rejects_unknown_kind_and_n_plus_one_bounds() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut unknown = fixture_value();
    timeline_mut(&mut unknown, "joint_definitions_and_mutations")["joints"][0]["definition"]["kind"] =
        json!("unknown_joint");
    let mut joints = fixture_value();
    let declarations = timeline_mut(&mut joints, "joint_definitions_and_mutations")["joints"]
        .as_array_mut()
        .expect("joint declarations should be an array");
    while declarations.len() <= RIGID_WORLD_MAXIMUM_JOINTS {
        let mut declaration = declarations[0].clone();
        declaration["joint_id"] = json!(format!("extra-joint-{}", declarations.len()));
        declarations.push(declaration);
    }
    let mut rope = fixture_value();
    let declaration = &mut timeline_mut(&mut rope, "standalone_rope_evolution")["ropes"][0];
    while declaration["vertices"]
        .as_array()
        .expect("vertices should be an array")
        .len()
        <= RIGID_WORLD_MAXIMUM_ROPE_VERTICES
    {
        declaration["vertices"]
            .as_array_mut()
            .expect("vertices should be an array")
            .push(json!({ "x_bits": 0, "y_bits": 0 }));
        declaration["masses_bits"]
            .as_array_mut()
            .expect("masses should be an array")
            .push(json!(0));
    }

    // Act
    let errors = [unknown, joints, rope].map(|value| {
        decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
            .expect_err("unknown or N+1 Phase 8 input must fail")
    });

    // Assert
    assert!(
        errors
            .iter()
            .all(|error| matches!(error, RigidWorldDecodeError::Codec(_)))
    );
}

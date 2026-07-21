use super::*;

fn action_index(value: &Value, action_id: &str) -> usize {
    value["scenario"]["timelines"][0]["actions"]
        .as_array()
        .expect("actions should be an array")
        .iter()
        .position(|action| action["action_id"] == action_id)
        .expect("action should exist")
}

fn execute_value(value: &Value) -> (liquidfun_test_protocol::RigidWorldRequestRecord, Value) {
    let limits = HarnessLimits::phase2_default_v1();
    let request = decode_rigid_world_request_jsonl(&encode_value(value), &limits)
        .expect("Phase 10 request should decode");
    let result =
        NativeRigidWorldExecutor::execute(&request).expect("Phase 10 request should execute");
    let result_value = serde_json::to_value(result).expect("Phase 10 result should encode");
    (request, result_value)
}

fn phase10_observation_indices(result: &Value) -> Vec<usize> {
    result["timelines"][0]["checkpoints"]
        .as_array()
        .expect("checkpoints should be an array")
        .last()
        .expect("a checkpoint should exist")["observations"]
        .as_array()
        .expect("observations should be an array")
        .iter()
        .enumerate()
        .filter_map(|(index, observation)| {
            (observation["kind"] == "particle_group").then_some(index)
        })
        .collect()
}

fn validate_mutated_result(
    request: &liquidfun_test_protocol::RigidWorldRequestRecord,
    result: &Value,
) -> Result<(), RigidWorldDecodeError> {
    let decoded = decode_rigid_world_result_jsonl(
        &encode_value(result),
        &HarnessLimits::phase2_default_v1(),
    )?;
    liquidfun_test_protocol::validate_rigid_world_result_against_request(request, &decoded)
}

fn insert_inspect_before(value: &mut Value, before_action_id: &str, action_id: &str) {
    let actions = value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array");
    let insertion_index = actions
        .iter()
        .position(|action| action["action_id"] == before_action_id)
        .expect("insertion action should exist");
    actions.insert(
        insertion_index,
        json!({
            "action_id": action_id,
            "phase": "phase10",
            "action": {
                "kind": "particle_group",
                "operation": { "kind": "inspect_state" }
            }
        }),
    );
}

fn copy_phase10_observation(result: &mut Value, source: usize, target: usize) {
    let indices = phase10_observation_indices(result);
    let observations = result["timelines"][0]["checkpoints"]
        .as_array_mut()
        .expect("checkpoints should be an array")
        .last_mut()
        .expect("a checkpoint should exist")["observations"]
        .as_array_mut()
        .expect("observations should be an array");
    observations[indices[target]] = observations[indices[source]].clone();
}

fn first_phase10_event_mut(result: &mut Value) -> &mut Value {
    let observation_index = phase10_observation_indices(result)[0];
    result["timelines"][0]["checkpoints"]
        .as_array_mut()
        .expect("checkpoints should be an array")
        .last_mut()
        .expect("a checkpoint should exist")["observations"][observation_index]["observation"]
        ["state"]["events"]
        .as_array_mut()
        .expect("events should be an array")
        .first_mut()
        .expect("group creation should emit an event")
}

#[test]
fn wire_rejects_destroying_a_system_before_its_groups() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut system_first = phase10_request_value();
    let actions = system_first["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array");
    let destroy_system =
        actions.remove(action_index(&phase10_request_value(), "p10-destroy-system"));
    let destroy_group_index = actions
        .iter()
        .position(|action| action["action_id"] == "p10-destroy-c")
        .expect("group destroy action should exist");
    actions.insert(destroy_group_index, destroy_system);
    let group_first = phase10_request_value();

    // Act
    let rejected = decode_rigid_world_request_jsonl(&encode_value(&system_first), &limits);
    let accepted = decode_rigid_world_request_jsonl(&encode_value(&group_first), &limits);

    // Assert
    assert_eq!(
        rejected
            .expect_err("a system with live groups must not be destroyed")
            .rigid_world_kind(),
        Some(liquidfun_test_protocol::RigidWorldErrorKind::InvalidParticleGroupAction)
    );
    assert!(accepted.is_ok());
}

#[test]
fn wire_requires_the_exact_phase10_label_for_group_operations() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut mislabeled = phase10_request_value();
    mislabeled["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .iter_mut()
        .find(|action| action["action_id"] == "p10-create-a")
        .expect("group creation should exist")["phase"] = json!("phase-ten");
    let canonical = phase10_request_value();

    // Act
    let rejected = decode_rigid_world_request_jsonl(&encode_value(&mislabeled), &limits);
    let accepted = decode_rigid_world_request_jsonl(&encode_value(&canonical), &limits);

    // Assert
    assert_eq!(
        rejected
            .expect_err("particle-group operations require the exact phase label")
            .rigid_world_kind(),
        Some(liquidfun_test_protocol::RigidWorldErrorKind::CheckpointPhaseMismatch)
    );
    assert!(accepted.is_ok());
}

#[test]
fn result_rejects_an_inspection_bound_to_a_future_group_identity() {
    // Arrange
    let mut value = phase10_request_value();
    insert_inspect_before(&mut value, "p10-create-b", "p10-inspect-early");
    let (request, mut result) = execute_value(&value);
    copy_phase10_observation(&mut result, 1, 0);

    // Act
    let rejected = validate_mutated_result(&request, &result);

    // Assert
    assert_eq!(
        rejected
            .expect_err("an early inspection must not bind future identities")
            .rigid_world_kind(),
        Some(liquidfun_test_protocol::RigidWorldErrorKind::ResultObservationMismatch)
    );
}

#[test]
fn result_rejects_a_joined_away_group_in_a_later_inspection() {
    // Arrange
    let mut value = phase10_request_value();
    insert_inspect_before(&mut value, "p10-join", "p10-inspect-before-join");
    let (request, mut result) = execute_value(&value);
    copy_phase10_observation(&mut result, 0, 1);

    // Act
    let rejected = validate_mutated_result(&request, &result);

    // Assert
    assert_eq!(
        rejected
            .expect_err("a joined-away group must not remain in a later state")
            .rigid_world_kind(),
        Some(liquidfun_test_protocol::RigidWorldErrorKind::ResultObservationMismatch)
    );
}

#[test]
fn result_rejects_a_destroyed_group_in_a_later_inspection() {
    // Arrange
    let mut value = phase10_request_value();
    let actions = value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array");
    let insertion_index = actions
        .iter()
        .position(|action| action["action_id"] == "p10-destroy-system")
        .expect("system destroy action should exist");
    actions.splice(
        insertion_index..insertion_index,
        [
            json!({
                "action_id": "p10-compact-destroyed",
                "phase": "phase10",
                "action": { "kind": "particle_group", "operation": {
                    "kind": "step",
                    "timestep_bits": bits(1.0 / 60.0).bits(),
                    "velocity_iterations": 8,
                    "position_iterations": 3,
                    "particle_iterations": 2
                } }
            }),
            json!({
                "action_id": "p10-inspect-after-destroy",
                "phase": "phase10",
                "action": { "kind": "particle_group", "operation": {
                    "kind": "inspect_state"
                } }
            }),
        ],
    );
    let (request, mut result) = execute_value(&value);
    copy_phase10_observation(&mut result, 0, 1);

    // Act
    let rejected = validate_mutated_result(&request, &result);

    // Assert
    assert_eq!(
        rejected
            .expect_err("a destroyed group must not remain in a later state")
            .rigid_world_kind(),
        Some(liquidfun_test_protocol::RigidWorldErrorKind::ResultObservationMismatch)
    );
}

#[test]
fn result_rejects_event_fields_for_the_wrong_kind() {
    // Arrange
    let value = phase10_request_value();
    let (_request, mut result) = execute_value(&value);
    first_phase10_event_mut(&mut result)["maybe_particle_id"] = json!("particle-a");

    // Act
    let rejected = decode_rigid_world_result_jsonl(
        &encode_value(&result),
        &HarnessLimits::phase2_default_v1(),
    );

    // Assert
    assert_eq!(
        rejected
            .expect_err("event fields must match the closed event kind")
            .rigid_world_kind(),
        Some(liquidfun_test_protocol::RigidWorldErrorKind::InvalidParticleGroupResult)
    );
}

#[test]
fn result_rejects_unknown_group_event_identity() {
    // Arrange
    let value = phase10_request_value();
    let (request, mut result) = execute_value(&value);
    first_phase10_event_mut(&mut result)["maybe_group_id"] = json!("group-future");

    // Act
    let rejected = validate_mutated_result(&request, &result);

    // Assert
    assert_eq!(
        rejected
            .expect_err("event group identities must exist in the inspection prefix")
            .rigid_world_kind(),
        Some(liquidfun_test_protocol::RigidWorldErrorKind::ResultObservationMismatch)
    );
}

#[test]
fn result_rejects_unknown_particle_event_identity() {
    // Arrange
    let value = phase10_request_value();
    let (request, mut result) = execute_value(&value);
    let event = first_phase10_event_mut(&mut result);
    event["kind"] = json!("particle_destroyed");
    event["maybe_group_id"] = Value::Null;
    event["maybe_particle_id"] = json!("particle-future");

    // Act
    let rejected = validate_mutated_result(&request, &result);

    // Assert
    assert_eq!(
        rejected
            .expect_err("event particle identities must exist in the inspection prefix")
            .rigid_world_kind(),
        Some(liquidfun_test_protocol::RigidWorldErrorKind::ResultObservationMismatch)
    );
}

#[test]
fn result_rejects_unknown_other_particle_event_identity() {
    // Arrange
    let value = phase10_request_value();
    let (request, mut result) = execute_value(&value);
    let event = first_phase10_event_mut(&mut result);
    event["kind"] = json!("particle_contact_begin");
    event["maybe_group_id"] = Value::Null;
    event["maybe_particle_id"] = json!("particle-a");
    event["maybe_other_particle_id"] = json!("particle-future");

    // Act
    let rejected = validate_mutated_result(&request, &result);

    // Assert
    assert_eq!(
        rejected
            .expect_err("both contact particle identities must exist in the prefix")
            .rigid_world_kind(),
        Some(liquidfun_test_protocol::RigidWorldErrorKind::ResultObservationMismatch)
    );
}

#[test]
fn result_rejects_unknown_body_event_identity() {
    // Arrange
    let value = phase10_request_value();
    let (request, mut result) = execute_value(&value);
    let event = first_phase10_event_mut(&mut result);
    event["kind"] = json!("body_contact_begin");
    event["maybe_group_id"] = Value::Null;
    event["maybe_particle_id"] = json!("particle-a");
    event["maybe_body_id"] = json!("body-future");

    // Act
    let rejected = validate_mutated_result(&request, &result);

    // Assert
    assert_eq!(
        rejected
            .expect_err("body-contact event identities must exist in the prefix")
            .rigid_world_kind(),
        Some(liquidfun_test_protocol::RigidWorldErrorKind::ResultObservationMismatch)
    );
}

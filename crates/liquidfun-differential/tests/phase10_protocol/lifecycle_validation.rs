use super::*;

fn action_index(value: &Value, action_id: &str) -> usize {
    value["scenario"]["timelines"][0]["actions"]
        .as_array()
        .expect("actions should be an array")
        .iter()
        .position(|action| action["action_id"] == action_id)
        .expect("action should exist")
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

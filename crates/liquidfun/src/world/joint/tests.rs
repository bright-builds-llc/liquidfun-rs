use super::*;
use crate::world::fixture::test_fixture_definition;
use crate::{BodyDef, RevoluteJointDef};

fn test_world_with_bodies() -> (World, BodyId, BodyId) {
    let mut world = World::new().expect("test world key should remain available");
    let body_a = world
        .create_body(&BodyDef::default())
        .expect("body A should fit");
    let body_b = world
        .create_body(&BodyDef::default())
        .expect("body B should fit");
    (world, body_a, body_b)
}

#[test]
fn locked_creation_is_rejected_without_adjacency_effects() {
    // Arrange
    let (mut world, body_a, body_b) = test_world_with_bodies();
    let definition = RevoluteJointDef::new(body_a, body_b)
        .expect("distinct bodies form a valid joint")
        .into();
    world.step_state.set_locked_for_test(true);

    // Act
    let result = world.create_joint(definition);
    world.step_state.set_locked_for_test(false);

    // Assert
    assert_eq!(result, Err(JointCreationError::Locked));
    assert!(world.body_mut_after_validation(body_a).joints.is_empty());
    assert!(world.body_mut_after_validation(body_b).joints.is_empty());
}

#[test]
fn poisoned_creation_is_rejected_without_adjacency_effects() {
    // Arrange
    let (mut world, body_a, body_b) = test_world_with_bodies();
    let definition = RevoluteJointDef::new(body_a, body_b)
        .expect("distinct bodies form a valid joint")
        .into();
    world.step_state.set_poisoned_for_test(true);

    // Act
    let result = world.create_joint(definition);
    world.step_state.set_poisoned_for_test(false);

    // Assert
    assert_eq!(result, Err(JointCreationError::Poisoned));
    assert!(world.body_mut_after_validation(body_a).joints.is_empty());
    assert!(world.body_mut_after_validation(body_b).joints.is_empty());
}

#[test]
fn collision_suppression_refilters_only_after_last_joint_is_removed() {
    // Arrange
    let (mut world, body_a, body_b) = test_world_with_bodies();
    let fixture = world
        .create_fixture(body_a, &test_fixture_definition())
        .expect("fixture should fit");
    let definition = RevoluteJointDef::new(body_a, body_b)
        .expect("distinct bodies form a valid joint")
        .into();
    let first = world.create_joint(definition).expect("joint should fit");
    let second = world.create_joint(definition).expect("joint should fit");
    world
        .fixtures
        .get_mut(fixture)
        .expect("fixture should remain live")
        .pending_refilter = false;

    // Act
    world.destroy_joint(second).expect("joint should be live");
    let after_first = world
        .fixtures
        .get(fixture)
        .expect("fixture should remain live")
        .pending_refilter;
    world.destroy_joint(first).expect("joint should be live");
    let after_last = world
        .fixtures
        .get(fixture)
        .expect("fixture should remain live")
        .pending_refilter;

    // Assert
    assert!(!after_first);
    assert!(after_last);
}

//! Black-box source-order and bounded island-construction witnesses.

#![cfg(feature = "differential-internals")]

use liquidfun::math::Vec2;
use liquidfun::{BodyDef, BodyId, BodyType, HandleError, World};

fn dynamic_definition() -> BodyDef {
    BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
        .expect("dynamic test body definition should be valid")
}

fn create_dynamic(world: &mut World) -> BodyId {
    world
        .create_body(&dynamic_definition())
        .expect("dynamic test body should fit")
}

#[test]
fn body_order_preserves_newest_first_creation_destruction_and_slot_reuse() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let first = create_dynamic(&mut world);
    let second = create_dynamic(&mut world);
    let third = create_dynamic(&mut world);
    let fourth = create_dynamic(&mut world);
    assert_eq!(
        world.rigid_body_order_diagnostic(),
        vec![fourth, third, second, first]
    );

    // Act and Assert: middle destruction.
    world
        .destroy_body(third)
        .expect("middle body should be live");
    assert_eq!(
        world.rigid_body_order_diagnostic(),
        vec![fourth, second, first]
    );

    // Act and Assert: head destruction.
    world
        .destroy_body(fourth)
        .expect("head body should be live");
    assert_eq!(world.rigid_body_order_diagnostic(), vec![second, first]);

    // Act and Assert: tail destruction.
    world.destroy_body(first).expect("tail body should be live");
    assert_eq!(world.rigid_body_order_diagnostic(), vec![second]);

    // Act and Assert: arena slot reuse still prepends by source-list semantics.
    let replacement = create_dynamic(&mut world);
    assert_ne!(replacement, first);
    assert_eq!(
        world.rigid_body_order_diagnostic(),
        vec![replacement, second]
    );
}

#[test]
fn body_order_rejects_cross_world_destruction_without_mutation() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let first = create_dynamic(&mut world);
    let second = create_dynamic(&mut world);
    let before = world.rigid_body_order_diagnostic();
    let mut other = World::new().expect("second world key should remain available");
    let foreign = create_dynamic(&mut other);

    // Act
    let result = world.destroy_body(foreign);

    // Assert
    assert_eq!(result, Err(HandleError::WrongWorld));
    assert_eq!(world.rigid_body_order_diagnostic(), before);
    assert_eq!(before, vec![second, first]);
    assert_eq!(other.rigid_body_order_diagnostic(), vec![foreign]);
}

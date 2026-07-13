//! Transactional world-origin translation evidence.

use liquidfun::math::Vec2;
use liquidfun::{BodyDef, BodyType, OriginShiftError, World};

fn body_definition(position: Vec2) -> BodyDef {
    BodyDef::new(BodyType::Dynamic, position, 0.0, true)
        .expect("test body definition should be valid")
}

#[test]
fn origin_shift_rejects_invalid_input_atomically() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let ordinary = world
        .create_body(&body_definition(Vec2::new(8.0, -3.0)))
        .expect("test body should fit");
    let overflowing = world
        .create_body(&body_definition(Vec2::new(-f32::MAX, 2.0)))
        .expect("test body should fit");
    let ordinary_before = world
        .body_snapshot(ordinary)
        .expect("ordinary body should remain live");
    let overflowing_before = world
        .body_snapshot(overflowing)
        .expect("overflowing body should remain live");

    // Act
    let non_finite_result = world.shift_origin(Vec2::new(f32::NAN, 0.0));
    let overflow_result = world.shift_origin(Vec2::new(f32::MAX, 0.0));

    // Assert
    assert_eq!(non_finite_result, Err(OriginShiftError::NonFiniteShift));
    assert_eq!(overflow_result, Err(OriginShiftError::NonFiniteBodyState));
    assert_eq!(world.body_snapshot(ordinary), Ok(ordinary_before));
    assert_eq!(world.body_snapshot(overflowing), Ok(overflowing_before));
}

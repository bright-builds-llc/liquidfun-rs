//! Focused native rigid adapter regression tests.

use liquidfun::math::Vec2;
use liquidfun::{BodyDef, BodyType, World};
use liquidfun_test_protocol::{FloatBits, Vec2Bits};

use super::{NativeRigidWorldError, catch_native_timeline_panic, native_body_mass_data};

#[test]
fn native_centered_inertia_defense_rejects_equality_before_world_mutation() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
        .expect("dynamic body definition should be valid");
    let body = world.create_body(&definition).expect("body should fit");
    let before = world.body_snapshot(body).expect("body should remain live");

    // Act
    let result = native_body_mass_data(
        FloatBits::from_f32(1.0),
        Vec2Bits {
            x_bits: FloatBits::from_f32(1.0),
            y_bits: FloatBits::from_f32(0.0),
        },
        FloatBits::from_f32(1.0),
    );
    let after = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert_eq!(
        result,
        Err(liquidfun::BodyMassDataError::NonPositiveCenteredRotationalInertia)
    );
    assert_eq!(after, before);
}

#[test]
fn native_timeline_panic_is_a_typed_fail_closed_error() {
    // Arrange
    let timeline_id: Box<str> = "phase10-panic".into();

    // Act
    let result = catch_native_timeline_panic::<()>(timeline_id, || {
        panic!("adapter panic fixture");
    });

    // Assert
    assert!(matches!(
        result,
        Err(NativeRigidWorldError::Panic { timeline_id })
            if timeline_id.as_ref() == "phase10-panic"
    ));
}

//! Black-box checks for checked rigid body and fixture definitions.

use liquidfun::math::Vec2;
use liquidfun::{BodyDef, BodyDefError, BodyMassData, BodyMassDataError, BodyType};

#[test]
fn body_definition_round_trips_each_body_type() {
    // Arrange
    let body_types = [BodyType::Static, BodyType::Kinematic, BodyType::Dynamic];

    // Act
    let definitions = body_types.map(|body_type| {
        BodyDef::new(body_type, Vec2::new(3.5, -2.25), -0.75, false)
            .expect("finite body definition should be accepted")
    });

    // Assert
    assert_eq!(
        definitions.map(|definition| definition.body_type()),
        body_types
    );
}

#[test]
fn body_definition_preserves_transform_and_active_bits() {
    // Arrange
    let position = Vec2::new(-0.0, f32::from_bits(0x3f80_0001));
    let angle = -0.0_f32;

    // Act
    let definition = BodyDef::new(BodyType::Dynamic, position, angle, false)
        .expect("finite body definition should be accepted");
    let snapshot = definition.snapshot();

    // Assert
    assert_eq!(definition.position().x.to_bits(), position.x.to_bits());
    assert_eq!(definition.position().y.to_bits(), position.y.to_bits());
    assert_eq!(definition.angle().to_bits(), angle.to_bits());
    assert!(!definition.is_active());
    assert_eq!(snapshot.position().x.to_bits(), position.x.to_bits());
    assert_eq!(snapshot.position().y.to_bits(), position.y.to_bits());
    assert_eq!(snapshot.angle().to_bits(), angle.to_bits());
    assert_eq!(snapshot.body_type(), BodyType::Dynamic);
    assert!(!snapshot.is_active());
}

#[test]
fn body_definition_rejects_non_finite_position_x() {
    // Arrange
    let position = Vec2::new(f32::INFINITY, 0.0);

    // Act
    let result = BodyDef::new(BodyType::Static, position, 0.0, true);

    // Assert
    assert_eq!(result, Err(BodyDefError::NonFinitePositionX));
}

#[test]
fn body_definition_rejects_non_finite_position_y() {
    // Arrange
    let position = Vec2::new(0.0, f32::NEG_INFINITY);

    // Act
    let result = BodyDef::new(BodyType::Static, position, 0.0, true);

    // Assert
    assert_eq!(result, Err(BodyDefError::NonFinitePositionY));
}

#[test]
fn body_definition_rejects_non_finite_angle() {
    // Arrange
    let angle = f32::NAN;

    // Act
    let result = BodyDef::new(BodyType::Static, Vec2::ZERO, angle, true);

    // Assert
    assert_eq!(result, Err(BodyDefError::NonFiniteAngle));
}

#[test]
fn body_mass_data_preserves_valid_source_values() {
    // Arrange
    let mass = 2.0_f32;
    let center = Vec2::new(0.5, -0.25);
    let rotational_inertia = 3.0_f32;

    // Act
    let data = BodyMassData::new(mass, center, rotational_inertia)
        .expect("positive centered inertia should be accepted");

    // Assert
    assert_eq!(data.mass().to_bits(), mass.to_bits());
    assert_eq!(data.center(), center);
    assert_eq!(
        data.rotational_inertia().to_bits(),
        rotational_inertia.to_bits()
    );
    assert_eq!(
        data.centered_rotational_inertia().to_bits(),
        (rotational_inertia - mass * center.dot(center)).to_bits()
    );
}

#[test]
fn body_mass_data_rejects_negative_mass() {
    // Arrange
    let mass = -1.0;

    // Act
    let result = BodyMassData::new(mass, Vec2::ZERO, 0.0);

    // Assert
    assert_eq!(result, Err(BodyMassDataError::NegativeMass));
}

#[test]
fn body_mass_data_rejects_negative_centered_inertia() {
    // Arrange
    let mass = 2.0;
    let center = Vec2::new(2.0, 0.0);
    let rotational_inertia = 7.0;

    // Act
    let result = BodyMassData::new(mass, center, rotational_inertia);

    // Assert
    assert_eq!(
        result,
        Err(BodyMassDataError::NegativeCenteredRotationalInertia)
    );
}

#[test]
fn body_mass_data_rejects_each_non_finite_lane() {
    // Arrange
    let values = [
        BodyMassData::new(f32::NAN, Vec2::ZERO, 0.0),
        BodyMassData::new(1.0, Vec2::new(f32::INFINITY, 0.0), 1.0),
        BodyMassData::new(1.0, Vec2::new(0.0, f32::NEG_INFINITY), 1.0),
        BodyMassData::new(1.0, Vec2::ZERO, f32::NAN),
    ];

    // Act
    let errors = values.map(|result| result.expect_err("invalid mass lane must be rejected"));

    // Assert
    assert_eq!(
        errors,
        [
            BodyMassDataError::NonFiniteMass,
            BodyMassDataError::NonFiniteCenterX,
            BodyMassDataError::NonFiniteCenterY,
            BodyMassDataError::NonFiniteRotationalInertia,
        ]
    );
}

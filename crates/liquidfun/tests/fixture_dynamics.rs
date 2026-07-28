//! Black-box checks for fixture broad-phase lifecycle and deferred side effects.

use liquidfun::collision::FilterData;
use liquidfun::collision::shape::{ChainShape, CircleShape, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    AggregateMassError, BodyActivationError, BodyDef, BodyMassData, BodyMassResetError,
    BodyTransformError, BodyType, BodyTypeChangeError, CreateObjectError, FixtureBoundsError,
    FixtureDef, FixtureDefError, FixtureDestructionError, FixtureMutationError, ObjectSnapshot,
    StepConfiguration, StepHook, StepLimits, World,
};

struct NoopHook;

impl StepHook for NoopHook {}

fn phase6_step_configuration() -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3).expect("fixed test configuration should be valid")
}

fn body_definition(body_type: BodyType, active: bool) -> BodyDef {
    BodyDef::new(body_type, Vec2::ZERO, 0.0, active)
        .expect("finite body definition should be accepted")
}

fn circle_fixture() -> FixtureDef {
    let shape = Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("circle should be valid"));
    FixtureDef::new(shape, 1.0, 0.2, 0.0, false, FilterData::default())
        .expect("fixture definition should be valid")
}

fn far_circle_fixture() -> FixtureDef {
    let shape = Shape::from(
        CircleShape::new(Vec2::new(f32::MAX, 0.0), 1.0).expect("circle should be valid"),
    );
    FixtureDef::new(shape, 0.0, 0.2, 0.0, false, FilterData::default())
        .expect("fixture definition should be valid")
}

fn high_density_circle_fixture(density: f32) -> FixtureDef {
    let shape = Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("circle should be valid"));
    FixtureDef::new(shape, density, 0.2, 0.0, false, FilterData::default())
        .expect("finite high-density fixture definition should be valid")
}

fn high_density_sensor_circle_fixture(density: f32) -> FixtureDef {
    let shape = Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("circle should be valid"));
    FixtureDef::new(shape, density, 0.2, 0.0, true, FilterData::default())
        .expect("finite high-density sensor fixture definition should be valid")
}

fn assert_mass_bits_equal(actual: liquidfun::BodySnapshot, expected: liquidfun::BodySnapshot) {
    assert_eq!(actual.mass().to_bits(), expected.mass().to_bits());
    assert_eq!(
        actual.local_center().x.to_bits(),
        expected.local_center().x.to_bits()
    );
    assert_eq!(
        actual.local_center().y.to_bits(),
        expected.local_center().y.to_bits()
    );
    assert_eq!(
        actual.rotational_inertia().to_bits(),
        expected.rotational_inertia().to_bits()
    );
}

#[test]
fn proxy_active_fixture_creation_tracks_each_shape_child() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Static, true))
        .expect("body should fit");

    // Act
    let fixture = world
        .create_fixture(body, &circle_fixture())
        .expect("fixture should fit");
    let snapshot = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");

    // Assert
    assert_eq!(snapshot.broad_phase_entry_count(), 1);
    assert_eq!(world.broad_phase_entry_count(), 1);
}

#[test]
fn proxy_inactive_creation_and_activation_transitions_are_deferred() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, false))
        .expect("body should fit");
    let fixture = world
        .create_fixture(body, &circle_fixture())
        .expect("fixture should fit");

    // Act
    let initial = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");
    world
        .set_body_active(body, true)
        .expect("activation should create entries");
    let active = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");
    world
        .set_body_active(body, false)
        .expect("deactivation should remove entries");
    let inactive = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");
    world
        .set_body_active(body, true)
        .expect("reactivation should recreate entries");
    let reactivated = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");

    // Assert
    assert_eq!(initial.broad_phase_entry_count(), 0);
    assert_eq!(active.broad_phase_entry_count(), 1);
    assert_eq!(inactive.broad_phase_entry_count(), 0);
    assert_eq!(reactivated.broad_phase_entry_count(), 1);
    assert_eq!(world.broad_phase_entry_count(), 1);
}

#[test]
fn proxy_chain_fixture_owns_one_entry_per_child() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Static, true))
        .expect("body should fit");
    let vertices = [
        Vec2::new(-2.0, 0.0),
        Vec2::new(-1.0, 1.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(2.0, 0.0),
    ];
    let chain = ChainShape::open(&vertices, None, None).expect("chain should be valid");
    let expected_children = chain.child_count();
    let definition = FixtureDef::new(
        Shape::from(chain),
        0.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("fixture definition should be valid");

    // Act
    let fixture = world
        .create_fixture(body, &definition)
        .expect("fixture should fit");
    let snapshot = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");

    // Assert
    assert_eq!(snapshot.broad_phase_entry_count(), expected_children);
    assert_eq!(world.broad_phase_entry_count(), expected_children);
}

#[test]
fn proxy_transform_synchronizes_entries_without_creating_contacts() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, true))
        .expect("body should fit");
    world
        .create_fixture(body, &circle_fixture())
        .expect("fixture should fit");

    // Act
    world
        .set_body_transform(body, Vec2::new(4.0, -3.0), 0.25)
        .expect("finite transform should synchronize entries");
    let snapshot = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert_eq!(snapshot.position(), Vec2::new(4.0, -3.0));
    assert_eq!(snapshot.angle().to_bits(), 0.25_f32.to_bits());
    assert_eq!(world.broad_phase_entry_count(), 1);
}

#[test]
fn proxy_transform_overflow_rejection_is_atomic() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, true))
        .expect("body should fit");
    world
        .create_fixture(body, &far_circle_fixture())
        .expect("fixture should fit");
    let before = world.body_snapshot(body).expect("body should remain live");

    // Act
    let result = world.set_body_transform(body, Vec2::new(f32::MAX, 0.0), 0.0);
    let after = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert_eq!(
        result,
        Err(BodyTransformError::InvalidFixtureBounds(
            FixtureBoundsError::NonFiniteDerivedBounds
        ))
    );
    assert_eq!(after, before);
    assert_eq!(world.broad_phase_entry_count(), 1);
}

#[test]
fn proxy_activation_overflow_rejection_is_atomic() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let definition = BodyDef::new(BodyType::Dynamic, Vec2::new(f32::MAX, 0.0), 0.0, false)
        .expect("finite body definition should be accepted");
    let body = world.create_body(&definition).expect("body should fit");
    world
        .create_fixture(body, &far_circle_fixture())
        .expect("inactive fixture should not need entries");

    // Act
    let result = world.set_body_active(body, true);
    let snapshot = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert_eq!(
        result,
        Err(BodyActivationError::InvalidFixtureBounds(
            FixtureBoundsError::NonFiniteDerivedBounds
        ))
    );
    assert!(!snapshot.is_active());
    assert_eq!(world.broad_phase_entry_count(), 0);
}

#[test]
fn proxy_type_change_preserves_entries_for_step_time_reconsideration() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, true))
        .expect("body should fit");
    world
        .create_fixture(body, &circle_fixture())
        .expect("fixture should fit");

    // Act
    world
        .set_body_type(body, BodyType::Static)
        .expect("body should remain live");
    let snapshot = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert_eq!(snapshot.body_type(), BodyType::Static);
    assert_eq!(world.broad_phase_entry_count(), 1);
}

#[test]
fn mass_positive_density_creation_and_every_destruction_reset_body_mass() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, true))
        .expect("body should fit");
    let definition = circle_fixture();
    let expected = definition
        .shape()
        .compute_mass(definition.density())
        .expect("checked fixture mass should remain valid");

    // Act
    let fixture = world
        .create_fixture(body, &definition)
        .expect("fixture should fit");
    let after_create = world.body_snapshot(body).expect("body should remain live");
    world
        .destroy_fixture(fixture)
        .expect("fixture should remain live");
    let after_destroy = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert_eq!(after_create.mass().to_bits(), expected.mass().to_bits());
    assert_eq!(after_create.local_center(), expected.center());
    assert_eq!(
        after_create.rotational_inertia().to_bits(),
        (expected.rotational_inertia()
            - expected.mass() * expected.center().dot(expected.center()))
        .to_bits()
    );
    assert_eq!(after_destroy.mass().to_bits(), 1.0_f32.to_bits());
    assert_eq!(after_destroy.local_center(), Vec2::ZERO);
    assert_eq!(
        after_destroy.rotational_inertia().to_bits(),
        0.0_f32.to_bits()
    );
}

#[test]
fn mass_zero_density_creation_preserves_dynamic_default() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, true))
        .expect("body should fit");
    let definition = FixtureDef::new(
        circle_fixture().shape().clone(),
        0.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("fixture definition should be valid");

    // Act
    world
        .create_fixture(body, &definition)
        .expect("fixture should fit");
    let snapshot = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert_eq!(snapshot.mass().to_bits(), 1.0_f32.to_bits());
    assert_eq!(snapshot.local_center(), Vec2::ZERO);
}

#[test]
fn mass_density_edit_waits_for_explicit_reset() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, true))
        .expect("body should fit");
    let fixture = world
        .create_fixture(body, &circle_fixture())
        .expect("fixture should fit");
    let before = world.body_snapshot(body).expect("body should remain live");

    // Act
    world
        .set_fixture_density(fixture, 3.0)
        .expect("finite density should be accepted");
    let before_reset = world.body_snapshot(body).expect("body should remain live");
    world
        .reset_body_mass_data(body)
        .expect("body should remain live");
    let after_reset = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert_eq!(before_reset.mass().to_bits(), before.mass().to_bits());
    assert_eq!(before_reset.local_center(), before.local_center());
    assert_eq!(
        before_reset.rotational_inertia().to_bits(),
        before.rotational_inertia().to_bits()
    );
    assert_eq!(
        world
            .fixture_snapshot(fixture)
            .expect("fixture should remain live")
            .density()
            .to_bits(),
        3.0_f32.to_bits()
    );
    assert_eq!(
        after_reset.mass().to_bits(),
        (3.0 * std::f32::consts::PI).to_bits()
    );
}

#[test]
fn mass_custom_override_is_dynamic_only_and_replaced_by_reset_triggers() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let dynamic = world
        .create_body(&body_definition(BodyType::Dynamic, true))
        .expect("body should fit");
    let static_body = world
        .create_body(&body_definition(BodyType::Static, true))
        .expect("body should fit");
    let custom =
        BodyMassData::new(5.0, Vec2::new(0.25, -0.5), 8.0).expect("custom mass should be valid");

    // Act
    world
        .set_body_mass_data(dynamic, custom)
        .expect("dynamic body should remain live");
    world
        .set_body_mass_data(static_body, custom)
        .expect("static custom mass is a no-op");
    let dynamic_override = world
        .body_snapshot(dynamic)
        .expect("dynamic body should remain live");
    let static_after = world
        .body_snapshot(static_body)
        .expect("static body should remain live");
    world
        .create_fixture(dynamic, &circle_fixture())
        .expect("positive-density fixture should fit");
    let after_fixture = world
        .body_snapshot(dynamic)
        .expect("dynamic body should remain live");
    world
        .set_body_type(dynamic, BodyType::Static)
        .expect("body should remain live");
    let after_type = world
        .body_snapshot(dynamic)
        .expect("body should remain live");

    // Assert
    assert_eq!(dynamic_override.mass().to_bits(), 5.0_f32.to_bits());
    assert_eq!(dynamic_override.local_center(), Vec2::new(0.25, -0.5));
    assert_eq!(
        dynamic_override.rotational_inertia().to_bits(),
        custom.centered_rotational_inertia().to_bits()
    );
    assert_eq!(static_after.mass().to_bits(), 0.0_f32.to_bits());
    assert_ne!(after_fixture.mass().to_bits(), 5.0_f32.to_bits());
    assert_eq!(after_type.mass().to_bits(), 0.0_f32.to_bits());
    assert_eq!(after_type.local_center(), Vec2::ZERO);
}

#[test]
fn mass_zero_custom_value_validates_against_the_effective_unit_mass() {
    // Arrange
    let center = Vec2::new(2.0, 0.0);

    // Act
    let result = BodyMassData::new(0.0, center, 3.0);

    // Assert
    assert_eq!(
        result,
        Err(liquidfun::BodyMassDataError::NonPositiveCenteredRotationalInertia)
    );
}

#[test]
fn custom_mass_rejects_derived_center_overflow_without_panicking_or_mutating() {
    // Arrange
    let definition = BodyDef::new(BodyType::Dynamic, Vec2::new(f32::MAX, 0.0), 0.0, true)
        .expect("finite transform should be accepted");
    let mut world = World::new().expect("world key should remain available");
    let body = world.create_body(&definition).expect("body should fit");
    let before = world.body_snapshot(body).expect("body should remain live");
    let custom = BodyMassData::new(1.0, Vec2::new(f32::MAX, 0.0), 0.0)
        .expect("finite custom mass should be accepted");

    // Act
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        world.set_body_mass_data(body, custom)
    }));
    let after = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert!(matches!(
        result,
        Ok(Err(liquidfun::BodyMassMutationError::InvalidDerivedMass(
            liquidfun::AggregateMassError::NonFiniteDerivedCenter
        )))
    ));
    assert_eq!(after, before);
}

#[test]
fn custom_mass_rejects_derived_velocity_overflow_without_mutating() {
    // Arrange
    let definition = body_definition(BodyType::Dynamic, true)
        .with_angular_velocity(2.0)
        .expect("finite angular velocity should be accepted");
    let mut world = World::new().expect("world key should remain available");
    let body = world.create_body(&definition).expect("body should fit");
    let before = world.body_snapshot(body).expect("body should remain live");
    let custom = BodyMassData::new(1.0, Vec2::new(f32::MAX, 0.0), 0.0)
        .expect("finite custom mass should be accepted");

    // Act
    let result = world.set_body_mass_data(body, custom);
    let after = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert!(matches!(
        result,
        Err(liquidfun::BodyMassMutationError::InvalidDerivedMass(
            liquidfun::AggregateMassError::NonFiniteDerivedVelocity
        ))
    ));
    assert_eq!(after, before);
}

#[test]
fn aggregate_mass_overflow_rejects_fixture_creation_without_effects() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, true))
        .expect("body should fit");
    let density = f32::MAX / 4.0;
    let definition = high_density_circle_fixture(density);
    let fixture = world
        .create_fixture(body, &definition)
        .expect("one high-density fixture should have finite mass");
    let body_before = world.body_snapshot(body).expect("body should remain live");
    let fixture_before = world
        .fixture_snapshot(fixture)
        .expect("fixture should remain live");
    let proxy_count_before = world.broad_phase_entry_count();
    let contact_count_before = world.contact_count();

    // Act
    let result = world.create_fixture(body, &definition);

    // Assert
    assert_eq!(
        result,
        Err(CreateObjectError::InvalidAggregateMass(
            AggregateMassError::NonFiniteMass
        ))
    );
    assert_mass_bits_equal(
        world.body_snapshot(body).expect("body should remain live"),
        body_before,
    );
    assert_eq!(
        world
            .fixture_snapshot(fixture)
            .expect("fixture should remain live"),
        fixture_before
    );
    assert_eq!(world.broad_phase_entry_count(), proxy_count_before);
    assert_eq!(world.contact_count(), contact_count_before);

    let records = world.destroy_body(body).expect("body should remain live");
    assert!(matches!(
        records.last().map(liquidfun::DestructionRecord::snapshot),
        Some(ObjectSnapshot::Body { fixtures, .. }) if fixtures == &[fixture]
    ));
}

#[test]
fn aggregate_mass_overflow_rejects_explicit_reset_without_effects() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&body_definition(BodyType::Dynamic, true))
        .expect("body should fit");
    let density = f32::MAX / 4.0;
    let first = world
        .create_fixture(body, &high_density_circle_fixture(density))
        .expect("one high-density fixture should have finite mass");
    let second = world
        .create_fixture(body, &high_density_circle_fixture(0.0))
        .expect("zero-density fixture should not reset mass");
    world
        .set_fixture_density(second, density)
        .expect("individual high-density fixture mass should remain finite");
    let body_before = world.body_snapshot(body).expect("body should remain live");
    let first_before = world
        .fixture_snapshot(first)
        .expect("first fixture should remain live");
    let second_before = world
        .fixture_snapshot(second)
        .expect("second fixture should remain live");
    let proxy_count_before = world.broad_phase_entry_count();
    let contact_count_before = world.contact_count();

    // Act
    let result = world.reset_body_mass_data(body);

    // Assert
    assert_eq!(
        result,
        Err(BodyMassResetError::InvalidAggregateMass(
            AggregateMassError::NonFiniteMass
        ))
    );
    assert_mass_bits_equal(
        world.body_snapshot(body).expect("body should remain live"),
        body_before,
    );
    assert_eq!(
        world
            .fixture_snapshot(first)
            .expect("first fixture should remain live"),
        first_before
    );
    assert_eq!(
        world
            .fixture_snapshot(second)
            .expect("second fixture should remain live"),
        second_before
    );
    assert_eq!(world.broad_phase_entry_count(), proxy_count_before);
    assert_eq!(world.contact_count(), contact_count_before);

    let records = world.destroy_body(body).expect("body should remain live");
    assert!(matches!(
        records.last().map(liquidfun::DestructionRecord::snapshot),
        Some(ObjectSnapshot::Body { fixtures, .. }) if fixtures == &[second, first]
    ));
}

include!("fixture_dynamics/aggregate_mutation.rs");

//! Source-ordered discrete island-solver and atomic world-step witnesses.

use liquidfun::collision::{CircleShape, FilterData, PolygonShape, Shape};
use liquidfun::math::Vec2;
#[cfg(feature = "differential-internals")]
use liquidfun::rigid_differential::RigidStepFailureInjection;
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, StepConfiguration, StepError, StepHook, StepLimits,
    WakePolicy, World,
};

#[derive(Default)]
struct NoopHook;

impl StepHook for NoopHook {}

fn step_configuration(time_step: f32) -> StepConfiguration {
    StepConfiguration::new(time_step, 8, 3).expect("test step configuration should be valid")
}

fn body_definition(body_type: BodyType, position: Vec2) -> BodyDef {
    BodyDef::new(body_type, position, 0.0, true).expect("test body definition should be valid")
}

fn create_body(world: &mut World, body_type: BodyType, position: Vec2) -> BodyId {
    world
        .create_body(&body_definition(body_type, position))
        .expect("test body should fit")
}

fn circle_fixture(radius: f32, friction: f32, restitution: f32) -> FixtureDef {
    let shape = Shape::from(
        CircleShape::new(Vec2::ZERO, radius).expect("test circle shape should be valid"),
    );
    FixtureDef::new(
        shape,
        1.0,
        friction,
        restitution,
        false,
        FilterData::default(),
    )
    .expect("test fixture definition should be valid")
}

fn attach_circle(world: &mut World, body: BodyId) -> liquidfun::FixtureId {
    world
        .create_fixture(body, &circle_fixture(1.0, 0.4, 0.0))
        .expect("test fixture should fit")
}

fn box_fixture(friction: f32, restitution: f32) -> FixtureDef {
    let shape = Shape::from(
        PolygonShape::new(&[
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(-1.0, 1.0),
        ])
        .expect("test polygon should be valid"),
    );
    FixtureDef::new(
        shape,
        1.0,
        friction,
        restitution,
        false,
        FilterData::default(),
    )
    .expect("test fixture definition should be valid")
}

#[test]
fn constraints_dynamic_dynamic_contact_solves() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let first = create_body(&mut world, BodyType::Dynamic, Vec2::ZERO);
    let second = create_body(&mut world, BodyType::Dynamic, Vec2::new(1.5, 0.0));
    let _first_fixture = attach_circle(&mut world, first);
    let _second_fixture = attach_circle(&mut world, second);
    let mut hook = NoopHook;

    // Act
    let report = world
        .step(
            step_configuration(1.0 / 60.0),
            &mut hook,
            StepLimits::default(),
        )
        .expect("dynamic contact should solve in one island");

    // Assert
    assert_eq!(report.contact_solves().len(), 1);
    assert_eq!(report.contact_solves()[0].contact().points().len(), 1);
    assert!(world.body_snapshot(first).is_ok());
    assert!(world.body_snapshot(second).is_ok());
}

#[test]
fn constraints_multi_contact_island_preserves_manager_order() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let boundary = create_body(&mut world, BodyType::Static, Vec2::ZERO);
    let middle = create_body(&mut world, BodyType::Dynamic, Vec2::new(1.5, 0.0));
    let outer = create_body(&mut world, BodyType::Dynamic, Vec2::new(3.0, 0.0));
    let boundary_fixture = attach_circle(&mut world, boundary);
    let middle_fixture = attach_circle(&mut world, middle);
    let outer_fixture = attach_circle(&mut world, outer);
    let mut hook = NoopHook;

    // Act
    let report = world
        .step(
            step_configuration(1.0 / 60.0),
            &mut hook,
            StepLimits::default(),
        )
        .expect("multi-contact island should solve");

    // Assert
    assert_eq!(report.contact_solves().len(), 2);
    assert_eq!(
        report.contact_solves()[0].contact().fixtures(),
        [middle_fixture, outer_fixture]
    );
    assert_eq!(
        report.contact_solves()[1].contact().fixtures(),
        [boundary_fixture, middle_fixture]
    );
}

#[test]
fn constraints_integrate_force_gravity_and_pade_damping_before_motion() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    world
        .set_gravity(Vec2::new(0.0, -10.0))
        .expect("test gravity should be valid");
    let definition = body_definition(BodyType::Dynamic, Vec2::ZERO)
        .with_linear_velocity(Vec2::new(4.0, -2.0))
        .expect("test velocity should be valid")
        .with_linear_damping(3.0)
        .expect("test damping should be valid");
    let body = world
        .create_body(&definition)
        .expect("test body should fit");
    world
        .apply_body_force_to_center(body, Vec2::new(6.0, 0.0), WakePolicy::Wake)
        .expect("test force should be valid");
    let mut hook = NoopHook;
    let time_step = 0.25;
    let expected_velocity = (Vec2::new(4.0, -2.0)
        + time_step * (Vec2::new(0.0, -10.0) + Vec2::new(6.0, 0.0)))
        * (1.0 / (1.0 + time_step * 3.0));

    // Act
    world
        .step(
            step_configuration(time_step),
            &mut hook,
            StepLimits::default(),
        )
        .expect("unconstrained island should integrate");
    let snapshot = world
        .body_snapshot(body)
        .expect("integrated body should remain live");

    // Assert
    assert_eq!(
        snapshot.linear_velocity().x.to_bits(),
        expected_velocity.x.to_bits()
    );
    assert_eq!(
        snapshot.linear_velocity().y.to_bits(),
        expected_velocity.y.to_bits()
    );
    assert_eq!(
        snapshot.position().x.to_bits(),
        (time_step * expected_velocity.x).to_bits()
    );
    assert_eq!(
        snapshot.position().y.to_bits(),
        (time_step * expected_velocity.y).to_bits()
    );
}

#[test]
fn constraints_kinematic_velocity_stays_user_driven_while_dynamic_body_solves() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let kinematic_definition = body_definition(BodyType::Kinematic, Vec2::ZERO)
        .with_linear_velocity(Vec2::new(2.0, 0.0))
        .expect("kinematic velocity should be valid");
    let kinematic = world
        .create_body(&kinematic_definition)
        .expect("kinematic body should fit");
    let dynamic = create_body(&mut world, BodyType::Dynamic, Vec2::new(1.5, 0.0));
    let _kinematic_fixture = attach_circle(&mut world, kinematic);
    let _dynamic_fixture = attach_circle(&mut world, dynamic);
    let mut hook = NoopHook;
    let time_step = 0.1;

    // Act
    let report = world
        .step(
            step_configuration(time_step),
            &mut hook,
            StepLimits::default(),
        )
        .expect("kinematic/dynamic island should solve");
    let kinematic_after = world
        .body_snapshot(kinematic)
        .expect("kinematic body should remain live");

    // Assert
    assert_eq!(report.contact_solves().len(), 1);
    assert_eq!(
        kinematic_after.linear_velocity().x.to_bits(),
        2.0_f32.to_bits()
    );
    assert_eq!(
        kinematic_after.position().x.to_bits(),
        (time_step * 2.0).to_bits()
    );
}

#[test]
fn constraints_two_point_contact_preserves_manifold_order_and_material() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let boundary = create_body(&mut world, BodyType::Static, Vec2::ZERO);
    let dynamic = create_body(&mut world, BodyType::Dynamic, Vec2::new(0.0, 1.5));
    let _boundary_fixture = world
        .create_fixture(boundary, &box_fixture(0.25, 0.125))
        .expect("boundary fixture should fit");
    let _dynamic_fixture = world
        .create_fixture(dynamic, &box_fixture(1.0, 0.75))
        .expect("dynamic fixture should fit");
    let mut hook = NoopHook;

    // Act
    let report = world
        .step(
            step_configuration(1.0 / 60.0),
            &mut hook,
            StepLimits::default(),
        )
        .expect("two-point constraint should solve");
    let solved = report.contact_solves()[0].contact();

    // Assert
    assert_eq!(solved.points().len(), 2);
    assert_ne!(
        solved.points()[0].feature_id(),
        solved.points()[1].feature_id()
    );
    assert_eq!(solved.friction().to_bits(), 0.5_f32.to_bits());
    assert_eq!(solved.restitution().to_bits(), 0.75_f32.to_bits());
}

#[test]
fn constraints_friction_reduces_tangent_motion_after_normal_impact() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let boundary = create_body(&mut world, BodyType::Static, Vec2::ZERO);
    let dynamic_definition = body_definition(BodyType::Dynamic, Vec2::new(1.5, 0.0))
        .with_linear_velocity(Vec2::new(-2.0, 3.0))
        .expect("impact velocity should be valid");
    let dynamic = world
        .create_body(&dynamic_definition)
        .expect("dynamic body should fit");
    let _boundary_fixture = attach_circle(&mut world, boundary);
    let _dynamic_fixture = attach_circle(&mut world, dynamic);
    let mut hook = NoopHook;

    // Act
    world
        .step(
            step_configuration(1.0 / 60.0),
            &mut hook,
            StepLimits::default(),
        )
        .expect("friction constraint should solve");
    let velocity = world
        .body_snapshot(dynamic)
        .expect("dynamic body should remain live")
        .linear_velocity();

    // Assert
    assert!(velocity.y.abs() < 3.0);
}

#[test]
fn constraints_restitution_uses_the_maximum_mixed_value() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let boundary = create_body(&mut world, BodyType::Static, Vec2::ZERO);
    let dynamic_definition = body_definition(BodyType::Dynamic, Vec2::new(1.5, 0.0))
        .with_linear_velocity(Vec2::new(-2.0, 0.0))
        .expect("impact velocity should be valid");
    let dynamic = world
        .create_body(&dynamic_definition)
        .expect("dynamic body should fit");
    world
        .create_fixture(boundary, &circle_fixture(1.0, 0.0, 0.25))
        .expect("boundary fixture should fit");
    world
        .create_fixture(dynamic, &circle_fixture(1.0, 0.0, 0.75))
        .expect("dynamic fixture should fit");
    let mut hook = NoopHook;

    // Act
    let report = world
        .step(
            step_configuration(1.0 / 60.0),
            &mut hook,
            StepLimits::default(),
        )
        .expect("restitution constraint should solve");
    let velocity = world
        .body_snapshot(dynamic)
        .expect("dynamic body should remain live")
        .linear_velocity();

    // Assert
    assert_eq!(
        report.contact_solves()[0].contact().restitution().to_bits(),
        0.75_f32.to_bits()
    );
    assert!(velocity.x > 0.0);
}

#[test]
fn constraints_disabled_warm_start_still_stores_new_impulses() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    world
        .set_warm_starting_enabled(false)
        .expect("warm starting control should be available");
    let boundary = create_body(&mut world, BodyType::Static, Vec2::ZERO);
    let dynamic_definition = body_definition(BodyType::Dynamic, Vec2::new(1.5, 0.0))
        .with_linear_velocity(Vec2::new(-2.0, 0.0))
        .expect("impact velocity should be valid");
    let dynamic = world
        .create_body(&dynamic_definition)
        .expect("dynamic body should fit");
    let _boundary_fixture = attach_circle(&mut world, boundary);
    let _dynamic_fixture = attach_circle(&mut world, dynamic);
    let mut hook = NoopHook;

    // Act
    let report = world
        .step(
            step_configuration(1.0 / 60.0),
            &mut hook,
            StepLimits::default(),
        )
        .expect("cold-start constraint should solve");
    let point = report.contact_solves()[0].contact().points()[0];

    // Assert
    assert!(!world.is_warm_starting_enabled());
    assert!(point.normal_impulse() > 0.0);
    assert!(point.normal_impulse().is_finite());
    assert!(point.tangent_impulse().is_finite());
}

#[test]
fn atomic_successful_step_synchronizes_proxies_before_finding_new_contacts() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let moving_definition = body_definition(BodyType::Dynamic, Vec2::ZERO)
        .with_linear_velocity(Vec2::new(4.0, 0.0))
        .expect("moving velocity should be valid");
    let moving = world
        .create_body(&moving_definition)
        .expect("moving body should fit");
    let boundary = create_body(&mut world, BodyType::Static, Vec2::new(3.5, 0.0));
    let _moving_fixture = attach_circle(&mut world, moving);
    let _boundary_fixture = attach_circle(&mut world, boundary);
    let mut hook = NoopHook;

    // Act
    let report = world
        .step(step_configuration(0.5), &mut hook, StepLimits::default())
        .expect("moving island should commit and synchronize");

    // Assert
    assert!(report.contact_transitions().is_empty());
    assert_eq!(world.contact_count(), 1);
    assert_eq!(
        world
            .body_snapshot(moving)
            .expect("moving body should remain live")
            .position()
            .x
            .to_bits(),
        2.0_f32.to_bits()
    );
}

#[cfg(feature = "differential-internals")]
#[test]
fn atomic_late_island_failure_preserves_every_body_and_impulse_lane() {
    // Arrange
    let mut world = two_disconnected_contact_islands();
    let mut hook = NoopHook;
    world
        .step(step_configuration(0.0), &mut hook, StepLimits::default())
        .expect("zero step should discover both contact islands");
    let body_ids = world.rigid_body_order_diagnostic();
    for body in &body_ids {
        let snapshot = world
            .body_snapshot(*body)
            .expect("diagnostic body should remain live");
        if snapshot.body_type() == BodyType::Dynamic {
            world
                .set_body_linear_velocity(*body, Vec2::new(-2.0, 0.0))
                .expect("impact velocity should be valid");
        }
    }
    let bodies_before = body_ids
        .iter()
        .map(|body| world.body_snapshot(*body).expect("body should remain live"))
        .collect::<Vec<_>>();
    let contacts_before = world.rigid_contact_diagnostics();
    let limits = StepLimits::default()
        .with_rigid_failure_injection(RigidStepFailureInjection::LateIsland { solved_islands: 1 });

    // Act
    let result = world.step(step_configuration(1.0 / 60.0), &mut hook, limits);

    // Assert
    assert!(matches!(
        result,
        Err(StepError::NonFiniteSolverState { .. })
    ));
    assert_eq!(
        body_ids
            .iter()
            .map(|body| world.body_snapshot(*body).expect("body should remain live"))
            .collect::<Vec<_>>(),
        bodies_before
    );
    assert_eq!(world.rigid_contact_diagnostics(), contacts_before);
}

#[cfg(feature = "differential-internals")]
#[test]
fn atomic_proxy_bound_failure_preserves_motion_impulses_and_contact_topology() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let moving_definition = body_definition(BodyType::Dynamic, Vec2::ZERO)
        .with_linear_velocity(Vec2::new(2.0, 0.0))
        .expect("moving velocity should be valid");
    let moving = world
        .create_body(&moving_definition)
        .expect("moving body should fit");
    let fixture = attach_circle(&mut world, moving);
    let body_before = world
        .body_snapshot(moving)
        .expect("moving body should remain live");
    let contacts_before = world.rigid_contact_diagnostics();
    let limits = StepLimits::default()
        .with_rigid_failure_injection(RigidStepFailureInjection::ProxyBounds { fixture });
    let mut hook = NoopHook;

    // Act
    let result = world.step(step_configuration(0.25), &mut hook, limits);

    // Assert
    assert!(matches!(
        result,
        Err(StepError::InvalidSolverProxyBounds { .. })
    ));
    assert_eq!(
        world
            .body_snapshot(moving)
            .expect("moving body should remain live"),
        body_before
    );
    assert_eq!(world.rigid_contact_diagnostics(), contacts_before);
    assert_eq!(world.contact_count(), 0);
}

fn two_disconnected_contact_islands() -> World {
    let mut world = World::new().expect("world key should remain available");
    for offset in [0.0, 10.0] {
        let boundary = create_body(&mut world, BodyType::Static, Vec2::new(offset, 0.0));
        let dynamic = create_body(&mut world, BodyType::Dynamic, Vec2::new(offset + 1.5, 0.0));
        let _boundary_fixture = attach_circle(&mut world, boundary);
        let _dynamic_fixture = attach_circle(&mut world, dynamic);
    }
    world
}

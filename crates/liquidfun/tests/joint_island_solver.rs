#![cfg(feature = "differential-internals")]

//! Mixed joint-island ordering and exhaustive solver-dispatch regressions.

use liquidfun::StepError;
use liquidfun::math::Vec2;
use liquidfun::rigid_differential::RigidStepFailureInjection;
use liquidfun::{
    BodyDef, BodyType, DistanceJointDef, FrictionJointDef, GearJointDef, JointDef, MotorJointDef,
    MouseJointDef, PrismaticJointDef, PulleyJointDef, RevoluteJointDef, RopeJointDef,
    StepConfiguration, StepHook, StepLimits, WeldJointDef, WheelJointDef, World,
};

struct NoopHook;

impl StepHook for NoopHook {}

fn dynamic_body(position: Vec2) -> BodyDef {
    BodyDef::new(BodyType::Dynamic, position, 0.0, true)
        .expect("test body definition should be valid")
}

#[test]
fn newest_first_joint_adjacency_connects_one_discrete_island() {
    // Arrange
    let mut world = World::new().expect("test world should be available");
    let first = world
        .create_body(&dynamic_body(Vec2::new(-1.0, 0.0)))
        .expect("first body should fit");
    let second = world
        .create_body(&dynamic_body(Vec2::ZERO))
        .expect("second body should fit");
    let third = world
        .create_body(&dynamic_body(Vec2::new(1.0, 0.0)))
        .expect("third body should fit");
    world
        .create_joint(JointDef::from(
            RevoluteJointDef::new(first, second).expect("joint should be valid"),
        ))
        .expect("first joint should fit");
    world
        .create_joint(JointDef::from(
            RevoluteJointDef::new(second, third).expect("joint should be valid"),
        ))
        .expect("second joint should fit");

    // Act
    let islands = world
        .rigid_island_diagnostics()
        .expect("joint graph should build");
    let mut hook = NoopHook;
    world
        .step(
            StepConfiguration::new(1.0 / 60.0, 8, 3).expect("configuration should be valid"),
            &mut hook,
            StepLimits::default(),
        )
        .expect("joint island should solve");

    // Assert
    assert_eq!(islands.len(), 1);
    assert_eq!(islands[0].body_ids(), &[third, second, first]);
    assert_eq!(islands[0].joint_count(), 2);
}

#[test]
fn all_eleven_joint_kinds_dispatch_in_one_discrete_island() {
    // Arrange
    let mut world = World::new().expect("test world should be available");
    let base_a = world
        .create_body(&dynamic_body(Vec2::new(-3.0, 0.0)))
        .expect("base A should fit");
    let moving_a = world
        .create_body(&dynamic_body(Vec2::new(-1.0, 0.0)))
        .expect("moving A should fit");
    let base_b = world
        .create_body(&dynamic_body(Vec2::new(1.0, 0.0)))
        .expect("base B should fit");
    let moving_b = world
        .create_body(&dynamic_body(Vec2::new(3.0, 0.0)))
        .expect("moving B should fit");
    let revolute = world
        .create_joint(
            RevoluteJointDef::new(base_a, moving_a)
                .expect("revolute")
                .into(),
        )
        .expect("revolute should fit");
    let prismatic = world
        .create_joint(
            PrismaticJointDef::new(base_b, moving_b)
                .expect("prismatic")
                .into(),
        )
        .expect("prismatic should fit");
    let definitions: [JointDef; 8] = [
        DistanceJointDef::new(moving_a, moving_b)
            .expect("distance")
            .into(),
        PulleyJointDef::new(moving_a, moving_b)
            .expect("pulley")
            .into(),
        MouseJointDef::new(moving_a, moving_b)
            .expect("mouse")
            .into(),
        WheelJointDef::new(moving_a, moving_b)
            .expect("wheel")
            .into(),
        WeldJointDef::new(moving_a, moving_b).expect("weld").into(),
        FrictionJointDef::new(moving_a, moving_b)
            .expect("friction")
            .into(),
        RopeJointDef::new(moving_a, moving_b).expect("rope").into(),
        MotorJointDef::new(moving_a, moving_b)
            .expect("motor")
            .into(),
    ];
    for definition in definitions {
        world.create_joint(definition).expect("joint should fit");
    }
    world
        .create_joint(GearJointDef::new(revolute, prismatic).expect("gear").into())
        .expect("gear should fit");
    world
        .set_body_linear_velocity(moving_a, Vec2::new(4.0, 0.0))
        .expect("velocity A should be valid");
    world
        .set_body_linear_velocity(moving_b, Vec2::new(-2.0, 0.0))
        .expect("velocity B should be valid");

    // Act
    let mut hook = NoopHook;
    world
        .step(
            StepConfiguration::new(1.0 / 60.0, 8, 3).expect("configuration should be valid"),
            &mut hook,
            StepLimits::default(),
        )
        .expect("all joint variants should dispatch");
    let islands = world
        .rigid_island_diagnostics()
        .expect("joint graph should remain coherent");

    // Assert
    assert_eq!(world.joint_count(), 11);
    assert_eq!(islands.len(), 1);
    assert_eq!(islands[0].joint_count(), 11);
    for body in [base_a, moving_a, base_b, moving_b] {
        let snapshot = world.body_snapshot(body).expect("body should remain live");
        assert!(snapshot.position().is_valid());
        assert!(snapshot.linear_velocity().is_valid());
    }
}

#[test]
fn joint_warm_cache_survives_zero_step_and_late_failure_is_atomic() {
    // Arrange
    let mut world = World::new().expect("test world should be available");
    let first_a = world
        .create_body(&dynamic_body(Vec2::new(-4.0, 0.0)))
        .expect("first A should fit");
    let first_b = world
        .create_body(&dynamic_body(Vec2::new(-2.0, 0.0)))
        .expect("first B should fit");
    let second_a = world
        .create_body(&dynamic_body(Vec2::new(2.0, 0.0)))
        .expect("second A should fit");
    let second_b = world
        .create_body(&dynamic_body(Vec2::new(4.0, 0.0)))
        .expect("second B should fit");
    for (body_a, body_b) in [(first_a, first_b), (second_a, second_b)] {
        world
            .create_joint(
                DistanceJointDef::new(body_a, body_b)
                    .expect("distance")
                    .into(),
            )
            .expect("distance should fit");
    }
    world
        .set_body_linear_velocity(first_a, Vec2::new(3.0, 0.0))
        .expect("velocity should be valid");
    let configuration =
        StepConfiguration::new(1.0 / 60.0, 8, 3).expect("configuration should be valid");
    let mut hook = NoopHook;
    world
        .step(configuration, &mut hook, StepLimits::default())
        .expect("cold step should solve");
    let cache = world.rigid_joint_solver_impulse_diagnostics();
    assert!(
        cache
            .iter()
            .any(|(_joint, linear, _angular)| *linear != Vec2::ZERO)
    );
    world
        .step(
            StepConfiguration::new(0.0, 1, 1).expect("zero step should be valid"),
            &mut hook,
            StepLimits::default(),
        )
        .expect("zero step should preserve caches");
    let before_bodies = [first_a, first_b, second_a, second_b]
        .map(|body| world.body_snapshot(body).expect("body should remain live"));

    // Act
    let result = world.step(
        configuration,
        &mut hook,
        StepLimits::default().with_rigid_failure_injection(RigidStepFailureInjection::LateIsland {
            solved_islands: 1,
        }),
    );

    // Assert
    assert!(matches!(
        result,
        Err(StepError::NonFiniteSolverState { .. })
    ));
    assert_eq!(world.rigid_joint_solver_impulse_diagnostics(), cache);
    assert_eq!(
        [first_a, first_b, second_a, second_b]
            .map(|body| world.body_snapshot(body).expect("body should remain live")),
        before_bodies
    );
}

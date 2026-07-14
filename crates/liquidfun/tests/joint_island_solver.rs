#![cfg(feature = "differential-internals")]

//! Mixed joint-island ordering and exhaustive solver-dispatch regressions.

use liquidfun::StepError;
use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::rigid_differential::RigidStepFailureInjection;
use liquidfun::{
    BodyDef, BodyType, DistanceJointDef, FixtureDef, FrictionJointDef, GearJointDef, JointDef,
    MotorJointDef, MouseJointDef, PrismaticJointDef, PulleyJointDef, RevoluteJointDef,
    RopeJointDef, StepConfiguration, StepHook, StepLimits, WeldJointDef, WheelJointDef, World,
};

struct NoopHook;

impl StepHook for NoopHook {}

fn dynamic_body(position: Vec2) -> BodyDef {
    BodyDef::new(BodyType::Dynamic, position, 0.0, true)
        .expect("test body definition should be valid")
}

fn attach_mass(world: &mut World, body: liquidfun::BodyId) {
    let shape = Shape::from(CircleShape::new(Vec2::ZERO, 0.5).expect("circle"));
    let fixture = FixtureDef::new(shape, 1.0, 0.0, 0.0, false, FilterData::default())
        .expect("fixture definition");
    world.create_fixture(body, &fixture).expect("fixture");
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
    let joints = [(first_a, first_b), (second_a, second_b)].map(|(body_a, body_b)| {
        world
            .create_joint(
                DistanceJointDef::new(body_a, body_b)
                    .expect("distance")
                    .into(),
            )
            .expect("distance should fit")
    });
    world
        .set_body_linear_velocity(first_a, Vec2::new(3.0, 0.0))
        .expect("velocity should be valid");
    let configuration =
        StepConfiguration::new(1.0 / 60.0, 8, 3).expect("configuration should be valid");
    let mut hook = NoopHook;
    world
        .step(configuration, &mut hook, StepLimits::default())
        .expect("cold step should solve");
    let cache = joints.map(|joint| {
        world
            .joint_reaction_force(joint, 60.0)
            .expect("distance reaction should remain observable")
    });
    assert!(cache.into_iter().any(|reaction| reaction != Vec2::ZERO));
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
    assert_eq!(
        joints.map(|joint| world
            .joint_reaction_force(joint, 60.0)
            .expect("distance reaction should remain observable")),
        cache
    );
    assert_eq!(
        [first_a, first_b, second_a, second_b]
            .map(|body| world.body_snapshot(body).expect("body should remain live")),
        before_bodies
    );
}

#[test]
fn live_revolute_and_prismatic_runtimes_are_atomic_on_late_island_failure() {
    // Arrange
    let mut world = World::new().expect("test world should be available");
    let first_a = world
        .create_body(
            &BodyDef::new(BodyType::Static, Vec2::new(-4.0, 0.0), 0.0, true).expect("first A"),
        )
        .expect("first A should fit");
    let first_b = world
        .create_body(&dynamic_body(Vec2::new(-3.0, 0.5)))
        .expect("first B should fit");
    let second_a = world
        .create_body(
            &BodyDef::new(BodyType::Static, Vec2::new(3.0, 0.0), 0.0, true).expect("second A"),
        )
        .expect("second A should fit");
    let second_b = world
        .create_body(&dynamic_body(Vec2::new(4.5, -0.5)))
        .expect("second B should fit");
    attach_mass(&mut world, first_b);
    attach_mass(&mut world, second_b);
    let revolute = world
        .create_joint(
            RevoluteJointDef::new(first_a, first_b)
                .expect("revolute")
                .with_frame(Vec2::new(0.5, 0.0), Vec2::new(-0.25, 0.5), 0.0)
                .expect("revolute frame")
                .with_motor(true, 2.0, 10.0)
                .expect("revolute motor")
                .into(),
        )
        .expect("revolute should fit");
    let prismatic = world
        .create_joint(
            PrismaticJointDef::new(second_a, second_b)
                .expect("prismatic")
                .with_frame(
                    Vec2::new(0.25, 0.0),
                    Vec2::new(-0.5, 0.5),
                    Vec2::new(1.0, 0.0),
                    0.0,
                )
                .expect("prismatic frame")
                .with_motor(true, -2.0, 10.0)
                .expect("prismatic motor")
                .into(),
        )
        .expect("prismatic should fit");
    let configuration =
        StepConfiguration::new(1.0 / 60.0, 8, 3).expect("configuration should be valid");
    let mut hook = NoopHook;
    world
        .step(configuration, &mut hook, StepLimits::default())
        .expect("cold step should solve");
    world
        .set_body_angular_velocity(first_b, -3.0)
        .expect("revolute velocity");
    world
        .set_body_linear_velocity(second_b, Vec2::new(-4.0, 1.0))
        .expect("prismatic velocity");
    let before_joints = [revolute, prismatic].map(|joint| {
        world
            .joint_snapshot(joint)
            .expect("joint should remain live")
    });
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
    assert_eq!(
        [revolute, prismatic].map(|joint| world
            .joint_snapshot(joint)
            .expect("joint should remain live")),
        before_joints
    );
    assert_eq!(
        [first_a, first_b, second_a, second_b]
            .map(|body| world.body_snapshot(body).expect("body should remain live")),
        before_bodies
    );
}

fn create_distance_pulley_mouse_joints(
    world: &mut World,
    body_a: liquidfun::BodyId,
    body_b: liquidfun::BodyId,
) -> [liquidfun::JointId; 3] {
    let distance = world
        .create_joint(
            DistanceJointDef::new(body_a, body_b)
                .expect("distance")
                .with_anchors(Vec2::new(0.5, -0.25), Vec2::new(-0.5, 0.5))
                .expect("distance anchors")
                .with_length(1.5)
                .expect("distance length")
                .into(),
        )
        .expect("distance should fit");
    let pulley = world
        .create_joint(
            PulleyJointDef::new(body_a, body_b)
                .expect("pulley")
                .with_geometry(
                    Vec2::new(-5.0, 4.0),
                    Vec2::new(0.0, 5.0),
                    Vec2::new(0.25, 0.5),
                    Vec2::new(-0.5, 0.25),
                    4.0,
                    5.0,
                    2.0,
                )
                .expect("pulley geometry")
                .into(),
        )
        .expect("pulley should fit");
    let mouse = world
        .create_joint(
            MouseJointDef::new(body_a, body_b)
                .expect("mouse")
                .with_target(Vec2::new(-0.5, 1.0))
                .expect("mouse target")
                .with_max_force(10.0)
                .expect("mouse force")
                .into(),
        )
        .expect("mouse should fit");
    [distance, pulley, mouse]
}

#[test]
fn live_distance_pulley_and_mouse_runtimes_are_atomic_on_late_island_failure() {
    // Arrange
    let mut world = World::new().expect("test world should be available");
    let first_a = world
        .create_body(&dynamic_body(Vec2::new(-4.0, 0.0)))
        .expect("first A should fit");
    let first_b = world
        .create_body(&dynamic_body(Vec2::new(-1.0, 0.5)))
        .expect("first B should fit");
    let second_a = world
        .create_body(&dynamic_body(Vec2::new(3.0, 0.0)))
        .expect("second A should fit");
    let second_b = world
        .create_body(&dynamic_body(Vec2::new(5.0, 0.0)))
        .expect("second B should fit");
    for body in [first_a, first_b, second_a, second_b] {
        attach_mass(&mut world, body);
    }
    let [distance, pulley, mouse] =
        create_distance_pulley_mouse_joints(&mut world, first_a, first_b);
    world
        .create_joint(
            DistanceJointDef::new(second_a, second_b)
                .expect("second distance")
                .into(),
        )
        .expect("second distance should fit");
    let configuration =
        StepConfiguration::new(1.0 / 60.0, 8, 3).expect("configuration should be valid");
    let mut hook = NoopHook;
    world
        .step(configuration, &mut hook, StepLimits::default())
        .expect("cold step should solve");
    world
        .set_body_linear_velocity(first_a, Vec2::new(4.0, -1.0))
        .expect("velocity A");
    world
        .set_body_angular_velocity(first_b, -3.0)
        .expect("velocity B");
    world
        .set_mouse_target(mouse, Vec2::new(-6.0, -2.0))
        .expect("mouse target update");
    let before_joints = [distance, pulley, mouse].map(|joint| {
        world
            .joint_snapshot(joint)
            .expect("joint should remain live")
    });
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
    assert_eq!(
        [distance, pulley, mouse].map(|joint| world
            .joint_snapshot(joint)
            .expect("joint should remain live")),
        before_joints
    );
    assert_eq!(
        [first_a, first_b, second_a, second_b]
            .map(|body| world.body_snapshot(body).expect("body should remain live")),
        before_bodies
    );
}

fn create_four_body_gear(world: &mut World) -> ([liquidfun::BodyId; 4], liquidfun::JointId) {
    let body_c = world
        .create_body(&dynamic_body(Vec2::new(-3.0, 0.0)))
        .expect("body C should fit");
    let body_a = world
        .create_body(&dynamic_body(Vec2::new(-1.0, 0.5)))
        .expect("body A should fit");
    let body_d = world
        .create_body(&dynamic_body(Vec2::new(1.0, -0.5)))
        .expect("body D should fit");
    let body_b = world
        .create_body(&dynamic_body(Vec2::new(3.0, 0.0)))
        .expect("body B should fit");
    for body in [body_a, body_b, body_c, body_d] {
        attach_mass(world, body);
    }
    let source_a = world
        .create_joint(
            RevoluteJointDef::new(body_c, body_a)
                .expect("source A")
                .into(),
        )
        .expect("source A should fit");
    let source_b = world
        .create_joint(
            PrismaticJointDef::new(body_d, body_b)
                .expect("source B")
                .into(),
        )
        .expect("source B should fit");
    let gear = world
        .create_joint(
            GearJointDef::new(source_a, source_b)
                .expect("gear")
                .with_ratio(-2.0)
                .expect("ratio")
                .into(),
        )
        .expect("gear should fit");
    ([body_a, body_b, body_c, body_d], gear)
}

#[test]
fn four_body_gear_runtime_and_bodies_are_atomic_on_late_failure() {
    // Arrange
    let mut world = World::new().expect("test world should be available");
    let other_a = world
        .create_body(&dynamic_body(Vec2::new(-8.0, 0.0)))
        .expect("other A should fit");
    let other_b = world
        .create_body(&dynamic_body(Vec2::new(-6.0, 0.0)))
        .expect("other B should fit");
    attach_mass(&mut world, other_a);
    attach_mass(&mut world, other_b);
    world
        .create_joint(
            DistanceJointDef::new(other_a, other_b)
                .expect("other distance")
                .into(),
        )
        .expect("other distance should fit");
    let ([body_a, body_b, body_c, body_d], gear) = create_four_body_gear(&mut world);
    let configuration =
        StepConfiguration::new(1.0 / 60.0, 8, 3).expect("configuration should be valid");
    let mut hook = NoopHook;
    world
        .set_body_angular_velocity(body_a, 3.0)
        .expect("initial A velocity");
    world
        .set_body_linear_velocity(body_b, Vec2::new(-4.0, 1.0))
        .expect("initial B velocity");
    world
        .step(configuration, &mut hook, StepLimits::default())
        .expect("cold gear step should solve");
    world
        .set_body_angular_velocity(body_a, -5.0)
        .expect("candidate A velocity");
    world
        .set_body_linear_velocity(body_b, Vec2::new(7.0, -2.0))
        .expect("candidate B velocity");
    let before_joint = world.joint_snapshot(gear).expect("gear remains live");
    let before_reaction = (
        world.joint_reaction_force(gear, 60.0).expect("gear force"),
        world
            .joint_reaction_torque(gear, 60.0)
            .expect("gear torque"),
    );
    let before_bodies = [body_a, body_b, body_c, body_d, other_a, other_b]
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
    assert_eq!(world.joint_snapshot(gear), Ok(before_joint));
    assert_eq!(
        (
            world.joint_reaction_force(gear, 60.0).expect("gear force"),
            world
                .joint_reaction_torque(gear, 60.0)
                .expect("gear torque"),
        ),
        before_reaction
    );
    assert_eq!(
        [body_a, body_b, body_c, body_d, other_a, other_b]
            .map(|body| world.body_snapshot(body).expect("body should remain live")),
        before_bodies
    );
}

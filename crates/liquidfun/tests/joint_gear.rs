//! Integration coverage for gear ownership, inspection, mutation, and cascades.

use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyId, BodyType, DestroyedId, DestructionCause, GearJointDef, HandleError, JointId,
    JointKind, JointMutationError, JointSpecificSnapshot, PrismaticJointDef, RevoluteJointDef,
    World,
};

fn body(world: &mut World, position: Vec2, angle: f32) -> BodyId {
    world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, position, angle, true)
                .expect("finite body definition"),
        )
        .expect("body should fit")
}

fn revolute(world: &mut World, body_a: BodyId, body_b: BodyId) -> JointId {
    world
        .create_joint(
            RevoluteJointDef::new(body_a, body_b)
                .expect("distinct bodies")
                .into(),
        )
        .expect("revolute joint should fit")
}

fn prismatic(world: &mut World, body_a: BodyId, body_b: BodyId) -> JointId {
    world
        .create_joint(
            PrismaticJointDef::new(body_a, body_b)
                .expect("distinct bodies")
                .into(),
        )
        .expect("prismatic joint should fit")
}

#[test]
fn all_source_combinations_and_ratio_signs_are_inspectable() {
    // Arrange
    let combinations = [
        (false, false, 2.0),
        (false, true, -0.5),
        (true, false, 0.0),
        (true, true, 3.0),
    ];

    for (first_prismatic, second_prismatic, ratio) in combinations {
        let mut world = World::new().expect("world");
        let body_c = body(&mut world, Vec2::ZERO, 0.25);
        let body_a = body(&mut world, Vec2::new(1.0, 0.0), 0.75);
        let body_d = body(&mut world, Vec2::new(0.0, 2.0), -0.5);
        let body_b = body(&mut world, Vec2::new(3.0, 2.0), 0.5);
        let joint1 = if first_prismatic {
            prismatic(&mut world, body_c, body_a)
        } else {
            revolute(&mut world, body_c, body_a)
        };
        let joint2 = if second_prismatic {
            prismatic(&mut world, body_d, body_b)
        } else {
            revolute(&mut world, body_d, body_b)
        };

        // Act
        let gear = world
            .create_joint(
                GearJointDef::new(joint1, joint2)
                    .expect("distinct sources")
                    .with_ratio(ratio)
                    .expect("finite ratio")
                    .into(),
            )
            .expect("gear should fit");
        let snapshot = world.joint_snapshot(gear).expect("gear snapshot");

        // Assert
        assert_eq!(snapshot.kind(), JointKind::Gear);
        assert_eq!(snapshot.bodies(), [body_a, body_b]);
        let JointSpecificSnapshot::Gear(state) = snapshot.specific() else {
            panic!("expected gear runtime snapshot");
        };
        assert_eq!(state.source_joints(), [joint1, joint2]);
        assert_eq!(state.source_bodies(), [body_a, body_b, body_c, body_d]);
        assert_eq!(state.ratio().to_bits(), ratio.to_bits());
        assert_eq!(
            state.constant().to_bits(),
            (state.coordinate1() + ratio * state.coordinate2()).to_bits()
        );
    }
}

#[test]
fn invalid_dependency_inputs_leave_existing_graph_unchanged() {
    // Arrange
    let mut world = World::new().expect("world");
    let body_a = body(&mut world, Vec2::ZERO, 0.0);
    let body_b = body(&mut world, Vec2::new(1.0, 0.0), 0.0);
    let source = revolute(&mut world, body_a, body_b);
    let wrong_kind = world
        .create_joint(
            liquidfun::DistanceJointDef::new(body_a, body_b)
                .expect("distinct bodies")
                .into(),
        )
        .expect("distance joint");
    let before = world.joint_count();
    let mut other = World::new().expect("other world");
    let other_a = body(&mut other, Vec2::ZERO, 0.0);
    let other_b = body(&mut other, Vec2::new(1.0, 0.0), 0.0);
    let foreign = revolute(&mut other, other_a, other_b);

    // Act
    let wrong = world.create_joint(
        GearJointDef::new(source, wrong_kind)
            .expect("distinct sources")
            .into(),
    );
    let cross_world = world.create_joint(
        GearJointDef::new(source, foreign)
            .expect("distinct sources")
            .into(),
    );

    // Assert
    assert!(matches!(
        wrong,
        Err(liquidfun::JointCreationError::WrongDependencyKind { .. })
    ));
    assert_eq!(
        cross_world,
        Err(liquidfun::JointCreationError::InvalidHandle(
            HandleError::WrongWorld
        ))
    );
    assert_eq!(world.joint_count(), before);
}

#[test]
fn stale_source_and_explicit_gear_removal_leave_no_reverse_edges() {
    // Arrange
    let mut world = World::new().expect("world");
    let body_c = body(&mut world, Vec2::ZERO, 0.0);
    let body_a = body(&mut world, Vec2::new(1.0, 0.0), 0.0);
    let body_d = body(&mut world, Vec2::new(0.0, 1.0), 0.0);
    let body_b = body(&mut world, Vec2::new(1.0, 1.0), 0.0);
    let joint1 = revolute(&mut world, body_c, body_a);
    let joint2 = prismatic(&mut world, body_d, body_b);
    let definition = GearJointDef::new(joint1, joint2).expect("sources");
    let gear = world
        .create_joint(definition.into())
        .expect("gear should fit");

    // Act
    let gear_records = world.destroy_joint(gear).expect("gear remains live");
    let source_records = world.destroy_joint(joint1).expect("source remains live");
    let stale = world.create_joint(definition.into());

    // Assert
    assert_eq!(gear_records.len(), 1);
    assert_eq!(source_records.len(), 1);
    assert_eq!(source_records[0].destroyed(), DestroyedId::Joint(joint1));
    assert_eq!(
        stale,
        Err(liquidfun::JointCreationError::InvalidHandle(
            HandleError::StaleOrDestroyed
        ))
    );
    assert!(world.contains_joint(joint2));
}

#[test]
fn ratio_mutation_is_checked_and_preserves_the_creation_constant() {
    // Arrange
    let mut world = World::new().expect("world");
    let body_c = body(&mut world, Vec2::ZERO, 0.0);
    let body_a = body(&mut world, Vec2::new(1.0, 0.0), 1.0);
    let body_d = body(&mut world, Vec2::new(0.0, 1.0), 0.0);
    let body_b = body(&mut world, Vec2::new(2.0, 1.0), 2.0);
    let joint1 = revolute(&mut world, body_c, body_a);
    let joint2 = revolute(&mut world, body_d, body_b);
    let gear = world
        .create_joint(GearJointDef::new(joint1, joint2).expect("sources").into())
        .expect("gear");
    let before = world.joint_snapshot(gear).expect("before");
    let JointSpecificSnapshot::Gear(before_state) = before.specific() else {
        panic!("expected gear");
    };

    // Act
    world.set_gear_ratio(gear, -3.0).expect("finite ratio");
    let invalid = world.set_gear_ratio(gear, f32::NAN);
    let after = world.joint_snapshot(gear).expect("after");

    // Assert
    let JointSpecificSnapshot::Gear(after_state) = after.specific() else {
        panic!("expected gear");
    };
    assert_eq!(after_state.ratio().to_bits(), (-3.0_f32).to_bits());
    assert_eq!(
        after_state.constant().to_bits(),
        before_state.constant().to_bits()
    );
    assert_eq!(invalid, Err(JointMutationError::InvalidValue));
    assert_eq!(
        world
            .gear_joint_ratio(gear)
            .expect("ratio remains live")
            .to_bits(),
        (-3.0_f32).to_bits()
    );
}

#[test]
fn source_destruction_returns_newest_first_dependent_gears_before_source() {
    // Arrange
    let mut world = World::new().expect("world");
    let body_c = body(&mut world, Vec2::ZERO, 0.0);
    let body_a = body(&mut world, Vec2::new(1.0, 0.0), 0.0);
    let body_d = body(&mut world, Vec2::new(0.0, 1.0), 0.0);
    let body_b = body(&mut world, Vec2::new(1.0, 1.0), 0.0);
    let joint1 = revolute(&mut world, body_c, body_a);
    let joint2 = prismatic(&mut world, body_d, body_b);
    let first = world
        .create_joint(GearJointDef::new(joint1, joint2).expect("sources").into())
        .expect("first gear");
    let second = world
        .create_joint(GearJointDef::new(joint1, joint2).expect("sources").into())
        .expect("second gear");

    // Act
    let records = world.destroy_joint(joint1).expect("source remains live");

    // Assert
    assert_eq!(
        records
            .iter()
            .map(liquidfun::DestructionRecord::destroyed)
            .collect::<Vec<_>>(),
        vec![
            DestroyedId::Joint(second),
            DestroyedId::Joint(first),
            DestroyedId::Joint(joint1),
        ]
    );
    assert_eq!(
        records[0].cause(),
        DestructionCause::GearDependencyCascade { source: joint1 }
    );
    assert!(!world.contains_joint(first));
    assert!(!world.contains_joint(second));
    assert!(world.contains_joint(joint2));
}

#[test]
fn body_cascade_deduplicates_gears_before_ordinary_joint_removal() {
    // Arrange
    let mut world = World::new().expect("world");
    let root = body(&mut world, Vec2::ZERO, 0.0);
    let body_a = body(&mut world, Vec2::new(1.0, 0.0), 0.0);
    let body_b = body(&mut world, Vec2::new(0.0, 1.0), 0.0);
    let source1 = revolute(&mut world, root, body_a);
    let source2 = prismatic(&mut world, root, body_b);
    let gear = world
        .create_joint(GearJointDef::new(source1, source2).expect("sources").into())
        .expect("gear");

    // Act
    let records = world.destroy_body(root).expect("root remains live");
    let destroyed = records
        .iter()
        .map(liquidfun::DestructionRecord::destroyed)
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(
        destroyed
            .iter()
            .filter(|id| **id == DestroyedId::Joint(gear))
            .count(),
        1
    );
    assert_eq!(destroyed[0], DestroyedId::Joint(gear));
    assert!(!world.contains_joint(source1));
    assert!(!world.contains_joint(source2));
}

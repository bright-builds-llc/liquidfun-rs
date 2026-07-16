//! Black-box consumer evidence for typed identity, destruction, and associations.

use std::any::TypeId;

use liquidfun::collision::FilterData;
use liquidfun::collision::shape::{CircleShape, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    AssociationMap, BodyDef, BodyId, CreateObjectError, DestroyedId, DestructionCause, FixtureDef,
    FixtureId, HandleError, JointId, ObjectSnapshot, ParticleGroupId, ParticleId, ParticleSystemId,
    World,
};

fn test_world() -> World {
    World::new().expect("test world key should remain available")
}

fn fixture_definition() -> FixtureDef {
    let shape =
        Shape::from(CircleShape::new(Vec2::ZERO, 0.5).expect("test circle should be valid"));
    FixtureDef::new(shape, 0.0, 0.2, 0.0, false, FilterData::default())
        .expect("test fixture definition should be valid")
}

#[test]
fn public_handle_kinds_are_distinct_types() {
    // Arrange
    let kinds = [
        TypeId::of::<BodyId>(),
        TypeId::of::<FixtureId>(),
        TypeId::of::<JointId>(),
        TypeId::of::<ParticleSystemId>(),
        TypeId::of::<ParticleGroupId>(),
        TypeId::of::<ParticleId>(),
    ];

    // Act
    let all_distinct = kinds
        .iter()
        .enumerate()
        .all(|(index, kind)| !kinds[index + 1..].contains(kind));

    // Assert
    assert!(all_distinct);
}

#[test]
fn destroyed_handle_stays_stale_after_slot_reuse() {
    // Arrange
    let mut world = test_world();
    let stale = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    world.destroy_body(stale).expect("body should be live");
    let replacement = world
        .create_body(&BodyDef::default())
        .expect("reused slot should fit");

    // Act
    let result = world.destroy_body(stale);

    // Assert
    assert_eq!(result, Err(HandleError::StaleOrDestroyed));
    assert!(world.contains_body(replacement));
}

#[test]
fn cross_world_handle_fails_without_mutating_local_state() {
    // Arrange
    let mut world = test_world();
    let local = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let mut other = test_world();
    let foreign = other
        .create_body(&BodyDef::default())
        .expect("body should fit");

    // Act
    let result = world.destroy_body(foreign);

    // Assert
    assert_eq!(result, Err(HandleError::WrongWorld));
    assert!(world.contains_body(local));
}

#[test]
fn body_destruction_returns_owned_ordered_cascade_evidence() {
    // Arrange
    let mut world = test_world();
    let root = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let survivor = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let first_fixture = world
        .create_fixture(root, &fixture_definition())
        .expect("fixture should fit");
    let second_fixture = world
        .create_fixture(root, &fixture_definition())
        .expect("fixture should fit");
    let first_joint = world
        .create_joint(
            liquidfun::RevoluteJointDef::new(root, survivor)
                .expect("distinct bodies form a valid joint")
                .into(),
        )
        .expect("joint should fit");
    let second_joint = world
        .create_joint(
            liquidfun::RevoluteJointDef::new(root, survivor)
                .expect("distinct bodies form a valid joint")
                .into(),
        )
        .expect("joint should fit");

    // Act
    let records = world.destroy_body(root).expect("root should be live");

    // Assert
    assert_eq!(
        records
            .iter()
            .map(liquidfun::DestructionRecord::destroyed)
            .collect::<Vec<_>>(),
        vec![
            DestroyedId::Joint(second_joint),
            DestroyedId::Joint(first_joint),
            DestroyedId::Fixture(second_fixture),
            DestroyedId::Fixture(first_fixture),
            DestroyedId::Body(root),
        ]
    );
    assert!(matches!(
        records.last().map(liquidfun::DestructionRecord::snapshot),
        Some(ObjectSnapshot::Body {
            fixtures, joints, ..
        })
            if fixtures == &[second_fixture, first_fixture]
                && joints == &[second_joint, first_joint]
    ));
    assert_eq!(
        records.last().map(liquidfun::DestructionRecord::cause),
        Some(DestructionCause::Explicit)
    );
    assert!(world.contains_body(survivor));
}

#[test]
fn typed_association_cleanup_follows_destruction_records() {
    // Arrange
    let mut world = test_world();
    let body = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let survivor = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let first_fixture = world
        .create_fixture(body, &fixture_definition())
        .expect("fixture should fit");
    let second_fixture = world
        .create_fixture(body, &fixture_definition())
        .expect("fixture should fit");
    let first_joint = world
        .create_joint(
            liquidfun::RevoluteJointDef::new(body, survivor)
                .expect("distinct bodies form a valid joint")
                .into(),
        )
        .expect("joint should fit");
    let second_joint = world
        .create_joint(
            liquidfun::RevoluteJointDef::new(body, survivor)
                .expect("distinct bodies form a valid joint")
                .into(),
        )
        .expect("joint should fit");
    let mut body_names = AssociationMap::new();
    let mut fixture_names = AssociationMap::new();
    let mut joint_names = AssociationMap::new();
    body_names.insert(body, "body");
    fixture_names.insert(first_fixture, "first fixture");
    fixture_names.insert(second_fixture, "second fixture");
    joint_names.insert(first_joint, "first joint");
    joint_names.insert(second_joint, "second joint");
    let records = world.destroy_body(body).expect("body should be live");

    // Act
    let removed_joints = joint_names.cleanup(&records);
    let removed_fixtures = fixture_names.cleanup(&records);
    let removed_bodies = body_names.cleanup(&records);

    // Assert
    assert_eq!(removed_joints, vec!["second joint", "first joint"]);
    assert_eq!(removed_fixtures, vec!["second fixture", "first fixture"]);
    assert_eq!(removed_bodies, vec!["body"]);
    assert!(joint_names.is_empty());
    assert!(fixture_names.is_empty());
    assert!(body_names.is_empty());
    assert!(world.contains_body(survivor));
}

#[test]
fn particle_system_cascade_preserves_owned_snapshots_and_cleanup_order() {
    // Arrange
    let mut world = test_world();
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let group = world
        .create_particle_group(system)
        .expect("particle group should fit");
    let grouped = world
        .create_particle(system, Some(group))
        .expect("particle should fit")
        .created_particle();
    let ungrouped = world
        .create_particle(system, None)
        .expect("particle should fit")
        .created_particle();
    let mut system_names = AssociationMap::new();
    let mut group_names = AssociationMap::new();
    let mut particle_names = AssociationMap::new();
    system_names.insert(system, "system");
    group_names.insert(group, "group");
    particle_names.insert(grouped, "grouped");
    particle_names.insert(ungrouped, "ungrouped");

    // Act
    let records = world
        .destroy_particle_system(system)
        .expect("particle system should be live");
    let removed_groups = group_names.cleanup(&records);
    let removed_particles = particle_names.cleanup(&records);
    let removed_systems = system_names.cleanup(&records);

    // Assert
    assert_eq!(
        records
            .iter()
            .map(liquidfun::DestructionRecord::destroyed)
            .collect::<Vec<_>>(),
        vec![
            DestroyedId::ParticleGroup(group),
            DestroyedId::Particle(grouped),
            DestroyedId::Particle(ungrouped),
            DestroyedId::ParticleSystem(system),
        ]
    );
    assert!(matches!(
        records.first().map(liquidfun::DestructionRecord::snapshot),
        Some(ObjectSnapshot::ParticleGroup {
            system: snapshot_system,
            particles,
        }) if *snapshot_system == system && particles == &[grouped]
    ));
    assert!(matches!(
        records.get(1).map(liquidfun::DestructionRecord::snapshot),
        Some(ObjectSnapshot::Particle {
            system: snapshot_system,
            maybe_group,
        }) if *snapshot_system == system && *maybe_group == Some(group)
    ));
    assert!(matches!(
        records.get(2).map(liquidfun::DestructionRecord::snapshot),
        Some(ObjectSnapshot::Particle {
            system: snapshot_system,
            maybe_group,
        }) if *snapshot_system == system && maybe_group.is_none()
    ));
    assert!(matches!(
        records.last().map(liquidfun::DestructionRecord::snapshot),
        Some(ObjectSnapshot::ParticleSystem { groups, particles })
            if groups == &[group] && particles == &[grouped, ungrouped]
    ));
    assert!(!world.contains_particle_group(group));
    assert!(!world.contains_particle(grouped));
    assert!(!world.contains_particle(ungrouped));
    assert!(!world.contains_particle_system(system));
    assert_eq!(removed_groups, vec!["group"]);
    assert_eq!(removed_particles, vec!["grouped", "ungrouped"]);
    assert_eq!(removed_systems, vec!["system"]);
    assert!(group_names.is_empty());
    assert!(particle_names.is_empty());
    assert!(system_names.is_empty());
}

#[test]
fn particle_group_owner_mismatch_reports_particle_system_scope() {
    // Arrange
    let mut world = test_world();
    let first_system = world
        .create_particle_system()
        .expect("particle system should fit");
    let second_system = world
        .create_particle_system()
        .expect("particle system should fit");
    let first_group = world
        .create_particle_group(first_system)
        .expect("particle group should fit");

    // Act
    let result = world.create_particle(second_system, Some(first_group));

    // Assert
    assert_eq!(
        result,
        Err(CreateObjectError::InvalidHandle(
            HandleError::WrongParticleSystem
        ))
    );
    assert!(world.contains_particle_system(first_system));
    assert!(world.contains_particle_system(second_system));
    assert!(world.contains_particle_group(first_group));
}

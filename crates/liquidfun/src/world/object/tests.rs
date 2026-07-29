use super::*;

fn test_world() -> World {
    World::new().expect("test world key should remain available")
}

fn create_test_group(world: &mut World, system: ParticleSystemId) -> (ParticleGroupId, ParticleId) {
    let source = crate::particle::ParticleGroupSource::positions(vec![Vec2::ZERO])
        .expect("one finite position is valid");
    let recipe = crate::particle::ParticleGroupRecipe::new(
        source,
        crate::particle::ParticleGroupDestination::New,
    );
    let group = world
        .create_particle_group(system, &recipe)
        .expect("particle group should fit");
    let particle = world
        .particle_group_view(group)
        .expect("particle group remains live")
        .member_ids()[0];
    (group, particle)
}

#[test]
fn body_destruction_cascades_joints_then_fixtures_and_preserves_other_body() {
    // Arrange
    let mut world = test_world();
    let root = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let survivor = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let first_fixture = world
        .create_fixture(root, &test_fixture_definition())
        .expect("fixture should fit");
    let second_fixture = world
        .create_fixture(root, &test_fixture_definition())
        .expect("fixture should fit");
    let first_joint = world
        .create_joint(
            crate::RevoluteJointDef::new(root, survivor)
                .expect("distinct bodies form a valid joint")
                .into(),
        )
        .expect("joint should fit");
    let second_joint = world
        .create_joint(
            crate::RevoluteJointDef::new(root, survivor)
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
            .map(DestructionRecord::destroyed)
            .collect::<Vec<_>>(),
        vec![
            DestroyedId::Joint(second_joint),
            DestroyedId::Joint(first_joint),
            DestroyedId::Fixture(second_fixture),
            DestroyedId::Fixture(first_fixture),
            DestroyedId::Body(root),
        ]
    );
    assert!(!world.contains_body(root));
    assert!(!world.contains_joint(first_joint));
    assert!(!world.contains_joint(second_joint));
    assert!(!world.contains_fixture(first_fixture));
    assert!(!world.contains_fixture(second_fixture));
    assert!(world.contains_body(survivor));
    assert!(
        world
            .bodies
            .get(survivor)
            .expect("survivor remains live")
            .joints
            .is_empty()
    );
    assert!(matches!(
        records.last().map(DestructionRecord::snapshot),
        Some(ObjectSnapshot::Body {
            fixtures, joints, ..
        })
            if fixtures == &[second_fixture, first_fixture]
                && joints == &[second_joint, first_joint]
    ));
}

#[test]
fn invalid_body_destruction_is_state_preserving() {
    // Arrange
    let mut world = test_world();
    let body = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let fixture = world
        .create_fixture(body, &test_fixture_definition())
        .expect("fixture should fit");
    let mut other = test_world();
    let foreign = other
        .create_body(&BodyDef::default())
        .expect("body should fit");

    // Act
    let result = world.destroy_body(foreign);

    // Assert
    assert_eq!(result, Err(HandleError::WrongWorld));
    assert!(world.contains_body(body));
    assert!(world.contains_fixture(fixture));
    assert_eq!(world.bodies.iter().count(), 1);
    assert_eq!(world.fixtures.iter().count(), 1);
}

#[test]
fn stale_body_destruction_is_state_preserving() {
    // Arrange
    let mut world = test_world();
    let stale = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    world.destroy_body(stale).expect("body should be live");
    let survivor = world
        .create_body(&BodyDef::default())
        .expect("body should fit");

    // Act
    let result = world.destroy_body(stale);

    // Assert
    assert_eq!(result, Err(HandleError::StaleOrDestroyed));
    assert!(world.contains_body(survivor));
    assert_eq!(world.bodies.iter().count(), 1);
}

#[test]
fn particle_system_destruction_cascades_groups_then_particles() {
    // Arrange
    let mut world = test_world();
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let (group, grouped) = create_test_group(&mut world, system);
    let ungrouped = world
        .create_particle(system, None)
        .expect("particle should fit")
        .created_particle();

    // Act
    let records = world
        .destroy_particle_system(system)
        .expect("system should be live");

    // Assert
    assert_eq!(
        records
            .iter()
            .map(DestructionRecord::destroyed)
            .collect::<Vec<_>>(),
        vec![
            DestroyedId::ParticleGroup(group),
            DestroyedId::Particle(grouped),
            DestroyedId::Particle(ungrouped),
            DestroyedId::ParticleSystem(system),
        ]
    );
    assert!(!world.contains_particle_system(system));
    assert!(!world.contains_particle_group(group));
    assert!(!world.contains_particle(grouped));
    assert!(!world.contains_particle(ungrouped));
    assert!(matches!(
        records.first().map(DestructionRecord::snapshot),
        Some(ObjectSnapshot::ParticleGroup {
            system: snapshot_system,
            particles,
        }) if *snapshot_system == system && particles == &[grouped]
    ));
    assert!(matches!(
        records.get(1).map(DestructionRecord::snapshot),
        Some(ObjectSnapshot::Particle {
            system: snapshot_system,
            maybe_group,
        }) if *snapshot_system == system && *maybe_group == Some(group)
    ));
    assert!(matches!(
        records.get(2).map(DestructionRecord::snapshot),
        Some(ObjectSnapshot::Particle {
            system: snapshot_system,
            maybe_group,
        }) if *snapshot_system == system && maybe_group.is_none()
    ));
    assert!(matches!(
        records.last().map(DestructionRecord::snapshot),
        Some(ObjectSnapshot::ParticleSystem { groups, particles })
            if groups == &[group] && particles == &[grouped, ungrouped]
    ));
}

#[test]
fn invalid_particle_system_destruction_is_state_preserving() {
    // Arrange
    let mut world = test_world();
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let particle = world
        .create_particle(system, None)
        .expect("particle should fit")
        .created_particle();
    let mut other = test_world();
    let foreign = other
        .create_particle_system()
        .expect("particle system should fit");

    // Act
    let result = world.destroy_particle_system(foreign);

    // Assert
    assert_eq!(result, Err(HandleError::WrongWorld));
    assert!(world.contains_particle_system(system));
    assert!(world.contains_particle(particle));
    assert_eq!(world.particle_systems.iter().count(), 1);
    assert_eq!(
        world
            .particle_systems
            .get(system)
            .expect("system remains live")
            .storage
            .len(),
        1
    );
}

#[test]
fn direct_dependent_destruction_updates_all_adjacency() {
    // Arrange
    let mut world = test_world();
    let first = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let second = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let fixture = world
        .create_fixture(first, &test_fixture_definition())
        .expect("fixture should fit");
    let joint = world
        .create_joint(
            crate::RevoluteJointDef::new(first, second)
                .expect("distinct bodies form a valid joint")
                .into(),
        )
        .expect("joint should fit");
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let (group, particle) = create_test_group(&mut world, system);

    // Act
    world
        .destroy_fixture(fixture)
        .expect("fixture should be live");
    world.destroy_joint(joint).expect("joint should be live");
    world
        .destroy_particle(particle)
        .expect("particle should be live");
    world
        .destroy_particle_group(group)
        .expect("group should be live");

    // Assert
    assert!(
        world
            .bodies
            .get(first)
            .expect("body remains live")
            .fixtures
            .is_empty()
    );
    assert!(
        world
            .bodies
            .get(first)
            .expect("body remains live")
            .joints
            .is_empty()
    );
    assert!(
        world
            .bodies
            .get(second)
            .expect("body remains live")
            .joints
            .is_empty()
    );
    let system = world
        .particle_systems
        .get(system)
        .expect("system remains live");
    assert!(system.groups.is_empty());
    assert_eq!(system.storage.len(), 0);
}

#[test]
fn owned_records_remain_usable_after_slot_reuse() {
    // Arrange
    let mut world = test_world();
    let body = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let fixture = world
        .create_fixture(body, &test_fixture_definition())
        .expect("fixture should fit");

    // Act
    let records = world.destroy_body(body).expect("body should be live");
    let replacement = world
        .create_body(&BodyDef::default())
        .expect("reused slot should fit");

    // Assert
    assert_ne!(body, replacement);
    assert_eq!(records[0].destroyed(), DestroyedId::Fixture(fixture));
    assert_eq!(records[1].destroyed(), DestroyedId::Body(body));
    assert!(matches!(
        records[0].snapshot(),
        ObjectSnapshot::Fixture {
            body: snapshot_body,
            ..
        } if *snapshot_body == body
    ));
}

#[test]
fn diagnostic_identity_exhaustion_rejects_insertion() {
    // Arrange
    let mut world = test_world();
    world.set_next_diagnostic_id_for_test(u64::MAX - 1);
    world
        .create_body(&BodyDef::default())
        .expect("penultimate ID should remain valid");
    world
        .create_body(&BodyDef::default())
        .expect("maximum ID should remain valid");

    // Act
    let result = world.create_body(&BodyDef::default());

    // Assert
    assert_eq!(result, Err(ArenaInsertError::DiagnosticIdExhausted));
    assert_eq!(world.bodies.iter().count(), 2);
    assert_eq!(
        world
            .bodies
            .iter()
            .map(|(_body, record)| record.diagnostic_id)
            .collect::<Vec<_>>(),
        vec![u64::MAX - 1, u64::MAX]
    );
}

#[test]
fn failed_particle_replacement_preserves_the_eviction_candidate() {
    // Arrange
    let mut world = test_world();
    let definition = ParticleSystemDef::default()
        .with_maximum_count(1)
        .expect("one particle is a valid maximum")
        .with_destruction_by_age(true);
    let system = world
        .create_particle_system_with_def(&definition)
        .expect("particle system should fit");
    let victim = world
        .create_particle_with_def(
            system,
            None,
            &crate::ParticleDef::default().with_flags(crate::ParticleFlags::DESTRUCTION_LISTENER),
        )
        .expect("first particle should fit")
        .created_particle();
    world.set_next_diagnostic_id_for_test(u64::MAX);
    world
        .create_body(&BodyDef::default())
        .expect("maximum diagnostic ID should remain valid");
    let before_particle = world
        .particle_snapshot(victim)
        .expect("eviction candidate should be live");
    let before_system = world
        .particle_system_snapshot(system)
        .expect("particle system should be live");
    let before_body_count = world.bodies.iter().count();

    // Act
    let result = world.create_particle(system, None);

    // Assert
    assert_eq!(
        result,
        Err(CreateObjectError::Arena(
            ArenaInsertError::DiagnosticIdExhausted
        ))
    );
    assert_eq!(world.particle_snapshot(victim), Ok(before_particle));
    assert_eq!(world.particle_system_snapshot(system), Ok(before_system));
    assert_eq!(world.bodies.iter().count(), before_body_count);
    assert_eq!(world.next_diagnostic_id, None);
}

#[test]
fn sensor_change_records_pending_body_wake() {
    // Arrange
    let mut world = test_world();
    let body = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let fixture = world
        .create_fixture(body, &test_fixture_definition())
        .expect("fixture should fit");

    // Act
    world
        .set_fixture_sensor(fixture, true)
        .expect("fixture should remain live");

    // Assert
    assert!(
        world
            .bodies
            .get(body)
            .expect("body should remain live")
            .pending_wake
    );
}

#[test]
fn filter_change_records_refilter_and_touches_without_entry_churn() {
    // Arrange
    let mut world = test_world();
    let body = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let fixture = world
        .create_fixture(body, &test_fixture_definition())
        .expect("fixture should fit");
    let before_count = world.broad_phase.proxy_count();

    // Act
    world
        .set_fixture_filter(fixture, FilterData::new(0x0002, 0x0004, -1))
        .expect("fixture should remain live");

    // Assert
    assert!(
        world
            .fixtures
            .get(fixture)
            .expect("fixture should remain live")
            .pending_refilter
    );
    assert_eq!(world.broad_phase.proxy_count(), before_count);
}

#[test]
fn type_change_records_pending_wake_and_contact_destruction() {
    // Arrange
    let mut world = test_world();
    let definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
        .expect("body definition should be valid");
    let body = world.create_body(&definition).expect("body should fit");

    // Act
    world
        .set_body_type(body, BodyType::Static)
        .expect("body should remain live");

    // Assert
    let record = world.bodies.get(body).expect("body should remain live");
    assert!(record.pending_wake);
    assert!(record.pending_contact_destruction);
}

#[test]
fn deactivation_records_pending_contact_destruction() {
    // Arrange
    let mut world = test_world();
    let body = world
        .create_body(&BodyDef::default())
        .expect("body should fit");

    // Act
    world
        .set_body_active(body, false)
        .expect("body should remain live");

    // Assert
    assert!(
        world
            .bodies
            .get(body)
            .expect("body should remain live")
            .pending_contact_destruction
    );
}

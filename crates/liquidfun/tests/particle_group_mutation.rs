//! Black-box stable-identity and rollback evidence for particle-group mutations.

use std::panic::{AssertUnwindSafe, catch_unwind};

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::particle::{
    ParticleGroupDestination, ParticleGroupFlags, ParticleGroupMutationError, ParticleGroupRecipe,
    ParticleGroupSource,
};
use liquidfun::{
    AssociationMap, BodyDef, BodyType, DestroyedId, FixtureDef, HandleError, ParticleFlags,
    ParticleGroupId, ParticleSystemId, StepConfiguration, StepHook, StepLifecycleEvent, StepLimits,
    World,
};
use proptest::prelude::*;

fn recipe(positions: Vec<Vec2>, flags: ParticleGroupFlags) -> ParticleGroupRecipe {
    let source = ParticleGroupSource::positions(positions).expect("positions are finite");
    ParticleGroupRecipe::new(source, ParticleGroupDestination::New).with_group_flags(flags)
}

fn create_group(
    world: &mut World,
    system: ParticleSystemId,
    positions: Vec<Vec2>,
    flags: ParticleGroupFlags,
) -> ParticleGroupId {
    world
        .create_particle_group(system, &recipe(positions, flags))
        .expect("test group should fit")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GroupState {
    flags: ParticleGroupFlags,
    members: Vec<liquidfun::ParticleId>,
    depths: Option<Vec<u32>>,
    transform: [u32; 4],
    center: [u32; 2],
    velocity: [u32; 2],
    angular_velocity: u32,
    mass: u32,
    inertia: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParticleState {
    id: liquidfun::ParticleId,
    maybe_group: Option<ParticleGroupId>,
    position: [u32; 2],
    velocity: [u32; 2],
    flags: ParticleFlags,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticSnapshot {
    particles: Vec<ParticleState>,
    groups: Vec<(ParticleGroupId, Result<GroupState, HandleError>)>,
    contacts: Vec<([liquidfun::ParticleId; 2], ParticleFlags, u32, [u32; 2])>,
    pairs: Vec<([liquidfun::ParticleId; 2], ParticleFlags, u32, u32)>,
    triads: Vec<([liquidfun::ParticleId; 3], ParticleFlags, [u32; 8])>,
    group_count: usize,
}

fn bits(vector: Vec2) -> [u32; 2] {
    [vector.x.to_bits(), vector.y.to_bits()]
}

fn semantic_snapshot(
    world: &World,
    system: ParticleSystemId,
    known_groups: &[ParticleGroupId],
) -> SemanticSnapshot {
    let system_view = world
        .particle_system_view(system)
        .expect("property system remains live");
    let particles = particle_states(world, &system_view);
    let groups = group_states(world, known_groups);
    let contacts = system_view
        .particle_contacts()
        .map(|contact| {
            (
                contact.particles(),
                contact.flags(),
                contact.weight().to_bits(),
                bits(contact.normal()),
            )
        })
        .collect();
    let pairs = system_view
        .pairs()
        .map(|pair| {
            (
                pair.particles(),
                pair.flags(),
                pair.strength().to_bits(),
                pair.distance().to_bits(),
            )
        })
        .collect();
    let triads = system_view
        .triads()
        .map(|triad| {
            (
                triad.particles(),
                triad.flags(),
                [
                    triad.strength().to_bits(),
                    triad.pa().x.to_bits(),
                    triad.pa().y.to_bits(),
                    triad.pb().x.to_bits(),
                    triad.pb().y.to_bits(),
                    triad.pc().x.to_bits(),
                    triad.pc().y.to_bits(),
                    triad.s().to_bits(),
                ],
            )
        })
        .collect();
    let group_count = world
        .particle_system_statistics(system)
        .expect("property system remains live")
        .group_count();
    SemanticSnapshot {
        particles,
        groups,
        contacts,
        pairs,
        triads,
        group_count,
    }
}

fn particle_states(
    world: &World,
    system_view: &liquidfun::ParticleSystemView<'_>,
) -> Vec<ParticleState> {
    system_view
        .particle_ids()
        .iter()
        .copied()
        .map(|particle| {
            let snapshot = world
                .particle_snapshot(particle)
                .expect("snapshot particle remains live");
            ParticleState {
                id: particle,
                maybe_group: snapshot.maybe_group(),
                position: bits(snapshot.position()),
                velocity: bits(snapshot.velocity()),
                flags: snapshot.flags(),
            }
        })
        .collect()
}

fn group_states(
    world: &World,
    known_groups: &[ParticleGroupId],
) -> Vec<(ParticleGroupId, Result<GroupState, HandleError>)> {
    known_groups
        .iter()
        .copied()
        .map(|group| {
            let state = world.particle_group_view(group).map(|view| {
                let transform = view.transform();
                GroupState {
                    flags: view.flags(),
                    members: view.member_ids().to_vec(),
                    depths: view
                        .maybe_depths()
                        .map(|depths| depths.iter().map(|depth| depth.to_bits()).collect()),
                    transform: [
                        transform.position().x.to_bits(),
                        transform.position().y.to_bits(),
                        transform.rotation().sine().to_bits(),
                        transform.rotation().cosine().to_bits(),
                    ],
                    center: bits(view.center()),
                    velocity: bits(view.linear_velocity()),
                    angular_velocity: view.angular_velocity().to_bits(),
                    mass: view.mass().to_bits(),
                    inertia: view.inertia().to_bits(),
                }
            });
            (group, state)
        })
        .collect()
}

#[test]
fn join_preserves_first_identity_members_flags_and_invalidates_second() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world.create_particle_system().expect("system should fit");
    let group_a = create_group(
        &mut world,
        system,
        vec![Vec2::new(-0.5, 0.0), Vec2::ZERO],
        ParticleGroupFlags::RIGID,
    );
    let group_b = create_group(
        &mut world,
        system,
        vec![Vec2::new(0.5, 0.0), Vec2::new(1.0, 0.0)],
        ParticleGroupFlags::SOLID,
    );
    let members_a = world
        .particle_group_view(group_a)
        .expect("first group remains live")
        .member_ids()
        .to_vec();
    let members_b = world
        .particle_group_view(group_b)
        .expect("second group remains live")
        .member_ids()
        .to_vec();
    let mut associations = AssociationMap::new();
    associations.insert(group_b, "second");

    // Act
    let report = world
        .join_particle_groups(group_a, group_b)
        .expect("compatible groups should join");
    let joined = *report.value();
    let view = world
        .particle_group_view(joined)
        .expect("surviving group remains live");
    let removed_association = match &report.lifecycle()[0] {
        StepLifecycleEvent::Destruction(record) => associations.cleanup_record(record),
        other => panic!("join should emit group destruction, got {other:?}"),
    };

    // Assert
    assert_eq!(joined, group_a);
    assert_eq!(
        view.member_ids(),
        members_a
            .iter()
            .chain(&members_b)
            .copied()
            .collect::<Vec<_>>()
    );
    assert_eq!(
        view.flags(),
        ParticleGroupFlags::RIGID | ParticleGroupFlags::SOLID
    );
    assert_eq!(
        world.particle_group_view(group_b).map(|view| view.id()),
        Err(HandleError::StaleOrDestroyed)
    );
    assert_eq!(removed_association, Some("second"));
    assert!(members_a.iter().chain(&members_b).all(|particle| {
        world
            .particle_snapshot(*particle)
            .expect("joined member remains live")
            .maybe_group()
            == Some(group_a)
    }));
}

#[test]
fn split_preserves_original_first_and_allocates_later_components_in_source_order() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world.create_particle_system().expect("system should fit");
    let flags =
        ParticleGroupFlags::SOLID | ParticleGroupFlags::RIGID | ParticleGroupFlags::CAN_BE_EMPTY;
    let group = create_group(
        &mut world,
        system,
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(0.5, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(20.0, 0.0),
        ],
        flags,
    );
    let source_members = world
        .particle_group_view(group)
        .expect("source group remains live")
        .member_ids()
        .to_vec();
    let mut associations = AssociationMap::new();
    associations.insert(group, String::from("source"));

    // Act
    let groups = world
        .split_particle_group_with_association(group, &mut associations)
        .expect("disconnected components should split");
    let member_groups = source_members
        .iter()
        .map(|particle| {
            world
                .particle_snapshot(*particle)
                .expect("split member remains live")
                .maybe_group()
        })
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0], group);
    assert_eq!(
        member_groups,
        vec![Some(group), Some(group), Some(groups[1]), Some(groups[2])]
    );
    assert_eq!(
        world
            .particle_group_view(group)
            .expect("original component remains live")
            .member_ids(),
        &source_members[..2]
    );
    assert_eq!(
        world
            .particle_group_view(groups[1])
            .expect("first later component remains live")
            .member_ids(),
        &source_members[2..3]
    );
    assert_eq!(
        world
            .particle_group_view(groups[2])
            .expect("second later component remains live")
            .member_ids(),
        &source_members[3..]
    );
    assert!(groups.iter().all(|candidate| {
        world
            .particle_group_view(*candidate)
            .expect("every split component remains live")
            .flags()
            == flags
    }));
    assert!(
        groups
            .iter()
            .all(|candidate| associations.get(candidate).map(String::as_str) == Some("source"))
    );
}

#[test]
fn flags_and_retained_empty_shell_follow_explicit_lifecycle_rules() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world.create_particle_system().expect("system should fit");
    let group = create_group(
        &mut world,
        system,
        vec![Vec2::ZERO],
        ParticleGroupFlags::CAN_BE_EMPTY,
    );
    let populated_before = semantic_snapshot(&world, system, &[group]);

    // Act
    let populated_destruction = world.destroy_empty_particle_group(group);
    world
        .destroy_particle_group_particles(group, false)
        .expect("member destruction should be scheduled");
    world
        .compact_pending_particles(system)
        .expect("pending member should compact");
    world
        .set_particle_group_flags(
            group,
            ParticleGroupFlags::RIGID | ParticleGroupFlags::CAN_BE_EMPTY,
        )
        .expect("public flags should replace atomically");
    let empty_view = world
        .particle_group_view(group)
        .expect("retained empty group remains live");
    let empty_state = (
        empty_view.flags(),
        empty_view.member_count(),
        empty_view.center(),
        empty_view.mass().to_bits(),
    );
    let record = world
        .destroy_empty_particle_group(group)
        .expect("empty shell should be explicitly destroyable");

    // Assert
    assert_eq!(
        populated_destruction,
        Err(ParticleGroupMutationError::GroupNotEmpty)
    );
    assert_eq!(
        semantic_snapshot(&world, system, &[group]).groups[0].1,
        Err(HandleError::StaleOrDestroyed)
    );
    assert_eq!(populated_before.group_count, 1);
    assert_eq!(
        empty_state,
        (
            ParticleGroupFlags::RIGID | ParticleGroupFlags::CAN_BE_EMPTY,
            0,
            Vec2::ZERO,
            0.0_f32.to_bits(),
        )
    );
    assert_eq!(record.destroyed(), DestroyedId::ParticleGroup(group));
}

#[test]
fn stale_foreign_cross_system_and_same_group_failures_are_typed_and_effect_free() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let first_system = world.create_particle_system().expect("system should fit");
    let second_system = world.create_particle_system().expect("system should fit");
    let first = create_group(
        &mut world,
        first_system,
        vec![Vec2::ZERO],
        ParticleGroupFlags::empty(),
    );
    let second = create_group(
        &mut world,
        second_system,
        vec![Vec2::ZERO],
        ParticleGroupFlags::empty(),
    );
    let mut foreign_world = World::new().expect("foreign world key remains available");
    let foreign_system = foreign_world
        .create_particle_system()
        .expect("foreign system should fit");
    let foreign = create_group(
        &mut foreign_world,
        foreign_system,
        vec![Vec2::ZERO],
        ParticleGroupFlags::empty(),
    );
    let known = [first, second, foreign];
    let before = semantic_snapshot(&world, first_system, &known);

    // Act
    let same = world.join_particle_groups(first, first);
    let cross_system = world.join_particle_groups(first, second);
    let wrong_world = world.set_particle_group_flags(foreign, ParticleGroupFlags::SOLID);

    // Assert
    assert_eq!(same, Err(ParticleGroupMutationError::SameGroup));
    assert_eq!(
        cross_system,
        Err(ParticleGroupMutationError::InvalidHandle(
            HandleError::WrongParticleSystem
        ))
    );
    assert_eq!(
        wrong_world,
        Err(ParticleGroupMutationError::InvalidHandle(
            HandleError::WrongWorld
        ))
    );
    assert_eq!(semantic_snapshot(&world, first_system, &known), before);
}

struct PanickingHook;

impl StepHook for PanickingHook {
    fn observe(&mut self, _contact: liquidfun::ContactView<'_>) {
        panic!("intentional group mutation poison witness");
    }
}

#[test]
fn poisoned_world_rejects_group_mutation_without_reclassifying_the_cause() {
    // Arrange
    let mut world = touching_world();
    let system = world.create_particle_system().expect("system should fit");
    let group = create_group(
        &mut world,
        system,
        vec![Vec2::ZERO],
        ParticleGroupFlags::empty(),
    );

    // Act
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _report = world.step(
            StepConfiguration::new(0.0, 8, 3).expect("zero step is valid"),
            &mut PanickingHook,
            StepLimits::default(),
        );
    }));
    let mutation = world.set_particle_group_flags(group, ParticleGroupFlags::SOLID);

    // Assert
    assert!(panic.is_err());
    assert_eq!(
        mutation,
        Err(ParticleGroupMutationError::InvalidHandle(
            HandleError::WorldPoisoned
        ))
    );
}

fn touching_world() -> World {
    let mut world = World::new().expect("world key remains available");
    world
        .set_continuous_physics_enabled(false)
        .expect("world configuration remains mutable");
    let static_body = world
        .create_body(
            &BodyDef::new(BodyType::Static, Vec2::ZERO, 0.0, true)
                .expect("static body definition is valid"),
        )
        .expect("static body fits");
    let dynamic_body = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(1.5, 0.0), 0.0, true)
                .expect("dynamic body definition is valid"),
        )
        .expect("dynamic body fits");
    let shape =
        Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("test circle geometry is valid"));
    let definition = FixtureDef::new(shape, 1.0, 0.2, 0.0, false, FilterData::default())
        .expect("fixture definition is valid");
    world
        .create_fixture(static_body, &definition)
        .expect("static fixture fits");
    world
        .create_fixture(dynamic_body, &definition)
        .expect("dynamic fixture fits");
    world
}

proptest! {
    #[test]
    fn public_operation_sequences_preserve_exact_snapshots_on_every_failure(
        operations in prop::collection::vec((0_u8..4, any::<u8>(), any::<u8>()), 1..24),
    ) {
        // Arrange
        let mut world = World::new().expect("world key remains available");
        let system = world.create_particle_system().expect("system should fit");
        let other_system = world.create_particle_system().expect("other system should fit");
        let mut known_groups = vec![
            create_group(
                &mut world,
                system,
                vec![Vec2::ZERO, Vec2::new(0.5, 0.0), Vec2::new(10.0, 0.0)],
                ParticleGroupFlags::CAN_BE_EMPTY,
            ),
            create_group(
                &mut world,
                system,
                vec![Vec2::new(20.0, 0.0)],
                ParticleGroupFlags::empty(),
            ),
            create_group(
                &mut world,
                other_system,
                vec![Vec2::ZERO],
                ParticleGroupFlags::empty(),
            ),
        ];

        // Act
        for (kind, first, second) in operations {
            let before = semantic_snapshot(&world, system, &known_groups);
            let first_group = known_groups[usize::from(first) % known_groups.len()];
            let second_group = known_groups[usize::from(second) % known_groups.len()];
            let result = match kind {
                0 => world
                    .join_particle_groups(first_group, second_group)
                    .map(|_report| Vec::new()),
                1 => world.split_particle_group(first_group),
                2 => world
                    .set_particle_group_flags(
                        first_group,
                        if second & 1 == 0 {
                            ParticleGroupFlags::SOLID | ParticleGroupFlags::CAN_BE_EMPTY
                        } else {
                            ParticleGroupFlags::RIGID
                        },
                    )
                    .map(|()| Vec::new()),
                _ => world
                    .destroy_empty_particle_group(first_group)
                    .map(|_record| Vec::new()),
            };
            match result {
                Ok(created) => known_groups.extend(created.into_iter().skip(1)),
                Err(_error) => {
                    let after = semantic_snapshot(&world, system, &known_groups);
                    prop_assert_eq!(after, before);
                }
            }
        }

        // Assert
        prop_assert!(world.particle_system_snapshot(system).is_ok());
    }
}

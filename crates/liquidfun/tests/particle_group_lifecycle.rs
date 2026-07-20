//! Black-box particle-group zombie, callback, and empty-retention regressions.

use liquidfun::math::Vec2;
use liquidfun::particle::{
    ParticleGroupDestination, ParticleGroupFlags, ParticleGroupRecipe, ParticleGroupSource,
    ParticleSystemDef,
};
use liquidfun::{
    AssociationMap, CreateObjectError, DestroyedId, HandleError, NoDecisionHook, ParticleFlags,
    ParticleGroupId, StepConfiguration, StepError, StepLifecycleEvent, StepLimits, World,
};

fn positions_recipe(
    positions: Vec<Vec2>,
    destination: ParticleGroupDestination,
) -> ParticleGroupRecipe {
    let source = ParticleGroupSource::positions(positions).expect("positions are finite");
    ParticleGroupRecipe::new(source, destination)
}

fn positive_step() -> StepConfiguration {
    StepConfiguration::new(0.01, 8, 3).expect("positive step configuration is valid")
}

fn paused_system(world: &mut World) -> liquidfun::ParticleSystemId {
    world
        .create_particle_system_with_def(&ParticleSystemDef::default().with_paused(true))
        .expect("paused particle system fits")
}

#[test]
fn ordinary_group_destruction_is_deferred_and_callbacks_are_source_ordered_once() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = paused_system(&mut world);
    let recipe = positions_recipe(
        vec![Vec2::new(-1.0, 0.0), Vec2::ZERO, Vec2::new(1.0, 0.0)],
        ParticleGroupDestination::New,
    )
    .with_user_association(String::from("ordinary"));
    let mut associations = AssociationMap::<ParticleGroupId, String>::new();
    let group = world
        .create_particle_group_with_association(system, recipe, &mut associations)
        .expect("group and association commit");
    let members = world
        .particle_group_view(group)
        .expect("group is live")
        .member_ids()
        .to_vec();

    // Act
    world
        .destroy_particle_group_particles(group, true)
        .expect("members become pending");
    world
        .destroy_particle_group_particles(group, true)
        .expect("repeated source request remains idempotent");
    let pending_members = world
        .particle_group_view(group)
        .expect("group remains inspectable before compaction")
        .member_ids()
        .to_vec();
    let pending_flags = world
        .particle_system_view(system)
        .expect("system remains live")
        .flags()
        .to_vec();
    let mut hook = NoDecisionHook;
    let report = world
        .step(positive_step(), &mut hook, StepLimits::default())
        .expect("lifecycle step compacts the group");
    let cleanup = associations.cleanup(report.destructions());
    let second_report = world
        .step(positive_step(), &mut hook, StepLimits::default())
        .expect("second lifecycle step is stable");

    // Assert
    assert_eq!(pending_members, members);
    assert!(pending_flags.iter().all(|flags| {
        flags.contains(ParticleFlags::ZOMBIE) && flags.contains(ParticleFlags::DESTRUCTION_LISTENER)
    }));
    assert!(members.iter().all(|particle| {
        world.particle_snapshot(*particle) == Err(HandleError::StaleOrDestroyed)
    }));
    assert!(matches!(
        world.particle_group_view(group),
        Err(HandleError::StaleOrDestroyed)
    ));
    assert_eq!(
        report
            .lifecycle()
            .iter()
            .filter_map(|event| match event {
                StepLifecycleEvent::ParticleDestruction(record)
                | StepLifecycleEvent::Destruction(record) => Some(record.destroyed()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        members
            .iter()
            .copied()
            .map(DestroyedId::Particle)
            .chain(std::iter::once(DestroyedId::ParticleGroup(group)))
            .collect::<Vec<_>>()
    );
    assert_eq!(cleanup, vec![String::from("ordinary")]);
    assert!(second_report.lifecycle().is_empty());
}

#[test]
fn can_be_empty_group_retains_exact_zero_state_and_accepts_a_later_append() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = paused_system(&mut world);
    let target_recipe = positions_recipe(
        vec![Vec2::new(-0.1, 0.0), Vec2::new(0.1, 0.0)],
        ParticleGroupDestination::New,
    )
    .with_group_flags(ParticleGroupFlags::SOLID | ParticleGroupFlags::CAN_BE_EMPTY);
    let target = world
        .create_particle_group(system, &target_recipe)
        .expect("retained target fits");
    let mut hook = NoDecisionHook;

    // Act
    world
        .destroy_particle_group_particles(target, false)
        .expect("target members become pending");
    let report = world
        .step(positive_step(), &mut hook, StepLimits::default())
        .expect("target becomes retained empty");
    let empty = world
        .particle_group_view(target)
        .expect("retained target remains live");
    let empty_state = (
        empty.member_count(),
        empty.mass().to_bits(),
        empty.inertia().to_bits(),
        empty.center().x.to_bits(),
        empty.center().y.to_bits(),
        empty.linear_velocity().x.to_bits(),
        empty.linear_velocity().y.to_bits(),
        empty.angular_velocity().to_bits(),
        empty.maybe_depths().map(<[f32]>::len),
    );
    let append = positions_recipe(
        vec![Vec2::new(1.0, 0.0), Vec2::new(2.0, 0.0)],
        ParticleGroupDestination::AppendTo(target),
    );
    let returned = world
        .create_particle_group(system, &append)
        .expect("append into retained empty target succeeds");
    let appended_members = world
        .particle_group_view(target)
        .expect("target remains live after append")
        .member_ids()
        .to_vec();

    // Assert
    assert_eq!(returned, target);
    assert_eq!(empty_state, (0, 0, 0, 0, 0, 0, 0, 0, Some(0)));
    assert!(report.lifecycle().is_empty());
    assert_eq!(appended_members.len(), 2);
    assert!(world.contains_particle_group(target));
}

#[test]
fn wrong_world_group_destruction_preserves_both_worlds() {
    // Arrange
    let mut owner = World::new().expect("owner world key remains available");
    let owner_system = paused_system(&mut owner);
    let group = owner
        .create_particle_group(
            owner_system,
            &positions_recipe(vec![Vec2::ZERO], ParticleGroupDestination::New),
        )
        .expect("owner group fits");
    let owner_members = owner
        .particle_group_view(group)
        .expect("owner group remains live")
        .member_ids()
        .to_vec();
    let mut foreign = World::new().expect("foreign world key remains available");
    let foreign_system = paused_system(&mut foreign);
    let foreign_before = foreign
        .particle_system_statistics(foreign_system)
        .expect("foreign system remains live");

    // Act
    let result = foreign.destroy_particle_group_particles(group, true);

    // Assert
    assert_eq!(
        result,
        Err(CreateObjectError::InvalidHandle(HandleError::WrongWorld))
    );
    assert_eq!(
        owner
            .particle_group_view(group)
            .expect("owner group remains unchanged")
            .member_ids(),
        owner_members
    );
    assert_eq!(
        foreign
            .particle_system_statistics(foreign_system)
            .expect("foreign system remains unchanged"),
        foreign_before
    );
}

#[test]
fn lifecycle_limit_failure_rolls_back_group_compaction_for_exactly_one_retry() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = paused_system(&mut world);
    let group = world
        .create_particle_group(
            system,
            &positions_recipe(
                vec![Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0)],
                ParticleGroupDestination::New,
            ),
        )
        .expect("group fits");
    let members = world
        .particle_group_view(group)
        .expect("group remains live")
        .member_ids()
        .to_vec();
    world
        .destroy_particle_group_particles(group, true)
        .expect("members become pending");
    let mut hook = NoDecisionHook;
    let constrained_limits = StepLimits::new(2, 0).expect("two events are within hard limits");

    // Act
    let failed = world.step(positive_step(), &mut hook, constrained_limits);
    let pending_after_failure = world
        .particle_group_view(group)
        .expect("failed step preserves the pending group")
        .member_ids()
        .to_vec();
    let retry = world
        .step(positive_step(), &mut hook, StepLimits::default())
        .expect("retry with sufficient journal capacity succeeds");

    // Assert
    assert_eq!(
        failed,
        Err(StepError::LimitExceeded {
            resource: "event",
            limit: 2,
        })
    );
    assert_eq!(pending_after_failure, members);
    assert_eq!(
        retry
            .lifecycle()
            .iter()
            .filter_map(|event| match event {
                StepLifecycleEvent::ParticleDestruction(record)
                | StepLifecycleEvent::Destruction(record) => Some(record.destroyed()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        members
            .into_iter()
            .map(DestroyedId::Particle)
            .chain(std::iter::once(DestroyedId::ParticleGroup(group)))
            .collect::<Vec<_>>()
    );
}

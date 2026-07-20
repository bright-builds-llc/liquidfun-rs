//! Black-box particle-group creation, append, view, and rollback evidence.

use liquidfun::collision::{CircleShape, EdgeShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleCapacity, ParticleColor, ParticleGroupDestination, ParticleGroupFlags,
    ParticleGroupRecipe, ParticleGroupSource, ParticleSystemDef,
};
use liquidfun::{
    AssociationMap, CreateObjectError, HandleError, NoDecisionHook, ParticleFlags, ParticleGroupId,
    StepConfiguration, StepLimits, World,
};

fn positions_recipe(
    positions: Vec<Vec2>,
    destination: ParticleGroupDestination,
) -> ParticleGroupRecipe {
    let source = ParticleGroupSource::positions(positions).expect("positions are finite");
    ParticleGroupRecipe::new(source, destination)
}

fn stable_snapshot(
    world: &World,
    system: liquidfun::ParticleSystemId,
) -> (
    Vec<liquidfun::ParticleId>,
    usize,
    Vec<Vec2>,
    Vec<ParticleFlags>,
) {
    let statistics = world
        .particle_system_statistics(system)
        .expect("system remains live");
    let view = world
        .particle_system_view(system)
        .expect("system remains live");
    (
        statistics.particle_ids().to_vec(),
        statistics.group_count(),
        view.positions().to_vec(),
        view.flags().to_vec(),
    )
}

#[test]
fn explicit_group_exposes_every_recipe_and_view_field_in_source_order() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let definition = ParticleSystemDef::default()
        .with_density(2.0)
        .expect("density is positive")
        .with_radius(0.25)
        .expect("radius is positive");
    let system = world
        .create_particle_system_with_def(&definition)
        .expect("system should fit");
    let transform = Transform::from_position_angle(Vec2::new(3.0, -2.0), 0.25);
    let source_positions = vec![Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0)];
    let recipe = positions_recipe(source_positions.clone(), ParticleGroupDestination::New)
        .with_particle_flags(ParticleFlags::SPRING | ParticleFlags::VISCOUS)
        .with_group_flags(ParticleGroupFlags::RIGID)
        .with_transform(transform)
        .expect("transform is finite")
        .with_linear_velocity(Vec2::new(2.0, -1.0))
        .expect("velocity is finite")
        .with_angular_velocity(0.5)
        .expect("angular velocity is finite")
        .with_color(ParticleColor::new(4, 3, 2, 1))
        .with_strength(0.75)
        .expect("strength is valid")
        .with_stride(0.2)
        .expect("stride is positive")
        .with_lifetime(4.0)
        .expect("lifetime is finite");

    // Act
    let group = world
        .create_particle_group(system, &recipe)
        .expect("group should be created atomically");
    let view = world
        .particle_group_view(group)
        .expect("group should remain live");
    let member_snapshots = view
        .member_ids()
        .iter()
        .map(|particle| {
            world
                .particle_snapshot(*particle)
                .expect("member should remain live")
        })
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(view.id(), group);
    assert_eq!(view.flags(), ParticleGroupFlags::RIGID);
    assert_eq!(view.transform(), transform);
    assert_eq!(view.position(), transform.position());
    assert_eq!(view.member_count(), source_positions.len());
    assert_eq!(
        member_snapshots
            .iter()
            .map(|snapshot| snapshot.position())
            .collect::<Vec<_>>(),
        source_positions
            .into_iter()
            .map(|position| transform.apply(position))
            .collect::<Vec<_>>()
    );
    assert!(member_snapshots.iter().all(|snapshot| {
        snapshot.flags() == ParticleFlags::SPRING | ParticleFlags::VISCOUS
            && snapshot.color() == ParticleColor::new(4, 3, 2, 1)
    }));
    assert!(view.mass() > 0.0);
    assert!(view.inertia() > 0.0);
    assert!(view.center().is_valid());
    assert!(view.linear_velocity().is_valid());
    assert!(view.angular_velocity().is_finite());
    assert!(view.maybe_depths().is_none());
}

#[test]
fn filled_and_stroke_sources_use_the_public_atomic_workflow() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world.create_particle_system().expect("system should fit");
    let filled_source = ParticleGroupSource::filled_shapes(vec![Shape::Circle(
        CircleShape::new(Vec2::ZERO, 0.6).expect("circle is valid"),
    )])
    .expect("filled source is valid");
    let filled_recipe = ParticleGroupRecipe::new(filled_source, ParticleGroupDestination::New)
        .with_stride(0.5)
        .expect("stride is positive");
    let stroke_source = ParticleGroupSource::stroke_shape(Shape::Edge(
        EdgeShape::new(Vec2::new(-1.0, 1.0), Vec2::new(1.0, 1.0)).expect("edge is valid"),
    ))
    .expect("stroke source is valid");
    let stroke_recipe = ParticleGroupRecipe::new(stroke_source, ParticleGroupDestination::New)
        .with_stride(0.6)
        .expect("stride is positive");

    // Act
    let filled = world
        .create_particle_group(system, &filled_recipe)
        .expect("filled group should fit");
    let stroke = world
        .create_particle_group(system, &stroke_recipe)
        .expect("stroke group should fit");
    let filled_members = world
        .particle_group_view(filled)
        .expect("filled group should remain live")
        .member_ids()
        .to_vec();
    let stroke_positions = world
        .particle_group_view(stroke)
        .expect("stroke group should remain live")
        .member_ids()
        .iter()
        .map(|particle| {
            world
                .particle_snapshot(*particle)
                .expect("stroke member remains live")
                .position()
        })
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(filled_members.len(), 5);
    assert_eq!(
        stroke_positions,
        vec![
            Vec2::new(-1.0, 1.0),
            Vec2::new(-0.399_999_98, 1.0),
            Vec2::new(0.200_000_05, 1.0),
            Vec2::new(0.800_000_1, 1.0),
        ]
    );
    assert_eq!(
        world
            .particle_system_statistics(system)
            .expect("system remains live")
            .group_count(),
        2
    );
}

#[test]
fn append_returns_the_original_group_and_exposes_no_temporary_identity() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world.create_particle_system().expect("system should fit");
    let initial_recipe = positions_recipe(
        vec![Vec2::new(-1.0, 0.0), Vec2::new(0.0, 0.0)],
        ParticleGroupDestination::New,
    );
    let target = world
        .create_particle_group(system, &initial_recipe)
        .expect("target group should fit");
    let original_members = world
        .particle_group_view(target)
        .expect("target remains live")
        .member_ids()
        .to_vec();
    let append_recipe = positions_recipe(
        vec![Vec2::new(1.0, 0.0), Vec2::new(2.0, 0.0)],
        ParticleGroupDestination::AppendTo(target),
    )
    .with_group_flags(ParticleGroupFlags::SOLID);

    // Act
    let returned = world
        .create_particle_group(system, &append_recipe)
        .expect("append should commit once");
    let view = world
        .particle_group_view(target)
        .expect("target remains live");

    // Assert
    assert_eq!(returned, target);
    assert_eq!(
        &view.member_ids()[..original_members.len()],
        original_members
    );
    assert_eq!(view.member_count(), 4);
    assert!(view.flags().contains(ParticleGroupFlags::SOLID));
    assert_eq!(
        world
            .particle_system_statistics(system)
            .expect("system remains live")
            .group_count(),
        1
    );
}

#[test]
fn association_capable_creation_commits_the_side_table_only_with_the_group() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world.create_particle_system().expect("system should fit");
    let recipe = positions_recipe(vec![Vec2::new(1.0, 2.0)], ParticleGroupDestination::New)
        .with_user_association(String::from("primary"));
    let mut associations = AssociationMap::<ParticleGroupId, String>::new();

    // Act
    let group = world
        .create_particle_group_with_association(system, recipe, &mut associations)
        .expect("group and association should commit together");

    // Assert
    assert_eq!(
        associations.get(&group).map(String::as_str),
        Some("primary")
    );
    assert!(world.contains_particle_group(group));
}

#[test]
fn stale_and_cross_system_append_targets_leave_every_public_snapshot_unchanged() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let first_system = world.create_particle_system().expect("system should fit");
    let second_system = world.create_particle_system().expect("system should fit");
    let target = world
        .create_particle_group(
            first_system,
            &positions_recipe(vec![Vec2::ZERO], ParticleGroupDestination::New),
        )
        .expect("target group should fit");
    let before = stable_snapshot(&world, second_system);
    let wrong_system_recipe = positions_recipe(
        vec![Vec2::new(1.0, 0.0)],
        ParticleGroupDestination::AppendTo(target),
    );

    // Act
    let wrong_system = world.create_particle_group(second_system, &wrong_system_recipe);
    world
        .destroy_particle_group(target)
        .expect("target should be destroyable");
    let stale = world.create_particle_group(first_system, &wrong_system_recipe);

    // Assert
    assert_eq!(
        wrong_system,
        Err(CreateObjectError::InvalidHandle(
            HandleError::WrongParticleSystem
        ))
    );
    assert_eq!(
        stale,
        Err(CreateObjectError::InvalidHandle(
            HandleError::StaleOrDestroyed
        ))
    );
    assert_eq!(stable_snapshot(&world, second_system), before);
}

#[test]
fn capacity_failure_preserves_identity_storage_topology_and_lifecycle_state() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let definition = ParticleSystemDef::default()
        .with_capacity(ParticleCapacity::fixed(1).expect("capacity is positive"))
        .expect("fixed capacity is valid");
    let system = world
        .create_particle_system_with_def(&definition)
        .expect("system should fit");
    let before = stable_snapshot(&world, system);
    let recipe = positions_recipe(
        vec![Vec2::ZERO, Vec2::new(1.0, 0.0)],
        ParticleGroupDestination::New,
    );
    let mut hook = NoDecisionHook;

    // Act
    let result = world.create_particle_group(system, &recipe);
    let report = world
        .step(
            StepConfiguration::new(0.0, 8, 3).expect("zero-duration step is valid"),
            &mut hook,
            StepLimits::default(),
        )
        .expect("unchanged world should step");

    // Assert
    assert!(matches!(
        result,
        Err(CreateObjectError::Arena(
            liquidfun::ArenaInsertError::CapacityExceeded { limit: 1 }
        ))
    ));
    assert_eq!(stable_snapshot(&world, system), before);
    assert!(report.lifecycle().is_empty());
}

#[test]
fn topology_failure_preserves_group_and_particle_identity_counts() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world.create_particle_system().expect("system should fit");
    let before = stable_snapshot(&world, system);
    let widely_spaced = (0..64)
        .scan(0.0_f32, |x, _ordinal| {
            let position = Vec2::new(*x, 0.0);
            *x += 1_000.0;
            Some(position)
        })
        .collect::<Vec<_>>();
    let recipe = positions_recipe(widely_spaced, ParticleGroupDestination::New)
        .with_particle_flags(ParticleFlags::ELASTIC);

    // Act
    let result = world.create_particle_group(system, &recipe);

    // Assert
    assert_eq!(result, Err(CreateObjectError::InvalidParticleGroupTopology));
    assert_eq!(stable_snapshot(&world, system), before);
}

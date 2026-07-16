//! Black-box owned particle-buffer adoption and teardown regressions.

use liquidfun::math::Vec2;
use liquidfun::{
    ArenaInsertError, BodyDef, CreateObjectError, ParticleBufferAdoptionErrorKind,
    ParticleBufferBundle, ParticleBufferErrorKind, ParticleBufferLanes, ParticleBufferMode,
    ParticleColor, ParticleDef, ParticleFlags, ParticleSystemDef, ParticleSystemDefError, World,
};

fn lanes_with_capacities(
    position_capacity: usize,
    velocity_capacity: usize,
    flag_capacity: usize,
    maybe_color_capacity: Option<usize>,
) -> ParticleBufferLanes {
    ParticleBufferLanes::new(
        Vec::with_capacity(position_capacity),
        Vec::with_capacity(velocity_capacity),
        Vec::with_capacity(flag_capacity),
        maybe_color_capacity.map(Vec::with_capacity),
    )
}

fn particle(value: f32, color: ParticleColor) -> ParticleDef {
    ParticleDef::default()
        .with_position(Vec2::new(value, -value))
        .expect("position is valid")
        .with_velocity(Vec2::new(value + 1.0, value + 2.0))
        .expect("velocity is valid")
        .with_color(color)
}

#[test]
fn adoption_returns_owned_lanes_with_final_semantic_contents() {
    // Arrange
    let lanes = lanes_with_capacities(2, 3, 4, Some(5));
    let buffers = ParticleBufferBundle::fixed(2, lanes).expect("fixed lanes are complete");
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system_with_buffers(&ParticleSystemDef::default(), buffers)
        .expect("owned buffers can be adopted");
    let first = ParticleDef::default()
        .with_position(Vec2::new(1.0, 2.0))
        .expect("position is valid")
        .with_velocity(Vec2::new(3.0, 4.0))
        .expect("velocity is valid")
        .with_color(ParticleColor::new(5, 6, 7, 8));
    let _particle = world
        .create_particle_with_def(system, None, &first)
        .expect("particle fits the declared fixed capacity")
        .created_particle();

    // Act
    let receipt = world
        .destroy_particle_system_with_buffers(system)
        .expect("the live system returns its adopted buffers");
    let record_count = receipt.records().len();
    let lanes = receipt.into_lanes();

    // Assert
    assert_eq!(record_count, 2);
    assert_eq!(lanes.positions(), &[Vec2::new(1.0, 2.0)]);
    assert_eq!(lanes.velocities(), &[Vec2::new(3.0, 4.0)]);
    assert_eq!(lanes.flags(), &[ParticleFlags::WATER]);
    assert_eq!(
        lanes.maybe_colors(),
        Some(&[ParticleColor::new(5, 6, 7, 8)][..])
    );
}

#[test]
fn adoption_rejects_incomplete_lane_lengths_and_returns_ownership() {
    // Arrange
    let lanes = ParticleBufferLanes::new(
        vec![Vec2::new(1.0, 2.0)],
        Vec::new(),
        vec![ParticleFlags::WATER],
        None,
    );

    // Act
    let error = ParticleBufferBundle::growable(1, lanes)
        .expect_err("incomplete semantic rows must be rejected before adoption");
    let kind = error.kind();
    let lanes = error.into_lanes();

    // Assert
    assert_eq!(kind, ParticleBufferErrorKind::LaneLengthMismatch);
    assert_eq!(lanes.positions(), &[Vec2::new(1.0, 2.0)]);
    assert!(lanes.velocities().is_empty());
    assert_eq!(lanes.flags(), &[ParticleFlags::WATER]);
}

#[test]
fn fixed_capacity_uses_declared_limit_and_preserves_original_allocations_on_failure() {
    // Arrange
    let lanes = lanes_with_capacities(2, 5, 3, Some(4));
    let original_positions = lanes.positions().as_ptr();
    let buffers = ParticleBufferBundle::fixed(2, lanes).expect("every lane can hold two rows");
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system_with_buffers(&ParticleSystemDef::default(), buffers)
        .expect("fixed buffers can be adopted");
    let first = world
        .create_particle_with_def(system, None, &particle(1.0, ParticleColor::ZERO))
        .expect("first row fits")
        .created_particle();
    let second = world
        .create_particle_with_def(system, None, &particle(2.0, ParticleColor::ZERO))
        .expect("second row exactly fills the declared limit")
        .created_particle();
    let before = [
        world.particle_snapshot(first).expect("first row is live"),
        world.particle_snapshot(second).expect("second row is live"),
    ];

    // Act
    let rejected = world.create_particle_with_def(
        system,
        None,
        &particle(3.0, ParticleColor::new(1, 2, 3, 4)),
    );
    let after = [
        world
            .particle_snapshot(first)
            .expect("first row is unchanged"),
        world
            .particle_snapshot(second)
            .expect("second row is unchanged"),
    ];
    let body = world
        .create_body(&BodyDef::default())
        .expect("rejected particle did not consume the next diagnostic identity");
    let receipt = world
        .destroy_particle_system_with_buffers(system)
        .expect("fixed buffers return after the no-effect failure");
    let (particle_records, bundle) = receipt.into_parts();
    let lanes = bundle.into_lanes();
    let body_records = world.destroy_body(body).expect("body remains live");

    // Assert
    assert_eq!(
        rejected,
        Err(CreateObjectError::Arena(
            ArenaInsertError::CapacityExceeded { limit: 2 }
        ))
    );
    assert_eq!(after, before);
    assert_eq!(
        lanes.positions(),
        &[Vec2::new(1.0, -1.0), Vec2::new(2.0, -2.0)]
    );
    assert_eq!(lanes.positions().as_ptr(), original_positions);
    assert_eq!(lanes.maybe_colors(), Some(&[ParticleColor::ZERO; 2][..]));
    assert_eq!(body_records.len(), 1);
    assert_eq!(
        body_records[0].diagnostic_id(),
        particle_records[1].diagnostic_id() + 1
    );
}

#[test]
fn fixed_bundle_rejects_any_undersized_required_or_optional_lane() {
    // Arrange
    let lanes = lanes_with_capacities(2, 3, 4, Some(1));

    // Act
    let error = ParticleBufferBundle::fixed(2, lanes)
        .expect_err("the allocated optional lane must satisfy the same fixed contract");
    let kind = error.kind();
    let lanes = error.into_lanes();

    // Assert
    assert_eq!(
        kind,
        ParticleBufferErrorKind::InsufficientLaneCapacity { required: 2 }
    );
    assert_eq!(lanes.maybe_colors(), Some(&[][..]));
}

#[test]
fn absent_optional_color_lane_stays_lazy_for_zero_color_rows() {
    // Arrange
    let lanes = lanes_with_capacities(1, 2, 3, None);
    let buffers = ParticleBufferBundle::fixed(1, lanes).expect("required lanes are complete");
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system_with_buffers(&ParticleSystemDef::default(), buffers)
        .expect("buffers can be adopted");

    // Act
    let _particle = world
        .create_particle_with_def(system, None, &particle(1.0, ParticleColor::ZERO))
        .expect("zero-color row fits")
        .created_particle();
    let lanes = world
        .destroy_particle_system_with_buffers(system)
        .expect("buffers return")
        .into_lanes();

    // Assert
    assert_eq!(lanes.maybe_colors(), None);
}

#[test]
fn growable_buffers_expand_until_the_explicit_system_maximum() {
    // Arrange
    let definition = ParticleSystemDef::default()
        .with_destruction_by_age(false)
        .with_maximum_count(3)
        .expect("maximum is valid");
    let lanes = lanes_with_capacities(1, 1, 1, None);
    let buffers = ParticleBufferBundle::growable(1, lanes).expect("initial allocation is present");
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system_with_buffers(&definition, buffers)
        .expect("growable buffers can be adopted");

    // Act
    for value in [1.0, 2.0, 3.0] {
        let _particle = world
            .create_particle_with_def(system, None, &particle(value, ParticleColor::ZERO))
            .expect("growth below the maximum succeeds")
            .created_particle();
    }
    let rejected =
        world.create_particle_with_def(system, None, &particle(4.0, ParticleColor::ZERO));
    let receipt = world
        .destroy_particle_system_with_buffers(system)
        .expect("grown buffers return");
    let mode = receipt.mode();
    let lanes = receipt.into_lanes();

    // Assert
    assert_eq!(
        rejected,
        Err(CreateObjectError::Arena(
            ArenaInsertError::CapacityExceeded { limit: 3 }
        ))
    );
    assert_eq!(
        mode,
        ParticleBufferMode::Growable {
            initial_capacity: 1
        }
    );
    assert_eq!(lanes.positions().len(), 3);
}

#[test]
fn fixed_maximum_conflict_returns_bundle_without_world_mutation() {
    // Arrange
    let definition = ParticleSystemDef::default()
        .with_maximum_count(3)
        .expect("maximum is valid");
    let lanes = lanes_with_capacities(2, 3, 4, None);
    let buffers = ParticleBufferBundle::fixed(2, lanes).expect("fixed lanes are complete");
    let mut world = World::new().expect("world key remains available");

    // Act
    let error = world
        .create_particle_system_with_buffers(&definition, buffers)
        .expect_err("maximum cannot exceed fixed supplied capacity");
    let kind = error.kind();
    let returned = error.into_bundle();

    // Assert
    assert_eq!(
        kind,
        ParticleBufferAdoptionErrorKind::Definition(
            ParticleSystemDefError::MaximumExceedsFixedCapacity {
                maximum: 3,
                capacity: 2,
            }
        )
    );
    assert_eq!(returned.mode(), ParticleBufferMode::Fixed { capacity: 2 });
    assert_eq!(world.particle_system_ids().len(), 0);
}

#[test]
fn teardown_returns_survivor_lanes_after_transactional_compaction() {
    // Arrange
    let lanes = lanes_with_capacities(3, 4, 5, Some(6));
    let original_positions = lanes.positions().as_ptr();
    let original_colors = lanes.maybe_colors().expect("test supplies colors").as_ptr();
    let buffers = ParticleBufferBundle::fixed(3, lanes).expect("fixed lanes are complete");
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system_with_buffers(&ParticleSystemDef::default(), buffers)
        .expect("buffers can be adopted");
    let first = world
        .create_particle_with_def(system, None, &particle(1.0, ParticleColor::new(1, 0, 0, 1)))
        .expect("first particle fits")
        .created_particle();
    let removed = world
        .create_particle_with_def(system, None, &particle(2.0, ParticleColor::new(2, 0, 0, 2)))
        .expect("middle particle fits")
        .created_particle();
    let third = world
        .create_particle_with_def(system, None, &particle(3.0, ParticleColor::new(3, 0, 0, 3)))
        .expect("third particle fits")
        .created_particle();
    world
        .mark_particle_for_destruction(removed)
        .expect("middle particle becomes pending");

    // Act
    world
        .compact_pending_particles(system)
        .expect("one total permutation removes the pending row");
    let survivor_ids = [
        world.particle_snapshot(first).expect("first survives").id(),
        world.particle_snapshot(third).expect("third survives").id(),
    ];
    let lanes = world
        .destroy_particle_system_with_buffers(system)
        .expect("compacted buffers return")
        .into_lanes();

    // Assert
    assert_eq!(survivor_ids, [first, third]);
    assert_eq!(
        lanes.positions(),
        &[Vec2::new(1.0, -1.0), Vec2::new(3.0, -3.0)]
    );
    assert_eq!(lanes.positions().as_ptr(), original_positions);
    assert_eq!(
        lanes.maybe_colors(),
        Some(
            &[
                ParticleColor::new(1, 0, 0, 1),
                ParticleColor::new(3, 0, 0, 3),
            ][..]
        )
    );
    assert_eq!(
        lanes
            .maybe_colors()
            .expect("colors remain allocated")
            .as_ptr(),
        original_colors
    );
}

#[test]
fn returned_lanes_can_be_cleared_and_adopted_repeatedly() {
    // Arrange
    let lanes = lanes_with_capacities(2, 3, 4, Some(5));
    let original_positions = lanes.positions().as_ptr();
    let mut world = World::new().expect("world key remains available");

    // Act
    let mut returned = lanes;
    for value in [1.0, 2.0] {
        returned.clear();
        let buffers =
            ParticleBufferBundle::fixed(2, returned).expect("cleared lanes remain reusable");
        let system = world
            .create_particle_system_with_buffers(&ParticleSystemDef::default(), buffers)
            .expect("same owned lanes can be adopted again");
        let _particle = world
            .create_particle_with_def(system, None, &particle(value, ParticleColor::ZERO))
            .expect("one particle fits")
            .created_particle();
        returned = world
            .destroy_particle_system_with_buffers(system)
            .expect("same lanes return after each cycle")
            .into_lanes();
    }

    // Assert
    assert_eq!(returned.positions(), &[Vec2::new(2.0, -2.0)]);
    assert_eq!(returned.positions().as_ptr(), original_positions);
}

//! Black-box owned particle-buffer adoption and teardown regressions.

use liquidfun::math::Vec2;
use liquidfun::{
    ParticleBufferBundle, ParticleBufferErrorKind, ParticleBufferLanes, ParticleColor, ParticleDef,
    ParticleFlags, ParticleSystemDef, World,
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
    world
        .create_particle_with_def(system, None, &first)
        .expect("particle fits the declared fixed capacity");

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

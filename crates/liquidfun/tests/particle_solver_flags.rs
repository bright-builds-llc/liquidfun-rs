//! Public flag-boundary witnesses for the closed particle solver ledger.

use liquidfun::math::Vec2;
use liquidfun::particle::{
    ParticleGroupDestination, ParticleGroupFlags, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{NoDecisionHook, ParticleDef, ParticleFlags, StepConfiguration, StepLimits, World};

fn recipe(flags: ParticleFlags, group_flags: ParticleGroupFlags) -> ParticleGroupRecipe {
    let source = ParticleGroupSource::positions(vec![Vec2::ZERO, Vec2::new(0.5, 0.0)])
        .expect("positions are finite");
    ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_particle_flags(flags)
        .with_group_flags(group_flags)
}

fn positive_step() -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3)
        .expect("configuration is valid")
        .with_particle_iterations(2)
        .expect("iteration count is valid")
}

#[test]
fn public_water_control_remains_zero_valued() {
    // Arrange / Act / Assert
    assert_eq!(ParticleFlags::WATER.bits(), 0);
    assert!(ParticleFlags::WATER.is_empty());
    assert_eq!(ParticleFlags::from_bits_retain(0), ParticleFlags::WATER);
}

#[test]
fn public_particle_flags_round_trip_through_particle_creation() {
    // Arrange
    let flags = [
        ParticleFlags::ZOMBIE,
        ParticleFlags::WALL,
        ParticleFlags::SPRING,
        ParticleFlags::ELASTIC,
        ParticleFlags::VISCOUS,
        ParticleFlags::POWDER,
        ParticleFlags::TENSILE,
        ParticleFlags::COLOR_MIXING,
        ParticleFlags::DESTRUCTION_LISTENER,
        ParticleFlags::BARRIER,
        ParticleFlags::STATIC_PRESSURE,
        ParticleFlags::REACTIVE,
        ParticleFlags::REPULSIVE,
        ParticleFlags::FIXTURE_CONTACT_LISTENER,
        ParticleFlags::PARTICLE_CONTACT_LISTENER,
        ParticleFlags::FIXTURE_CONTACT_FILTER,
        ParticleFlags::PARTICLE_CONTACT_FILTER,
    ];
    let mut world = World::new().expect("world key remains available");
    let system = world.create_particle_system().expect("system should fit");

    // Act
    let particles = flags.map(|flag| {
        world
            .create_particle_with_def(system, None, &ParticleDef::default().with_flags(flag))
            .expect("flagged particle should fit")
            .created_particle()
    });

    // Assert
    assert!(
        particles
            .into_iter()
            .zip(flags)
            .all(|(particle, expected)| {
                world
                    .particle_snapshot(particle)
                    .expect("particle remains live")
                    .flags()
                    == expected
            })
    );
}

#[test]
fn public_group_flags_hide_private_bits_and_preserve_unknown_public_bits() {
    // Arrange
    let unknown_public_bit = 1_u32 << 31;
    let private_bits = 0x0018;
    let flags = ParticleGroupFlags::from_bits_retain(
        ParticleGroupFlags::all().bits() | unknown_public_bit | private_bits,
    );
    let mut world = World::new().expect("world key remains available");
    let system = world.create_particle_system().expect("system should fit");

    // Act
    let group = world
        .create_particle_group(system, &recipe(ParticleFlags::WATER, flags))
        .expect("group should fit");
    let observed = world
        .particle_group_view(group)
        .expect("group remains live")
        .flags();

    // Assert
    assert_eq!(observed.bits() & private_bits, 0);
    assert_eq!(observed.bits() & unknown_public_bit, unknown_public_bit);
    assert!(observed.contains(ParticleGroupFlags::all()));
}

#[test]
fn wall_and_zombie_flags_have_distinct_public_step_outcomes() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    world
        .set_gravity(Vec2::ZERO)
        .expect("zero gravity is valid");
    let system = world.create_particle_system().expect("system should fit");
    let wall = world
        .create_particle_group(
            system,
            &recipe(ParticleFlags::WALL, ParticleGroupFlags::empty())
                .with_linear_velocity(Vec2::new(3.0, 0.0))
                .expect("velocity is finite"),
        )
        .expect("wall group should fit");
    let zombie = world
        .create_particle_group(
            system,
            &recipe(ParticleFlags::ZOMBIE, ParticleGroupFlags::empty()),
        )
        .expect("zombie group should fit");
    let wall_members = world
        .particle_group_view(wall)
        .expect("wall group remains live")
        .member_ids()
        .to_vec();
    let zombie_members = world
        .particle_group_view(zombie)
        .expect("zombie group remains live")
        .member_ids()
        .to_vec();

    // Act
    world
        .step(positive_step(), &mut NoDecisionHook, StepLimits::default())
        .expect("flagged step succeeds");

    // Assert
    assert!(wall_members.into_iter().all(|particle| {
        world
            .particle_snapshot(particle)
            .is_ok_and(|snapshot| snapshot.velocity() == Vec2::ZERO)
    }));
    assert!(zombie_members.into_iter().all(|particle| {
        world.particle_snapshot(particle) == Err(liquidfun::HandleError::StaleOrDestroyed)
    }));
}

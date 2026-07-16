//! Source-compatible particle lifetime clock and ordering behavior.

use liquidfun::particle::{
    ParticleDestructionOccurrence, ParticleLifetimeClock, ParticleLifetimeError, ParticleSystemDef,
};
use liquidfun::{
    ParticleDestructionOccurrence as RootParticleDestructionOccurrence, ParticleId,
    ParticleLifetimeClock as RootParticleLifetimeClock, World,
};

#[test]
fn destruction_occurrence_is_reachable_from_curated_exports() {
    // Arrange
    let module_projection: fn(ParticleDestructionOccurrence) -> ParticleId =
        ParticleDestructionOccurrence::particle;
    let root_projection: fn(RootParticleDestructionOccurrence) -> ParticleId =
        RootParticleDestructionOccurrence::particle;

    // Act
    let projection_sizes = (
        std::mem::size_of_val(&module_projection),
        std::mem::size_of_val(&root_projection),
    );

    // Assert
    assert_eq!(projection_sizes.0, projection_sizes.1);
}

#[test]
fn lifetime_clock_is_public_and_preserves_source_quantization() {
    // Arrange
    let definition = ParticleSystemDef::default()
        .with_lifetime_granularity(0.5)
        .expect("granularity is positive");
    let mut clock = ParticleLifetimeClock::from_system_definition(definition);
    let root_clock = RootParticleLifetimeClock::from_system_definition(definition);

    // Act
    let first_tick = clock.advance(0.25).expect("finite positive step advances");
    let finite = clock
        .expiration_time(1.49)
        .expect("finite lifetime quantizes");
    let zero = clock.expiration_time(0.0).expect("zero is infinite");
    let negative = clock
        .expiration_time(-1.49)
        .expect("negative lifetime is infinite");
    let second_tick = clock.advance(0.25).expect("fractional remainder carries");

    // Assert
    assert_eq!(root_clock.quantized_time_elapsed(), 0);
    assert_eq!(first_tick, 0);
    assert_eq!(finite, 2);
    assert_eq!(zero, 0);
    assert_eq!(negative, -2);
    assert_eq!(second_tick, 1);
}

#[test]
fn invalid_clock_inputs_leave_elapsed_time_unchanged() {
    // Arrange
    let definition = ParticleSystemDef::default();
    let mut clock = ParticleLifetimeClock::from_system_definition(definition);
    clock.advance(0.5).expect("baseline step advances");
    let before = clock;

    // Act
    let negative = clock.advance(-0.25);
    let non_finite = clock.advance(f32::NAN);

    // Assert
    assert_eq!(negative, Err(ParticleLifetimeError::NegativeTimeStep));
    assert_eq!(non_finite, Err(ParticleLifetimeError::NonFiniteTimeStep));
    assert_eq!(clock, before);
}

#[test]
fn canonical_equal_expiration_oldest_order_is_not_rust_sort_accidental() {
    // Arrange
    let mut world = World::new().expect("world key remains available");
    let system = world
        .create_particle_system()
        .expect("particle system fits");
    let particles = (0..8)
        .map(|_| {
            world
                .create_particle(system, None)
                .expect("particle fits")
                .created_particle()
        })
        .collect::<Vec<_>>();
    let definition = ParticleSystemDef::default()
        .with_lifetime_granularity(1.0)
        .expect("granularity is positive");
    let mut ordering = ParticleLifetimeClock::from_system_definition(definition).ordering();
    for particle in &particles {
        ordering
            .set_expiration(*particle, 2)
            .expect("stable identity is accepted once");
    }

    // Act
    let oldest = (0..particles.len())
        .map(|rank| ordering.oldest_particle(rank).expect("rank is present"))
        .collect::<Vec<_>>();

    // Assert
    // Pinned witness 08d41d25... records particle-7 through particle-0.
    assert_eq!(oldest, particles.iter().rev().copied().collect::<Vec<_>>());
}

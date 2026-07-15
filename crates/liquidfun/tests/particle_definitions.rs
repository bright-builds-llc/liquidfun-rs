//! Black-box coverage for checked particle definitions.

use liquidfun::math::Vec2;
use liquidfun::particle::{
    ParticleCapacity, ParticleColor, ParticleDef, ParticleDefError, ParticleFlags,
    ParticleSystemDef, ParticleSystemDefError,
};
use liquidfun::{ParticleDef as RootParticleDef, ParticleSystemDef as RootParticleSystemDef};

#[test]
fn particle_contracts_are_reachable_from_module_and_crate_root() {
    // Arrange / Act
    let module_definition = ParticleDef::default();
    let root_definition = RootParticleDef::default();
    let module_system = ParticleSystemDef::default();
    let root_system = RootParticleSystemDef::default();

    // Assert
    assert_eq!(module_definition, root_definition);
    assert_eq!(module_system, root_system);
}

#[test]
fn particle_flags_preserve_pinned_bits_and_unknown_bits() {
    // Arrange
    let known = [
        (ParticleFlags::WATER, 0),
        (ParticleFlags::ZOMBIE, 1 << 1),
        (ParticleFlags::WALL, 1 << 2),
        (ParticleFlags::SPRING, 1 << 3),
        (ParticleFlags::ELASTIC, 1 << 4),
        (ParticleFlags::VISCOUS, 1 << 5),
        (ParticleFlags::POWDER, 1 << 6),
        (ParticleFlags::TENSILE, 1 << 7),
        (ParticleFlags::COLOR_MIXING, 1 << 8),
        (ParticleFlags::DESTRUCTION_LISTENER, 1 << 9),
        (ParticleFlags::BARRIER, 1 << 10),
        (ParticleFlags::STATIC_PRESSURE, 1 << 11),
        (ParticleFlags::REACTIVE, 1 << 12),
        (ParticleFlags::REPULSIVE, 1 << 13),
        (ParticleFlags::FIXTURE_CONTACT_LISTENER, 1 << 14),
        (ParticleFlags::PARTICLE_CONTACT_LISTENER, 1 << 15),
        (ParticleFlags::FIXTURE_CONTACT_FILTER, 1 << 16),
        (ParticleFlags::PARTICLE_CONTACT_FILTER, 1 << 17),
    ];
    let unknown_bit = 1 << 31;

    // Act
    let retained = ParticleFlags::from_bits_retain(ParticleFlags::ELASTIC.bits() | unknown_bit);

    // Assert
    for (flag, bits) in known {
        assert_eq!(flag.bits(), bits);
    }
    assert_eq!(retained.bits(), (1 << 4) | unknown_bit);
}

#[test]
fn particle_color_preserves_exact_components_and_zero_state() {
    // Arrange
    let zero = ParticleColor::ZERO;

    // Act
    let color = ParticleColor::new(1, 2, 3, 4);

    // Assert
    assert!(zero.is_zero());
    assert!(!color.is_zero());
    assert_eq!(color.components(), [1, 2, 3, 4]);
}

#[test]
fn particle_system_defaults_match_pinned_values() {
    // Arrange / Act
    let definition = ParticleSystemDef::default();

    // Assert
    assert!(!definition.is_paused());
    assert!(!definition.uses_strict_contact_check());
    assert_eq!(definition.density().to_bits(), 1.0_f32.to_bits());
    assert_eq!(definition.gravity_scale().to_bits(), 1.0_f32.to_bits());
    assert_eq!(definition.radius().to_bits(), 1.0_f32.to_bits());
    assert_eq!(definition.damping().to_bits(), 1.0_f32.to_bits());
    assert_eq!(definition.static_pressure_iterations(), 8);
    assert!(definition.destroys_by_age());
    assert_eq!(
        definition.lifetime_granularity().to_bits(),
        (1.0_f32 / 60.0).to_bits()
    );
    assert_eq!(
        definition.capacity(),
        ParticleCapacity::growable(0).expect("zero initial growable capacity should be valid")
    );
    assert_eq!(definition.maximum_count(), None);
}

#[test]
fn particle_system_builders_preserve_checked_controls() {
    // Arrange
    let fixed = ParticleCapacity::fixed(64).expect("positive bounded capacity should be valid");

    // Act
    let definition = ParticleSystemDef::default()
        .with_paused(true)
        .with_strict_contact_check(true)
        .with_density(2.5)
        .expect("positive density should be valid")
        .with_gravity_scale(-0.5)
        .expect("finite gravity scale should be valid")
        .with_radius(0.125)
        .expect("positive radius should be valid")
        .with_damping(1.25)
        .expect("non-negative damping should be valid")
        .with_static_pressure_iterations(4)
        .expect("positive iteration count should be valid")
        .with_destruction_by_age(false)
        .with_lifetime_granularity(0.25)
        .expect("positive granularity should be valid")
        .with_capacity(fixed)
        .expect("capacity should be compatible")
        .with_maximum_count(32)
        .expect("maximum should fit the fixed capacity");

    // Assert
    assert!(definition.is_paused());
    assert!(definition.uses_strict_contact_check());
    assert_eq!(definition.density().to_bits(), 2.5_f32.to_bits());
    assert_eq!(definition.gravity_scale().to_bits(), (-0.5_f32).to_bits());
    assert_eq!(definition.radius().to_bits(), 0.125_f32.to_bits());
    assert_eq!(definition.damping().to_bits(), 1.25_f32.to_bits());
    assert_eq!(definition.static_pressure_iterations(), 4);
    assert!(!definition.destroys_by_age());
    assert_eq!(
        definition.lifetime_granularity().to_bits(),
        0.25_f32.to_bits()
    );
    assert_eq!(definition.capacity(), fixed);
    assert_eq!(definition.maximum_count(), Some(32));
}

#[test]
fn particle_system_rejects_invalid_physical_controls() {
    // Arrange
    let definition = ParticleSystemDef::default();

    // Act
    let density_finite = definition.with_density(f32::NAN);
    let density_positive = definition.with_density(0.0);
    let gravity = definition.with_gravity_scale(f32::INFINITY);
    let radius_finite = definition.with_radius(f32::NEG_INFINITY);
    let radius_positive = definition.with_radius(-1.0);
    let damping_finite = definition.with_damping(f32::NAN);
    let damping_positive = definition.with_damping(-0.5);
    let granularity_finite = definition.with_lifetime_granularity(f32::INFINITY);
    let granularity_positive = definition.with_lifetime_granularity(0.0);
    let iterations = definition.with_static_pressure_iterations(0);

    // Assert
    assert_eq!(
        density_finite,
        Err(ParticleSystemDefError::NonFiniteDensity)
    );
    assert_eq!(
        density_positive,
        Err(ParticleSystemDefError::NonPositiveDensity)
    );
    assert_eq!(gravity, Err(ParticleSystemDefError::NonFiniteGravityScale));
    assert_eq!(radius_finite, Err(ParticleSystemDefError::NonFiniteRadius));
    assert_eq!(
        radius_positive,
        Err(ParticleSystemDefError::NonPositiveRadius)
    );
    assert_eq!(
        damping_finite,
        Err(ParticleSystemDefError::NonFiniteDamping)
    );
    assert_eq!(
        damping_positive,
        Err(ParticleSystemDefError::NegativeDamping)
    );
    assert_eq!(
        granularity_finite,
        Err(ParticleSystemDefError::NonFiniteLifetimeGranularity)
    );
    assert_eq!(
        granularity_positive,
        Err(ParticleSystemDefError::NonPositiveLifetimeGranularity)
    );
    assert_eq!(iterations, Err(ParticleSystemDefError::ZeroIterations));
}

#[test]
fn particle_system_rejects_invalid_capacity_relationships() {
    // Arrange
    let fixed = ParticleCapacity::fixed(8).expect("positive capacity should be valid");
    let fixed_definition = ParticleSystemDef::default()
        .with_capacity(fixed)
        .expect("default maximum should fit");
    let limited_definition = ParticleSystemDef::default()
        .with_maximum_count(4)
        .expect("small maximum should be valid");

    // Act
    let zero_fixed = ParticleCapacity::fixed(0);
    let excessive_maximum = fixed_definition.with_maximum_count(9);
    let undersized_capacity = limited_definition
        .with_capacity(ParticleCapacity::fixed(3).expect("positive capacity should be valid"));

    // Assert
    assert_eq!(zero_fixed, Err(ParticleSystemDefError::ZeroFixedCapacity));
    assert_eq!(
        excessive_maximum,
        Err(ParticleSystemDefError::MaximumExceedsFixedCapacity {
            maximum: 9,
            capacity: 8,
        })
    );
    assert_eq!(
        undersized_capacity,
        Err(ParticleSystemDefError::MaximumExceedsFixedCapacity {
            maximum: 4,
            capacity: 3,
        })
    );
}

#[test]
fn particle_definition_defaults_match_pinned_values() {
    // Arrange / Act
    let definition = ParticleDef::default();

    // Assert
    assert_eq!(definition.flags(), ParticleFlags::WATER);
    assert_eq!(definition.position(), Vec2::ZERO);
    assert_eq!(definition.velocity(), Vec2::ZERO);
    assert_eq!(definition.color(), ParticleColor::ZERO);
    assert_eq!(definition.lifetime().to_bits(), 0.0_f32.to_bits());
    assert!(definition.maybe_user_association().is_none());
}

#[test]
fn particle_definition_builders_preserve_inputs_and_typed_association() {
    // Arrange
    let flags = ParticleFlags::VISCOUS | ParticleFlags::COLOR_MIXING;

    // Act
    let definition = ParticleDef::default()
        .with_position(Vec2::new(1.25, -2.5))
        .expect("finite position should be valid")
        .with_velocity(Vec2::new(-3.75, 4.5))
        .expect("finite velocity should be valid")
        .with_color(ParticleColor::new(10, 20, 30, 40))
        .with_flags(flags)
        .with_lifetime(12.5)
        .expect("finite lifetime should be valid")
        .with_user_association(String::from("particle-a"));

    // Assert
    assert_eq!(definition.position(), Vec2::new(1.25, -2.5));
    assert_eq!(definition.velocity(), Vec2::new(-3.75, 4.5));
    assert_eq!(definition.color().components(), [10, 20, 30, 40]);
    assert_eq!(definition.flags(), flags);
    assert_eq!(definition.lifetime().to_bits(), 12.5_f32.to_bits());
    assert_eq!(
        definition.maybe_user_association().map(String::as_str),
        Some("particle-a")
    );
}

#[test]
fn particle_definition_rejects_non_finite_inputs() {
    // Arrange
    let definition = ParticleDef::default();

    // Act
    let position_x = definition.with_position(Vec2::new(f32::NAN, 0.0));
    let position_y = definition.with_position(Vec2::new(0.0, f32::INFINITY));
    let velocity_x = definition.with_velocity(Vec2::new(f32::NEG_INFINITY, 0.0));
    let velocity_y = definition.with_velocity(Vec2::new(0.0, f32::NAN));
    let lifetime = definition.with_lifetime(f32::INFINITY);

    // Assert
    assert_eq!(position_x, Err(ParticleDefError::NonFinitePositionX));
    assert_eq!(position_y, Err(ParticleDefError::NonFinitePositionY));
    assert_eq!(velocity_x, Err(ParticleDefError::NonFiniteVelocityX));
    assert_eq!(velocity_y, Err(ParticleDefError::NonFiniteVelocityY));
    assert_eq!(lifetime, Err(ParticleDefError::NonFiniteLifetime));
}

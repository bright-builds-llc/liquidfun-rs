//! Black-box coverage for source-ordered particle neighborhoods and contacts.

use liquidfun::collision::Aabb;
use liquidfun::math::Vec2;
use liquidfun::{
    ParticleDef, ParticleNeighborPair, ParticleNeighborhood, ParticleProxyError, ParticleSystemId,
    World,
};
use proptest::prelude::*;

fn create_particle(
    world: &mut World,
    system: ParticleSystemId,
    position: Vec2,
) -> liquidfun::ParticleId {
    let definition = ParticleDef::default()
        .with_position(position)
        .expect("test position should be finite");
    world
        .create_particle_with_def(system, None, &definition)
        .expect("particle should fit")
}

#[test]
fn proxy_neighborhood_preserves_source_row_order_and_stable_ids() {
    // Arrange
    let mut world = World::new().expect("test world key remains available");
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let first = create_particle(&mut world, system, Vec2::new(0.0, 0.0));
    let right = create_particle(&mut world, system, Vec2::new(0.5, 0.0));
    let outside = create_particle(&mut world, system, Vec2::new(2.0, 0.0));
    let below_left = create_particle(&mut world, system, Vec2::new(-0.5, 1.0));
    let view = world
        .particle_system_view(system)
        .expect("particle system should remain live");

    // Act
    let neighborhood = ParticleNeighborhood::from_view(&view, 1.0)
        .expect("finite in-range positions should build proxies");

    // Assert
    let expected = [
        ParticleNeighborPair::new(first, right),
        ParticleNeighborPair::new(first, below_left),
        ParticleNeighborPair::new(right, below_left),
    ];
    assert_eq!(neighborhood.system(), system);
    assert_eq!(neighborhood.pairs(), expected);
    assert!(!neighborhood.pairs().iter().any(|pair| {
        let particles = pair.particles();
        particles.contains(&outside)
    }));
}

#[test]
fn proxy_equal_tags_keep_dense_tie_order_without_exposing_tags() {
    // Arrange
    let mut world = World::new().expect("test world key remains available");
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let first = create_particle(&mut world, system, Vec2::new(0.1, 0.1));
    let second = create_particle(&mut world, system, Vec2::new(0.2, 0.2));
    let third = create_particle(&mut world, system, Vec2::new(0.3, 0.3));
    let view = world
        .particle_system_view(system)
        .expect("particle system should remain live");

    // Act
    let neighborhood = ParticleNeighborhood::from_view(&view, 1.0)
        .expect("finite in-range positions should build proxies");

    // Assert
    assert_eq!(
        neighborhood.pairs(),
        [
            ParticleNeighborPair::new(first, second),
            ParticleNeighborPair::new(first, third),
            ParticleNeighborPair::new(second, third),
        ]
    );
}

#[test]
fn proxy_bounds_return_source_expanded_candidates_in_proxy_order() {
    // Arrange
    let mut world = World::new().expect("test world key remains available");
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    let first = create_particle(&mut world, system, Vec2::new(0.0, 0.0));
    let right = create_particle(&mut world, system, Vec2::new(0.5, 0.0));
    let _outside = create_particle(&mut world, system, Vec2::new(2.0, 0.0));
    let below_left = create_particle(&mut world, system, Vec2::new(-0.5, 1.0));
    let view = world
        .particle_system_view(system)
        .expect("particle system should remain live");
    let neighborhood = ParticleNeighborhood::from_view(&view, 1.0)
        .expect("finite in-range positions should build proxies");
    let bounds =
        Aabb::new(Vec2::new(-0.1, -0.1), Vec2::new(0.6, 0.1)).expect("bounds should be valid");

    // Act
    let candidates = neighborhood
        .particle_candidates_in_bounds(bounds)
        .expect("expanded bounds should remain representable");

    // Assert
    assert_eq!(candidates, vec![first, right, below_left]);
}

#[test]
fn proxy_construction_rejects_invalid_scale_or_unrepresentable_positions() {
    // Arrange
    let mut world = World::new().expect("test world key remains available");
    let system = world
        .create_particle_system()
        .expect("particle system should fit");
    create_particle(&mut world, system, Vec2::new(2048.0, 0.0));
    let view = world
        .particle_system_view(system)
        .expect("particle system should remain live");

    // Act
    let zero = ParticleNeighborhood::from_view(&view, 0.0);
    let infinite = ParticleNeighborhood::from_view(&view, f32::INFINITY);
    let out_of_range = ParticleNeighborhood::from_view(&view, 1.0);

    // Assert
    assert_eq!(zero, Err(ParticleProxyError::NonPositiveDiameter));
    assert_eq!(infinite, Err(ParticleProxyError::NonFiniteDiameter));
    assert_eq!(out_of_range, Err(ParticleProxyError::PositionOutOfTagRange));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn proxy_neighborhood_is_deterministic_for_finite_cell_positions(
        coordinates in prop::collection::vec((-8_i8..=8, -8_i8..=8), 0..32),
    ) {
        // Arrange
        let mut world = World::new().expect("test world key remains available");
        let system = world
            .create_particle_system()
            .expect("particle system should fit");
        for (x, y) in coordinates {
            create_particle(
                &mut world,
                system,
                Vec2::new(f32::from(x) * 0.25, f32::from(y) * 0.25),
            );
        }
        let view = world
            .particle_system_view(system)
            .expect("particle system should remain live");

        // Act
        let first = ParticleNeighborhood::from_view(&view, 1.0)
            .expect("generated positions remain representable");
        let second = ParticleNeighborhood::from_view(&view, 1.0)
            .expect("generated positions remain representable");

        // Assert
        prop_assert_eq!(first, second);
    }
}

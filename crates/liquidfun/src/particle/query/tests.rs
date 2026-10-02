//! Query-only indexing retains the full neighborhood's checked traversal.

use super::*;
use crate::{ParticleDef, ParticleSystemDef};

fn system(world: &mut World, positions: &[Vec2]) -> ParticleSystemId {
    let definition = ParticleSystemDef::default()
        .with_radius(0.5)
        .expect("radius");
    let system = world
        .create_particle_system_with_def(&definition)
        .expect("system");
    for position in positions {
        let receipt = world
            .create_particle_with_def(
                system,
                None,
                &ParticleDef::default()
                    .with_position(*position)
                    .expect("position"),
            )
            .expect("particle");
        assert!(receipt.destruction_occurrences().is_empty());
    }
    system
}

fn legacy_occurrences(
    view: &ParticleSystemView<'_>,
    diameter: f32,
    bounds: Aabb,
) -> Result<Vec<ParticleQueryOccurrence>, ParticleQueryError> {
    let neighborhood = ParticleNeighborhood::from_view(view, diameter)?;
    let candidates = neighborhood.particle_candidates_in_bounds(bounds)?;
    let lower = bounds.lower_bound();
    let upper = bounds.upper_bound();
    Ok(candidates
        .into_iter()
        .filter_map(|particle| {
            let position = position_for(view, particle).expect("same immutable view");
            (lower.x < position.x
                && position.x < upper.x
                && lower.y < position.y
                && position.y < upper.y)
                .then_some(ParticleQueryOccurrence {
                    system: view.system(),
                    particle,
                })
        })
        .collect())
}

#[test]
fn query_index_skips_pairs_while_public_neighborhood_retains_source_pairs() {
    // Arrange
    let mut world = World::new().expect("world");
    let system = system(&mut world, &[Vec2::ZERO; 3]);
    let view = world.particle_system_view(system).expect("view");
    let ids = view.particle_ids();
    let bounds = Aabb::new(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0)).expect("bounds");

    // Act
    let full = ParticleNeighborhood::from_view(&view, 1.0).expect("full");
    let query = ParticleNeighborhood::from_view_for_query(&view, 1.0).expect("query");

    // Assert
    assert_eq!(
        full.pairs(),
        [
            crate::ParticleNeighborPair::new(ids[0], ids[1]),
            crate::ParticleNeighborPair::new(ids[0], ids[2]),
            crate::ParticleNeighborPair::new(ids[1], ids[2]),
        ]
    );
    assert!(
        query.pair_rows().is_empty(),
        "queries must omit unused pair enumeration"
    );
    assert_eq!(
        query.particle_candidates_in_bounds(bounds),
        full.particle_candidates_in_bounds(bounds)
    );
}

#[test]
fn aabb_callbacks_match_legacy_order_equal_tags_and_strict_edges() {
    // Arrange
    let mut world = World::new().expect("world");
    let system = system(
        &mut world,
        &[
            Vec2::new(0.75, 0.25),
            Vec2::new(-0.25, 0.25),
            Vec2::new(0.25, 0.25),
            Vec2::new(0.25, 0.25),
            Vec2::new(-0.5, 0.25),
            Vec2::new(1.0, 0.25),
            Vec2::new(0.25, 0.0),
            Vec2::new(0.25, 1.0),
        ],
    );
    let view = world.particle_system_view(system).expect("view");
    let bounds = Aabb::new(Vec2::new(-0.5, 0.0), Vec2::new(1.0, 1.0)).expect("bounds");
    let expected = legacy_occurrences(&view, 1.0, bounds).expect("legacy");
    let mut actual = Vec::new();

    // Act
    let terminated = query_aabb(&view, 1.0, bounds, &mut |occurrence| {
        actual.push(*occurrence);
        QueryDirective::Continue
    })
    .expect("query");

    // Assert
    assert!(!terminated);
    assert_eq!(actual, expected);
    assert_eq!(
        actual
            .iter()
            .map(|occurrence| occurrence.particle())
            .collect::<Vec<_>>(),
        [
            view.particle_ids()[1],
            view.particle_ids()[2],
            view.particle_ids()[3],
            view.particle_ids()[0]
        ]
    );
}

#[test]
fn aabb_termination_reports_only_the_legacy_first_occurrence() {
    // Arrange
    let mut world = World::new().expect("world");
    let system = system(&mut world, &[Vec2::new(0.75, 0.25), Vec2::new(0.25, 0.25)]);
    let view = world.particle_system_view(system).expect("view");
    let bounds = Aabb::new(Vec2::ZERO, Vec2::new(1.0, 1.0)).expect("bounds");
    let expected = legacy_occurrences(&view, 1.0, bounds).expect("legacy");
    let mut actual = Vec::new();

    // Act
    let terminated = query_aabb(&view, 1.0, bounds, &mut |occurrence| {
        actual.push(*occurrence);
        QueryDirective::Terminate
    })
    .expect("query");

    // Assert
    assert!(terminated);
    assert_eq!(actual, expected[..1]);
}

#[test]
fn query_geometry_failures_preserve_legacy_validation_order_and_no_callbacks() {
    // Arrange
    let mut world = World::new().expect("world");
    let system = system(&mut world, &[Vec2::new(4_096.0, 0.0)]);
    let view = world.particle_system_view(system).expect("view");
    let bounds = Aabb::new(Vec2::ZERO, Vec2::new(4_096.0, 1.0)).expect("bounds");

    // Act / Assert
    for (diameter, error) in [
        (f32::NAN, ParticleProxyError::NonFiniteDiameter),
        (0.0, ParticleProxyError::NonPositiveDiameter),
        (-1.0, ParticleProxyError::NonPositiveDiameter),
        (f32::MIN_POSITIVE, ParticleProxyError::DiameterOutOfRange),
        (f32::MAX, ParticleProxyError::DiameterOutOfRange),
        (1.0, ParticleProxyError::PositionOutOfTagRange),
    ] {
        let mut count = 0;
        let actual = query_aabb(&view, diameter, bounds, &mut |_| {
            count += 1;
            QueryDirective::Continue
        });
        assert_eq!(actual, Err(ParticleQueryError::InvalidProxyGeometry(error)));
        assert_eq!(
            actual,
            legacy_occurrences(&view, diameter, bounds).map(|_| false)
        );
        assert_eq!(count, 0);
    }
}

#[test]
fn empty_query_still_checks_bound_expansion_before_returning() {
    // Arrange
    let mut world = World::new().expect("world");
    let system = system(&mut world, &[]);
    let view = world.particle_system_view(system).expect("view");
    let bounds = Aabb::new(Vec2::ZERO, Vec2::new(2_048.0, 1.0)).expect("bounds");
    let mut count = 0;

    // Act
    let actual = query_aabb(&view, 1.0, bounds, &mut |_| {
        count += 1;
        QueryDirective::Continue
    });

    // Assert
    assert_eq!(
        actual,
        Err(ParticleQueryError::InvalidProxyGeometry(
            ParticleProxyError::PositionOutOfTagRange
        ))
    );
    assert_eq!(
        actual,
        legacy_occurrences(&view, 1.0, bounds).map(|_| false)
    );
    assert_eq!(count, 0);
}

#[test]
fn shape_destruction_keeps_other_systems_and_particles_outside_the_shape() {
    // Arrange
    let mut world = World::new().expect("world");
    let target = system(
        &mut world,
        &[Vec2::ZERO, Vec2::new(0.4, 0.4), Vec2::new(0.6, 0.0)],
    );
    let ids = world
        .particle_system_view(target)
        .expect("view")
        .particle_ids()
        .to_vec();
    let other = system(&mut world, &[Vec2::ZERO]);
    let other_ids = world
        .particle_system_view(other)
        .expect("other view")
        .particle_ids()
        .to_vec();
    let shape = crate::collision::Shape::from(
        crate::collision::CircleShape::new(Vec2::ZERO, 0.5).expect("circle"),
    );

    // Act
    let destroyed = world
        .destroy_particles_in_shape(target, &shape, crate::math::Transform::IDENTITY)
        .expect("checked destruction");

    // Assert
    assert_eq!(destroyed, 1);
    assert!(world.particle_snapshot(ids[0]).is_err());
    assert_eq!(
        world
            .particle_system_view(target)
            .expect("view")
            .particle_ids(),
        &ids[1..]
    );
    assert_eq!(
        world
            .particle_system_view(other)
            .expect("other view")
            .particle_ids(),
        other_ids
    );
}

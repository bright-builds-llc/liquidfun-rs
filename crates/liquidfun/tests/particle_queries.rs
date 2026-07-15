//! Streaming particle-query contract evidence.

use liquidfun::collision::{Aabb, RayCastInput};
use liquidfun::math::Vec2;
use liquidfun::{
    ParticleDef, ParticleId, ParticleQueryOccurrence, ParticleRayCastError, ParticleRayHit,
    ParticleSystemDef, ParticleSystemId, QueryDirective, RayCastDirective, RayCastFraction, World,
};

fn bounds(lower: Vec2, upper: Vec2) -> Aabb {
    Aabb::new(lower, upper).expect("test bounds should be valid")
}

fn ray(start: Vec2, end: Vec2) -> RayCastInput {
    RayCastInput::new(start, end, 1.0).expect("test ray should be valid")
}

fn create_system(world: &mut World) -> ParticleSystemId {
    let definition = ParticleSystemDef::default()
        .with_radius(0.5)
        .expect("test radius should be valid");
    world
        .create_particle_system_with_def(&definition)
        .expect("test system should fit")
}

fn create_particle(world: &mut World, system: ParticleSystemId, position: Vec2) -> ParticleId {
    let definition = ParticleDef::default()
        .with_position(position)
        .expect("test position should be valid");
    world
        .create_particle_with_def(system, None, &definition)
        .expect("test particle should fit")
}

fn assert_public_particle_query(_query: &ParticleQueryOccurrence) {}

fn assert_public_particle_ray(_ray: &ParticleRayHit) {}

#[test]
fn per_system_aabb_returns_stable_particle_identities() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let system = create_system(&mut world);
    let inside = create_particle(&mut world, system, Vec2::new(0.25, 0.5));
    let outside = create_particle(&mut world, system, Vec2::new(2.0, 0.5));
    let query = bounds(Vec2::ZERO, Vec2::new(1.0, 1.0));
    let mut hits = Vec::new();

    // Act
    world
        .query_particle_system_aabb(system, query, |occurrence| {
            assert_public_particle_query(occurrence);
            hits.push((occurrence.system(), occurrence.particle()));
            QueryDirective::Continue
        })
        .expect("live system query should succeed");

    // Assert
    assert_eq!(hits, [(system, inside)]);
    assert!(!hits.iter().any(|(_, particle)| *particle == outside));
}

#[test]
fn per_system_aabb_uses_strict_bounds_and_honors_termination() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let system = create_system(&mut world);
    create_particle(&mut world, system, Vec2::ZERO);
    create_particle(&mut world, system, Vec2::new(0.25, 0.25));
    create_particle(&mut world, system, Vec2::new(0.75, 0.75));
    create_particle(&mut world, system, Vec2::new(1.0, 1.0));
    let query = bounds(Vec2::ZERO, Vec2::new(1.0, 1.0));
    let mut count = 0;

    // Act
    world
        .query_particle_system_aabb(system, query, |_occurrence| {
            count += 1;
            QueryDirective::Terminate
        })
        .expect("live system query should succeed");

    // Assert
    assert_eq!(count, 1);
}

#[test]
fn per_system_ray_continue_reports_each_intersection() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let system = create_system(&mut world);
    let first = create_particle(&mut world, system, Vec2::new(-1.0, 0.0));
    let second = create_particle(&mut world, system, Vec2::new(2.0, 0.0));
    let miss = create_particle(&mut world, system, Vec2::new(0.0, 3.0));
    let input = ray(Vec2::new(-4.0, 0.0), Vec2::new(4.0, 0.0));
    let mut hits = Vec::new();

    // Act
    world
        .ray_cast_particle_system(system, input, |hit| {
            assert_public_particle_ray(hit);
            hits.push(hit.particle());
            RayCastDirective::Continue
        })
        .expect("finite particle ray should succeed");

    // Assert
    assert_eq!(hits.len(), 2);
    assert!(hits.contains(&first));
    assert!(hits.contains(&second));
    assert!(!hits.contains(&miss));
}

#[test]
fn per_system_ray_excludes_particle_containing_start() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let system = create_system(&mut world);
    create_particle(&mut world, system, Vec2::ZERO);
    let input = ray(Vec2::ZERO, Vec2::new(4.0, 0.0));
    let mut count = 0;

    // Act
    world
        .ray_cast_particle_system(system, input, |_hit| {
            count += 1;
            RayCastDirective::Continue
        })
        .expect("finite particle ray should succeed");

    // Assert
    assert_eq!(count, 0);
}

#[test]
fn per_system_ray_ignore_preserves_later_hits() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let system = create_system(&mut world);
    let ignored = create_particle(&mut world, system, Vec2::new(-1.0, 0.0));
    let retained = create_particle(&mut world, system, Vec2::new(2.0, 0.0));
    let input = ray(Vec2::new(-4.0, 0.0), Vec2::new(4.0, 0.0));
    let mut hits = Vec::new();

    // Act
    world
        .ray_cast_particle_system(system, input, |hit| {
            hits.push(hit.particle());
            if hit.particle() == ignored {
                RayCastDirective::Ignore
            } else {
                RayCastDirective::Continue
            }
        })
        .expect("ignore should preserve the ray interval");

    // Assert
    assert!(hits.contains(&ignored));
    assert!(hits.contains(&retained));
}

#[test]
fn per_system_ray_terminate_stops_immediately() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let system = create_system(&mut world);
    create_particle(&mut world, system, Vec2::new(-1.0, 0.0));
    create_particle(&mut world, system, Vec2::new(2.0, 0.0));
    let input = ray(Vec2::new(-4.0, 0.0), Vec2::new(4.0, 0.0));
    let mut count = 0;

    // Act
    world
        .ray_cast_particle_system(system, input, |_hit| {
            count += 1;
            RayCastDirective::Terminate
        })
        .expect("termination should be successful");

    // Assert
    assert_eq!(count, 1);
}

#[test]
fn per_system_ray_clip_narrows_later_candidates() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let system = create_system(&mut world);
    let nearest = create_particle(&mut world, system, Vec2::new(-1.0, 0.0));
    create_particle(&mut world, system, Vec2::new(2.0, 0.0));
    let input = ray(Vec2::new(-4.0, 0.0), Vec2::new(4.0, 0.0));
    let mut maybe_closest = None;

    // Act
    world
        .ray_cast_particle_system(system, input, |hit| {
            maybe_closest = Some(hit.particle());
            RayCastDirective::Clip(hit.fraction())
        })
        .expect("checked hit fractions should clip safely");

    // Assert
    assert_eq!(maybe_closest, Some(nearest));
}

#[test]
fn per_system_ray_hit_exposes_finite_semantic_geometry() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let system = create_system(&mut world);
    let particle = create_particle(&mut world, system, Vec2::ZERO);
    let input = ray(Vec2::new(-2.0, 0.0), Vec2::new(2.0, 0.0));
    let mut maybe_hit = None;

    // Act
    world
        .ray_cast_particle_system(system, input, |hit| {
            maybe_hit = Some(*hit);
            RayCastDirective::Continue
        })
        .expect("finite particle ray should succeed");
    let hit = maybe_hit.expect("particle should be intersected");

    // Assert
    assert_eq!(hit.system(), system);
    assert_eq!(hit.particle(), particle);
    assert!(hit.point().is_valid());
    assert!(hit.normal().is_valid());
    assert!((0.0..=1.0).contains(&hit.fraction().get()));
}

#[test]
fn per_system_ray_clip_rejects_widening_without_world_mutation() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let system = create_system(&mut world);
    create_particle(&mut world, system, Vec2::new(-2.0, 0.0));
    let input = RayCastInput::new(Vec2::new(-4.0, 0.0), Vec2::new(4.0, 0.0), 0.5)
        .expect("test ray should be valid");
    let widening = RayCastFraction::new(0.75).expect("normalized fraction should be valid");
    let positions_before = world
        .particle_system_view(system)
        .expect("system should be live")
        .positions()
        .to_vec();
    let mut callbacks = 0;

    // Act
    let result = world.ray_cast_particle_system(system, input, |_hit| {
        callbacks += 1;
        RayCastDirective::Clip(widening)
    });
    let positions_after = world
        .particle_system_view(system)
        .expect("system should remain live")
        .positions()
        .to_vec();

    // Assert
    assert_eq!(
        result,
        Err(ParticleRayCastError::ClipOutsideCurrentInterval)
    );
    assert_eq!(callbacks, 1);
    assert_eq!(positions_after, positions_before);
}

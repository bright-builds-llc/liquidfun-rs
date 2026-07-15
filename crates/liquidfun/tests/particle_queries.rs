//! Streaming particle-query contract evidence.

use liquidfun::collision::shape::{CircleShape, Shape};
use liquidfun::collision::{Aabb, FilterData, RayCastInput};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, FixtureDef, ParticleDef, ParticleId, ParticleQueryOccurrence,
    ParticleRayCastError, ParticleRayHit, ParticleSystemDef, ParticleSystemId, QueryDirective,
    RayCastDirective, RayCastFraction, World, WorldQueryOccurrence, WorldRayCastOccurrence,
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

fn create_fixture(world: &mut World, position: Vec2) {
    let body_definition =
        BodyDef::new(BodyType::Static, position, 0.0, true).expect("test body should be valid");
    let body = world
        .create_body(&body_definition)
        .expect("test body should fit");
    let shape =
        Shape::from(CircleShape::new(Vec2::ZERO, 0.25).expect("test circle should be valid"));
    let fixture_definition = FixtureDef::new(shape, 0.0, 0.2, 0.0, false, FilterData::default())
        .expect("test fixture should be valid");
    world
        .create_fixture(body, &fixture_definition)
        .expect("test fixture should fit");
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

#[test]
fn mixed_world_aabb_visits_fixtures_before_newest_first_particle_systems() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    create_fixture(&mut world, Vec2::ZERO);
    let older = create_system(&mut world);
    create_particle(&mut world, older, Vec2::new(-0.5, 0.0));
    let newer = create_system(&mut world);
    create_particle(&mut world, newer, Vec2::new(0.5, 0.0));
    let query = bounds(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));
    let mut domains = Vec::new();

    // Act
    world
        .query_aabb_with_particles(query, |occurrence| {
            domains.push(match occurrence {
                WorldQueryOccurrence::Fixture(_fixture) => None,
                WorldQueryOccurrence::Particle(particle) => Some(particle.system()),
            });
            QueryDirective::Continue
        })
        .expect("mixed query should succeed");

    // Assert
    assert_eq!(domains, [None, Some(newer), Some(older)]);
}

#[test]
fn mixed_world_aabb_culls_nonoverlapping_invalid_proxy_system() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let far_system = create_system(&mut world);
    create_particle(&mut world, far_system, Vec2::new(3_000.0, 0.0));
    let near_system = create_system(&mut world);
    let near = create_particle(&mut world, near_system, Vec2::ZERO);
    let query = bounds(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));
    let mut particles = Vec::new();

    // Act
    world
        .query_aabb_with_particles(query, |occurrence| {
            if let WorldQueryOccurrence::Particle(particle) = occurrence {
                particles.push(particle.particle());
            }
            QueryDirective::Continue
        })
        .expect("culled invalid proxy system must not affect the query");

    // Assert
    assert_eq!(particles, [near]);
}

#[test]
fn mixed_world_aabb_fixture_termination_skips_particles() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    create_fixture(&mut world, Vec2::ZERO);
    let system = create_system(&mut world);
    create_particle(&mut world, system, Vec2::ZERO);
    let query = bounds(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));
    let mut occurrences = 0;

    // Act
    world
        .query_aabb_with_particles(query, |_occurrence| {
            occurrences += 1;
            QueryDirective::Terminate
        })
        .expect("termination should be successful");

    // Assert
    assert_eq!(occurrences, 1);
}

#[test]
fn mixed_world_aabb_particle_termination_skips_older_systems() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let older = create_system(&mut world);
    create_particle(&mut world, older, Vec2::new(-0.5, 0.0));
    let newer = create_system(&mut world);
    let first = create_particle(&mut world, newer, Vec2::new(0.5, 0.0));
    let query = bounds(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));
    let mut particles = Vec::new();

    // Act
    world
        .query_aabb_with_particles(query, |occurrence| {
            if let WorldQueryOccurrence::Particle(particle) = occurrence {
                particles.push(particle.particle());
                return QueryDirective::Terminate;
            }
            QueryDirective::Continue
        })
        .expect("particle termination should be successful");

    // Assert
    assert_eq!(particles, [first]);
}

#[test]
fn mixed_world_ray_visits_fixtures_before_particles() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    create_fixture(&mut world, Vec2::new(-1.0, 0.0));
    let system = create_system(&mut world);
    create_particle(&mut world, system, Vec2::new(2.0, 0.0));
    let input = ray(Vec2::new(-4.0, 0.0), Vec2::new(4.0, 0.0));
    let mut domains = Vec::new();

    // Act
    world
        .ray_cast_with_particles(input, |occurrence| {
            domains.push(matches!(occurrence, WorldRayCastOccurrence::Particle(_hit)));
            RayCastDirective::Continue
        })
        .expect("mixed ray should succeed");

    // Assert
    assert_eq!(domains, [false, true]);
}

#[test]
fn mixed_world_ray_fixture_clip_propagates_to_particle_systems() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    create_fixture(&mut world, Vec2::new(-1.0, 0.0));
    let system = create_system(&mut world);
    create_particle(&mut world, system, Vec2::new(2.0, 0.0));
    let input = ray(Vec2::new(-4.0, 0.0), Vec2::new(4.0, 0.0));
    let mut domains = Vec::new();

    // Act
    world
        .ray_cast_with_particles(input, |occurrence| match occurrence {
            WorldRayCastOccurrence::Fixture(hit) => {
                domains.push("fixture");
                RayCastDirective::Clip(hit.fraction())
            }
            WorldRayCastOccurrence::Particle(_hit) => {
                domains.push("particle");
                RayCastDirective::Continue
            }
        })
        .expect("fixture clip should remain valid across particle systems");

    // Assert
    assert_eq!(domains, ["fixture"]);
}

#[test]
fn mixed_world_ray_newest_particle_clip_propagates_to_older_system() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let older = create_system(&mut world);
    create_particle(&mut world, older, Vec2::new(2.0, 0.0));
    let newer = create_system(&mut world);
    let nearest = create_particle(&mut world, newer, Vec2::new(-1.0, 0.0));
    let input = ray(Vec2::new(-4.0, 0.0), Vec2::new(4.0, 0.0));
    let mut particles = Vec::new();

    // Act
    world
        .ray_cast_with_particles(input, |occurrence| match occurrence {
            WorldRayCastOccurrence::Fixture(_hit) => RayCastDirective::Continue,
            WorldRayCastOccurrence::Particle(hit) => {
                particles.push(hit.particle());
                RayCastDirective::Clip(hit.fraction())
            }
        })
        .expect("particle clip should remain valid across systems");

    // Assert
    assert_eq!(particles, [nearest]);
}

#[test]
fn mixed_world_ray_particle_termination_skips_older_systems() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let older = create_system(&mut world);
    create_particle(&mut world, older, Vec2::new(2.0, 0.0));
    let newer = create_system(&mut world);
    let first = create_particle(&mut world, newer, Vec2::new(-1.0, 0.0));
    let input = ray(Vec2::new(-4.0, 0.0), Vec2::new(4.0, 0.0));
    let mut particles = Vec::new();

    // Act
    world
        .ray_cast_with_particles(input, |occurrence| match occurrence {
            WorldRayCastOccurrence::Fixture(_hit) => RayCastDirective::Continue,
            WorldRayCastOccurrence::Particle(hit) => {
                particles.push(hit.particle());
                RayCastDirective::Terminate
            }
        })
        .expect("particle termination should be successful");

    // Assert
    assert_eq!(particles, [first]);
}

#[test]
fn mixed_world_ray_culls_nonoverlapping_invalid_proxy_system() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let far_system = create_system(&mut world);
    create_particle(&mut world, far_system, Vec2::new(3_000.0, 0.0));
    let near_system = create_system(&mut world);
    let near = create_particle(&mut world, near_system, Vec2::ZERO);
    let input = ray(Vec2::new(-2.0, 0.0), Vec2::new(2.0, 0.0));
    let mut particles = Vec::new();

    // Act
    world
        .ray_cast_with_particles(input, |occurrence| {
            if let WorldRayCastOccurrence::Particle(hit) = occurrence {
                particles.push(hit.particle());
            }
            RayCastDirective::Continue
        })
        .expect("culled invalid proxy system must not affect the ray");

    // Assert
    assert_eq!(particles, [near]);
}

#[test]
fn mixed_world_rigid_only_results_equal_existing_rigid_apis() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    create_fixture(&mut world, Vec2::new(-1.0, 0.0));
    create_fixture(&mut world, Vec2::new(1.0, 0.0));
    let query = bounds(Vec2::new(-2.0, -1.0), Vec2::new(2.0, 1.0));
    let input = ray(Vec2::new(-3.0, 0.0), Vec2::new(3.0, 0.0));
    let mut rigid_query = Vec::new();
    let mut mixed_query = Vec::new();
    let mut rigid_ray = Vec::new();
    let mut mixed_ray = Vec::new();

    // Act
    world.query_aabb(query, |hit| {
        rigid_query.push((hit.fixture(), hit.child_index()));
        QueryDirective::Continue
    });
    world
        .query_aabb_with_particles(query, |hit| {
            if let WorldQueryOccurrence::Fixture(hit) = hit {
                mixed_query.push((hit.fixture(), hit.child_index()));
            }
            QueryDirective::Continue
        })
        .expect("rigid-only mixed query should succeed");
    world
        .ray_cast(input, |hit| {
            rigid_ray.push((hit.fixture(), hit.fraction().get().to_bits()));
            RayCastDirective::Continue
        })
        .expect("rigid ray should succeed");
    world
        .ray_cast_with_particles(input, |hit| {
            if let WorldRayCastOccurrence::Fixture(hit) = hit {
                mixed_ray.push((hit.fixture(), hit.fraction().get().to_bits()));
            }
            RayCastDirective::Continue
        })
        .expect("rigid-only mixed ray should succeed");

    // Assert
    assert_eq!(mixed_query, rigid_query);
    assert_eq!(mixed_ray, rigid_ray);
}

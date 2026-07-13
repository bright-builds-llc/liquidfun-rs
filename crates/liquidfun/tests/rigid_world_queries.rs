//! Streaming world-query contract evidence.

use liquidfun::collision::shape::{ChainShape, CircleShape, Shape};
use liquidfun::collision::{Aabb, CollisionError, FilterData, RayCastInput};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, FixtureDef, FixtureId, FixtureQueryOccurrence, QueryDirective,
    RayCastDirective, RayCastFraction, RayCastFractionError, World, WorldRayCastError, WorldRayHit,
};

fn body_definition(position: Vec2) -> BodyDef {
    BodyDef::new(BodyType::Static, position, 0.0, true)
        .expect("test body definition should be valid")
}

fn circle_fixture(filter: FilterData) -> FixtureDef {
    let shape = Shape::from(
        CircleShape::new(Vec2::ZERO, 0.5).expect("test circle geometry should be valid"),
    );
    FixtureDef::new(shape, 0.0, 0.2, 0.0, false, filter)
        .expect("test fixture definition should be valid")
}

fn query_bounds(lower: Vec2, upper: Vec2) -> Aabb {
    Aabb::new(lower, upper).expect("test query bounds should be valid")
}

fn create_circle(world: &mut World, position: Vec2, filter: FilterData) -> FixtureId {
    let body = world
        .create_body(&body_definition(position))
        .expect("test body should fit");
    world
        .create_fixture(body, &circle_fixture(filter))
        .expect("test fixture should fit")
}

fn collect_aabb(world: &World, aabb: Aabb) -> Vec<(FixtureId, usize)> {
    let mut hits = Vec::new();
    world.query_aabb(aabb, |occurrence: &FixtureQueryOccurrence| {
        hits.push((occurrence.fixture(), occurrence.child_index().get()));
        QueryDirective::Continue
    });
    hits
}

fn ray_input(start: Vec2, end: Vec2) -> RayCastInput {
    RayCastInput::new(start, end, 1.0).expect("test ray should be valid")
}

#[test]
fn aabb_query_on_empty_world_streams_no_occurrences() {
    // Arrange
    let world = World::new().expect("test world key should remain available");
    let aabb = query_bounds(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));

    // Act
    let hits = collect_aabb(&world, aabb);

    // Assert
    assert!(hits.is_empty());
}

#[test]
fn aabb_query_streams_full_and_partial_fixture_sets() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let left = create_circle(&mut world, Vec2::new(-2.0, 0.0), FilterData::default());
    let right = create_circle(&mut world, Vec2::new(2.0, 0.0), FilterData::default());
    let full = query_bounds(Vec2::new(-3.0, -1.0), Vec2::new(3.0, 1.0));
    let partial = query_bounds(Vec2::new(-3.0, -1.0), Vec2::new(-1.0, 1.0));

    // Act
    let full_hits = collect_aabb(&world, full);
    let partial_hits = collect_aabb(&world, partial);

    // Assert
    assert_eq!(full_hits.len(), 2);
    assert!(full_hits.contains(&(left, 0)));
    assert!(full_hits.contains(&(right, 0)));
    assert_eq!(partial_hits, [(left, 0)]);
}

#[test]
fn aabb_query_terminate_stops_after_one_occurrence() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    create_circle(&mut world, Vec2::new(-1.0, 0.0), FilterData::default());
    create_circle(&mut world, Vec2::new(1.0, 0.0), FilterData::default());
    let aabb = query_bounds(Vec2::new(-2.0, -1.0), Vec2::new(2.0, 1.0));
    let mut count = 0;

    // Act
    world.query_aabb(aabb, |_occurrence| {
        count += 1;
        QueryDirective::Terminate
    });

    // Assert
    assert_eq!(count, 1);
}

#[test]
fn aabb_query_leaves_collision_filtering_to_the_visitor() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let included = create_circle(
        &mut world,
        Vec2::new(-1.0, 0.0),
        FilterData::new(0x0002, 0x0004, 0),
    );
    let excluded = create_circle(
        &mut world,
        Vec2::new(1.0, 0.0),
        FilterData::new(0x0008, 0x0010, 0),
    );
    let aabb = query_bounds(Vec2::new(-2.0, -1.0), Vec2::new(2.0, 1.0));
    let mut all = Vec::new();
    let mut explicitly_filtered = Vec::new();

    // Act
    world.query_aabb(aabb, |occurrence| {
        all.push(occurrence.fixture());
        if occurrence.fixture() == included {
            explicitly_filtered.push(occurrence.fixture());
        }
        QueryDirective::Continue
    });

    // Assert
    assert_eq!(all.len(), 2);
    assert!(all.contains(&included));
    assert!(all.contains(&excluded));
    assert_eq!(explicitly_filtered, [included]);
}

#[test]
fn aabb_query_preserves_repeated_children_of_one_fixture() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let body = world
        .create_body(&body_definition(Vec2::ZERO))
        .expect("test body should fit");
    let chain = ChainShape::open(
        &[Vec2::new(-1.0, 0.0), Vec2::ZERO, Vec2::new(1.0, 0.0)],
        None,
        None,
    )
    .expect("test chain should be valid");
    let definition = FixtureDef::new(
        Shape::from(chain),
        0.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("test fixture definition should be valid");
    let fixture = world
        .create_fixture(body, &definition)
        .expect("test fixture should fit");
    let aabb = query_bounds(Vec2::new(-2.0, -1.0), Vec2::new(2.0, 1.0));

    // Act
    let mut hits = collect_aabb(&world, aabb);
    hits.sort_by_key(|(_fixture, child)| *child);

    // Assert
    assert_eq!(hits, [(fixture, 0), (fixture, 1)]);
}

#[test]
fn ray_continue_streams_each_real_hit_and_skips_non_intersections() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let left = create_circle(&mut world, Vec2::new(-1.0, 0.0), FilterData::default());
    let right = create_circle(&mut world, Vec2::new(2.0, 0.0), FilterData::default());
    let miss = create_circle(&mut world, Vec2::new(0.0, 3.0), FilterData::default());
    let input = ray_input(Vec2::new(-4.0, 0.0), Vec2::new(4.0, 0.0));
    let mut hits = Vec::new();

    // Act
    world
        .ray_cast(input, |hit: &WorldRayHit| {
            hits.push(hit.fixture());
            RayCastDirective::Continue
        })
        .expect("finite ray should succeed");

    // Assert
    assert_eq!(hits.len(), 2);
    assert!(hits.contains(&left));
    assert!(hits.contains(&right));
    assert!(!hits.contains(&miss));
}

#[test]
fn ray_ignore_preserves_the_interval_for_later_hits() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let ignored = create_circle(&mut world, Vec2::new(-1.0, 0.0), FilterData::default());
    let retained = create_circle(&mut world, Vec2::new(2.0, 0.0), FilterData::default());
    let input = ray_input(Vec2::new(-4.0, 0.0), Vec2::new(4.0, 0.0));
    let mut hits = Vec::new();

    // Act
    world
        .ray_cast(input, |hit| {
            hits.push(hit.fixture());
            if hit.fixture() == ignored {
                RayCastDirective::Ignore
            } else {
                RayCastDirective::Continue
            }
        })
        .expect("ignore should preserve the current interval");

    // Assert
    assert_eq!(hits.len(), 2);
    assert!(hits.contains(&ignored));
    assert!(hits.contains(&retained));
}

#[test]
fn ray_terminate_stops_after_one_real_hit() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    create_circle(&mut world, Vec2::new(-1.0, 0.0), FilterData::default());
    create_circle(&mut world, Vec2::new(2.0, 0.0), FilterData::default());
    let input = ray_input(Vec2::new(-4.0, 0.0), Vec2::new(4.0, 0.0));
    let mut count = 0;

    // Act
    world
        .ray_cast(input, |_hit| {
            count += 1;
            RayCastDirective::Terminate
        })
        .expect("termination should be a successful ray cast");

    // Assert
    assert_eq!(count, 1);
}

#[test]
fn ray_clip_selects_the_nearest_hit_without_promising_visit_order() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let nearest = create_circle(&mut world, Vec2::new(-1.0, 0.0), FilterData::default());
    create_circle(&mut world, Vec2::new(2.0, 0.0), FilterData::default());
    let input = ray_input(Vec2::new(-4.0, 0.0), Vec2::new(4.0, 0.0));
    let mut maybe_closest = None;

    // Act
    world
        .ray_cast(input, |hit| {
            maybe_closest = Some(hit.fixture());
            RayCastDirective::Clip(hit.fraction())
        })
        .expect("checked hit fractions should clip safely");

    // Assert
    assert_eq!(maybe_closest, Some(nearest));
}

#[test]
fn ray_hit_owns_semantic_point_normal_and_fraction_data() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let fixture = create_circle(&mut world, Vec2::ZERO, FilterData::default());
    let input = ray_input(Vec2::new(-2.0, 0.0), Vec2::new(2.0, 0.0));
    let mut maybe_owned_hit = None;

    // Act
    world
        .ray_cast(input, |hit| {
            maybe_owned_hit = Some(*hit);
            RayCastDirective::Continue
        })
        .expect("finite exact hit should succeed");
    let owned_hit = maybe_owned_hit.expect("circle should produce one hit");

    // Assert
    assert_eq!(owned_hit.fixture(), fixture);
    assert_eq!(owned_hit.child_index().get(), 0);
    assert_eq!(owned_hit.point(), Vec2::new(-0.5, 0.0));
    assert_eq!(owned_hit.normal(), Vec2::new(-1.0, 0.0));
    assert_eq!(owned_hit.fraction().get().to_bits(), 0.375_f32.to_bits());
}

#[test]
fn ray_equal_fraction_ties_remain_distinct_semantic_hits() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let body = world
        .create_body(&body_definition(Vec2::ZERO))
        .expect("test body should fit");
    let definition = circle_fixture(FilterData::default());
    let first = world
        .create_fixture(body, &definition)
        .expect("first fixture should fit");
    let second = world
        .create_fixture(body, &definition)
        .expect("second fixture should fit");
    let input = ray_input(Vec2::new(-2.0, 0.0), Vec2::new(2.0, 0.0));
    let mut hits = Vec::new();

    // Act
    world
        .ray_cast(input, |hit| {
            hits.push((hit.fixture(), hit.fraction().get().to_bits()));
            RayCastDirective::Continue
        })
        .expect("equal-distance hits should be supported");

    // Assert
    assert_eq!(hits.len(), 2);
    assert!(hits.iter().any(|(fixture, _fraction)| *fixture == first));
    assert!(hits.iter().any(|(fixture, _fraction)| *fixture == second));
    assert_eq!(hits[0].1, hits[1].1);
}

#[test]
fn ray_chain_vertex_reports_each_intersected_child() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let body = world
        .create_body(&body_definition(Vec2::ZERO))
        .expect("test body should fit");
    let chain = ChainShape::open(
        &[Vec2::new(-1.0, 0.0), Vec2::ZERO, Vec2::new(1.0, 0.0)],
        None,
        None,
    )
    .expect("test chain should be valid");
    let definition = FixtureDef::new(
        Shape::from(chain),
        0.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("test fixture definition should be valid");
    let fixture = world
        .create_fixture(body, &definition)
        .expect("test fixture should fit");
    let input = ray_input(Vec2::new(0.0, -2.0), Vec2::new(0.0, 2.0));
    let mut children = Vec::new();

    // Act
    world
        .ray_cast(input, |hit| {
            assert_eq!(hit.fixture(), fixture);
            children.push(hit.child_index().get());
            RayCastDirective::Continue
        })
        .expect("chain ray should succeed");
    children.sort_unstable();

    // Assert
    assert_eq!(children, [0, 1]);
}

#[test]
fn ray_degenerate_geometry_fails_before_callbacks_without_world_effects() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let body = world
        .create_body(&body_definition(Vec2::ZERO))
        .expect("test body should fit");
    world
        .create_fixture(body, &circle_fixture(FilterData::default()))
        .expect("test fixture should fit");
    let before = world
        .body_snapshot(body)
        .expect("test body should remain live");
    let input = ray_input(Vec2::ZERO, Vec2::ZERO);
    let mut callback_count = 0;

    // Act
    let result = world.ray_cast(input, |_hit| {
        callback_count += 1;
        RayCastDirective::Continue
    });

    // Assert
    assert_eq!(result, Err(WorldRayCastError::DegenerateRay));
    assert_eq!(callback_count, 0);
    assert_eq!(world.body_snapshot(body), Ok(before));
}

#[test]
fn ray_and_clip_construction_reject_invalid_values_without_world_effects() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let body = world
        .create_body(&body_definition(Vec2::ZERO))
        .expect("test body should fit");
    let before = world
        .body_snapshot(body)
        .expect("test body should remain live");

    // Act
    let ray_result = RayCastInput::new(Vec2::new(f32::NAN, 0.0), Vec2::ZERO, 1.0);
    let non_finite_clip = RayCastFraction::new(f32::INFINITY);
    let out_of_range_clip = RayCastFraction::new(1.25);

    // Assert
    assert_eq!(ray_result, Err(CollisionError::NonFiniteValue));
    assert_eq!(non_finite_clip, Err(RayCastFractionError::NonFinite));
    assert_eq!(out_of_range_clip, Err(RayCastFractionError::OutOfRange));
    assert_eq!(world.body_snapshot(body), Ok(before));
}

#[test]
fn ray_clip_outside_current_interval_is_rejected_without_application() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let body = world
        .create_body(&body_definition(Vec2::ZERO))
        .expect("test body should fit");
    world
        .create_fixture(body, &circle_fixture(FilterData::default()))
        .expect("test fixture should fit");
    let before = world
        .body_snapshot(body)
        .expect("test body should remain live");
    let input = RayCastInput::new(Vec2::new(-2.0, 0.0), Vec2::new(2.0, 0.0), 0.5)
        .expect("test clipped ray should be valid");
    let invalid_for_interval =
        RayCastFraction::new(0.75).expect("fraction is valid in the normalized domain");
    let mut callback_effects = 0;

    // Act
    let result = world.ray_cast(input, |_hit| {
        callback_effects += 1;
        RayCastDirective::Clip(invalid_for_interval)
    });

    // Assert
    assert_eq!(result, Err(WorldRayCastError::ClipOutsideCurrentInterval));
    assert_eq!(callback_effects, 1);
    assert_eq!(world.body_snapshot(body), Ok(before));
}

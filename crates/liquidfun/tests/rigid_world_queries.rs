//! Streaming world-query contract evidence.

use liquidfun::collision::shape::{ChainShape, CircleShape, Shape};
use liquidfun::collision::{Aabb, FilterData};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, FixtureDef, FixtureId, FixtureQueryOccurrence, QueryDirective, World,
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

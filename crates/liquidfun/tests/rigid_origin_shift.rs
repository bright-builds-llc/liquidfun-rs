//! Transactional world-origin translation evidence.

use liquidfun::collision::shape::{ChainShape, CircleShape, Shape};
use liquidfun::collision::{Aabb, FilterData, RayCastInput};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, FixtureDef, FixtureId, FixtureQueryOccurrence, OriginShiftError,
    QueryDirective, RayCastDirective, World, WorldRayHit,
};
use liquidfun::{StepConfiguration, StepHook, StepLimits};

struct NoopHook;

impl StepHook for NoopHook {}

fn body_definition(position: Vec2) -> BodyDef {
    BodyDef::new(BodyType::Dynamic, position, 0.0, true)
        .expect("test body definition should be valid")
}

fn static_body_definition(position: Vec2) -> BodyDef {
    BodyDef::new(BodyType::Static, position, 0.0, true)
        .expect("test body definition should be valid")
}

fn circle_fixture() -> FixtureDef {
    let shape = Shape::from(
        CircleShape::new(Vec2::ZERO, 0.5).expect("test circle geometry should be valid"),
    );
    FixtureDef::new(shape, 0.0, 0.2, 0.0, false, FilterData::default())
        .expect("test fixture definition should be valid")
}

fn query_bounds(lower: Vec2, upper: Vec2) -> Aabb {
    Aabb::new(lower, upper).expect("test query bounds should be valid")
}

fn collect_query(world: &World, aabb: Aabb) -> Vec<(FixtureId, usize)> {
    let mut occurrences = Vec::new();
    world.query_aabb(aabb, |occurrence: &FixtureQueryOccurrence| {
        occurrences.push((occurrence.fixture(), occurrence.child_index().get()));
        QueryDirective::Continue
    });
    occurrences
}

fn assert_occurrence_multisets_equal(
    expected: &[(FixtureId, usize)],
    actual: &[(FixtureId, usize)],
) {
    let mut unmatched = actual.to_vec();
    for occurrence in expected {
        let position = unmatched
            .iter()
            .position(|candidate| candidate == occurrence)
            .expect("translated query should retain each semantic occurrence");
        unmatched.remove(position);
    }
    assert!(unmatched.is_empty());
}

#[derive(Clone, Copy)]
struct SemanticRayHit {
    fixture: FixtureId,
    child: usize,
    point: Vec2,
    normal: Vec2,
    fraction_bits: u32,
}

fn semantic_ray_hit(hit: &WorldRayHit) -> SemanticRayHit {
    SemanticRayHit {
        fixture: hit.fixture(),
        child: hit.child_index().get(),
        point: hit.point(),
        normal: hit.normal(),
        fraction_bits: hit.fraction().get().to_bits(),
    }
}

fn collect_ray(
    world: &World,
    input: RayCastInput,
    maybe_ignored: Option<FixtureId>,
) -> Vec<SemanticRayHit> {
    let mut hits = Vec::new();
    world
        .ray_cast(input, |hit| {
            hits.push(semantic_ray_hit(hit));
            if maybe_ignored == Some(hit.fixture()) {
                RayCastDirective::Ignore
            } else {
                RayCastDirective::Continue
            }
        })
        .expect("finite test ray should succeed");
    hits
}

fn closest_ray_hit(world: &World, input: RayCastInput) -> SemanticRayHit {
    let mut maybe_closest = None;
    world
        .ray_cast(input, |hit| {
            maybe_closest = Some(semantic_ray_hit(hit));
            RayCastDirective::Clip(hit.fraction())
        })
        .expect("checked hit fractions should clip safely");
    maybe_closest.expect("test ray should hit at least one fixture")
}

fn assert_ray_hits_covariant(expected: &[SemanticRayHit], actual: &[SemanticRayHit], shift: Vec2) {
    let mut unmatched = actual.to_vec();
    for expected_hit in expected {
        let position = unmatched
            .iter()
            .position(|actual_hit| {
                actual_hit.fixture == expected_hit.fixture && actual_hit.child == expected_hit.child
            })
            .expect("translated ray should retain each semantic hit");
        let actual_hit = unmatched.remove(position);
        assert_eq!(actual_hit.fraction_bits, expected_hit.fraction_bits);
        assert_eq!(actual_hit.normal, expected_hit.normal);
        assert_eq!(actual_hit.point, expected_hit.point - shift);
    }
    assert!(unmatched.is_empty());
}

#[test]
fn origin_shift_rejects_invalid_input_atomically() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let ordinary = world
        .create_body(&body_definition(Vec2::new(8.0, -3.0)))
        .expect("test body should fit");
    let overflowing = world
        .create_body(&body_definition(Vec2::new(-f32::MAX, 2.0)))
        .expect("test body should fit");
    let ordinary_before = world
        .body_snapshot(ordinary)
        .expect("ordinary body should remain live");
    let overflowing_before = world
        .body_snapshot(overflowing)
        .expect("overflowing body should remain live");

    // Act
    let non_finite_result = world.shift_origin(Vec2::new(f32::NAN, 0.0));
    let overflow_result = world.shift_origin(Vec2::new(f32::MAX, 0.0));

    // Assert
    assert_eq!(non_finite_result, Err(OriginShiftError::NonFiniteShift));
    assert_eq!(overflow_result, Err(OriginShiftError::NonFiniteBodyState));
    assert_eq!(world.body_snapshot(ordinary), Ok(ordinary_before));
    assert_eq!(world.body_snapshot(overflowing), Ok(overflowing_before));
}

#[test]
fn origin_shift_preserves_query_multisets_and_termination_semantics() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let chain_body = world
        .create_body(&static_body_definition(Vec2::ZERO))
        .expect("test body should fit");
    let chain = ChainShape::open(
        &[Vec2::new(-2.0, 0.0), Vec2::ZERO, Vec2::new(2.0, 0.0)],
        None,
        None,
    )
    .expect("test chain should be valid");
    let chain_fixture = FixtureDef::new(
        Shape::from(chain),
        0.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("test fixture definition should be valid");
    let chain_fixture = world
        .create_fixture(chain_body, &chain_fixture)
        .expect("test fixture should fit");
    let sleeping_body = world
        .create_body(&body_definition(Vec2::ZERO))
        .expect("test body should fit");
    let sleeping_fixture = world
        .create_fixture(sleeping_body, &circle_fixture())
        .expect("test fixture should fit");
    world
        .step(
            StepConfiguration::new(0.0, 8, 3).expect("zero-duration step should be valid"),
            &mut NoopHook,
            StepLimits::default(),
        )
        .expect("contact discovery should succeed");
    world
        .set_body_awake(sleeping_body, false)
        .expect("test body should remain live");
    let bounds = query_bounds(Vec2::new(-3.0, -1.0), Vec2::new(5.0, 1.0));
    let before = collect_query(&world, bounds);
    let shift = Vec2::new(32.0, -16.0);
    let chain_before = world
        .body_snapshot(chain_body)
        .expect("chain body should remain live");
    let sleeping_before = world
        .body_snapshot(sleeping_body)
        .expect("sleeping body should remain live");
    let contacts_before = world.contact_count();
    let mut termination_before = 0;
    world.query_aabb(bounds, |_occurrence| {
        termination_before += 1;
        QueryDirective::Terminate
    });

    // Act
    world
        .shift_origin(shift)
        .expect("finite translated state should remain valid");
    let translated_bounds =
        query_bounds(bounds.lower_bound() - shift, bounds.upper_bound() - shift);
    let after = collect_query(&world, translated_bounds);
    let mut termination_after = 0;
    world.query_aabb(translated_bounds, |_occurrence| {
        termination_after += 1;
        QueryDirective::Terminate
    });

    // Assert
    assert_occurrence_multisets_equal(&before, &after);
    assert_eq!(before.len(), 3);
    assert_eq!(
        before
            .iter()
            .filter(|(fixture, _child)| *fixture == chain_fixture)
            .count(),
        2
    );
    assert!(before.contains(&(sleeping_fixture, 0)));
    assert_eq!(termination_before, 1);
    assert_eq!(termination_after, 1);
    assert!(contacts_before > 0);
    assert_eq!(world.contact_count(), contacts_before);
    let chain_after = world
        .body_snapshot(chain_body)
        .expect("chain body should remain live");
    let sleeping_after = world
        .body_snapshot(sleeping_body)
        .expect("sleeping body should remain live");
    assert_eq!(chain_after.position(), chain_before.position() - shift);
    assert_eq!(
        sleeping_after.position(),
        sleeping_before.position() - shift
    );
    assert!(!sleeping_after.is_awake());
}

#[test]
fn origin_shift_preserves_continue_ignore_and_clip_ray_results() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let nearest_body = world
        .create_body(&static_body_definition(Vec2::new(-1.0, 0.0)))
        .expect("test body should fit");
    let nearest = world
        .create_fixture(nearest_body, &circle_fixture())
        .expect("test fixture should fit");
    let farther_body = world
        .create_body(&static_body_definition(Vec2::new(3.0, 0.0)))
        .expect("test body should fit");
    let farther = world
        .create_fixture(farther_body, &circle_fixture())
        .expect("test fixture should fit");
    let start = Vec2::new(-5.0, 0.0);
    let end = Vec2::new(6.0, 0.0);
    let input = RayCastInput::new(start, end, 1.0).expect("test ray should be valid");
    let continue_before = collect_ray(&world, input, None);
    let ignore_before = collect_ray(&world, input, Some(farther));
    let closest_before = closest_ray_hit(&world, input);
    let shift = Vec2::new(16.0, -8.0);

    // Act
    world
        .shift_origin(shift)
        .expect("finite translated state should remain valid");
    let translated_input = RayCastInput::new(start - shift, end - shift, 1.0)
        .expect("translated test ray should remain valid");
    let continue_after = collect_ray(&world, translated_input, None);
    let ignore_after = collect_ray(&world, translated_input, Some(farther));
    let closest_after = closest_ray_hit(&world, translated_input);

    // Assert
    assert_ray_hits_covariant(&continue_before, &continue_after, shift);
    assert_ray_hits_covariant(&ignore_before, &ignore_after, shift);
    assert_eq!(closest_before.fixture, nearest);
    assert_eq!(closest_after.fixture, nearest);
    assert_eq!(closest_after.child, closest_before.child);
    assert_eq!(closest_after.fraction_bits, closest_before.fraction_bits);
    assert_eq!(closest_after.normal, closest_before.normal);
    assert_eq!(closest_after.point, closest_before.point - shift);
}

#[test]
fn empty_large_and_repeated_inverse_origin_shifts_preserve_observations() {
    // Arrange
    let mut empty = World::new().expect("test world key should remain available");
    let mut world = World::new().expect("test world key should remain available");
    let static_body = world
        .create_body(&static_body_definition(Vec2::new(-256.0, 512.0)))
        .expect("test body should fit");
    world
        .create_fixture(static_body, &circle_fixture())
        .expect("test fixture should fit");
    let sleeping_body = world
        .create_body(&body_definition(Vec2::new(256.0, -512.0)))
        .expect("test body should fit");
    world
        .create_fixture(sleeping_body, &circle_fixture())
        .expect("test fixture should fit");
    world
        .set_body_awake(sleeping_body, false)
        .expect("test body should remain live");
    let static_before = world
        .body_snapshot(static_body)
        .expect("static body should remain live");
    let sleeping_before = world
        .body_snapshot(sleeping_body)
        .expect("sleeping body should remain live");
    let bounds = query_bounds(Vec2::new(-300.0, -600.0), Vec2::new(300.0, 600.0));
    let query_before = collect_query(&world, bounds);
    let large_shift = Vec2::new(1_048_576.0, -1_048_576.0);

    // Act
    empty
        .shift_origin(Vec2::new(f32::MAX, -f32::MAX))
        .expect("an empty world should accept every finite shift");
    for _iteration in 0..3 {
        world
            .shift_origin(large_shift)
            .expect("large translated state should remain finite");
        world
            .shift_origin(-large_shift)
            .expect("inverse translated state should remain finite");
    }

    // Assert
    assert_eq!(world.body_snapshot(static_body), Ok(static_before));
    assert_eq!(world.body_snapshot(sleeping_body), Ok(sleeping_before));
    assert_occurrence_multisets_equal(&query_before, &collect_query(&world, bounds));
}

//! Filtered iteration-0 CCD hits for moving fixtures equal the full particle scan.
use super::*;
use crate::collision::{ChainShape, CircleShape, EdgeShape, FilterData, PolygonShape};
use crate::{
    BodyDef, BodyType, FixtureDef, NoDecisionHook, ParticleDef, ParticleSystemDef, StepLimits,
};

const DIAMETER: f32 = 0.1;
const TIME_STEP: f32 = 1.0 / 60.0;
const GRID_SIDE: u16 = 20;
/// Upward speed large enough that every grid column below a fixture edge has a
/// particle whose ray crosses it, so the hit lists are never trivially empty.
const RISE_SPEED: f32 = 27.0;
/// Body-local point the particle grid is centered on.
const LOCAL_FOCUS: Vec2 = Vec2::new(1.6, 0.0);
/// Grid half-extent: the fixtures reach about 0.8 m by 0.4 m from the focus,
/// and the grid extends 2 m beyond them.
const GRID_HALF_EXTENT: Vec2 = Vec2::new(2.8, 2.4);

struct Grid {
    body: BodyId,
    fixture: FixtureId,
    candidate: BoundaryCandidate,
    proxies: Vec<ContactProxy>,
}

fn previous_transform() -> Transform {
    Transform::from_position_angle(Vec2::new(0.3, 0.1), 0.2)
}

fn rotated_and_translated(previous: Transform, angle: f32, translation: Vec2) -> Transform {
    Transform::from_position_angle(
        previous.position() + translation,
        previous.rotation().angle() + angle,
    )
}

/// 400 particles on a grid around the fixtures in the previous body frame,
/// all rising at about `rise_speed`.
fn grid(previous: Transform, rise_speed: f32) -> Grid {
    let mut world = World::new().expect("world");
    let body = world
        .create_body(&BodyDef::new(BodyType::Kinematic, Vec2::ZERO, 0.0, true).expect("body"))
        .expect("body");
    let fixture = world
        .create_fixture(
            body,
            &FixtureDef::new(
                Shape::from(PolygonShape::box_shape(0.4, 0.3).expect("box")),
                1.0,
                0.2,
                0.0,
                false,
                FilterData::default(),
            )
            .expect("fixture"),
        )
        .expect("fixture");
    let system = world
        .create_particle_system_with_def(
            &ParticleSystemDef::default()
                .with_radius(0.5 * DIAMETER)
                .expect("radius"),
        )
        .expect("system");
    let center = previous.apply(LOCAL_FOCUS);
    let lower = center - GRID_HALF_EXTENT;
    let step = Vec2::new(
        2.0 * GRID_HALF_EXTENT.x / f32::from(GRID_SIDE - 1),
        2.0 * GRID_HALF_EXTENT.y / f32::from(GRID_SIDE - 1),
    );
    for row in 0..GRID_SIDE {
        for column in 0..GRID_SIDE {
            let position = Vec2::new(
                lower.x + step.x * f32::from(column),
                lower.y + step.y * f32::from(row),
            );
            let drift = 0.5 * f32::from((row + column) % 3) - 0.5;
            let receipt = world
                .create_particle_with_def(
                    system,
                    None,
                    &ParticleDef::default()
                        .with_position(position)
                        .expect("position")
                        .with_velocity(Vec2::new(drift, rise_speed))
                        .expect("velocity"),
                )
                .expect("particle");
            assert!(receipt.destruction_occurrences().is_empty());
        }
    }
    let storage = &world.particle_systems.get(system).expect("system").storage;
    let candidate = BoundaryCandidate::new(
        system,
        storage.particle_ids(),
        storage.positions(),
        storage.velocities(),
        storage.forces(),
        storage.flags(),
        storage.groups(),
        storage.group_records(),
        storage.has_pending_system_force(),
        storage.len(),
    )
    .expect("candidate");
    let mut proxies = Vec::new();
    contact_scan::fill_stored_contacts(
        storage.positions(),
        storage.flags(),
        storage.particle_ids(),
        DIAMETER,
        &mut proxies,
        &mut Vec::new(),
        &mut |_contact| true,
    )
    .expect("proxies");
    Grid {
        body,
        fixture,
        candidate,
        proxies,
    }
}

fn record(
    grid: &Grid,
    shape: Shape,
    previous: Transform,
    current: Transform,
    body_local_center: Vec2,
) -> CcdFixtureRecord {
    let children = (0..shape.child_count())
        .map(|child| {
            let child = shape.child_index(child).expect("child");
            let expansion = Vec2::new(DIAMETER, DIAMETER);
            (
                child,
                expanded_shape_aabb(&shape, current, child, expansion),
            )
        })
        .collect();
    CcdFixtureRecord {
        body: grid.body,
        fixture: grid.fixture,
        previous_transform: previous,
        current_transform: current,
        body_local_center,
        is_circle: matches!(shape, Shape::Circle(_)),
        shape,
        children,
    }
}

fn polygon() -> Shape {
    Shape::from(PolygonShape::oriented_box(0.4, 0.3, Vec2::new(1.5, 0.0), 0.0).expect("box"))
}

fn hits_with(
    grid: &Grid,
    records: &[CcdFixtureRecord],
    particle_iteration: u32,
    proxies: &[ContactProxy],
) -> Vec<FilteredCollisionHit> {
    let mut hook = NoDecisionHook;
    let mut hook_run = ContactHookRun::new(&mut hook, StepLimits::default());
    let mut hits = Vec::new();
    collect_fixture_hits(
        &grid.candidate,
        records,
        TIME_STEP,
        particle_iteration,
        &mut hook_run,
        proxies,
        DIAMETER,
        &mut hits,
    )
    .expect("hits");
    hits
}

/// An empty proxy slice fails `proxies_match`, which forces the full scan.
fn full_scan(
    grid: &Grid,
    records: &[CcdFixtureRecord],
    particle_iteration: u32,
) -> Vec<FilteredCollisionHit> {
    hits_with(grid, records, particle_iteration, &[])
}

fn filtered(
    grid: &Grid,
    records: &[CcdFixtureRecord],
    particle_iteration: u32,
) -> Vec<FilteredCollisionHit> {
    hits_with(grid, records, particle_iteration, &grid.proxies)
}

type HitBits = (
    usize,
    BodyId,
    [u32; 4],
    [u32; 4],
    [u32; 2],
    bool,
    u32,
    [u32; 2],
);

fn transform_bits(transform: Transform) -> [u32; 4] {
    [
        transform.position().x.to_bits(),
        transform.position().y.to_bits(),
        transform.rotation().sine().to_bits(),
        transform.rotation().cosine().to_bits(),
    ]
}

fn hit_bits(hits: &[FilteredCollisionHit]) -> Vec<HitBits> {
    hits.iter()
        .map(|hit| {
            (
                hit.particle,
                hit.body,
                transform_bits(hit.previous_transform),
                transform_bits(hit.current_transform),
                [
                    hit.body_local_center.x.to_bits(),
                    hit.body_local_center.y.to_bits(),
                ],
                hit.is_circle,
                hit.fraction.to_bits(),
                [hit.normal.x.to_bits(), hit.normal.y.to_bits()],
            )
        })
        .collect()
}

fn motion(grid: &Grid) -> f32 {
    max_particle_motion(&grid.candidate.velocities, TIME_STEP)
}

fn pad_for(grid: &Grid, record: &CcdFixtureRecord, child: usize) -> Option<f32> {
    let aabb = record.children[child].1.expect("child aabb");
    moving_fixture_query_pad(
        aabb,
        motion(grid),
        record.previous_transform,
        record.current_transform,
        record.body_local_center,
        record.is_circle,
    )
}

fn visited_rows(grid: &Grid, aabb: Aabb, pad: f32) -> usize {
    let mut visited = 0;
    let queried = query_particles_for_fixture(&grid.proxies, DIAMETER, aabb, pad, |_row| {
        visited += 1;
        Ok(())
    })
    .expect("query");
    assert!(queried);
    visited
}

#[test]
fn rotating_polygon_filtered_hits_equal_full_scan() {
    // Arrange
    let previous = previous_transform();
    let current = rotated_and_translated(previous, 0.05, Vec2::new(0.02, -0.01));
    let grid = grid(previous, RISE_SPEED);
    let records = [record(&grid, polygon(), previous, current, Vec2::ZERO)];
    let pad = pad_for(&grid, &records[0], 0).expect("pad engages");
    let aabb = records[0].children[0].1.expect("child aabb");

    // Act
    let visited = visited_rows(&grid, aabb, pad);
    let full = full_scan(&grid, &records, 0);
    let filtered = filtered(&grid, &records, 0);

    // Assert
    assert_eq!(
        grid.candidate.positions.len(),
        usize::from(GRID_SIDE * GRID_SIDE)
    );
    assert!(visited < 400, "visited {visited} rows");
    assert!(!full.is_empty());
    assert_eq!(hit_bits(&filtered), hit_bits(&full));
}

#[test]
fn washing_machine_rotation_pad_converges() {
    // Arrange
    let previous = Transform::from_position_angle(Vec2::new(0.0, 0.5), 0.7);
    let current = rotated_and_translated(previous, 0.035, Vec2::ZERO);
    let rib_center = previous.position() + Vec2::new(1.5 * 0.9_f32.cos(), 1.5 * 0.9_f32.sin());
    let half = Vec2::new(0.05, 0.05);
    let aabb = Aabb::new(rib_center - half, rib_center + half).expect("aabb");

    // Act
    let pad = moving_fixture_query_pad(aabb, 0.02, previous, current, Vec2::ZERO, false)
        .expect("pad converges");

    // Assert
    let padding = Vec2::new(pad, pad);
    let lower = aabb.lower_bound() - padding;
    let upper = aabb.upper_bound() + padding;
    let largest = [
        lower,
        Vec2::new(upper.x, lower.y),
        upper,
        Vec2::new(lower.x, upper.y),
    ]
    .into_iter()
    .map(|corner| {
        let start = collision_start_from_previous_transform(
            corner,
            previous,
            current,
            Vec2::ZERO,
            false,
            0,
        )
        .expect("start");
        (start.x - corner.x).abs().max((start.y - corner.y).abs())
    })
    .fold(0.0_f32, f32::max);
    assert!(pad < 0.1, "pad {pad}");
    assert!(
        pad >= largest,
        "pad {pad} below corner displacement {largest}"
    );
}

#[test]
fn fast_rotation_falls_back_to_full_scan() {
    // Arrange
    let previous = previous_transform();
    let current = rotated_and_translated(previous, 1.2, Vec2::ZERO);
    let grid = grid(previous, RISE_SPEED);
    let records = [record(&grid, polygon(), previous, current, Vec2::ZERO)];

    // Act
    let maybe_pad = pad_for(&grid, &records[0], 0);
    let full = full_scan(&grid, &records, 0);
    let filtered = filtered(&grid, &records, 0);

    // Assert
    assert_eq!(maybe_pad, None);
    assert_eq!(hit_bits(&filtered), hit_bits(&full));
}

#[test]
fn translating_circle_filtered_hits_equal_full_scan() {
    // Arrange
    let previous = previous_transform();
    let current = rotated_and_translated(previous, 0.05, Vec2::new(0.03, 0.02));
    let grid = grid(previous, RISE_SPEED);
    let circle = Shape::from(CircleShape::new(Vec2::new(1.5, 0.0), 0.35).expect("circle"));
    let records = [record(
        &grid,
        circle,
        previous,
        current,
        Vec2::new(1.3, 0.1),
    )];

    // Act
    let maybe_pad = pad_for(&grid, &records[0], 0);
    let full = full_scan(&grid, &records, 0);
    let filtered = filtered(&grid, &records, 0);

    // Assert
    assert!(maybe_pad.is_some());
    assert!(!full.is_empty());
    assert_eq!(hit_bits(&filtered), hit_bits(&full));
}

#[test]
fn chain_and_edge_children_filtered_hits_equal_full_scan() {
    // Arrange
    let previous = previous_transform();
    let current = rotated_and_translated(previous, -0.05, Vec2::new(-0.02, 0.01));
    let grid = grid(previous, RISE_SPEED);
    let chain = Shape::from(
        ChainShape::open(
            &[
                Vec2::new(0.8, -0.3),
                Vec2::new(1.2, -0.25),
                Vec2::new(1.6, -0.35),
                Vec2::new(2.0, -0.3),
                Vec2::new(2.4, -0.28),
            ],
            None,
            None,
        )
        .expect("chain"),
    );
    let edge =
        Shape::from(EdgeShape::new(Vec2::new(0.9, 0.3), Vec2::new(2.1, 0.35)).expect("edge"));
    let records = [
        record(&grid, chain, previous, current, Vec2::ZERO),
        record(&grid, edge, previous, current, Vec2::ZERO),
    ];

    // Act
    let pads = (0..records[0].children.len())
        .map(|child| pad_for(&grid, &records[0], child))
        .collect::<Vec<_>>();
    let full = full_scan(&grid, &records, 0);
    let filtered = filtered(&grid, &records, 0);

    // Assert
    assert_eq!(pads.len(), 4);
    assert!(pads.iter().all(Option::is_some));
    assert!(full.len() >= 2, "{} hits", full.len());
    assert_eq!(hit_bits(&filtered), hit_bits(&full));
}

#[test]
fn static_and_later_iteration_paths_unchanged() {
    // Arrange
    let previous = previous_transform();
    let moved = rotated_and_translated(previous, 0.05, Vec2::new(0.02, -0.01));
    let grid = grid(previous, RISE_SPEED);
    let fixed = [record(&grid, polygon(), previous, previous, Vec2::ZERO)];
    let moving = [record(&grid, polygon(), previous, moved, Vec2::ZERO)];

    // Act
    let static_full = full_scan(&grid, &fixed, 0);
    let static_filtered = filtered(&grid, &fixed, 0);
    let later_full = full_scan(&grid, &moving, 1);
    let later_filtered = filtered(&grid, &moving, 1);

    // Assert
    assert!(!static_full.is_empty());
    assert!(!later_full.is_empty());
    assert_eq!(hit_bits(&static_filtered), hit_bits(&static_full));
    assert_eq!(hit_bits(&later_filtered), hit_bits(&later_full));
}

#[test]
fn swept_edge_hits_beyond_motion_pad_are_kept() {
    // Arrange: a thin edge sweeps 0.3 rad over slow particles, so some hits
    // start far outside the edge AABB padded by the particle travel alone.
    let previous = Transform::from_position_angle(Vec2::new(0.3, 0.1), -0.3);
    let current = Transform::from_position_angle(Vec2::new(0.3, 0.1), 0.0);
    let grid = grid(previous, 0.6);
    let edge = Shape::from(EdgeShape::new(Vec2::new(1.0, 0.0), Vec2::new(2.2, 0.0)).expect("edge"));
    let records = [record(&grid, edge, previous, current, Vec2::ZERO)];
    let aabb = records[0].children[0].1.expect("child aabb");
    let travel = Vec2::new(motion(&grid), motion(&grid));
    let travel_only =
        Aabb::new(aabb.lower_bound() - travel, aabb.upper_bound() + travel).expect("aabb");
    let outside_travel_only = |particle: usize| {
        let position = grid.candidate.positions[particle];
        position.x < travel_only.lower_bound().x
            || position.x > travel_only.upper_bound().x
            || position.y < travel_only.lower_bound().y
            || position.y > travel_only.upper_bound().y
    };

    // Act
    let maybe_pad = pad_for(&grid, &records[0], 0);
    let full = full_scan(&grid, &records, 0);
    let filtered = filtered(&grid, &records, 0);

    // Assert
    assert!(maybe_pad.is_some());
    assert!(full.iter().any(|hit| outside_travel_only(hit.particle)));
    assert_eq!(hit_bits(&filtered), hit_bits(&full));
}

//! Black-box source-order and bounded island-construction witnesses.

#![cfg(feature = "differential-internals")]

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::rigid_differential::RigidIslandBuildError;
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, HandleError, PreSolveDirective, StepConfiguration,
    StepHook, StepLimits, World,
};

fn dynamic_definition() -> BodyDef {
    BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
        .expect("dynamic test body definition should be valid")
}

fn create_dynamic(world: &mut World) -> BodyId {
    world
        .create_body(&dynamic_definition())
        .expect("dynamic test body should fit")
}

fn create_body(world: &mut World, body_type: BodyType, position: Vec2) -> BodyId {
    let definition = BodyDef::new(body_type, position, 0.0, true)
        .expect("positioned body definition should be valid");
    world
        .create_body(&definition)
        .expect("positioned body should fit")
}

fn attach_circle(world: &mut World, body: BodyId, sensor: bool) {
    let shape =
        Shape::from(CircleShape::new(Vec2::ZERO, 0.55).expect("test circle should be valid"));
    let definition = FixtureDef::new(shape, 1.0, 0.2, 0.0, sensor, FilterData::default())
        .expect("test fixture should be valid");
    world
        .create_fixture(body, &definition)
        .expect("test fixture should fit");
}

fn discover_contacts<H: StepHook>(world: &mut World, hook: &mut H) {
    let configuration =
        StepConfiguration::new(0.0, 8, 3).expect("zero-duration step should be valid");
    world
        .step(configuration, hook, StepLimits::default())
        .expect("zero-duration contact discovery should succeed");
}

#[derive(Default)]
struct NoopHook;

impl StepHook for NoopHook {}

struct DisableHook;

impl StepHook for DisableHook {
    fn pre_solve(&mut self, _contact: liquidfun::ContactView<'_>) -> PreSolveDirective {
        PreSolveDirective::Disable
    }
}

#[test]
fn body_order_preserves_newest_first_creation_destruction_and_slot_reuse() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let first = create_dynamic(&mut world);
    let second = create_dynamic(&mut world);
    let third = create_dynamic(&mut world);
    let fourth = create_dynamic(&mut world);
    assert_eq!(
        world.rigid_body_order_diagnostic(),
        vec![fourth, third, second, first]
    );

    // Act and Assert: middle destruction.
    world
        .destroy_body(third)
        .expect("middle body should be live");
    assert_eq!(
        world.rigid_body_order_diagnostic(),
        vec![fourth, second, first]
    );

    // Act and Assert: head destruction.
    world
        .destroy_body(fourth)
        .expect("head body should be live");
    assert_eq!(world.rigid_body_order_diagnostic(), vec![second, first]);

    // Act and Assert: tail destruction.
    world.destroy_body(first).expect("tail body should be live");
    assert_eq!(world.rigid_body_order_diagnostic(), vec![second]);

    // Act and Assert: arena slot reuse still prepends by source-list semantics.
    let replacement = create_dynamic(&mut world);
    assert_ne!(replacement, first);
    assert_eq!(
        world.rigid_body_order_diagnostic(),
        vec![replacement, second]
    );
}

#[test]
fn body_order_rejects_cross_world_destruction_without_mutation() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let first = create_dynamic(&mut world);
    let second = create_dynamic(&mut world);
    let before = world.rigid_body_order_diagnostic();
    let mut other = World::new().expect("second world key should remain available");
    let foreign = create_dynamic(&mut other);

    // Act
    let result = world.destroy_body(foreign);

    // Assert
    assert_eq!(result, Err(HandleError::WrongWorld));
    assert_eq!(world.rigid_body_order_diagnostic(), before);
    assert_eq!(before, vec![second, first]);
    assert_eq!(other.rigid_body_order_diagnostic(), vec![foreign]);
}

#[test]
fn dfs_preserves_source_order_and_stops_propagation_at_static_bodies() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let left = create_body(&mut world, BodyType::Static, Vec2::new(0.0, 0.0));
    let first = create_body(&mut world, BodyType::Dynamic, Vec2::new(1.0, 0.0));
    let second = create_body(&mut world, BodyType::Dynamic, Vec2::new(2.0, 0.0));
    let right = create_body(&mut world, BodyType::Static, Vec2::new(3.0, 0.0));
    for body in [left, first, second, right] {
        attach_circle(&mut world, body, false);
    }
    discover_contacts(&mut world, &mut NoopHook);

    // Act
    let islands = world
        .rigid_island_diagnostics()
        .expect("bounded graph should build");

    // Assert
    assert_eq!(islands.len(), 1);
    assert_eq!(islands[0].body_ids(), &[second, first, left, right]);
    assert_eq!(islands[0].contact_occurrences(), &[3, 2, 1]);
    assert_eq!(islands[0].position_count(), 4);
    assert_eq!(islands[0].velocity_count(), 4);
    assert_eq!(islands[0].joint_count(), 0);
}

#[test]
fn dfs_reuses_a_shared_static_across_later_islands() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let boundary = create_body(&mut world, BodyType::Static, Vec2::ZERO);
    let left = create_body(&mut world, BodyType::Dynamic, Vec2::new(-1.0, 0.0));
    let right = create_body(&mut world, BodyType::Dynamic, Vec2::new(1.0, 0.0));
    for body in [boundary, left, right] {
        attach_circle(&mut world, body, false);
    }
    discover_contacts(&mut world, &mut NoopHook);

    // Act
    let islands = world
        .rigid_island_diagnostics()
        .expect("shared-static graph should build");

    // Assert
    assert_eq!(islands.len(), 2);
    assert_eq!(islands[0].body_ids(), &[right, boundary]);
    assert_eq!(islands[1].body_ids(), &[left, boundary]);
    assert_eq!(islands[0].contact_occurrences().len(), 1);
    assert_eq!(islands[1].contact_occurrences().len(), 1);
}

#[test]
fn dfs_wakes_a_reached_body_only_in_candidate_scratch() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let sleeping_definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
        .expect("sleeping definition should be valid")
        .with_awake(false);
    let sleeping = world
        .create_body(&sleeping_definition)
        .expect("sleeping body should fit");
    let awake = create_body(&mut world, BodyType::Dynamic, Vec2::new(1.0, 0.0));
    attach_circle(&mut world, sleeping, false);
    attach_circle(&mut world, awake, false);
    discover_contacts(&mut world, &mut NoopHook);

    // Act
    let islands = world
        .rigid_island_diagnostics()
        .expect("connected awake graph should build");

    // Assert
    assert_eq!(islands.len(), 1);
    assert_eq!(islands[0].body_ids(), &[awake, sleeping]);
    assert!(islands[0].body_snapshots()[1].is_awake());
    assert!(
        !world
            .body_snapshot(sleeping)
            .expect("sleeping body remains live")
            .is_awake()
    );
}

#[test]
fn dfs_skips_inactive_asleep_sensor_and_disabled_paths() {
    // Arrange: seed eligibility.
    let mut seed_world = World::new().expect("world key should remain available");
    let awake = create_dynamic(&mut seed_world);
    let inactive_definition = BodyDef::new(BodyType::Dynamic, Vec2::new(3.0, 0.0), 0.0, false)
        .expect("inactive definition should be valid");
    let inactive = seed_world
        .create_body(&inactive_definition)
        .expect("inactive body should fit");
    let asleep_definition = BodyDef::new(BodyType::Dynamic, Vec2::new(6.0, 0.0), 0.0, true)
        .expect("asleep definition should be valid")
        .with_awake(false);
    let asleep = seed_world
        .create_body(&asleep_definition)
        .expect("asleep body should fit");

    // Act: seed eligibility.
    let seed_islands = seed_world
        .rigid_island_diagnostics()
        .expect("disconnected seed graph should build");

    // Assert: inactive and asleep bodies are not seeds.
    assert_eq!(seed_islands.len(), 1);
    assert_eq!(seed_islands[0].body_ids(), &[awake]);
    assert!(seed_world.body_snapshot(inactive).is_ok());
    assert!(seed_world.body_snapshot(asleep).is_ok());

    // Arrange: sensor and disabled contact paths.
    let mut sensor_world = touching_pair(true);
    discover_contacts(&mut sensor_world, &mut NoopHook);
    let mut disabled_world = touching_pair(false);
    discover_contacts(&mut disabled_world, &mut DisableHook);

    // Act
    let sensor_island = sensor_world
        .rigid_island_diagnostics()
        .expect("sensor graph should build");
    let disabled_island = disabled_world
        .rigid_island_diagnostics()
        .expect("disabled graph should build");

    // Assert
    assert_eq!(sensor_island.len(), 1);
    assert_eq!(disabled_island.len(), 1);
    assert!(sensor_island[0].contact_occurrences().is_empty());
    assert!(disabled_island[0].contact_occurrences().is_empty());
    assert_eq!(sensor_island[0].body_ids().len(), 1);
    assert_eq!(disabled_island[0].body_ids().len(), 1);
}

#[test]
fn dfs_capacity_n_and_n_plus_one_are_typed_and_state_preserving() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let first = create_dynamic(&mut world);
    let second = create_dynamic(&mut world);
    let order_before = world.rigid_body_order_diagnostic();
    let snapshots_before = [
        world.body_snapshot(first).expect("first body remains live"),
        world
            .body_snapshot(second)
            .expect("second body remains live"),
    ];

    // Act
    let accepted = world.rigid_island_diagnostics_with_limits(2, 0);
    let rejected = world.rigid_island_diagnostics_with_limits(1, 0);

    // Assert
    assert_eq!(accepted.expect("N bodies should fit").len(), 2);
    assert_eq!(
        rejected,
        Err(RigidIslandBuildError::CapacityExceeded {
            resource: "island bodies",
            limit: 1,
        })
    );
    assert_eq!(world.rigid_body_order_diagnostic(), order_before);
    assert_eq!(
        [
            world.body_snapshot(first).expect("first body remains live"),
            world
                .body_snapshot(second)
                .expect("second body remains live"),
        ],
        snapshots_before
    );

    // Arrange and Act: one contact fits exactly, zero rejects it.
    let mut contact_world = touching_pair(false);
    discover_contacts(&mut contact_world, &mut NoopHook);
    let contact_accepted = contact_world.rigid_island_diagnostics_with_limits(2, 1);
    let contact_rejected = contact_world.rigid_island_diagnostics_with_limits(2, 0);

    // Assert
    assert_eq!(contact_accepted.expect("N contacts should fit").len(), 1);
    assert_eq!(
        contact_rejected,
        Err(RigidIslandBuildError::CapacityExceeded {
            resource: "island contacts",
            limit: 0,
        })
    );
}

fn touching_pair(sensor: bool) -> World {
    let mut world = World::new().expect("world key should remain available");
    let boundary = create_body(&mut world, BodyType::Static, Vec2::ZERO);
    let dynamic = create_body(&mut world, BodyType::Dynamic, Vec2::new(1.0, 0.0));
    attach_circle(&mut world, boundary, sensor);
    attach_circle(&mut world, dynamic, sensor);
    world
}

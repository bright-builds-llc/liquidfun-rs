use liquidfun::math::Vec2;
use liquidfun::{BodyType, JointDef, JointId, JointKind, WorldObservationLimits};

use super::build;
use crate::scene::{SceneId, build_scene};
use crate::session::{SessionCore, SessionError};

const WAIST_TOP: f32 = 0.98;
const WAIST_EXIT: f32 = 0.90;
const CHAMBER_LEFT: f32 = -0.55;
const DIVIDER_INNER_X: f32 = 0.48;
const HUB: Vec2 = Vec2::new(0.0, 0.40);
const HUB_RADIUS: f32 = 0.12;
const SHAFT_WALL_X: f32 = 0.56;
const PROOF_BATCHES: usize = 30;
const ANGLE_FLOOR: f32 = 0.05;
const PLATE_TRANSLATION_LIMIT: f32 = 0.01;
const SPEED_TOLERANCE: f32 = 1.0e-5;

#[test]
fn reservoir_starts_still_above_the_waist() {
    // Arrange
    let session =
        SessionCore::create(SceneId::LiquidBubbler).expect("liquid bubbler should construct");

    // Act
    let (live_count, positions, velocities) = session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should be live");
        (
            view.particle_ids().len(),
            view.positions().to_vec(),
            view.velocities().to_vec(),
        )
    });
    let angle = revolute_angle(&session);
    let translation = plate_translation(&session);

    // Assert
    assert!(live_count > 0, "the reservoir should start with water");
    assert!(
        velocities.iter().copied().all(velocity_is_zero),
        "a fresh drip has no particle velocity"
    );
    assert!(
        positions
            .iter()
            .all(|position| position.y > WAIST_TOP && position.x < DIVIDER_INNER_X),
        "every particle starts above the waist and left of the divider"
    );
    assert_eq!(angle.to_bits(), 0.0_f32.to_bits());
    assert_eq!(translation.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn drip_crosses_the_waist_and_turns_the_wheel() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::LiquidBubbler).expect("liquid bubbler should construct");
    let started_above = session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should be live");
        view.particle_ids()
            .iter()
            .copied()
            .zip(view.positions().iter().copied())
            .filter(|(_id, position)| position.y > WAIST_TOP)
            .collect::<Vec<_>>()
    });
    let start_count = session
        .live_particle_count()
        .expect("the reservoir should start with water");

    // Act
    advance_proof(&mut session);
    let end_count = session
        .live_particle_count()
        .expect("the same water should still be live");
    let crossed = session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should still be live");
        started_above.iter().any(|(id, _start)| {
            view.particle_ids()
                .iter()
                .zip(view.positions())
                .any(|(live, position)| live == id && is_below_the_waist_outside_the_hub(*position))
        })
    });
    let angle = revolute_angle(&session);

    // Assert
    assert_eq!(end_count, start_count, "the live count stays unchanged");
    assert!(
        crossed,
        "at least one original particle id is below the waist in the lower chamber"
    );
    assert!(
        angle.abs() >= ANGLE_FLOOR,
        "the wheel angle should leave 0, got {angle}"
    );
}

#[test]
fn plate_stays_down_during_the_proof() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::LiquidBubbler).expect("liquid bubbler should construct");

    // Act
    advance_proof(&mut session);
    let translation = plate_translation(&session);
    let speed = prismatic_motor_speed(&session);

    // Assert
    assert!(
        translation.abs() < PLATE_TRANSLATION_LIMIT,
        "the plate stays down during the proof, translation {translation}"
    );
    assert!(
        (speed - 0.0).abs() < SPEED_TOLERANCE,
        "the plate motor stays at 0 during the dwell, speed {speed}"
    );
}

#[test]
fn wheel_motor_stays_off_and_the_plate_is_beside_the_wheel() {
    // Arrange
    let session =
        SessionCore::create(SceneId::LiquidBubbler).expect("liquid bubbler should construct");

    // Act
    let (motor_enabled, axis_count, dynamic_positions) =
        session.read_particles(|world, _system| {
            let observation = world
                .world_observation(WorldObservationLimits::reviewed())
                .expect("reviewed observation should include the joints");
            let revolute = observation
                .joints()
                .iter()
                .find(|joint| joint.snapshot().kind() == JointKind::Revolute)
                .expect("the wheel rides a revolute joint");
            let JointDef::Revolute(revolute_definition) = revolute.snapshot().definition() else {
                panic!("the wheel joint should be revolute");
            };
            let axis_count = observation
                .joints()
                .iter()
                .filter(|joint| joint.snapshot().kind() == JointKind::Prismatic)
                .filter(|joint| {
                    let JointDef::Prismatic(definition) = joint.snapshot().definition() else {
                        return false;
                    };
                    definition.local_axis_a().x.to_bits() == 0.0_f32.to_bits()
                        && definition.local_axis_a().y.to_bits() == 1.0_f32.to_bits()
                })
                .count();
            let dynamic_positions = observation
                .bodies()
                .iter()
                .filter(|body| body.snapshot().body_type() == BodyType::Dynamic)
                .map(|body| body.snapshot().position())
                .collect::<Vec<_>>();
            (
                revolute_definition.is_motor_enabled(),
                axis_count,
                dynamic_positions,
            )
        });

    // Assert
    assert!(!motor_enabled, "the revolute motor stays disabled");
    assert_eq!(axis_count, 1, "exactly one prismatic joint points world-up");
    assert_eq!(
        dynamic_positions.len(),
        2,
        "the wheel and the plate are dynamic"
    );
    assert!(
        dynamic_positions
            .iter()
            .any(|position| position.x > SHAFT_WALL_X),
        "the plate sits beside the wheel"
    );
}

#[test]
fn rebuild_restores_the_reservoir() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::LiquidBubbler).expect("liquid bubbler should construct");
    advance_proof(&mut session);
    drop(session);

    // Act
    let rebuilt = SessionCore::create(SceneId::LiquidBubbler)
        .expect("a new session should restore the reservoir");
    let (positions, velocities) = rebuilt.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the rebuilt water system should be live");
        (view.positions().to_vec(), view.velocities().to_vec())
    });
    let angle = revolute_angle(&rebuilt);
    let translation = plate_translation(&rebuilt);

    // Assert
    assert_eq!(angle.to_bits(), 0.0_f32.to_bits());
    assert_eq!(translation.to_bits(), 0.0_f32.to_bits());
    assert!(
        velocities.iter().copied().all(velocity_is_zero),
        "rebuild restores a still reservoir"
    );
    assert!(
        positions
            .iter()
            .all(|position| position.y > WAIST_TOP && position.x < DIVIDER_INNER_X),
        "rebuild puts every particle back above the waist"
    );
}

#[test]
fn unknown_controls_and_pointer_leave_the_drip_alone() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::LiquidBubbler).expect("liquid bubbler should construct");
    let before = session
        .live_particle_count()
        .expect("the starting water should be countable");

    // Act
    let waist = session.apply_control("waist", "0.2");
    let color = session.apply_control("color", "amber");
    let wheel = session.apply_action("wheel");
    let pointer = session.apply_pointer("up", 0.0, 1.2);
    let after = session
        .live_particle_count()
        .expect("pointer up should leave the water in place");

    // Assert
    assert_eq!(waist, Err(SessionError::UnknownControl));
    assert_eq!(color, Err(SessionError::UnknownControl));
    assert_eq!(wheel, Err(SessionError::UnknownControl));
    assert_eq!(pointer, Ok(()));
    assert_eq!(after, before);
}

#[test]
fn source_lifts_with_a_prismatic_plate() {
    // Arrange
    let source = include_str!("../liquid_bubbler.rs");

    // Act
    let lifts = source.contains("set_prismatic_motor_speed");
    let keeps_particles = source.contains("with_destruction_by_age(false)");
    let spins_the_wheel = source.contains("set_revolute_motor_speed");
    let spawns_singles = source.contains("create_particle_with_def");
    let writes_positions = source.contains("set_particle_position");
    let mixes_color = source.contains("with_color_mixing_strength");

    // Assert
    assert!(lifts, "the plate speed is written from simulation time");
    assert!(keeps_particles, "age destruction stays off");
    assert!(!spins_the_wheel, "the wheel is not motor-driven");
    assert!(!spawns_singles, "the return does not emit a jet");
    assert!(!writes_positions, "the return does not teleport particles");
    assert!(!mixes_color, "the drip is one plain color");
}

#[test]
fn non_empty_presets_are_rejected_and_gravity_is_stripped() {
    // Arrange
    let waist = [("waist".to_owned(), "0.2".to_owned())];
    let gravity = [("gravity".to_owned(), "10".to_owned())];

    // Act
    let rejected = build(&waist);
    let built = build_scene(SceneId::LiquidBubbler, &gravity);

    // Assert
    assert!(matches!(rejected, Err(SessionError::UnknownControl)));
    let built = built.expect("gravity is stripped before the scene builder");
    assert_eq!(built.world.gravity().x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(built.world.gravity().y.to_bits(), (-10.0_f32).to_bits());
}

fn advance_proof(session: &mut SessionCore) {
    for _ in 0..PROOF_BATCHES {
        session
            .advance(4)
            .expect("the two-second proof window should step");
    }
}

fn is_below_the_waist_outside_the_hub(position: Vec2) -> bool {
    let offset_x = position.x - HUB.x;
    let offset_y = position.y - HUB.y;
    let distance_sq = offset_x * offset_x + offset_y * offset_y;
    position.y < WAIST_EXIT
        && position.x > CHAMBER_LEFT
        && position.x < DIVIDER_INNER_X
        && distance_sq > HUB_RADIUS * HUB_RADIUS
}

fn velocity_is_zero(velocity: Vec2) -> bool {
    velocity.x.to_bits() == 0.0_f32.to_bits() && velocity.y.to_bits() == 0.0_f32.to_bits()
}

fn revolute_angle(session: &SessionCore) -> f32 {
    session.read_particles(|world, _system| {
        let joint = sole_joint(
            world,
            JointKind::Revolute,
            "the wheel rides one revolute joint",
        );
        world
            .revolute_joint_angle(joint)
            .expect("the wheel angle should be readable")
    })
}

fn plate_translation(session: &SessionCore) -> f32 {
    session.read_particles(|world, _system| {
        let joint = sole_joint(
            world,
            JointKind::Prismatic,
            "the plate rides one prismatic joint",
        );
        world
            .prismatic_joint_translation(joint)
            .expect("the plate translation should be readable")
    })
}

fn prismatic_motor_speed(session: &SessionCore) -> f32 {
    session.read_particles(|world, _system| {
        let observation = world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the plate joint");
        let joint = observation
            .joints()
            .iter()
            .find(|joint| joint.snapshot().kind() == JointKind::Prismatic)
            .expect("the plate rides a prismatic joint");
        let JointDef::Prismatic(definition) = joint.snapshot().definition() else {
            panic!("the plate joint should be prismatic");
        };
        definition.motor_speed()
    })
}

fn sole_joint(world: &liquidfun::World, kind: JointKind, label: &str) -> JointId {
    let observation = world
        .world_observation(WorldObservationLimits::reviewed())
        .expect("reviewed observation should include the scene joints");
    let mut joints = observation
        .joints()
        .iter()
        .filter(|joint| joint.snapshot().kind() == kind);
    let joint = joints.next().expect(label);
    assert!(
        joints.next().is_none(),
        "{label} is the only joint of that kind"
    );
    joint.id()
}

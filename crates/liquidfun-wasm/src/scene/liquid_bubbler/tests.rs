use liquidfun::math::Vec2;
use liquidfun::{BodyType, JointDef, JointId, JointKind, WorldObservationLimits};

use super::build;
use crate::scene::{SceneId, build_scene};
use crate::session::{SessionCore, SessionError};

const CHAMBER_LEFT: f32 = -0.55;
const DIVIDER_INNER_X: f32 = 0.48;
const SHAFT_WALL_X: f32 = 0.56;
const PROOF_BATCHES: usize = 30;
const ANGLE_FLOOR: f32 = 0.05;
const PLATE_TRANSLATION_LIMIT: f32 = 0.01;
const SPEED_TOLERANCE: f32 = 1.0e-5;

#[test]
fn reservoir_holds_three_thousand_finer_particles() {
    // Arrange / Act
    let session =
        SessionCore::create(SceneId::LiquidBubbler).expect("liquid bubbler should construct");

    // Assert
    assert_eq!(
        session.particle_count(),
        3_000,
        "the upper chamber should start with 3000 particles"
    );
    assert_eq!(
        session
            .live_particle_count()
            .expect("the reservoir should be live"),
        3_000
    );
    assert!(
        super::PARTICLE_RADIUS < 0.025,
        "particles should be finer than the original 0.025 m drip"
    );
}

#[test]
fn reservoir_starts_still_above_the_top_shelf() {
    // Arrange
    let session =
        SessionCore::create(SceneId::LiquidBubbler).expect("liquid bubbler should construct");
    let shelf_top = super::LEVELS[0].shelf_top;

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
    let angles = revolute_angles(&session);
    let translation = plate_translation(&session);

    // Assert
    assert!(live_count > 0, "the reservoir should start with water");
    assert!(
        velocities.iter().copied().all(velocity_is_zero),
        "a fresh drip has no particle velocity"
    );
    assert!(
        positions.iter().all(|position| {
            position.y > shelf_top
                && position.y < super::DIVIDER_TOP_Y
                && position.x < DIVIDER_INNER_X
        }),
        "every particle starts above the top shelf, below the spill lip, and left of the divider"
    );
    assert_eq!(angles.len(), super::LEVEL_COUNT);
    assert!(
        angles
            .iter()
            .all(|angle| angle.to_bits() == 0.0_f32.to_bits()),
        "every spinner starts at rest"
    );
    assert_eq!(translation.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn drip_crosses_the_top_shelf_and_turns_a_wheel() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::LiquidBubbler).expect("liquid bubbler should construct");
    let shelf_top = super::LEVELS[0].shelf_top;
    let started_above = session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should be live");
        view.particle_ids()
            .iter()
            .copied()
            .zip(view.positions().iter().copied())
            .filter(|(_id, position)| position.y > shelf_top)
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
                .any(|(live, position)| {
                    live == id && is_below_the_top_shelf_outside_the_hub(*position)
                })
        })
    });
    let angles = revolute_angles(&session);

    // Assert
    assert_eq!(end_count, start_count, "the live count stays unchanged");
    assert!(
        crossed,
        "at least one original particle id is below the top shelf"
    );
    assert!(
        angles.iter().any(|angle| angle.abs() >= ANGLE_FLOOR),
        "a spinner angle should leave 0, got {angles:?}"
    );
}

#[test]
fn return_lifts_an_original_particle_above_the_top_shelf() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::LiquidBubbler).expect("liquid bubbler should construct");
    let shelf_top = super::LEVELS[0].shelf_top;
    let started_above = session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should be live");
        view.particle_ids()
            .iter()
            .copied()
            .zip(view.positions().iter().copied())
            .filter(|(_id, position)| position.y > shelf_top)
            .map(|(id, _position)| id)
            .collect::<Vec<_>>()
    });
    let start_count = session
        .live_particle_count()
        .expect("the reservoir should start with water");

    // Act
    advance_return(&mut session);
    let end_count = session
        .live_particle_count()
        .expect("the same water should still be live");
    let returned = session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should still be live");
        started_above.iter().any(|id| {
            view.particle_ids()
                .iter()
                .zip(view.positions())
                .any(|(live, position)| {
                    live == id && position.y > shelf_top && position.x < DIVIDER_INNER_X
                })
        })
    });
    let motors_enabled = revolute_motors_enabled(&session);

    // Assert
    assert_eq!(end_count, start_count, "the live count stays unchanged");
    assert!(
        returned,
        "an original above-shelf particle is back above the top shelf and left of the divider"
    );
    assert!(
        motors_enabled.iter().all(|enabled| !enabled),
        "every spinner motor stays disabled"
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
fn plate_speed_is_four_times_the_original_cruise() {
    // Arrange
    let original_cruise = 0.15_f32;

    // Act
    let speed = super::PLATE_SPEED;

    // Assert
    assert_eq!(speed.to_bits(), (original_cruise * 4.0).to_bits());
}

#[test]
fn plate_pauses_at_the_top_before_descending() {
    // Arrange
    let rise_end = super::DWELL + super::RISE_SECONDS;
    let top_end = rise_end + super::TOP_DWELL;

    // Act
    let during_pause = super::scheduled_plate_speed(rise_end + 1.0);
    let last_pause_step = super::scheduled_plate_speed(top_end - super::SIM_DT);
    let descent = super::scheduled_plate_speed(top_end + super::SIM_DT);

    // Assert
    assert_eq!(during_pause.to_bits(), 0.0_f32.to_bits());
    assert_eq!(last_pause_step.to_bits(), 0.0_f32.to_bits());
    assert_eq!(descent.to_bits(), (-super::PLATE_SPEED).to_bits());
    assert!(
        super::TOP_DWELL >= 3.0,
        "the top pause should last a few seconds"
    );
}

#[test]
fn plate_top_slants_two_degrees_toward_the_chamber() {
    // Arrange
    let expected = std::f32::consts::TAU * 2.0 / 360.0;
    let corners = super::plate_local_corners();
    let left_lip = corners[3];
    let right_lip = corners[2];
    let lowest = corners
        .iter()
        .map(|corner| corner.y)
        .fold(f32::MAX, f32::min);

    // Act
    let slant = super::PLATE_SLANT;

    // Assert
    assert_eq!(
        slant.to_bits(),
        expected.to_bits(),
        "the deck slants two degrees"
    );
    assert!(
        left_lip.y < right_lip.y,
        "the lip toward the chamber is the low side"
    );
    assert!(
        super::PLATE_CENTER.y + lowest >= super::PLATE_FLOOR_CLEARANCE,
        "the slanted low corner stays above the floor skin"
    );
}

#[test]
fn plate_motor_uses_the_faster_cruise_after_the_dwell() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::LiquidBubbler).expect("liquid bubbler should construct");

    // Act
    advance_seconds(&mut session, super::DWELL + 0.5);
    let speed = prismatic_motor_speed(&session);

    // Assert
    assert_eq!(speed.to_bits(), super::PLATE_SPEED.to_bits());
}

#[test]
fn each_level_puts_a_spinner_under_its_hole() {
    // Arrange
    let levels = super::LEVELS;

    // Act
    let placed = levels.map(|level| {
        let centered = (level.spinner_center.x - level.hole_center_x).abs() < 1.0e-4;
        let clearance = level.shelf_bottom() - (level.spinner_center.y + super::PADDLE_OUTER);
        let hole_width = level.hole_right() - level.hole_left();
        let lip_reaches_the_walls =
            level.hole_left() > CHAMBER_LEFT && level.hole_right() < DIVIDER_INNER_X;
        (centered, clearance, hole_width, lip_reaches_the_walls)
    });

    // Assert
    assert_eq!(levels.len(), super::LEVEL_COUNT);
    assert!(levels[0].hole_center_x < 0.0 && levels[2].hole_center_x < 0.0);
    assert!(
        levels[1].hole_center_x > 0.0,
        "the middle hole sits on the right"
    );
    for (centered, clearance, hole_width, lip_reaches_the_walls) in placed {
        assert!(centered, "the spinner is centered under the hole");
        assert!(
            clearance > super::PARTICLE_RADIUS * 2.0,
            "the spinner clears the shelf by more than one particle, clearance {clearance}"
        );
        assert!(
            hole_width > super::PARTICLE_RADIUS * 2.0,
            "the hole is wider than one particle"
        );
        assert!(lip_reaches_the_walls, "each shelf spans the left chamber");
    }
}

#[test]
fn wheel_motors_stay_off_and_the_plate_is_beside_the_wheels() {
    // Arrange
    let session =
        SessionCore::create(SceneId::LiquidBubbler).expect("liquid bubbler should construct");

    // Act
    let (motors_enabled, axis_count, dynamic_positions) =
        session.read_particles(|world, _system| {
            let observation = world
                .world_observation(WorldObservationLimits::reviewed())
                .expect("reviewed observation should include the joints");
            let motors_enabled = observation
                .joints()
                .iter()
                .filter(|joint| joint.snapshot().kind() == JointKind::Revolute)
                .map(|joint| {
                    let JointDef::Revolute(definition) = joint.snapshot().definition() else {
                        panic!("a spinner joint should be revolute");
                    };
                    definition.is_motor_enabled()
                })
                .collect::<Vec<_>>();
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
            (motors_enabled, axis_count, dynamic_positions)
        });

    // Assert
    assert_eq!(motors_enabled.len(), super::LEVEL_COUNT);
    assert!(
        motors_enabled.iter().all(|enabled| !enabled),
        "every spinner motor stays disabled"
    );
    assert_eq!(axis_count, 1, "exactly one prismatic joint points world-up");
    assert_eq!(
        dynamic_positions.len(),
        super::LEVEL_COUNT + 1,
        "the spinners and the plate are dynamic"
    );
    assert_eq!(
        dynamic_positions
            .iter()
            .filter(|position| position.x > SHAFT_WALL_X)
            .count(),
        1,
        "the plate sits beside the wheels"
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
    let shelf_top = super::LEVELS[0].shelf_top;
    let (positions, velocities) = rebuilt.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the rebuilt water system should be live");
        (view.positions().to_vec(), view.velocities().to_vec())
    });
    let angles = revolute_angles(&rebuilt);
    let translation = plate_translation(&rebuilt);

    // Assert
    assert_eq!(angles.len(), super::LEVEL_COUNT);
    assert!(
        angles
            .iter()
            .all(|angle| angle.to_bits() == 0.0_f32.to_bits())
    );
    assert_eq!(translation.to_bits(), 0.0_f32.to_bits());
    assert!(
        velocities.iter().copied().all(velocity_is_zero),
        "rebuild restores a still reservoir"
    );
    assert!(
        positions.iter().all(|position| {
            position.y > shelf_top
                && position.y < super::DIVIDER_TOP_Y
                && position.x < DIVIDER_INNER_X
        }),
        "rebuild puts every particle back above the top shelf and below the spill lip"
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
    assert!(!spins_the_wheel, "the wheels are not motor-driven");
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

fn advance_seconds(session: &mut SessionCore, seconds: f32) {
    let steps = (seconds / super::SIM_DT).ceil() as usize;
    let batches = steps.div_ceil(4);
    for _ in 0..batches {
        session.advance(4).expect("the timed window should step");
    }
}

fn advance_return(session: &mut SessionCore) {
    let seconds = super::DWELL + super::RISE_SECONDS + super::TOP_DWELL;
    advance_seconds(session, seconds);
}

fn revolute_motors_enabled(session: &SessionCore) -> Vec<bool> {
    session.read_particles(|world, _system| {
        let observation = world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the wheel joints");
        observation
            .joints()
            .iter()
            .filter(|joint| joint.snapshot().kind() == JointKind::Revolute)
            .map(|joint| {
                let JointDef::Revolute(definition) = joint.snapshot().definition() else {
                    panic!("a spinner joint should be revolute");
                };
                definition.is_motor_enabled()
            })
            .collect()
    })
}

fn is_below_the_top_shelf_outside_the_hub(position: Vec2) -> bool {
    let level = super::LEVELS[0];
    let offset_x = position.x - level.spinner_center.x;
    let offset_y = position.y - level.spinner_center.y;
    let distance_sq = offset_x * offset_x + offset_y * offset_y;
    position.y < level.shelf_bottom()
        && position.x > CHAMBER_LEFT
        && position.x < DIVIDER_INNER_X
        && distance_sq > super::HUB_RADIUS * super::HUB_RADIUS
}

fn velocity_is_zero(velocity: Vec2) -> bool {
    velocity.x.to_bits() == 0.0_f32.to_bits() && velocity.y.to_bits() == 0.0_f32.to_bits()
}

fn revolute_angles(session: &SessionCore) -> Vec<f32> {
    session.read_particles(|world, _system| {
        let observation = world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the wheel joints");
        observation
            .joints()
            .iter()
            .filter(|joint| joint.snapshot().kind() == JointKind::Revolute)
            .map(|joint| {
                world
                    .revolute_joint_angle(joint.id())
                    .expect("a wheel angle should be readable")
            })
            .collect()
    })
}

fn plate_translation(session: &SessionCore) -> f32 {
    session.read_particles(|world, _system| {
        let joint = sole_prismatic(world);
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

fn sole_prismatic(world: &liquidfun::World) -> JointId {
    let observation = world
        .world_observation(WorldObservationLimits::reviewed())
        .expect("reviewed observation should include the scene joints");
    let mut joints = observation
        .joints()
        .iter()
        .filter(|joint| joint.snapshot().kind() == JointKind::Prismatic);
    let joint = joints.next().expect("the plate rides one prismatic joint");
    assert!(
        joints.next().is_none(),
        "the plate rides the only prismatic joint"
    );
    joint.id()
}

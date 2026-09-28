//! Cascade proofs for Stacked Drip.

use liquidfun::math::Vec2;
use liquidfun::{BodyType, JointDef, JointId, JointKind, World, WorldObservationLimits};

use crate::scene::SceneId;
use crate::session::{SessionCore, SessionError};

const PLATE_TRANSLATION_LIMIT: f32 = 0.01;
const SPEED_TOLERANCE: f32 = 1.0e-5;
const PLATE_MIN_X: f32 = 0.98;

#[test]
fn reservoir_starts_still_above_the_top_tray() {
    // Arrange
    let session = fresh_session();

    // Act
    let (live_count, positions, velocities) = particle_state(&session);
    let angles = tray_angles(&session);
    let translation = plate_translation(&session);

    // Assert
    assert!(
        live_count == 2000,
        "the reservoir should hold 2000 particles, got {live_count}"
    );
    assert!(
        velocities.iter().copied().all(velocity_is_zero),
        "a fresh drip has no particle velocity"
    );
    assert!(
        positions
            .iter()
            .all(|position| position.y > super::TOP_TRAY_Y && position.x < super::DIVIDER_INNER_X),
        "every particle starts above the top tray and left of the divider"
    );
    for angle in angles {
        assert_eq!(angle.to_bits(), 0.0_f32.to_bits());
    }
    assert_eq!(translation.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn cohort_passes_below_the_bottom_tray() {
    // Arrange
    let mut session = fresh_session();
    let started_above = ids_above_the_top_tray(&session);
    let start_count = session
        .live_particle_count()
        .expect("the reservoir should start with water");

    // Act
    advance_proof(&mut session);
    let end_count = session
        .live_particle_count()
        .expect("the same water should still be live");
    let cohort = session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should still be live");
        started_above
            .iter()
            .filter_map(|id| {
                view.particle_ids()
                    .iter()
                    .zip(view.positions())
                    .find(|(live, _position)| *live == id)
                    .map(|(_live, position)| *position)
            })
            .collect::<Vec<_>>()
    });

    // Assert
    assert_eq!(end_count, start_count, "the live count stays unchanged");
    assert_eq!(
        cohort.len(),
        started_above.len(),
        "every original particle is still live"
    );
    assert!(
        cohort
            .iter()
            .all(|position| position.y < super::BOTTOM_TRAY_Y),
        "every original particle that began above the top tray is below the bottom tray, cohort {cohort:?}"
    );
    let bypassed = cohort
        .iter()
        .any(|position| position.x >= super::DIVIDER_INNER_X && position.y >= super::BOTTOM_TRAY_Y);
    assert!(
        !bypassed,
        "an original particle is right of the divider while still at or above the bottom tray, cohort {cohort:?}"
    );
}

#[test]
fn upper_tray_moves_before_the_lower_trays() {
    // Arrange
    let mut session = fresh_session();
    let start = tray_angles(&session);
    let mut first = [None, None, None];

    // Act
    for batch in 1..=super::PROOF_BATCHES {
        session.advance(4).expect("the cascade proof should step");
        let angles = tray_angles(&session);
        for (index, angle) in angles.iter().enumerate() {
            let moved = (angle - start[index]).abs() >= super::ANGLE_FLOOR;
            if first[index].is_none() && moved {
                first[index] = Some(batch);
            }
        }
    }
    let finals = tray_angles(&session);

    // Assert
    for angle in start {
        assert_eq!(angle.to_bits(), 0.0_f32.to_bits());
    }
    let Some(upper_batch) = first[0] else {
        panic!("the upper tray stayed under the angle floor, finals {finals:?}");
    };
    let Some(middle_batch) = first[1] else {
        panic!("the middle tray stayed under the angle floor, finals {finals:?}");
    };
    let Some(lower_batch) = first[2] else {
        panic!("the lower tray stayed under the angle floor, finals {finals:?}");
    };
    assert!(
        upper_batch < middle_batch,
        "upper batch {upper_batch} should precede middle batch {middle_batch}"
    );
    assert!(
        middle_batch < lower_batch,
        "middle batch {middle_batch} should precede lower batch {lower_batch}"
    );
    for angle in finals {
        assert!(
            (angle - 0.0).abs() >= super::ANGLE_FLOOR,
            "each tray should leave rest by at least the angle floor, got {angle}, finals {finals:?}"
        );
    }
}

#[test]
fn plate_stays_down_during_the_cascade() {
    // Arrange
    let mut session = fresh_session();

    // Act
    advance_proof(&mut session);
    let translation = plate_translation(&session);
    let speed = prismatic_motor_speed(&session);

    // Assert
    assert!(
        translation.abs() < PLATE_TRANSLATION_LIMIT,
        "the plate stays down during the cascade, translation {translation}"
    );
    assert!(
        (speed - 0.0).abs() < SPEED_TOLERANCE,
        "the plate motor stays at 0 during the dwell, speed {speed}"
    );
}

#[test]
fn plate_speed_eases_through_each_end_and_pauses_at_the_top() {
    // Arrange
    let rise = super::RISE_SECONDS;
    let top = super::DWELL + rise;
    let descent = top + super::TOP_DWELL;

    // Act
    let bottom = super::scheduled_plate_speed(super::DWELL * 0.5);
    let leaving_bottom = super::scheduled_plate_speed(super::DWELL + rise * 0.1);
    let mid_rise = super::scheduled_plate_speed(super::DWELL + rise * 0.5);
    let arriving_top = super::scheduled_plate_speed(top - super::SIM_DT);
    let held = super::scheduled_plate_speed(top + super::TOP_DWELL * 0.5);
    let leaving_top = super::scheduled_plate_speed(descent + rise * 0.1);
    let mid_descent = super::scheduled_plate_speed(descent + rise * 0.5);
    let arriving_bottom = super::scheduled_plate_speed(descent + rise - super::SIM_DT);

    // Assert
    assert!(
        bottom.abs() < SPEED_TOLERANCE,
        "the bottom pause is stopped"
    );
    assert!(held.abs() < SPEED_TOLERANCE, "the top pause is stopped");
    assert!(
        leaving_bottom > 0.0 && leaving_bottom < mid_rise * 0.5,
        "the rise leaves the bottom slower than the cruise, speed {leaving_bottom}"
    );
    assert!(
        (mid_rise - super::PLATE_SPEED).abs() < 1.0e-3,
        "the rise peaks at the cruise speed, speed {mid_rise}"
    );
    assert!(
        arriving_top > 0.0 && arriving_top < mid_rise * 0.5,
        "the rise arrives at the top slower than the cruise, speed {arriving_top}"
    );
    assert!(
        leaving_top < 0.0 && leaving_top.abs() < mid_descent.abs() * 0.5,
        "the descent leaves the top slower than the cruise, speed {leaving_top}"
    );
    assert!(
        (mid_descent + super::PLATE_SPEED).abs() < 1.0e-3,
        "the descent peaks at the cruise speed, speed {mid_descent}"
    );
    assert!(
        arriving_bottom < 0.0 && arriving_bottom.abs() < mid_descent.abs() * 0.5,
        "the descent arrives at the bottom slower than the cruise, speed {arriving_bottom}"
    );
}

#[test]
fn return_lifts_an_original_particle_above_the_top_tray() {
    // Arrange
    let mut session = fresh_session();
    let started_above = ids_above_the_top_tray(&session);
    let start_count = session
        .live_particle_count()
        .expect("the reservoir should start with water");

    // Act
    advance_return(&mut session);
    let end_count = session
        .live_particle_count()
        .expect("the same water should still be live");
    let cohort = session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should still be live");
        started_above
            .iter()
            .filter_map(|id| {
                view.particle_ids()
                    .iter()
                    .zip(view.positions())
                    .find(|(live, _position)| *live == id)
                    .map(|(_live, position)| *position)
            })
            .collect::<Vec<_>>()
    });
    let returned = cohort
        .iter()
        .any(|position| position.y > super::TOP_TRAY_Y && position.x < super::DIVIDER_INNER_X);
    let translation = plate_translation(&session);
    let motors_enabled = revolute_motors_enabled(&session);

    // Assert
    assert_eq!(end_count, start_count, "the live count stays unchanged");
    assert!(
        returned,
        "an original particle that started above the top tray is above it again and left of the divider, translation {translation}, cohort {cohort:?}"
    );
    assert!(
        motors_enabled.iter().all(|enabled| !enabled),
        "each tray motor stays disabled"
    );
}

#[test]
fn tray_motors_stay_off_and_the_plate_is_beside_the_stack() {
    // Arrange
    let session = fresh_session();

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
                        panic!("a revolute tray should carry a revolute definition");
                    };
                    definition.is_motor_enabled()
                })
                .collect::<Vec<_>>();
            let axis_count = observation
                .joints()
                .iter()
                .filter(|joint| joint.snapshot().kind() == JointKind::Prismatic)
                .filter(|joint| prismatic_axis_is_world_up(joint))
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
    assert_eq!(motors_enabled.len(), 3, "three revolute trays");
    assert!(
        motors_enabled.iter().all(|enabled| !enabled),
        "each tray motor stays disabled"
    );
    assert_eq!(axis_count, 1, "exactly one prismatic joint points world-up");
    assert_eq!(dynamic_positions.len(), 4, "three trays and one plate");
    assert!(
        dynamic_positions
            .iter()
            .any(|position| position.x > PLATE_MIN_X),
        "the plate sits on the shaft side of the divider"
    );
}

#[test]
fn rebuild_restores_the_reservoir() {
    // Arrange
    let mut session = fresh_session();
    advance_proof(&mut session);
    drop(session);

    // Act
    let rebuilt = fresh_session();
    let (_live_count, positions, velocities) = particle_state(&rebuilt);
    let angles = tray_angles(&rebuilt);
    let translation = plate_translation(&rebuilt);

    // Assert
    for angle in angles {
        assert_eq!(angle.to_bits(), 0.0_f32.to_bits());
    }
    assert_eq!(translation.to_bits(), 0.0_f32.to_bits());
    assert!(
        velocities.iter().copied().all(velocity_is_zero),
        "rebuild restores a still reservoir"
    );
    assert!(
        positions
            .iter()
            .all(|position| position.y > super::TOP_TRAY_Y && position.x < super::DIVIDER_INNER_X),
        "rebuild puts every particle back above the top tray"
    );
}

#[test]
fn unknown_controls_and_pointer_leave_the_drip_alone() {
    // Arrange
    let mut session = fresh_session();
    let before = session
        .live_particle_count()
        .expect("the starting water should be countable");

    // Act
    let tray = session.apply_control("tray", "1");
    let color = session.apply_control("color", "teal");
    let speed = session.apply_action("speed");
    let pointer = session.apply_pointer("up", 0.0, 1.7);
    let after = session
        .live_particle_count()
        .expect("pointer up should leave the water in place");

    // Assert
    assert_eq!(tray, Err(SessionError::UnknownControl));
    assert_eq!(color, Err(SessionError::UnknownControl));
    assert_eq!(speed, Err(SessionError::UnknownControl));
    assert_eq!(pointer, Ok(()));
    assert_eq!(after, before);
}

#[test]
fn source_lifts_with_a_prismatic_plate() {
    // Arrange
    let source = include_str!("../stacked_drip.rs");

    // Act
    let lifts = source.contains("set_prismatic_motor_speed");
    let keeps_particles = source.contains("with_destruction_by_age(false)");
    let spins_a_tray = source.contains("set_revolute_motor_speed");
    let spawns_singles = source.contains("create_particle_with_def");
    let writes_positions = source.contains("set_particle_position");
    let mixes_color = source.contains("with_color_mixing_strength");

    // Assert
    assert!(lifts, "the plate speed is written from simulation time");
    assert!(keeps_particles, "age destruction stays off");
    assert!(!spins_a_tray, "the trays are not motor-driven");
    assert!(!spawns_singles, "the cascade does not emit a jet");
    assert!(!writes_positions, "the cascade does not teleport particles");
    assert!(!mixes_color, "the drip is one plain color");
}

fn fresh_session() -> SessionCore {
    SessionCore::create(SceneId::StackedDrip).expect("stacked drip should construct")
}

fn advance_proof(session: &mut SessionCore) {
    for _ in 0..super::PROOF_BATCHES {
        session
            .advance(4)
            .expect("the cascade proof window should step");
    }
}

fn advance_return(session: &mut SessionCore) {
    let seconds = super::DWELL + super::RISE_SECONDS + super::SPILL_SAMPLE_SECONDS;
    let steps = (seconds / super::SIM_DT).ceil() as usize;
    let batches = steps.div_ceil(4);
    for _ in 0..batches {
        session.advance(4).expect("the return window should step");
    }
}

fn revolute_motors_enabled(session: &SessionCore) -> Vec<bool> {
    session.read_particles(|world, _system| {
        let observation = world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the tray joints");
        observation
            .joints()
            .iter()
            .filter(|joint| joint.snapshot().kind() == JointKind::Revolute)
            .map(|joint| {
                let JointDef::Revolute(definition) = joint.snapshot().definition() else {
                    panic!("a revolute tray should carry a revolute definition");
                };
                definition.is_motor_enabled()
            })
            .collect()
    })
}

fn particle_state(session: &SessionCore) -> (usize, Vec<Vec2>, Vec<Vec2>) {
    session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should be live");
        (
            view.particle_ids().len(),
            view.positions().to_vec(),
            view.velocities().to_vec(),
        )
    })
}

fn ids_above_the_top_tray(session: &SessionCore) -> Vec<liquidfun::ParticleId> {
    session.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("the water system should be live");
        view.particle_ids()
            .iter()
            .copied()
            .zip(view.positions().iter().copied())
            .filter(|(_id, position)| position.y > super::TOP_TRAY_Y)
            .map(|(id, _position)| id)
            .collect()
    })
}

fn tray_angles(session: &SessionCore) -> [f32; 3] {
    session.read_particles(|world, _system| {
        let ids = revolute_ids_top_to_bottom(world);
        [
            world
                .revolute_joint_angle(ids[0])
                .expect("the upper tray angle should be readable"),
            world
                .revolute_joint_angle(ids[1])
                .expect("the middle tray angle should be readable"),
            world
                .revolute_joint_angle(ids[2])
                .expect("the lower tray angle should be readable"),
        ]
    })
}

fn revolute_ids_top_to_bottom(world: &World) -> [JointId; 3] {
    let observation = world
        .world_observation(WorldObservationLimits::reviewed())
        .expect("reviewed observation should include the tray joints");
    let mut ranked = observation
        .joints()
        .iter()
        .filter(|joint| joint.snapshot().kind() == JointKind::Revolute)
        .map(|joint| {
            let JointDef::Revolute(definition) = joint.snapshot().definition() else {
                panic!("a revolute tray should carry a revolute definition");
            };
            (definition.local_anchor_a().y, joint.id())
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.0.total_cmp(&left.0));
    assert_eq!(ranked.len(), 3, "stacked drip has three revolute trays");
    [ranked[0].1, ranked[1].1, ranked[2].1]
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

fn sole_prismatic(world: &World) -> JointId {
    let observation = world
        .world_observation(WorldObservationLimits::reviewed())
        .expect("reviewed observation should include the plate joint");
    let mut joints = observation
        .joints()
        .iter()
        .filter(|joint| joint.snapshot().kind() == JointKind::Prismatic);
    let joint = joints.next().expect("the plate rides one prismatic joint");
    assert!(
        joints.next().is_none(),
        "stacked drip has one prismatic joint"
    );
    joint.id()
}

fn prismatic_axis_is_world_up(joint: &liquidfun::JointObservation) -> bool {
    let JointDef::Prismatic(definition) = joint.snapshot().definition() else {
        return false;
    };
    definition.local_axis_a().x.to_bits() == 0.0_f32.to_bits()
        && definition.local_axis_a().y.to_bits() == 1.0_f32.to_bits()
}

fn velocity_is_zero(velocity: Vec2) -> bool {
    velocity.x.to_bits() == 0.0_f32.to_bits() && velocity.y.to_bits() == 0.0_f32.to_bits()
}

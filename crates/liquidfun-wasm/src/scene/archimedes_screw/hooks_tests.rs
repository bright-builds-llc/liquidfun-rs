//! Debug-safe Archimedes Screw hooks tests: phase math, build, vane-train
//! motion, inputs and drawing (D-01, D-02, D-11, D-14).

use liquidfun::math::Vec2;
use liquidfun::{BodyId, BodyType, NoDecisionHook, StepLimits, World, WorldObservationLimits};

use super::geometry::{
    AXIS, NORMAL, PITCH, TUBE_ORIGIN, VANE_COUNT, vane_center, visible_vane_outline, wall_quads,
    water_positions,
};
use super::test_support::step_configuration;
use super::{
    DEFAULT_SCREW_RPM, SIM_DT, ScrewParts, advance_phase, axial_speed, build, build_parts,
};
use crate::ProofFrame;
use crate::scene::{PointerKind, SceneHooks, SceneId, build_scene};
use crate::session::{SessionCore, SessionError};

/// The scene's particle budget (D-12).
const MAX_SPAWN_COUNT: usize = 3_000;
const MOVE_STEPS: u32 = 30;
const SNAP_STEPS: u32 = 150;
const POSITION_TOLERANCE: f32 = 1e-3;
const LATTICE_TOLERANCE: f32 = 1e-5;
const PHASE_TOLERANCE: f32 = 1e-6;
const MAX_FRAME_SEGMENTS: usize = 512;

/// Runs `steps` fixed steps the way the session does: `on_advance`, then `world.step`.
fn drive(parts: &mut ScrewParts, steps: u32, mut after_step: impl FnMut(&World)) {
    let step = step_configuration();
    for _ in 0..steps {
        parts
            .hooks
            .on_advance(&mut parts.world, parts.particle_system)
            .expect("the vane train advances");
        parts
            .world
            .step(step, &mut NoDecisionHook, StepLimits::default())
            .expect("the screw world steps");
        after_step(&parts.world);
    }
}

fn train_origin(world: &World, train: BodyId) -> Vec2 {
    world.body_snapshot(train).expect("live train").position()
}

#[test]
fn advance_phase_snaps_back_a_pitch_past_the_top() {
    // Arrange / Act
    let (next, snap) = advance_phase(0.9, 0.2);

    // Assert
    assert!((next - 0.1).abs() < PHASE_TOLERANCE, "{next}");
    assert_eq!(snap.to_bits(), (-PITCH).to_bits());
}

#[test]
fn advance_phase_snaps_forward_a_pitch_below_zero() {
    // Arrange / Act
    let (next, snap) = advance_phase(0.05, -0.1);

    // Assert
    assert!((next - 0.95).abs() < PHASE_TOLERANCE, "{next}");
    assert_eq!(snap.to_bits(), PITCH.to_bits());
}

#[test]
fn advance_phase_moves_without_a_snap_inside_a_pitch() {
    // Arrange / Act
    let (next, snap) = advance_phase(0.3, 0.1);

    // Assert
    assert!((next - 0.4).abs() < PHASE_TOLERANCE, "{next}");
    assert_eq!(snap.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn advance_phase_holds_still_for_a_zero_step() {
    // Arrange / Act
    let (next, snap) = advance_phase(0.3, 0.0);

    // Assert
    assert_eq!(next.to_bits(), 0.3_f32.to_bits());
    assert_eq!(snap.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn axial_speed_is_one_pitch_per_revolution() {
    // Arrange
    let expected = PITCH * f32::from(DEFAULT_SCREW_RPM) / 60.0;

    // Act
    let forward = axial_speed(DEFAULT_SCREW_RPM, false);
    let reverse = axial_speed(DEFAULT_SCREW_RPM, true);

    // Assert
    assert_eq!(forward.to_bits(), expected.to_bits());
    assert_eq!(reverse.to_bits(), (-expected).to_bits());
}

#[test]
fn axial_speed_is_zero_when_stopped() {
    // Arrange / Act
    let speeds = [axial_speed(0, false), axial_speed(0, true)];

    // Assert
    assert!(speeds.iter().all(|speed| *speed == 0.0), "{speeds:?}");
}

#[test]
fn the_scene_builds_every_spawn_point_inside_the_budget() {
    // Arrange
    let expected = water_positions().len();

    // Act
    let session = SessionCore::create(SceneId::ArchimedesScrew).expect("screw scene");

    // Assert
    assert_eq!(session.particle_count(), expected);
    assert!(expected <= MAX_SPAWN_COUNT, "{expected}");
}

#[test]
fn the_vane_train_is_the_only_kinematic_body() {
    // Arrange
    let session = SessionCore::create(SceneId::ArchimedesScrew).expect("screw scene");

    // Act
    let kinematic_count = session.read_particles(|world, _system| {
        world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation")
            .bodies()
            .iter()
            .filter(|body| body.snapshot().body_type() == BodyType::Kinematic)
            .count()
    });

    // Assert
    assert_eq!(kinematic_count, 1);
}

#[test]
fn the_scene_builds_at_gravity_ten() {
    // Arrange / Act
    let built = build(&[]).expect("screw scene");

    // Assert
    assert_eq!(built.world.gravity().x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(built.world.gravity().y.to_bits(), (-10.0_f32).to_bits());
}

#[test]
fn non_empty_presets_are_rejected_and_gravity_is_stripped() {
    // Arrange
    let unknown = [("x".to_owned(), "1".to_owned())];
    let gravity = [("gravity".to_owned(), "10".to_owned())];

    // Act
    let rejected = build(&unknown);
    let stripped = build_scene(SceneId::ArchimedesScrew, &gravity);

    // Assert
    assert!(matches!(rejected, Err(SessionError::UnknownControl)));
    assert!(stripped.is_ok(), "gravity is stripped before the builder");
}

#[test]
fn the_train_moves_one_pitch_per_revolution_along_the_axis() {
    // Arrange
    let mut parts = build_parts().expect("screw parts");
    let expected = f32::from(u16::try_from(MOVE_STEPS).expect("small step count"))
        * SIM_DT
        * axial_speed(DEFAULT_SCREW_RPM, false);

    // Act
    drive(&mut parts, MOVE_STEPS, |_world| {});

    // Assert
    let travelled = (train_origin(&parts.world, parts.hooks.train) - TUBE_ORIGIN).dot(AXIS);
    let error = (travelled - expected).rem_euclid(PITCH);
    let error = error.min(PITCH - error);
    assert!(error < POSITION_TOLERANCE, "{travelled} vs {expected}");
}

#[test]
fn speed_zero_holds_the_vanes_still() {
    // Arrange
    let mut parts = build_parts().expect("screw parts");
    parts.hooks.rpm = 0;
    let before = train_origin(&parts.world, parts.hooks.train);

    // Act
    drive(&mut parts, MOVE_STEPS, |_world| {});

    // Assert
    let after = train_origin(&parts.world, parts.hooks.train);
    assert_eq!(after.x.to_bits(), before.x.to_bits());
    assert_eq!(after.y.to_bits(), before.y.to_bits());
}

#[test]
fn the_train_stays_within_one_pitch_across_snaps() {
    // Arrange
    let mut parts = build_parts().expect("screw parts");
    let train = parts.hooks.train;
    let mut offsets = Vec::new();

    // Act
    drive(&mut parts, SNAP_STEPS, |world| {
        let offset = train_origin(world, train) - TUBE_ORIGIN;
        offsets.push((offset.dot(AXIS), offset.dot(NORMAL)));
    });

    // Assert
    let outside: Vec<&(f32, f32)> = offsets
        .iter()
        .filter(|(along, across)| {
            *along < -LATTICE_TOLERANCE
                || *along > PITCH + LATTICE_TOLERANCE
                || across.abs() > LATTICE_TOLERANCE
        })
        .collect();
    assert!(outside.is_empty(), "{outside:?}");
}

#[test]
fn controls_and_actions_fail_closed() {
    // Arrange
    let mut parts = build_parts().expect("screw parts");
    let system = parts.particle_system;

    // Act
    let control = parts
        .hooks
        .apply_control(&mut parts.world, system, "screw-speed", "30");
    let action = parts
        .hooks
        .apply_action(&mut parts.world, system, "anything");

    // Assert
    assert!(matches!(control, Err(SessionError::UnknownControl)));
    assert!(matches!(action, Err(SessionError::UnknownControl)));
}

#[test]
fn pointer_input_is_accepted_and_changes_nothing() {
    // Arrange
    let mut parts = build_parts().expect("screw parts");
    let system = parts.particle_system;
    let before = train_origin(&parts.world, parts.hooks.train);
    let kinds = [
        PointerKind::Down,
        PointerKind::Move,
        PointerKind::Up,
        PointerKind::Cancel,
    ];

    // Act
    let results: Vec<_> = kinds
        .into_iter()
        .map(|kind| {
            parts
                .hooks
                .apply_pointer(&mut parts.world, system, kind, -3.0, 0.5)
        })
        .collect();

    // Assert
    assert!(results.iter().all(Result::is_ok));
    let after = train_origin(&parts.world, parts.hooks.train);
    assert_eq!(after.x.to_bits(), before.x.to_bits());
    assert_eq!(after.y.to_bits(), before.y.to_bits());
    assert_eq!(parts.hooks.phase.to_bits(), 0.0_f32.to_bits());
    assert_eq!(parts.hooks.rpm, DEFAULT_SCREW_RPM);
    assert!(!parts.hooks.reverse);
}

#[test]
fn the_frame_draws_walls_and_visible_vanes_only() {
    // Arrange
    let session = SessionCore::create(SceneId::ArchimedesScrew).expect("screw scene");
    let vane_edges: usize = (0..VANE_COUNT)
        .filter_map(|index| visible_vane_outline(vane_center(index, 0.0)))
        .map(|outline| outline.len())
        .sum();

    // Act
    let frame = ProofFrame::from(session.capture_frame().expect("frame"));

    // Assert
    let segment_count = frame.rigid_segments().len() / 4;
    assert_eq!(segment_count, wall_quads().len() * 4 + vane_edges);
    assert!(segment_count <= MAX_FRAME_SEGMENTS, "{segment_count}");
    assert!(frame.rigid_circles().is_empty());
}

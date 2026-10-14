//! Debug-safe Archimedes Screw tests: geometry invariants (D-03..D-08, D-12).

use std::f32::consts::TAU;

use liquidfun::collision::PolygonShape;
use liquidfun::math::{Transform, Vec2};

use liquidfun::{BodyId, BodyType, NoDecisionHook, StepLimits, World, WorldObservationLimits};

use super::geometry::{
    AXIS, FRAME_MAX_X, FRAME_MAX_Y, FRAME_MIN_X, FRAME_MIN_Y, HOUSING_LENGTH, INLET_LENGTH,
    INLET_SEAL_Y, NORMAL, OUTLET_LENGTH, PARTICLE_DIAMETER, PARTICLE_RADIUS, PITCH, SLAB_THICKNESS,
    TUBE_GAP, TUBE_LENGTH, TUBE_ORIGIN, VANE_COS, VANE_COUNT, VANE_EMBED, VANE_HALF_SPAN,
    VANE_OVERHANG, VANE_TAN, VANE_THICKNESS, WEIR_LIP_Y, bottom_housing, distance_to_boundary,
    floor_slab, in_basin, in_reservoir, quad_contains, top_housing, tube_point, vane_center,
    vane_quads, visible_vane_outline, wall_quads, water_positions,
};

use super::test_support::step_configuration;
use super::{
    DEFAULT_SCREW_RPM, SIM_DT, ScrewParts, advance_phase, axial_speed, build, build_parts,
};
use crate::ProofFrame;
use crate::scene::{PointerKind, SceneHooks, SceneId, build_scene};
use crate::session::{SessionCore, SessionError};

const LITERAL_TOLERANCE: f32 = 1e-6;
const FRAME_MARGIN: f32 = 0.1;
const MIN_SPAWN_COUNT: usize = 1_000;
const MAX_SPAWN_COUNT: usize = 3_000;

/// World point at vane coordinate `q` and height `t`.
fn vane_point(q: f32, t: f32) -> Vec2 {
    tube_point(q + (t - TUBE_GAP / 2.0) * VANE_TAN, t)
}

fn polygon(quad: &[Vec2; 4]) -> PolygonShape {
    PolygonShape::new(quad).expect("geometry quads are convex")
}

fn vane_at_phase(index: u8, phase: f32) -> [Vec2; 4] {
    vane_quads()[usize::from(index)].map(|corner| corner + TUBE_ORIGIN + AXIS * phase)
}

#[test]
fn tube_axis_literals_match_a_thirty_degree_tilt() {
    // Arrange
    let angle = TAU / 12.0;

    // Act
    let errors = [
        AXIS.x - angle.cos(),
        AXIS.y - angle.sin(),
        NORMAL.x + angle.sin(),
        NORMAL.y - angle.cos(),
    ];

    // Assert
    assert!(
        errors.iter().all(|error| error.abs() < LITERAL_TOLERANCE),
        "{errors:?}"
    );
}

#[test]
fn vane_slant_literals_match_twenty_degrees() {
    // Arrange
    let angle = TAU / 18.0;

    // Act
    let errors = [VANE_TAN - angle.tan(), VANE_COS - angle.cos()];

    // Assert
    assert!(
        errors.iter().all(|error| error.abs() < LITERAL_TOLERANCE),
        "{errors:?}"
    );
}

#[test]
fn the_vane_added_by_a_forward_snap_lies_inside_the_bottom_housing() {
    // Arrange
    let housing = polygon(&bottom_housing());

    // Act
    let vane = vane_at_phase(0, 0.0);

    // Assert
    for corner in vane {
        assert!(
            housing
                .test_point(Transform::IDENTITY, corner)
                .expect("finite corner"),
            "vane 0 corner {corner:?} is outside the bottom housing"
        );
    }
}

#[test]
fn the_vane_removed_by_a_forward_snap_lies_inside_the_top_housing() {
    // Arrange
    let housing = polygon(&top_housing());

    // Act
    let vane = vane_at_phase(VANE_COUNT - 1, PITCH - 1e-4);

    // Assert
    for corner in vane {
        assert!(
            housing
                .test_point(Transform::IDENTITY, corner)
                .expect("finite corner"),
            "last vane corner {corner:?} is outside the top housing"
        );
    }
}

#[test]
fn openings_are_longer_than_a_pitch_plus_the_vane_overhang() {
    // Arrange
    let minimum = PITCH + VANE_OVERHANG;

    // Act
    let lengths = [INLET_LENGTH, OUTLET_LENGTH];

    // Assert
    assert!(
        lengths.iter().all(|length| *length >= minimum),
        "{lengths:?} < {minimum}"
    );
}

#[test]
fn openings_run_up_to_the_housing_faces() {
    // Arrange
    let slab = floor_slab();
    let middle_t = -SLAB_THICKNESS / 2.0;
    let samples: Vec<f32> = (0..=140_u8).map(|step| f32::from(step) * 0.01).collect();

    // Act
    let floor_in_openings: Vec<f32> = samples
        .iter()
        .flat_map(|offset| [*offset, TUBE_LENGTH - *offset])
        .filter(|q| {
            let inside_inlet = *q < INLET_LENGTH;
            let inside_outlet = *q > TUBE_LENGTH - OUTLET_LENGTH;
            (inside_inlet || inside_outlet) && quad_contains(&slab, vane_point(*q, middle_t))
        })
        .collect();

    // Assert
    assert!(floor_in_openings.is_empty(), "{floor_in_openings:?}");
}

#[test]
fn vanes_are_thick_and_embedded_within_the_slabs() {
    // Arrange / Act
    let thick_enough = VANE_THICKNESS >= 3.0 * PARTICLE_DIAMETER;
    let embed_fits = VANE_EMBED <= SLAB_THICKNESS && VANE_EMBED >= 0.5 * PARTICLE_DIAMETER;

    // Assert
    assert!(thick_enough, "vane thickness {VANE_THICKNESS}");
    assert!(embed_fits, "vane embed {VANE_EMBED}");
}

#[test]
fn tube_gap_is_six_to_eight_particle_diameters() {
    // Arrange / Act
    let diameters = TUBE_GAP / PARTICLE_DIAMETER;

    // Assert
    assert!((6.0..=8.0).contains(&diameters), "{diameters} diameters");
}

#[test]
fn housings_hold_a_whole_vane_with_margin() {
    // Arrange / Act
    let vane_span = 2.0 * VANE_HALF_SPAN;

    // Assert
    assert!(HOUSING_LENGTH > vane_span, "{vane_span}");
}

#[test]
fn weir_lip_submerges_the_outlet_below_the_top_housing() {
    // Arrange
    let outlet_floor_y = vane_point(TUBE_LENGTH - OUTLET_LENGTH, 0.0).y;
    let housing_low_y = top_housing()
        .iter()
        .map(|corner| corner.y)
        .fold(f32::INFINITY, f32::min);

    // Act
    let between = WEIR_LIP_Y > outlet_floor_y && WEIR_LIP_Y < housing_low_y;

    // Assert
    assert!(
        between,
        "lip {WEIR_LIP_Y}, outlet floor {outlet_floor_y}, housing {housing_low_y}"
    );
}

#[test]
fn every_wall_vertex_lies_inside_the_frame() {
    // Arrange
    let quads = wall_quads();

    // Act
    let outside: Vec<Vec2> = quads
        .iter()
        .flatten()
        .copied()
        .filter(|corner| {
            corner.x < FRAME_MIN_X + FRAME_MARGIN
                || corner.x > FRAME_MAX_X - FRAME_MARGIN
                || corner.y < FRAME_MIN_Y + FRAME_MARGIN
                || corner.y > FRAME_MAX_Y - FRAME_MARGIN
        })
        .collect();

    // Assert
    assert!(outside.is_empty(), "{outside:?}");
}

#[test]
fn water_count_stays_inside_the_particle_budget() {
    // Arrange / Act
    let count = water_positions().len();

    // Assert
    assert!(
        (MIN_SPAWN_COUNT..=MAX_SPAWN_COUNT).contains(&count),
        "{count} particles"
    );
}

#[test]
fn water_spawns_clear_of_every_wall_and_vane() {
    // Arrange
    let mut quads: Vec<[Vec2; 4]> = wall_quads().to_vec();
    quads.extend((0..VANE_COUNT).map(|index| vane_at_phase(index, 0.0)));

    // Act
    let overlapping: Vec<Vec2> = water_positions()
        .into_iter()
        .filter(|point| {
            quads.iter().any(|quad| {
                quad_contains(quad, *point) || distance_to_boundary(quad, *point) < PARTICLE_RADIUS
            })
        })
        .collect();

    // Assert
    assert!(overlapping.is_empty(), "{overlapping:?}");
}

#[test]
fn the_basin_starts_empty() {
    // Arrange / Act
    let in_basin_count = water_positions()
        .into_iter()
        .filter(|point| in_basin(*point))
        .count();

    // Assert
    assert_eq!(in_basin_count, 0);
}

#[test]
fn the_reservoir_fills_to_at_least_the_inlet_seal() {
    // Arrange / Act
    let top = water_positions()
        .into_iter()
        .filter(|point| in_reservoir(*point))
        .map(|point| point.y)
        .fold(f32::NEG_INFINITY, f32::max);

    // Assert
    assert!(top >= INLET_SEAL_Y, "reservoir top {top} < {INLET_SEAL_Y}");
}

#[test]
fn a_vane_inside_a_housing_has_no_outline() {
    // Arrange
    let q_center = -HOUSING_LENGTH / 2.0;

    // Act
    let maybe_outline = visible_vane_outline(q_center);

    // Assert
    assert!(maybe_outline.is_none());
}

#[test]
fn a_vane_cut_by_a_housing_face_draws_a_narrower_outline() {
    // Arrange
    let q_center = 0.0;

    // Act
    let outline = visible_vane_outline(q_center).expect("half the vane is in the tube");

    // Assert
    let drawn_width = (outline[1] - outline[0]).length();
    assert!((drawn_width - VANE_HALF_SPAN).abs() < 1e-4, "{drawn_width}");
}

#[test]
fn a_mid_tube_vane_draws_its_full_outline() {
    // Arrange
    let q_center = TUBE_LENGTH / 2.0;

    // Act
    let outline = visible_vane_outline(q_center).expect("the vane is in the tube");

    // Assert
    let expected = [
        vane_point(q_center - VANE_HALF_SPAN, -VANE_EMBED),
        vane_point(q_center + VANE_HALF_SPAN, -VANE_EMBED),
        vane_point(q_center + VANE_HALF_SPAN, TUBE_GAP + VANE_EMBED),
        vane_point(q_center - VANE_HALF_SPAN, TUBE_GAP + VANE_EMBED),
    ];
    for (drawn, wanted) in outline.iter().zip(expected) {
        assert!((*drawn - wanted).length() < 1e-5, "{drawn:?} vs {wanted:?}");
    }
}

// Hooks: phase math, build, vane-train motion, inputs and drawing (D-01, D-02, D-11, D-14).

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
    let visible_vanes = (0..VANE_COUNT)
        .filter(|index| visible_vane_outline(vane_center(*index, 0.0)).is_some())
        .count();

    // Act
    let frame = ProofFrame::from(session.capture_frame().expect("frame"));

    // Assert
    let segment_count = frame.rigid_segments().len() / 4;
    assert_eq!(segment_count, wall_quads().len() * 4 + 4 * visible_vanes);
    assert!(segment_count <= MAX_FRAME_SEGMENTS, "{segment_count}");
    assert!(frame.rigid_circles().is_empty());
}

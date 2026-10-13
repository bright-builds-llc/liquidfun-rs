//! Valve schedule, gate lifecycle and step-order tests (Phase 40 D-01, D-03,
//! D-08).

use liquidfun::math::Vec2;
use liquidfun::{BodyDef, World};

use super::{REFERENCE_VALVE_POSE, VALVE_TEST_SPEC, ValveGate, open_step};
use crate::frame::ProofFrame;
use crate::puzzle::builder::build_level;
use crate::puzzle::first_pour::REFERENCE_PLANK_POSE;
use crate::puzzle::tools::{Placements, pose_placement};
use crate::puzzle::validate::validate_for_build;
use crate::scene::{BuiltScene, RigidSegment};
use crate::scene_spot::frame_hash::frame_hash;
use crate::session::SessionCore;

/// Reference plank plus the reference valve (opens at tick 4, step 120).
fn plank_and_valve() -> Placements {
    let poses: Vec<i32> = REFERENCE_PLANK_POSE
        .into_iter()
        .chain(REFERENCE_VALVE_POSE)
        .collect();
    validate_for_build(&VALVE_TEST_SPEC, &[1, 2], &poses).expect("plank and valve are valid")
}

fn valve_session() -> SessionCore {
    SessionCore::create_puzzle_from_spec(&VALVE_TEST_SPEC, plank_and_valve())
        .expect("valve test session builds")
}

fn advance_to(core: &mut SessionCore, step: u32) {
    while core.step_index() < step {
        core.advance(1).expect("puzzle step advances");
    }
}

fn segment_lane(core: &SessionCore) -> Vec<u32> {
    let frame = ProofFrame::from(core.capture_frame().expect("puzzle frame captures"));
    frame
        .rigid_segments()
        .iter()
        .map(|value| value.to_bits())
        .collect()
}

fn body_count(core: &SessionCore) -> usize {
    core.read_particles(|world, _system| world.world_diagnostics().body_count())
}

fn particles_below(core: &SessionCore, y: f32) -> usize {
    let frame = ProofFrame::from(core.capture_frame().expect("puzzle frame captures"));
    frame
        .particle_positions()
        .chunks_exact(2)
        .filter(|position| position[1] < y)
        .count()
}

fn reference_valve_edge_bits() -> Vec<u32> {
    let [v0, v1, v2, v3] = pose_placement(2, &REFERENCE_VALVE_POSE)
        .expect("reference valve pose describes a valve")
        .vertices();
    [(v0, v1), (v1, v2), (v2, v3), (v3, v0)]
        .iter()
        .flat_map(|(start, end)| [start.x, start.y, end.x, end.y])
        .map(f32::to_bits)
        .collect()
}

fn test_gate(open_tick: u8) -> ValveGate {
    let mut world = World::new().expect("test world builds");
    let body = world
        .create_body(&BodyDef::default())
        .expect("test body builds");
    let segment = RigidSegment {
        start: Vec2::new(0.0, 0.0),
        end: Vec2::new(1.0, 0.0),
    };
    ValveGate::new(open_tick, body, [segment; 4])
}

#[test]
fn open_tick_four_opens_at_step_one_hundred_twenty() {
    // Arrange
    let open_tick = 4;

    // Act
    let step = open_step(open_tick);

    // Assert
    assert_eq!(step, 120);
}

#[test]
fn a_closed_gate_is_due_from_its_open_step() {
    // Arrange
    let gate = test_gate(4);

    // Act
    let due = [119, 120, 121].map(|next_step| gate.is_due(next_step));

    // Assert
    assert_eq!(due, [false, true, true]);
}

#[test]
fn an_opened_gate_is_never_due_and_has_no_segments() {
    // Arrange
    let mut gate = test_gate(1);

    // Act
    let maybe_body = gate.take_body();

    // Assert
    assert!(maybe_body.is_some());
    assert!(!gate.is_closed());
    assert!(!gate.is_due(30));
    assert!(gate.closed_segments().is_empty());
    assert_eq!(gate.take_body(), None);
}

#[test]
fn on_advance_destroys_the_gate_before_step_one_hundred_twenty_runs() {
    // Arrange
    let BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = build_level(&VALVE_TEST_SPEC, &plank_and_valve()).expect("valve level builds");
    let bodies = |world: &World| world.world_diagnostics().body_count();
    for _ in 0..118 {
        hooks
            .on_after_step(&mut world, particle_system, &[])
            .expect("hooks count without stepping the world");
    }
    hooks
        .on_advance(&mut world, particle_system)
        .expect("pre-step hook runs before step 119");
    let closed_bodies = bodies(&world);
    let closed_segments = hooks.collect_segments(&world).expect("segments").len();
    hooks
        .on_after_step(&mut world, particle_system, &[])
        .expect("hooks count step 119");

    // Act
    hooks
        .on_advance(&mut world, particle_system)
        .expect("pre-step hook runs before step 120");

    // Assert
    let open_segments = hooks.collect_segments(&world).expect("segments").len();
    assert_eq!(bodies(&world), closed_bodies - 1);
    assert_eq!(open_segments, closed_segments - 4);
}

#[test]
fn the_frame_at_step_119_has_the_gate_and_the_frame_at_step_120_does_not() {
    // Arrange
    let mut core = valve_session();
    advance_to(&mut core, 119);
    let (closed_lane, closed_bodies) = (segment_lane(&core), body_count(&core));

    // Act
    core.advance(1).expect("step 120 advances");

    // Assert
    let open_lane = segment_lane(&core);
    assert_eq!(closed_lane.len(), open_lane.len() + 4 * 4);
    assert_eq!(&closed_lane[..open_lane.len()], open_lane.as_slice());
    assert_eq!(
        &closed_lane[open_lane.len()..],
        reference_valve_edge_bits().as_slice()
    );
    assert_eq!(body_count(&core), closed_bodies - 1);
}

#[test]
fn opening_the_valve_increases_flow_through_the_outlet() {
    // Arrange
    let mut core = valve_session();
    let outlet_y = 4.2;
    advance_to(&mut core, 90);
    let before_closed = particles_below(&core, outlet_y);
    advance_to(&mut core, 120);
    let at_open = particles_below(&core, outlet_y);

    // Act
    advance_to(&mut core, 150);

    // Assert
    let after_open = particles_below(&core, outlet_y);
    let closed_flow = at_open - before_closed;
    let open_flow = after_open - at_open;
    assert!(
        open_flow > closed_flow,
        "open flow {open_flow} vs closed flow {closed_flow}"
    );
}

#[test]
fn identical_valve_placements_give_identical_frames() {
    // Arrange
    let (mut first, mut second) = (valve_session(), valve_session());

    // Act
    advance_to(&mut first, 240);
    advance_to(&mut second, 240);

    // Assert
    let hash = |core: &SessionCore| {
        frame_hash(&ProofFrame::from(
            core.capture_frame().expect("puzzle frame captures"),
        ))
    };
    assert_eq!(hash(&first), hash(&second));
}

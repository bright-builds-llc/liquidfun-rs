//! Per-level proofs: first-pour's nudge sweep, and for each Phase 40 teaching
//! level its fixture replays, release-only nudge sweep and idea-necessity
//! control (D-15, D-16, D-18). timed-gate adds its open-tick window sweep and
//! a reproducibility check (D-08, D-13). float-up's buoyancy control runs in
//! every lane (D-12). grand-finale proves its valve is needed and that its
//! open time matters (D-09).

use super::super::LevelId;
use super::super::float_up::{FLOAT, FLOAT_UP, FLOAT_UP_DENSE};
use super::super::goal::Outcome;
use super::super::grand_finale::GRAND_FINALE_WITHOUT_PAD;
use super::super::jelly_pad::{JELLY_PAD_WITHOUT_PAD, pad_centroid};
use super::super::replay::fixture::{FixtureOutcome, FixturePlacement, placement_lanes};
use super::super::replay::replay_fixture_file;
use super::super::spec::LevelSpec;
use super::super::thick_syrup::SHALLOW_ANGLE;
use super::super::timed_gate::{PERMANENT_PLANK_POSE, TIMED_GATE_WITH_PLANK};
use super::super::tools::ToolKind;
use super::super::validate::validate_for_build;
use super::{
    assert_nudges_mostly_win, fixture_text, open_tick_sweep, parsed_fixture, run_outcome,
    run_spec_outcome, timing_window_ok,
};
use crate::frame::ProofFrame;
use crate::scene_spot::frame_hash::frame_hash;
use crate::session::SessionCore;

/// Steps both timed-gate sessions run before their frames are compared (D-08).
const REPRODUCIBLE_STEPS: u32 = 360;

/// The jelly pad stays in the stream's path: its centroid moves less than
/// this many meters before the decision step (D-11).
const PAD_DRIFT_LIMIT_METERS: f32 = 0.3;

/// The float-up ball seats in the drain hole 0.44 m above its start; by the
/// win it has risen at least this far (D-12).
const FLOAT_SEATED_RISE_METERS: f32 = 0.4;

/// A dense ball stays on the well floor: it never rises this far.
const DENSE_FLOAT_MAX_RISE_METERS: f32 = 0.1;

/// Debug runs are too slow for the sweep; run it with `--release`.
#[test]
#[cfg_attr(debug_assertions, ignore = "release-only nudge sweep (D-16)")]
fn first_pour_nudges_mostly_win() {
    assert_nudges_mostly_win("first-pour-solution.txt");
}

#[test]
fn thick_syrup_solution_fixture_replays() {
    replay_fixture_file(
        "thick-syrup-solution.txt",
        fixture_text("thick-syrup-solution.txt"),
    );
}

#[test]
fn thick_syrup_empty_fixture_replays() {
    replay_fixture_file(
        "thick-syrup-empty.txt",
        fixture_text("thick-syrup-empty.txt"),
    );
}

#[test]
fn jelly_pad_solution_fixture_replays() {
    replay_fixture_file(
        "jelly-pad-solution.txt",
        fixture_text("jelly-pad-solution.txt"),
    );
}

#[test]
fn jelly_pad_empty_fixture_replays() {
    replay_fixture_file("jelly-pad-empty.txt", fixture_text("jelly-pad-empty.txt"));
}

/// Debug runs are too slow for the sweep; run it with `--release`.
#[test]
#[cfg_attr(debug_assertions, ignore = "release-only nudge sweep (D-16)")]
fn thick_syrup_nudges_mostly_win() {
    assert_nudges_mostly_win("thick-syrup-solution.txt");
}

/// Debug runs are too slow for the sweep; run it with `--release`.
#[test]
#[cfg_attr(debug_assertions, ignore = "release-only nudge sweep (D-16)")]
fn jelly_pad_nudges_mostly_win() {
    assert_nudges_mostly_win("jelly-pad-solution.txt");
}

/// D-18 control: the thick-syrup reference planks re-angled to a shallow
/// -15° slope are still valid but do not win, so the steep path matters.
#[test]
#[cfg_attr(debug_assertions, ignore = "release-only idea control")]
fn thick_syrup_needs_a_steep_path() {
    // Arrange
    let reference = parsed_fixture("thick-syrup-solution.txt").placements;
    let shallow: Vec<FixturePlacement> = reference
        .iter()
        .map(|&placement| FixturePlacement {
            angle: SHALLOW_ANGLE,
            ..placement
        })
        .collect();

    // Act
    let maybe_outcome = run_outcome(LevelId::ThickSyrup, &shallow);

    // Assert
    assert_eq!(maybe_outcome, Some(FixtureOutcome::Lost));
}

/// D-18 control: the jelly-pad reference without the pad does not win, and
/// with the pad the pad stays in the stream's path through the decision step.
#[test]
#[cfg_attr(debug_assertions, ignore = "release-only idea control")]
fn jelly_pad_needs_the_pad() {
    // Arrange
    let fixture = parsed_fixture("jelly-pad-solution.txt");
    let decision_step = fixture
        .maybe_expectation
        .and_then(|expected| expected.maybe_brim_step)
        .expect("the jelly-pad solution records a Brim decision step");
    let (kinds, poses) = placement_lanes(&fixture.placements);
    let mut core =
        crate::build_puzzle_core(LevelId::JellyPad.as_str(), &kinds, &poses).expect("builds");
    let start = pad_centroid(&core);

    // Act
    let without_pad = run_spec_outcome(&JELLY_PAD_WITHOUT_PAD, &fixture.placements);
    let mut max_drift: f32 = 0.0;
    while core.step_index() < decision_step {
        core.advance(1).expect("level steps");
        max_drift = max_drift.max((pad_centroid(&core) - start).length());
    }

    // Assert
    assert_eq!(without_pad, Some(FixtureOutcome::Lost));
    assert!(
        max_drift < PAD_DRIFT_LIMIT_METERS,
        "pad centroid moved {max_drift} m by step {decision_step}"
    );
}

#[test]
fn float_up_solution_fixture_replays() {
    replay_fixture_file(
        "float-up-solution.txt",
        fixture_text("float-up-solution.txt"),
    );
}

#[test]
fn float_up_empty_fixture_replays() {
    replay_fixture_file("float-up-empty.txt", fixture_text("float-up-empty.txt"));
}

/// Debug runs are too slow for the sweep; run it with `--release`.
#[test]
#[cfg_attr(debug_assertions, ignore = "release-only nudge sweep (D-16)")]
fn float_up_nudges_mostly_win() {
    assert_nudges_mostly_win("float-up-solution.txt");
}

/// Height of float-up's one float, from the frame circles.
fn float_y(core: &SessionCore) -> f32 {
    let frame = ProofFrame::from(core.capture_frame().expect("puzzle frame captures"));
    let circles = frame.rigid_circles();
    assert_eq!(circles.len(), 3, "float-up has one float circle");
    circles[1]
}

/// Steps `spec` with `placements` to `last_step` and returns the float's
/// highest rise above its start, with the outcome at that step.
fn float_run(
    spec: &'static LevelSpec,
    placements: &[FixturePlacement],
    last_step: u32,
) -> (f32, Outcome) {
    let (kinds, poses) = placement_lanes(placements);
    let validated =
        validate_for_build(spec, &kinds, &poses).expect("the reference placements are valid");
    let mut core = SessionCore::create_puzzle_from_spec(spec, validated).expect("float-up builds");
    let mut max_rise: f32 = 0.0;
    while core.step_index() < last_step {
        core.advance(1).expect("level steps");
        max_rise = max_rise.max(float_y(&core) - FLOAT.center.y);
    }
    let outcome = core
        .maybe_puzzle_status()
        .expect("puzzle sessions report status")
        .outcome;
    (max_rise, outcome)
}

/// D-12 control: the recorded reference with a density-2.0 ball loses,
/// because the ball stays on the well floor and the drain stays open. With
/// the level's density-0.4 ball, the ball has risen into the drain hole by
/// the recorded win.
#[test]
fn float_up_needs_buoyancy() {
    // Arrange
    let fixture = parsed_fixture("float-up-solution.txt");
    let win_step = fixture
        .maybe_expectation
        .map(|expected| expected.outcome_step)
        .expect("the float-up solution records its outcome");

    // Act
    let (light_rise, _) = float_run(&FLOAT_UP, &fixture.placements, win_step);
    let (dense_rise, dense_outcome) =
        float_run(&FLOAT_UP_DENSE, &fixture.placements, FLOAT_UP.step_limit);

    // Assert
    assert!(
        matches!(dense_outcome, Outcome::Lost { .. }),
        "dense float: {dense_outcome:?}"
    );
    assert!(
        light_rise >= FLOAT_SEATED_RISE_METERS,
        "the ball rose only {light_rise} m by step {win_step}"
    );
    assert!(
        dense_rise < DENSE_FLOAT_MAX_RISE_METERS,
        "the dense ball rose {dense_rise} m"
    );
}

#[test]
fn timed_gate_solution_fixture_replays() {
    replay_fixture_file(
        "timed-gate-solution.txt",
        fixture_text("timed-gate-solution.txt"),
    );
}

#[test]
fn timed_gate_empty_fixture_replays() {
    replay_fixture_file("timed-gate-empty.txt", fixture_text("timed-gate-empty.txt"));
}

#[test]
fn timed_gate_early_fixture_replays() {
    replay_fixture_file("timed-gate-early.txt", fixture_text("timed-gate-early.txt"));
}

#[test]
fn timed_gate_late_fixture_replays() {
    replay_fixture_file("timed-gate-late.txt", fixture_text("timed-gate-late.txt"));
}

/// Debug runs are too slow for the sweep; run it with `--release`.
#[test]
#[cfg_attr(debug_assertions, ignore = "release-only nudge sweep (D-16)")]
fn timed_gate_nudges_mostly_win() {
    assert_nudges_mostly_win("timed-gate-solution.txt");
}

/// D-13: with the reference valve opening at every tick `1..=24`, the first
/// and last ticks lose and the winning ticks form one band of at least two
/// that contains the reference tick. `SWEEP` lines record the table.
#[test]
#[cfg_attr(debug_assertions, ignore = "release-only open-tick sweep (D-13)")]
fn timed_gate_open_time_decides_the_outcome() {
    // Arrange
    let fixture = parsed_fixture("timed-gate-solution.txt");
    let valve_index = fixture
        .placements
        .iter()
        .position(|placement| placement.kind == ToolKind::Valve)
        .expect("the timed-gate solution places a valve");
    let reference_tick = usize::try_from(fixture.placements[valve_index].open_tick)
        .expect("the reference open tick is positive");

    // Act
    let outcomes = open_tick_sweep(LevelId::TimedGate, &fixture.placements, valve_index);

    // Assert
    for (tick, maybe_outcome) in (1..).zip(&outcomes) {
        let token = maybe_outcome.map_or("invalid", FixtureOutcome::token);
        println!("SWEEP timed-gate open_tick {tick} {token}");
    }
    assert!(
        timing_window_ok(&outcomes),
        "no timing window: {outcomes:?}"
    );
    assert_eq!(
        outcomes.get(reference_tick - 1).copied().flatten(),
        Some(FixtureOutcome::Won),
        "the reference tick {reference_tick} is outside the winning band"
    );
}

/// D-18 control: a plank that never opens, standing in the valve slot, stops
/// the jet and holds the syrup for the whole run, so blocking alone loses.
#[test]
#[cfg_attr(debug_assertions, ignore = "release-only idea control")]
fn timed_gate_needs_the_valve_to_open() {
    // Arrange
    let [x, y, angle, open_tick] = PERMANENT_PLANK_POSE;
    let plank = [FixturePlacement {
        kind: ToolKind::Plank,
        x,
        y,
        angle,
        open_tick,
    }];

    // Act
    let maybe_outcome = run_spec_outcome(&TIMED_GATE_WITH_PLANK, &plank);

    // Assert
    assert_eq!(maybe_outcome, Some(FixtureOutcome::Lost));
}

fn timed_gate_reference_core() -> SessionCore {
    let fixture = parsed_fixture("timed-gate-solution.txt");
    let (kinds, poses) = placement_lanes(&fixture.placements);
    crate::build_puzzle_core(LevelId::TimedGate.as_str(), &kinds, &poses)
        .expect("the timed-gate solution builds")
}

fn frame_fingerprint(core: &SessionCore) -> u64 {
    let frame = ProofFrame::from(core.capture_frame().expect("puzzle frame captures"));
    frame_hash(&frame)
}

fn latched_outcome(core: &mut SessionCore) -> Outcome {
    loop {
        let status = core
            .maybe_puzzle_status()
            .expect("puzzle sessions report status");
        if status.outcome != Outcome::Running {
            return status.outcome;
        }
        core.advance(1).expect("level steps");
    }
}

/// D-08: the same opening time always gives the same outcome. Two builds
/// with identical placements have equal frames after 360 steps, past the
/// reference open step, and latch the same outcome at the same step.
#[test]
fn timed_gate_is_reproducible() {
    // Arrange
    let mut first = timed_gate_reference_core();
    let mut second = timed_gate_reference_core();

    // Act
    for _ in 0..REPRODUCIBLE_STEPS {
        first.advance(1).expect("level steps");
        second.advance(1).expect("level steps");
    }
    let fingerprints = (frame_fingerprint(&first), frame_fingerprint(&second));
    let outcomes = (latched_outcome(&mut first), latched_outcome(&mut second));

    // Assert
    assert_eq!(fingerprints.0, fingerprints.1);
    assert_eq!(outcomes.0, outcomes.1);
    assert!(matches!(outcomes.0, Outcome::Won { .. }), "{outcomes:?}");
}

#[test]
fn grand_finale_solution_fixture_replays() {
    replay_fixture_file(
        "grand-finale-solution.txt",
        fixture_text("grand-finale-solution.txt"),
    );
}

#[test]
fn grand_finale_empty_fixture_replays() {
    replay_fixture_file(
        "grand-finale-empty.txt",
        fixture_text("grand-finale-empty.txt"),
    );
}

/// Debug runs are too slow for the sweep; run it with `--release`.
#[test]
#[cfg_attr(debug_assertions, ignore = "release-only nudge sweep (D-16)")]
fn grand_finale_nudges_mostly_win() {
    assert_nudges_mostly_win("grand-finale-solution.txt");
}

/// Index of the valve in a fixture's placements.
fn valve_index(placements: &[FixturePlacement]) -> usize {
    placements
        .iter()
        .position(|placement| placement.kind == ToolKind::Valve)
        .expect("the solution places a valve")
}

/// D-18 control: the reference plank without the valve loses. The jet flies
/// past the cup and the syrup alone falls short.
#[test]
#[cfg_attr(debug_assertions, ignore = "release-only idea control")]
fn grand_finale_needs_the_valve() {
    // Arrange
    let fixture = parsed_fixture("grand-finale-solution.txt");
    let mut without_valve = fixture.placements.clone();
    without_valve.remove(valve_index(&fixture.placements));

    // Act
    let maybe_outcome = run_outcome(LevelId::GrandFinale, &without_valve);

    // Assert
    assert_eq!(maybe_outcome, Some(FixtureOutcome::Lost));
}

/// D-18 control: with the reference valve opening at every tick
/// `1..=30`, at least one tick does not win, so the open time matters.
/// `SWEEP` lines record the table.
#[test]
#[cfg_attr(debug_assertions, ignore = "release-only open-tick sweep (D-09)")]
fn grand_finale_valve_time_matters() {
    // Arrange
    let fixture = parsed_fixture("grand-finale-solution.txt");
    let index = valve_index(&fixture.placements);

    // Act
    let outcomes = open_tick_sweep(LevelId::GrandFinale, &fixture.placements, index);

    // Assert
    for (tick, maybe_outcome) in (1..).zip(&outcomes) {
        let token = maybe_outcome.map_or("invalid", FixtureOutcome::token);
        println!("SWEEP grand-finale open_tick {tick} {token}");
    }
    assert!(
        outcomes
            .iter()
            .any(|maybe_outcome| *maybe_outcome != Some(FixtureOutcome::Won)),
        "every open tick wins: {outcomes:?}"
    );
}

/// D-18 control: the grand-finale reference without the jelly pad does not
/// win, because the stream drains through the cradle slots.
#[test]
#[cfg_attr(debug_assertions, ignore = "release-only idea control")]
fn grand_finale_needs_the_pad() {
    // Arrange
    let fixture = parsed_fixture("grand-finale-solution.txt");

    // Act
    let maybe_outcome = run_spec_outcome(&GRAND_FINALE_WITHOUT_PAD, &fixture.placements);

    // Assert
    assert_eq!(maybe_outcome, Some(FixtureOutcome::Lost));
}

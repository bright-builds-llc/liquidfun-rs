//! Per-level proofs: first-pour's nudge sweep, and for each Phase 40 teaching
//! level its fixture replays, release-only nudge sweep and idea-necessity
//! control (D-15, D-16, D-18).

use super::super::LevelId;
use super::super::jelly_pad::{JELLY_PAD_WITHOUT_PAD, pad_centroid};
use super::super::replay::fixture::{FixtureOutcome, FixturePlacement, placement_lanes};
use super::super::replay::replay_fixture_file;
use super::super::thick_syrup::SHALLOW_ANGLE;
use super::{
    assert_nudges_mostly_win, fixture_text, parsed_fixture, run_outcome, run_spec_outcome,
};

/// The jelly pad stays in the stream's path: its centroid moves less than
/// this many meters before the decision step (D-11).
const PAD_DRIFT_LIMIT_METERS: f32 = 0.3;

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

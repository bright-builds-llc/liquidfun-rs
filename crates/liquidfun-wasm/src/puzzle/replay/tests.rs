//! Contract and recorded-fixture replay tests (D-18 to D-22, Phase 39 D-12).
//!
//! Grammar tests live in `replay/fixture/tests.rs`.

use super::*;
use crate::puzzle::{LevelId, level_spec};

const SOLUTION_FILE: &str = "first-pour-solution.txt";
const EMPTY_FILE: &str = "first-pour-empty.txt";
const SOLUTION_TEXT: &str = include_str!("../fixtures/first-pour-solution.txt");
const EMPTY_TEXT: &str = include_str!("../fixtures/first-pour-empty.txt");

fn first_pour_info() -> PuzzleLevelInfo {
    PuzzleLevelInfo::from_spec(level_spec(LevelId::FirstPour))
}

fn fixture_expecting(expectation: FixtureExpectation) -> Fixture {
    Fixture {
        level: LevelId::FirstPour,
        placements: Vec::new(),
        maybe_expectation: Some(expectation),
    }
}

/// The expectation a trace would record, so tests can perturb one field.
fn expectation_of(trace: &ReplayTrace) -> FixtureExpectation {
    FixtureExpectation {
        outcome: trace.outcome,
        outcome_step: trace.outcome_step,
        final_count: trace.final_count,
        peak_held: trace.peak_held,
        maybe_brim_step: trace.maybe_brim_step,
    }
}

/// Counts of 0 until `from_step`, then `held` through the margin window end.
fn held_counts(from_step: u32, held: u32, info: &PuzzleLevelInfo) -> Vec<u32> {
    let window_end = info.step_limit() - MARGIN_WIN_SLACK_STEPS;
    (1..=window_end)
        .map(|step| if step < from_step { 0 } else { held })
        .collect()
}

/// A won run without Brim, decided at the step limit.
fn won_trace(outcome_step: u32, held: u32, info: &PuzzleLevelInfo) -> ReplayTrace {
    let from_step = outcome_step + 1 - info.settle_steps();
    ReplayTrace {
        outcome: FixtureOutcome::Won,
        outcome_step,
        final_count: held,
        peak_held: held,
        maybe_brim_step: None,
        decided_step: info.step_limit(),
        counts: held_counts(from_step, held, info),
    }
}

/// A won run holding `held` from the start of its settle window, which
/// reaches Brim and is decided at `brim_step`.
fn brim_trace(outcome_step: u32, held: u32, brim_step: u32, info: &PuzzleLevelInfo) -> ReplayTrace {
    ReplayTrace {
        maybe_brim_step: Some(brim_step),
        decided_step: brim_step,
        ..won_trace(outcome_step, held, info)
    }
}

fn lost_trace(counts: Vec<u32>, info: &PuzzleLevelInfo) -> ReplayTrace {
    ReplayTrace {
        outcome: FixtureOutcome::Lost,
        outcome_step: info.step_limit(),
        final_count: counts.last().copied().unwrap_or(0),
        peak_held: 0,
        maybe_brim_step: None,
        decided_step: info.step_limit(),
        counts,
    }
}

fn raised_brim(info: &PuzzleLevelInfo) -> u32 {
    info.brim_count() + fill_margin_count(info.full_count())
}

#[test]
fn exact_contract_requires_equal_step_and_count() {
    // Arrange
    let info = first_pour_info();
    let raised = info.target_count() + fill_margin_count(info.full_count());
    let trace = won_trace(201, raised, &info);
    let fixture = fixture_expecting(FixtureExpectation {
        outcome_step: 200,
        ..expectation_of(&trace)
    });

    // Act
    let results = (
        check_contract(&fixture, &trace, &info, true),
        check_contract(&fixture, &trace, &info, false),
    );

    // Assert
    assert!(results.0.is_err(), "{:?}", results.0);
    assert_eq!(results.1, Ok(()));
}

#[test]
fn exact_contract_requires_the_recorded_peak_held() {
    // Arrange
    let info = first_pour_info();
    let trace = brim_trace(201, raised_brim(&info), 262, &info);
    let fixture = fixture_expecting(FixtureExpectation {
        peak_held: trace.peak_held + 1,
        ..expectation_of(&trace)
    });

    // Act
    let results = (
        check_contract(&fixture, &trace, &info, true),
        check_contract(&fixture, &trace, &info, false),
    );

    // Assert
    assert_eq!(
        results.0,
        Err(format!(
            "peak held {} differs from recorded {}",
            trace.peak_held,
            trace.peak_held + 1
        ))
    );
    assert_eq!(results.1, Ok(()));
}

#[test]
fn exact_contract_requires_the_recorded_brim_step() {
    // Arrange
    let info = first_pour_info();
    let trace = brim_trace(201, raised_brim(&info), 262, &info);
    let fixture = fixture_expecting(FixtureExpectation {
        maybe_brim_step: Some(263),
        ..expectation_of(&trace)
    });

    // Act
    let result = check_contract(&fixture, &trace, &info, true);

    // Assert
    assert_eq!(
        result,
        Err("Brim step 262 differs from recorded 263".to_owned())
    );
}

#[test]
fn debug_contract_accepts_a_brim_step_within_the_tolerance() {
    // Arrange
    let info = first_pour_info();
    let trace = brim_trace(201, raised_brim(&info), 262, &info);
    let within = fixture_expecting(FixtureExpectation {
        maybe_brim_step: Some(262 + DEBUG_OUTCOME_STEP_TOLERANCE),
        ..expectation_of(&trace)
    });
    let beyond = fixture_expecting(FixtureExpectation {
        maybe_brim_step: Some(262 + DEBUG_OUTCOME_STEP_TOLERANCE + 1),
        ..expectation_of(&trace)
    });

    // Act
    let results = (
        check_contract(&within, &trace, &info, false),
        check_contract(&beyond, &trace, &info, false),
    );

    // Assert
    assert_eq!(results.0, Ok(()));
    assert!(results.1.is_err(), "{:?}", results.1);
}

#[test]
fn debug_contract_requires_brim_exactly_when_recorded() {
    // Arrange
    let info = first_pour_info();
    let with_brim = brim_trace(201, raised_brim(&info), 262, &info);
    let without_brim = ReplayTrace {
        maybe_brim_step: None,
        ..with_brim.clone()
    };
    let recorded_brim = fixture_expecting(expectation_of(&with_brim));
    let recorded_none = fixture_expecting(expectation_of(&without_brim));

    // Act
    let results = (
        check_contract(&recorded_brim, &without_brim, &info, false),
        check_contract(&recorded_none, &with_brim, &info, false),
    );

    // Assert
    assert_eq!(
        results.0,
        Err("Brim step none differs from recorded 262".to_owned())
    );
    assert_eq!(
        results.1,
        Err("Brim step 262 differs from recorded none".to_owned())
    );
}

#[test]
fn brim_contract_requires_a_hold_at_the_raised_brim_count() {
    // Arrange
    let info = first_pour_info();
    let short_trace = brim_trace(201, raised_brim(&info) - 1, 262, &info);
    let enough_trace = brim_trace(201, raised_brim(&info), 262, &info);
    let short = fixture_expecting(expectation_of(&short_trace));
    let enough = fixture_expecting(expectation_of(&enough_trace));

    // Act
    let results = (
        check_contract(&short, &short_trace, &info, false),
        check_contract(&enough, &enough_trace, &info, false),
    );

    // Assert
    assert!(results.0.is_err(), "{:?}", results.0);
    assert_eq!(results.1, Ok(()));
}

#[test]
fn won_contract_requires_a_win_at_the_raised_target() {
    // Arrange
    let info = first_pour_info();
    let raised = info.target_count() + fill_margin_count(info.full_count());
    let short_trace = won_trace(200, raised - 1, &info);
    let enough_trace = won_trace(200, raised, &info);
    let short = fixture_expecting(expectation_of(&short_trace));
    let enough = fixture_expecting(expectation_of(&enough_trace));

    // Act
    let short_result = check_contract(&short, &short_trace, &info, true);
    let enough_result = check_contract(&enough, &enough_trace, &info, true);

    // Assert
    assert!(short_result.is_err(), "{short_result:?}");
    assert_eq!(enough_result, Ok(()));
}

#[test]
fn lost_contract_requires_counts_at_or_below_target_minus_margin() {
    // Arrange
    let info = first_pour_info();
    let ceiling = info.target_count() - fill_margin_count(info.full_count());
    let steps = usize::try_from(info.step_limit()).expect("step limit fits usize");
    let at_ceiling = lost_trace(vec![ceiling; steps], &info);
    let mut over = vec![ceiling; steps];
    over[steps / 2] = ceiling + 1;
    let over_ceiling = lost_trace(over, &info);
    let fixture = fixture_expecting(expectation_of(&at_ceiling));

    // Act
    let results = (
        check_contract(&fixture, &at_ceiling, &info, true),
        check_contract(&fixture, &over_ceiling, &info, true),
    );

    // Assert
    assert_eq!(results.0, Ok(()));
    assert!(results.1.is_err(), "{:?}", results.1);
}

#[test]
fn outcome_mismatch_always_fails() {
    // Arrange
    let info = first_pour_info();
    let trace = lost_trace(vec![0; 10], &info);
    let fixture = fixture_expecting(FixtureExpectation {
        outcome: FixtureOutcome::Won,
        ..expectation_of(&trace)
    });

    // Act
    let results = (
        check_contract(&fixture, &trace, &info, true),
        check_contract(&fixture, &trace, &info, false),
    );

    // Assert
    assert!(results.0.is_err(), "{:?}", results.0);
    assert!(results.1.is_err(), "{:?}", results.1);
}

#[test]
fn record_line_appends_peak_held_and_brim_step() {
    // Arrange
    let info = first_pour_info();
    let trace = brim_trace(201, raised_brim(&info), 262, &info);

    // Act
    let line = record_line("solution.txt", &trace);

    // Assert
    assert!(
        line.contains(&format!(
            "final_count {} peak_held {} brim_step 262 profile ",
            trace.final_count, trace.peak_held
        )),
        "{line}"
    );
}

#[test]
fn first_pour_solution_fixture_replays_to_its_recorded_win() {
    replay_fixture_file(SOLUTION_FILE, SOLUTION_TEXT);
}

#[test]
fn first_pour_empty_fixture_replays_to_a_loss_at_the_step_limit() {
    replay_fixture_file(EMPTY_FILE, EMPTY_TEXT);
}

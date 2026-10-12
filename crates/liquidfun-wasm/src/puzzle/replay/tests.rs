//! Fixture parser, contract and recorded-fixture replay tests (D-18 to D-22).

use super::*;
use crate::puzzle::level_spec;

const FULL_WON: &str =
    "level first-pour\nplacement plank 8 64 23\noutcome won\noutcome_step 412\nfinal_count 640\n";

const SOLUTION_FILE: &str = "first-pour-solution.txt";
const EMPTY_FILE: &str = "first-pour-empty.txt";
const SOLUTION_TEXT: &str = include_str!("../fixtures/first-pour-solution.txt");
const EMPTY_TEXT: &str = include_str!("../fixtures/first-pour-empty.txt");

fn first_pour_info() -> PuzzleLevelInfo {
    PuzzleLevelInfo::from_spec(level_spec(LevelId::FirstPour))
}

fn fixture_expecting(outcome: FixtureOutcome, outcome_step: u32, final_count: u32) -> Fixture {
    Fixture {
        level: LevelId::FirstPour,
        placements: Vec::new(),
        maybe_expectation: Some(FixtureExpectation {
            outcome,
            outcome_step,
            final_count,
        }),
    }
}

/// Counts of 0 until `from_step`, then `held` through the margin window end.
fn held_counts(from_step: u32, held: u32, info: &PuzzleLevelInfo) -> Vec<u32> {
    let window_end = info.step_limit() - MARGIN_WIN_SLACK_STEPS;
    (1..=window_end)
        .map(|step| if step < from_step { 0 } else { held })
        .collect()
}

fn won_trace(outcome_step: u32, held: u32, info: &PuzzleLevelInfo) -> ReplayTrace {
    let from_step = outcome_step + 1 - info.settle_steps();
    ReplayTrace {
        outcome: FixtureOutcome::Won,
        outcome_step,
        final_count: held,
        counts: held_counts(from_step, held, info),
    }
}

fn lost_trace(counts: Vec<u32>, info: &PuzzleLevelInfo) -> ReplayTrace {
    ReplayTrace {
        outcome: FixtureOutcome::Lost,
        outcome_step: info.step_limit(),
        final_count: counts.last().copied().unwrap_or(0),
        counts,
    }
}

fn is_record_mode() -> bool {
    std::env::var_os("LIQUIDFUN_PUZZLE_RECORD").is_some_and(|value| value == "1")
}

fn replay_fixture_file(file: &str, text: &str) {
    // Arrange
    let fixture = parse_fixture(text).expect("checked-in fixture parses");
    let info = PuzzleLevelInfo::from_spec(level_spec(fixture.level));

    // Act
    let trace = replay(&fixture);
    if is_record_mode() {
        println!("{}", record_line(file, &trace));
        return;
    }

    // Assert
    let result = check_contract(&fixture, &trace, &info, !cfg!(debug_assertions));
    assert_eq!(result, Ok(()), "{file} replay: {trace:?}");
}

#[test]
fn parses_a_full_won_fixture() {
    // Arrange
    let text = FULL_WON;

    // Act
    let fixture = parse_fixture(text);

    // Assert
    assert_eq!(
        fixture,
        Ok(Fixture {
            level: LevelId::FirstPour,
            placements: vec![FixturePlacement {
                kind: ToolKind::Plank,
                x: 8,
                y: 64,
                angle: 23,
            }],
            maybe_expectation: Some(FixtureExpectation {
                outcome: FixtureOutcome::Won,
                outcome_step: 412,
                final_count: 640,
            }),
        })
    );
}

#[test]
fn parses_an_input_only_fixture() {
    // Arrange
    let text = "level first-pour\n";

    // Act
    let fixture = parse_fixture(text);

    // Assert
    assert_eq!(
        fixture,
        Ok(Fixture {
            level: LevelId::FirstPour,
            placements: Vec::new(),
            maybe_expectation: None,
        })
    );
}

#[test]
fn rejects_an_unknown_key() {
    // Arrange
    let text = "level first-pour\ncolor red\n";

    // Act
    let result = parse_fixture(text);

    // Assert
    let error = result.expect_err("unknown key is rejected");
    assert!(error.contains("line 2"), "{error}");
}

#[test]
fn rejects_expectation_lines_out_of_order() {
    // Arrange
    let text = "level first-pour\noutcome_step 412\noutcome won\nfinal_count 640\n";

    // Act
    let result = parse_fixture(text);

    // Assert
    assert!(result.is_err(), "{result:?}");
}

#[test]
fn rejects_an_unknown_level_or_tool() {
    // Arrange
    let texts = [
        "level dam-break\n",
        "level first-pour\nplacement valve 1 2 3\n",
    ];

    // Act
    let results = texts.map(parse_fixture);

    // Assert
    for result in results {
        assert!(result.is_err(), "{result:?}");
    }
}

#[test]
fn rejects_malformed_integers() {
    // Arrange
    let texts = [
        "level first-pour\nplacement plank +8 64 23\n",
        "level first-pour\nplacement plank 8.0 64 23\n",
        "level first-pour\nplacement plank x 64 23\n",
        "level first-pour\nplacement plank  64 23\n",
        "level first-pour\noutcome won\noutcome_step -1\nfinal_count 640\n",
    ];

    // Act
    let results = texts.map(parse_fixture);

    // Assert
    for result in results {
        assert!(result.is_err(), "{result:?}");
    }
}

#[test]
fn rejects_blank_lines_and_carriage_returns() {
    // Arrange
    let texts = [
        "level first-pour\n\n",
        "level first-pour\n\nplacement plank 8 64 23\n",
        "level first-pour\r\n",
        "",
    ];

    // Act
    let results = texts.map(parse_fixture);

    // Assert
    for result in results {
        assert!(result.is_err(), "{result:?}");
    }
}

#[test]
fn lanes_flatten_placements_in_fixture_order() {
    // Arrange
    let fixture =
        parse_fixture("level first-pour\nplacement plank 8 64 23\nplacement plank -3 50 0\n")
            .expect("two-placement fixture parses");

    // Act
    let (kinds, poses) = lanes(&fixture);

    // Assert
    assert_eq!((kinds, poses), (vec![1, 1], vec![8, 64, 23, -3, 50, 0]));
}

#[test]
fn exact_contract_requires_equal_step_and_count() {
    // Arrange
    let info = first_pour_info();
    let raised = info.target_count() + fill_margin_count(info.full_count());
    let fixture = fixture_expecting(FixtureOutcome::Won, 200, raised);
    let trace = won_trace(201, raised, &info);

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
fn won_contract_requires_a_win_at_the_raised_target() {
    // Arrange
    let info = first_pour_info();
    let raised = info.target_count() + fill_margin_count(info.full_count());
    let short = (
        raised - 1,
        fixture_expecting(FixtureOutcome::Won, 200, raised - 1),
    );
    let enough = (raised, fixture_expecting(FixtureOutcome::Won, 200, raised));

    // Act
    let short_result = check_contract(&short.1, &won_trace(200, short.0, &info), &info, true);
    let enough_result = check_contract(&enough.1, &won_trace(200, enough.0, &info), &info, true);

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
    let fixture = fixture_expecting(FixtureOutcome::Lost, info.step_limit(), ceiling);
    let at_ceiling = lost_trace(vec![ceiling; steps], &info);
    let mut over = vec![ceiling; steps];
    over[steps / 2] = ceiling + 1;
    let over_ceiling = lost_trace(over, &info);

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
    let fixture = fixture_expecting(FixtureOutcome::Won, info.step_limit(), 0);
    let trace = lost_trace(vec![0; 10], &info);

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
fn first_pour_solution_fixture_replays_to_its_recorded_win() {
    replay_fixture_file(SOLUTION_FILE, SOLUTION_TEXT);
}

#[test]
fn first_pour_empty_fixture_replays_to_a_loss_at_the_step_limit() {
    replay_fixture_file(EMPTY_FILE, EMPTY_TEXT);
}

//! Fixture grammar tests (Phase 37 D-18, Phase 39 D-12).
//!
//! The rejection messages are mirrored in `web/tests/puzzle-replay-stars.test.ts`.

use super::*;

const FULL_WON: &str = "level first-pour\nplacement plank 8 64 23\noutcome won\noutcome_step 412\nfinal_count 640\npeak_held 811\nbrim_step 262\n";

const EXPECTATION_PREFIX: &str =
    "level first-pour\nplacement plank 8 64 23\noutcome won\noutcome_step 412\nfinal_count 640\n";

fn with_tail(tail: &str) -> String {
    format!("{EXPECTATION_PREFIX}{tail}")
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
                open_tick: 0,
            }],
            maybe_expectation: Some(FixtureExpectation {
                outcome: FixtureOutcome::Won,
                outcome_step: 412,
                final_count: 640,
                peak_held: 811,
                maybe_brim_step: Some(262),
            }),
        })
    );
}

#[test]
fn parses_brim_step_none_as_no_brim() {
    // Arrange
    let text = "level first-pour\noutcome lost\noutcome_step 720\nfinal_count 0\npeak_held 0\nbrim_step none\n";

    // Act
    let fixture = parse_fixture(text);

    // Assert
    let expectation = fixture
        .expect("brim_step none parses")
        .maybe_expectation
        .expect("fixture has an expectation");
    assert_eq!(
        (expectation.peak_held, expectation.maybe_brim_step),
        (0, None)
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
fn rejects_a_missing_peak_held_after_final_count() {
    // Arrange
    let text = EXPECTATION_PREFIX;

    // Act
    let result = parse_fixture(text);

    // Assert
    assert_eq!(
        result,
        Err("fixture line 6: missing `peak_held` line".to_owned())
    );
}

#[test]
fn rejects_a_missing_brim_step_after_peak_held() {
    // Arrange
    let text = with_tail("peak_held 811\n");

    // Act
    let result = parse_fixture(&text);

    // Assert
    assert_eq!(
        result,
        Err("fixture line 7: missing `brim_step` line".to_owned())
    );
}

#[test]
fn rejects_brim_step_before_peak_held() {
    // Arrange
    let text = with_tail("brim_step 262\npeak_held 811\n");

    // Act
    let result = parse_fixture(&text);

    // Assert
    assert_eq!(
        result,
        Err("fixture line 6: expected `peak_held`, found `brim_step`".to_owned())
    );
}

#[test]
fn rejects_malformed_brim_steps() {
    // Arrange
    let tails = [
        "peak_held 811\nbrim_step -1\n",
        "peak_held 811\nbrim_step nope\n",
    ];

    // Act
    let results = tails.map(|tail| parse_fixture(&with_tail(tail)));

    // Assert
    assert_eq!(
        results,
        [
            Err("fixture line 7: `-1` is not a non-negative integer".to_owned()),
            Err("fixture line 7: `nope` is not a non-negative integer".to_owned()),
        ]
    );
}

#[test]
fn rejects_a_malformed_peak_held() {
    // Arrange
    let text = with_tail("peak_held none\nbrim_step 262\n");

    // Act
    let result = parse_fixture(&text);

    // Assert
    assert_eq!(
        result,
        Err("fixture line 6: `none` is not a non-negative integer".to_owned())
    );
}

#[test]
fn rejects_a_line_after_brim_step() {
    // Arrange
    let text = format!("{FULL_WON}peak_held 811\n");

    // Act
    let result = parse_fixture(&text);

    // Assert
    assert_eq!(
        result,
        Err("fixture line 8: unexpected line after brim_step".to_owned())
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
    let text = "level first-pour\noutcome_step 412\noutcome won\nfinal_count 640\npeak_held 811\nbrim_step 262\n";

    // Act
    let result = parse_fixture(text);

    // Assert
    assert!(result.is_err(), "{result:?}");
}

#[test]
fn rejects_an_unknown_level() {
    // Arrange
    let text = "level dam-break\n";

    // Act
    let result = parse_fixture(text);

    // Assert
    assert_eq!(result, Err("fixture line 1: unknown level".to_owned()));
}

#[test]
fn rejects_an_unknown_tool() {
    // Arrange
    let text = "level first-pour\nplacement gate 1 2 3\n";

    // Act
    let result = parse_fixture(text);

    // Assert
    assert_eq!(result, Err("fixture line 2: unknown tool".to_owned()));
}

#[test]
fn parses_a_valve_line_with_its_open_tick() {
    // Arrange
    let text = "level first-pour\nplacement valve -16 87 0 4\n";

    // Act
    let fixture = parse_fixture(text).expect("a 6-token valve line parses");

    // Assert
    assert_eq!(
        fixture.placements,
        vec![FixturePlacement {
            kind: ToolKind::Valve,
            x: -16,
            y: 87,
            angle: 0,
            open_tick: 4,
        }]
    );
}

#[test]
fn a_plank_line_means_open_tick_zero() {
    // Arrange
    let text = "level first-pour\nplacement plank 8 64 23\n";

    // Act
    let fixture = parse_fixture(text).expect("a 5-token plank line parses");

    // Assert
    let open_ticks: Vec<i32> = fixture
        .placements
        .iter()
        .map(|placement| placement.open_tick)
        .collect();
    assert_eq!(open_ticks, vec![0]);
}

#[test]
fn rejects_a_plank_line_with_an_open_tick() {
    // Arrange
    let text = "level first-pour\nplacement plank 8 64 23 0\n";

    // Act
    let result = parse_fixture(text);

    // Assert
    assert_eq!(
        result,
        Err("fixture line 2: expected 5 space-separated tokens".to_owned())
    );
}

#[test]
fn rejects_a_valve_line_without_an_open_tick() {
    // Arrange
    let text = "level first-pour\nplacement valve 1 2 3\n";

    // Act
    let result = parse_fixture(text);

    // Assert
    assert_eq!(
        result,
        Err("fixture line 2: expected 6 space-separated tokens".to_owned())
    );
}

#[test]
fn rejects_a_bare_placement_key() {
    // Arrange
    let text = "level first-pour\nplacement\n";

    // Act
    let result = parse_fixture(text);

    // Assert
    assert_eq!(
        result,
        Err("fixture line 2: expected 5 space-separated tokens".to_owned())
    );
}

#[test]
fn rejects_malformed_integers() {
    // Arrange
    let texts = [
        "level first-pour\nplacement plank +8 64 23\n",
        "level first-pour\nplacement plank 8.0 64 23\n",
        "level first-pour\nplacement plank x 64 23\n",
        "level first-pour\nplacement plank  64 23\n",
        "level first-pour\noutcome won\noutcome_step -1\nfinal_count 640\npeak_held 811\nbrim_step 262\n",
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
    assert_eq!(
        (kinds, poses),
        (vec![1, 1], vec![8, 64, 23, 0, -3, 50, 0, 0])
    );
}

#[test]
fn lanes_carry_the_valve_open_tick() {
    // Arrange
    let fixture =
        parse_fixture("level first-pour\nplacement plank 8 64 23\nplacement valve -16 87 0 4\n")
            .expect("plank and valve fixture parses");

    // Act
    let (kinds, poses) = lanes(&fixture);

    // Assert
    assert_eq!(
        (kinds, poses),
        (vec![1, 2], vec![8, 64, 23, 0, -16, 87, 0, 4])
    );
}

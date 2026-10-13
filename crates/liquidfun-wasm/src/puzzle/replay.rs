//! Test-only reference-solution replay (Phase 37 D-18, D-19): fixtures replay
//! through `build_puzzle_core` and `ProofFrame` getters, the same surface the
//! Bun wasm replay uses.
//!
//! The fixture grammar lives in `replay/fixture.rs` and is shared
//! byte-for-byte with `web/scripts/puzzle-replay/fixture.ts`, so both
//! runtimes accept exactly the same text. Phase 39 D-12 adds the peak held
//! count and the Brim step at the decision step.

pub(in crate::puzzle) mod fixture;

use fixture::{Fixture, FixtureExpectation, FixtureOutcome, NO_BRIM_TOKEN, lanes, parse_fixture};

use crate::frame::ProofFrame;
use crate::puzzle::goal::{MARGIN_WIN_SLACK_STEPS, fill_margin_count, maybe_margin_win_step};
use crate::puzzle::{PuzzleLevelInfo, level_spec};
use crate::session::SessionCore;

/// What one fixture run produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ReplayTrace {
    pub(super) outcome: FixtureOutcome,
    pub(super) outcome_step: u32,
    pub(super) final_count: u32,
    /// Frame peak held count at `decided_step` (D-12).
    pub(super) peak_held: u32,
    /// Frame Brim step at `decided_step` (D-12).
    pub(super) maybe_brim_step: Option<u32>,
    /// The D-10 decision step, read from the frames.
    pub(super) decided_step: u32,
    /// `counts[i]` is the goal count after step `i + 1`, through the latch and
    /// on to the later of `decided_step` and `step_limit - MARGIN_WIN_SLACK_STEPS`.
    pub(super) counts: Vec<u32>,
}

/// Steps run after the latch and the margin window: two `advance(4)` calls on
/// the same session.
pub(super) const POST_LATCH_STEPS: u32 = 8;

const POST_LATCH_ADVANCE: u32 = 4;

/// Debug-profile outcome-step tolerance (D-22): `max(5, 2 * spread)`.
///
/// Measured on first-pour (Phase 37 Plan 08): the reference solution latched
/// Won at step 201 in both the release and the debug native replay, so the
/// spread is 0 and the floor of 5 applies. Phase 39 D-12 applies the same
/// tolerance to the Brim step.
pub(super) const DEBUG_OUTCOME_STEP_TOLERANCE: u32 = 5;

/// Puzzle scalars read from one checked frame.
struct FrameScalars {
    goal_count: u32,
    outcome: u8,
    maybe_outcome_step: Option<u32>,
    peak_held_count: u32,
    maybe_brim_step: Option<u32>,
    maybe_decided_step: Option<u32>,
}

/// Peak held count and Brim step read from the frame at the decision step.
#[derive(Debug, Clone, Copy)]
struct Decision {
    step: u32,
    peak_held: u32,
    maybe_brim_step: Option<u32>,
}

fn capture(core: &SessionCore, info: &PuzzleLevelInfo) -> FrameScalars {
    let frame = ProofFrame::from(core.capture_frame().expect("puzzle frames capture"));
    assert_eq!(
        frame.step_index(),
        core.step_index(),
        "frame step index must equal the session step index"
    );
    assert_eq!(
        frame.goal_target(),
        Some(info.target_count()),
        "goal target"
    );
    assert_eq!(frame.goal_full(), Some(info.full_count()), "goal full");
    assert_eq!(frame.step_limit(), Some(info.step_limit()), "step limit");
    assert_eq!(frame.brim_count(), Some(info.brim_count()), "brim count");
    let goal_count = frame
        .goal_count()
        .expect("puzzle frames report a goal count");
    let outcome = frame.outcome().expect("puzzle frames report an outcome");
    let maybe_outcome_step = frame.outcome_step();
    assert_eq!(
        maybe_outcome_step.is_some(),
        outcome != 0,
        "outcome step is present exactly when the outcome is latched"
    );
    FrameScalars {
        goal_count,
        outcome,
        maybe_outcome_step,
        peak_held_count: frame
            .peak_held_count()
            .expect("puzzle frames report a peak held count"),
        maybe_brim_step: frame.brim_step(),
        maybe_decided_step: frame.decided_step(),
    }
}

fn assert_latch_unchanged(scalars: &FrameScalars, outcome: u8, outcome_step: u32, step: u32) {
    assert_eq!(
        (scalars.outcome, scalars.maybe_outcome_step),
        (outcome, Some(outcome_step)),
        "latched outcome changed at step {step}"
    );
}

/// Records the decision from the frame whose step is the decided step, and
/// requires the decided step never to change once present.
fn observe_decision(
    maybe_decision: Option<Decision>,
    scalars: &FrameScalars,
    step: u32,
) -> Option<Decision> {
    if let Some(decision) = maybe_decision {
        assert_eq!(
            scalars.maybe_decided_step,
            Some(decision.step),
            "decided step changed at step {step}"
        );
        return Some(decision);
    }
    let decided = scalars.maybe_decided_step?;
    assert_eq!(decided, step, "decided step first appears on its own frame");
    Some(Decision {
        step,
        peak_held: scalars.peak_held_count,
        maybe_brim_step: scalars.maybe_brim_step,
    })
}

/// Replays a fixture through the wasm-facing helper and frame getters.
///
/// Steps with `advance(1)` until the outcome latches, keeps recording counts
/// on the same session until both the decision step was observed and
/// `step_limit - MARGIN_WIN_SLACK_STEPS` was reached, then runs
/// `POST_LATCH_STEPS` more steps, requiring the latch and the decision to
/// hold on every frame.
///
/// # Panics
///
/// Panics (test-only harness) when the fixture does not build, a puzzle
/// scalar is missing or inconsistent, the run passes the step limit without
/// latching or deciding, or the latched outcome or decision changes.
pub(super) fn replay(fixture: &Fixture) -> ReplayTrace {
    let (kinds, poses) = lanes(fixture);
    let mut core = crate::build_puzzle_core(fixture.level.as_str(), &kinds, &poses)
        .expect("fixture level and placements build");
    let info = core
        .level_info()
        .expect("puzzle sessions report level info");
    let mut counts = Vec::new();
    let mut maybe_decision = None;
    let (outcome, outcome_step, final_count) = loop {
        core.advance(1).expect("advance(1) succeeds");
        let scalars = capture(&core, &info);
        counts.push(scalars.goal_count);
        maybe_decision = observe_decision(maybe_decision, &scalars, core.step_index());
        if let Some(outcome_step) = scalars.maybe_outcome_step {
            break (scalars.outcome, outcome_step, scalars.goal_count);
        }
        assert!(
            core.step_index() <= info.step_limit(),
            "replay ran past step limit {} without latching",
            info.step_limit()
        );
    };
    let window_end = info
        .step_limit()
        .checked_sub(MARGIN_WIN_SLACK_STEPS)
        .expect("step limit covers the margin slack");
    while maybe_decision.is_none() || core.step_index() < window_end {
        assert!(
            maybe_decision.is_some() || core.step_index() < info.step_limit(),
            "replay reached step limit {} without a decision",
            info.step_limit()
        );
        core.advance(1).expect("advance(1) succeeds");
        let scalars = capture(&core, &info);
        assert_latch_unchanged(&scalars, outcome, outcome_step, core.step_index());
        counts.push(scalars.goal_count);
        maybe_decision = observe_decision(maybe_decision, &scalars, core.step_index());
    }
    let decision = maybe_decision.expect("the stepping loop ends only after the decision");
    for _ in 0..POST_LATCH_STEPS / POST_LATCH_ADVANCE {
        core.advance(POST_LATCH_ADVANCE)
            .expect("post-latch advance succeeds");
        let scalars = capture(&core, &info);
        assert_latch_unchanged(&scalars, outcome, outcome_step, core.step_index());
        observe_decision(Some(decision), &scalars, core.step_index());
    }
    ReplayTrace {
        outcome: FixtureOutcome::from_frame_code(outcome).expect("latched outcome is won or lost"),
        outcome_step,
        final_count,
        peak_held: decision.peak_held,
        maybe_brim_step: decision.maybe_brim_step,
        decided_step: decision.step,
        counts,
    }
}

/// Checks a replay against its fixture's recorded expectation.
///
/// Both forms require the recorded outcome and the D-03 margin: a win must
/// still win with the target raised by `fill_margin_count(full_count)`
/// (`maybe_margin_win_step`), and a loss must keep every count at or below
/// `target - margin`. A recorded Brim step must likewise still be reached
/// with the Brim count raised by the margin (D-12). `exact` also requires
/// the recorded outcome step, final count, peak held count and Brim step;
/// otherwise the outcome and Brim steps may differ by up to
/// `DEBUG_OUTCOME_STEP_TOLERANCE` and Brim must be reached exactly when
/// recorded.
///
/// # Errors
///
/// Returns a description of the first violated rule.
pub(super) fn check_contract(
    fixture: &Fixture,
    trace: &ReplayTrace,
    info: &PuzzleLevelInfo,
    exact: bool,
) -> Result<(), String> {
    let Some(expected) = fixture.maybe_expectation else {
        return Err("fixture has no recorded outcome; record with LIQUIDFUN_PUZZLE_RECORD=1 cargo test -p liquidfun-wasm --release --lib puzzle::replay -- --nocapture".to_owned());
    };
    if trace.outcome != expected.outcome {
        return Err(format!(
            "outcome {} differs from recorded {}",
            trace.outcome.token(),
            expected.outcome.token()
        ));
    }
    match expected.outcome {
        FixtureOutcome::Won => check_won_margin(trace, info)?,
        FixtureOutcome::Lost => check_lost_margin(trace, info)?,
    }
    if expected.maybe_brim_step.is_some() {
        check_brim_margin(trace, info)?;
    }
    if exact {
        return check_exact(trace, expected);
    }
    check_within_tolerance("outcome step", trace.outcome_step, expected.outcome_step)?;
    match (trace.maybe_brim_step, expected.maybe_brim_step) {
        (Some(brim_step), Some(recorded)) => {
            check_within_tolerance("Brim step", brim_step, recorded)
        }
        (None, None) => Ok(()),
        (maybe_brim_step, maybe_recorded) => Err(format!(
            "Brim step {} differs from recorded {}",
            brim_token(maybe_brim_step),
            brim_token(maybe_recorded)
        )),
    }
}

fn check_within_tolerance(label: &str, step: u32, recorded: u32) -> Result<(), String> {
    let spread = step.abs_diff(recorded);
    if spread > DEBUG_OUTCOME_STEP_TOLERANCE {
        return Err(format!(
            "{label} {step} is {spread} steps from recorded {recorded} (tolerance {DEBUG_OUTCOME_STEP_TOLERANCE})"
        ));
    }
    Ok(())
}

fn check_won_margin(trace: &ReplayTrace, info: &PuzzleLevelInfo) -> Result<(), String> {
    check_raised_hold("win", trace, info.target_count(), info)
}

fn check_brim_margin(trace: &ReplayTrace, info: &PuzzleLevelInfo) -> Result<(), String> {
    check_raised_hold("Brim", trace, info.brim_count(), info)
}

/// `count` raised by the fill margin must hold for `settle_steps` by
/// `step_limit - MARGIN_WIN_SLACK_STEPS`.
fn check_raised_hold(
    label: &str,
    trace: &ReplayTrace,
    count: u32,
    info: &PuzzleLevelInfo,
) -> Result<(), String> {
    maybe_margin_win_step(
        &trace.counts,
        count,
        info.full_count(),
        info.settle_steps(),
        info.step_limit(),
    )
    .map(|_| ())
    .ok_or_else(|| {
        format!(
            "{label} does not hold at the raised count {} by step {}",
            count + fill_margin_count(info.full_count()),
            info.step_limit().saturating_sub(MARGIN_WIN_SLACK_STEPS)
        )
    })
}

fn check_lost_margin(trace: &ReplayTrace, info: &PuzzleLevelInfo) -> Result<(), String> {
    if trace.outcome_step != info.step_limit() {
        return Err(format!(
            "loss latched at step {} instead of the step limit {}",
            trace.outcome_step,
            info.step_limit()
        ));
    }
    let ceiling = info
        .target_count()
        .saturating_sub(fill_margin_count(info.full_count()));
    match trace.counts.iter().position(|&count| count > ceiling) {
        Some(index) => Err(format!(
            "count {} after step {} exceeds the loss ceiling {ceiling}",
            trace.counts[index],
            index + 1
        )),
        None => Ok(()),
    }
}

fn check_exact(trace: &ReplayTrace, expected: FixtureExpectation) -> Result<(), String> {
    if trace.outcome_step != expected.outcome_step {
        return Err(format!(
            "outcome step {} differs from recorded {}",
            trace.outcome_step, expected.outcome_step
        ));
    }
    if trace.final_count != expected.final_count {
        return Err(format!(
            "final count {} differs from recorded {}",
            trace.final_count, expected.final_count
        ));
    }
    if trace.peak_held != expected.peak_held {
        return Err(format!(
            "peak held {} differs from recorded {}",
            trace.peak_held, expected.peak_held
        ));
    }
    if trace.maybe_brim_step != expected.maybe_brim_step {
        return Err(format!(
            "Brim step {} differs from recorded {}",
            brim_token(trace.maybe_brim_step),
            brim_token(expected.maybe_brim_step)
        ));
    }
    Ok(())
}

/// The fixture token for an optional Brim step: digits or `none`.
fn brim_token(maybe_brim_step: Option<u32>) -> String {
    maybe_brim_step.map_or_else(|| NO_BRIM_TOKEN.to_owned(), |step| step.to_string())
}

/// The `RECORD` line printed by record mode; never written to a file.
pub(super) fn record_line(file: &str, trace: &ReplayTrace) -> String {
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    format!(
        "RECORD {file} outcome {} outcome_step {} final_count {} peak_held {} brim_step {} profile {profile}",
        trace.outcome.token(),
        trace.outcome_step,
        trace.final_count,
        trace.peak_held,
        brim_token(trace.maybe_brim_step)
    )
}

/// Whether `LIQUIDFUN_PUZZLE_RECORD=1` asks fixture tests to print `RECORD`
/// lines instead of asserting.
pub(in crate::puzzle) fn is_record_mode() -> bool {
    std::env::var_os("LIQUIDFUN_PUZZLE_RECORD").is_some_and(|value| value == "1")
}

/// The decision frame is the Brim step on a Brim win and the outcome step on
/// a loss, and counts run through the later of it and the margin window.
///
/// # Panics
///
/// Panics (test-only harness) when the trace has another shape.
pub(in crate::puzzle) fn assert_trace_shape(trace: &ReplayTrace, info: &PuzzleLevelInfo) {
    let expected_decision = match (trace.outcome, trace.maybe_brim_step) {
        (FixtureOutcome::Won, Some(brim_step)) => brim_step,
        (FixtureOutcome::Won, None) => info.step_limit(),
        (FixtureOutcome::Lost, _) => trace.outcome_step,
    };
    assert_eq!(trace.decided_step, expected_decision, "decision step");
    let window_end = info.step_limit() - MARGIN_WIN_SLACK_STEPS;
    let expected_len = trace.decided_step.max(window_end);
    assert_eq!(
        u32::try_from(trace.counts.len()).ok(),
        Some(expected_len),
        "counts run through the decision and the margin window"
    );
}

/// The level and placement lines of a fixture. Record mode regenerates the
/// expectation, so it must not depend on the recorded lines' grammar.
pub(in crate::puzzle) fn input_lines(text: &str) -> String {
    let mut inputs = text
        .lines()
        .take_while(|line| !line.starts_with("outcome "))
        .collect::<Vec<_>>()
        .join("\n");
    inputs.push('\n');
    inputs
}

/// Replays a fixture's inputs and prints its `RECORD` line.
///
/// # Panics
///
/// Panics (test-only harness) when the inputs do not parse or replay.
pub(in crate::puzzle) fn record_fixture_file(file: &str, text: &str) {
    let fixture = parse_fixture(&input_lines(text)).expect("fixture inputs parse");
    println!("{}", record_line(file, &replay(&fixture)));
}

/// Replays a checked-in fixture against its recorded contract: exact in
/// release, the debug tolerance otherwise. Record mode prints instead.
///
/// # Panics
///
/// Panics (test-only harness) when the fixture does not parse, replay or
/// meet its contract.
pub(in crate::puzzle) fn replay_fixture_file(file: &str, text: &str) {
    if is_record_mode() {
        record_fixture_file(file, text);
        return;
    }

    // Arrange
    let fixture = parse_fixture(text).expect("checked-in fixture parses");
    let info = PuzzleLevelInfo::from_spec(level_spec(fixture.level));

    // Act
    let trace = replay(&fixture);

    // Assert
    assert_trace_shape(&trace, &info);
    let result = check_contract(&fixture, &trace, &info, !cfg!(debug_assertions));
    assert_eq!(result, Ok(()), "{file} replay: {trace:?}");
}

#[cfg(test)]
mod tests;

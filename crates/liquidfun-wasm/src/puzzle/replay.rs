//! Test-only reference-solution replay (Phase 37 D-18, D-19): fixtures replay
//! through `build_puzzle_core` and `ProofFrame` getters, the same surface the
//! Bun wasm replay uses.
//!
//! The fixture grammar is shared byte-for-byte with
//! `web/scripts/puzzle-replay/fixture.ts`, so both runtimes accept exactly the
//! same text.

use crate::frame::ProofFrame;
use crate::puzzle::goal::{MARGIN_WIN_SLACK_STEPS, fill_margin_count, maybe_margin_win_step};
use crate::puzzle::tools::ToolKind;
use crate::puzzle::{LevelId, PuzzleLevelInfo, parse_level_id};
use crate::session::SessionCore;

/// Recorded outcome of a fixture run; tokens `won` / `lost`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FixtureOutcome {
    /// Frame outcome code 1.
    Won,
    /// Frame outcome code 2.
    Lost,
}

impl FixtureOutcome {
    const fn token(self) -> &'static str {
        match self {
            Self::Won => "won",
            Self::Lost => "lost",
        }
    }

    fn from_token(token: &str) -> Option<Self> {
        match token {
            "won" => Some(Self::Won),
            "lost" => Some(Self::Lost),
            _ => None,
        }
    }

    fn from_frame_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Won),
            2 => Some(Self::Lost),
            _ => None,
        }
    }
}

/// One `placement <kind> <x> <y> <angle>` line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct FixturePlacement {
    pub(super) kind: ToolKind,
    pub(super) x: i32,
    pub(super) y: i32,
    pub(super) angle: i32,
}

/// The recorded `outcome`, `outcome_step` and `final_count` lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct FixtureExpectation {
    pub(super) outcome: FixtureOutcome,
    pub(super) outcome_step: u32,
    /// Frame goal count at `outcome_step`.
    pub(super) final_count: u32,
}

/// A parsed fixture; input-only fixtures have no expectation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Fixture {
    pub(super) level: LevelId,
    pub(super) placements: Vec<FixturePlacement>,
    pub(super) maybe_expectation: Option<FixtureExpectation>,
}

const KNOWN_KEYS: [&str; 5] = [
    "level",
    "placement",
    "outcome",
    "outcome_step",
    "final_count",
];

type Row<'a> = (usize, Vec<&'a str>);

fn line_error(number: usize, reason: &str) -> String {
    format!("fixture line {number}: {reason}")
}

/// Parses the strict D-18 line format.
///
/// # Errors
///
/// Returns `fixture line N: ...` (1-based) for any deviation from the grammar.
pub(super) fn parse_fixture(text: &str) -> Result<Fixture, String> {
    let rows = split_rows(text)?;
    let next_number = rows.len() + 1;
    let mut rows = rows.into_iter().peekable();
    let Some((level_number, level_tokens)) = rows.next() else {
        return Err(line_error(1, "missing level line"));
    };
    let level = parse_level_line(level_number, &level_tokens)?;
    let mut placements = Vec::new();
    while let Some((number, tokens)) = rows.next_if(|(_, tokens)| tokens[0] == "placement") {
        placements.push(parse_placement_line(number, &tokens)?);
    }
    let maybe_expectation = match rows.next() {
        None => None,
        Some(first) => {
            let (number, outcome_token) = expect_field(Some(first), "outcome", next_number)?;
            let outcome = FixtureOutcome::from_token(outcome_token)
                .ok_or_else(|| line_error(number, "outcome must be `won` or `lost`"))?;
            let (number, step_token) = expect_field(rows.next(), "outcome_step", next_number)?;
            let outcome_step = parse_u32(number, step_token)?;
            let (number, count_token) = expect_field(rows.next(), "final_count", next_number)?;
            let final_count = parse_u32(number, count_token)?;
            if let Some((number, _)) = rows.next() {
                return Err(line_error(number, "unexpected line after final_count"));
            }
            Some(FixtureExpectation {
                outcome,
                outcome_step,
                final_count,
            })
        }
    };
    Ok(Fixture {
        level,
        placements,
        maybe_expectation,
    })
}

fn split_rows(text: &str) -> Result<Vec<Row<'_>>, String> {
    let body = text.strip_suffix('\n').unwrap_or(text);
    let mut rows = Vec::new();
    for (number, line) in (1_usize..).zip(body.split('\n')) {
        if line.contains('\r') {
            return Err(line_error(number, "carriage return"));
        }
        if line.is_empty() {
            return Err(line_error(number, "blank line"));
        }
        rows.push((number, line.split(' ').collect()));
    }
    Ok(rows)
}

fn check_key(number: usize, tokens: &[&str], key: &str) -> Result<(), String> {
    let found = tokens[0];
    if !KNOWN_KEYS.contains(&found) {
        return Err(line_error(number, &format!("unknown key `{found}`")));
    }
    if found != key {
        return Err(line_error(
            number,
            &format!("expected `{key}`, found `{found}`"),
        ));
    }
    Ok(())
}

fn check_arity(number: usize, tokens: &[&str], arity: usize) -> Result<(), String> {
    if tokens.len() == arity {
        return Ok(());
    }
    Err(line_error(
        number,
        &format!("expected {arity} space-separated tokens"),
    ))
}

fn parse_level_line(number: usize, tokens: &[&str]) -> Result<LevelId, String> {
    check_key(number, tokens, "level")?;
    check_arity(number, tokens, 2)?;
    parse_level_id(tokens[1]).ok_or_else(|| line_error(number, "unknown level"))
}

fn parse_placement_line(number: usize, tokens: &[&str]) -> Result<FixturePlacement, String> {
    check_arity(number, tokens, 5)?;
    let kind = ToolKind::from_token(tokens[1]).ok_or_else(|| line_error(number, "unknown tool"))?;
    Ok(FixturePlacement {
        kind,
        x: parse_i32(number, tokens[2])?,
        y: parse_i32(number, tokens[3])?,
        angle: parse_i32(number, tokens[4])?,
    })
}

fn expect_field<'a>(
    maybe_row: Option<Row<'a>>,
    key: &str,
    missing_number: usize,
) -> Result<(usize, &'a str), String> {
    let Some((number, tokens)) = maybe_row else {
        return Err(line_error(missing_number, &format!("missing `{key}` line")));
    };
    check_key(number, &tokens, key)?;
    check_arity(number, &tokens, 2)?;
    Ok((number, tokens[1]))
}

fn is_digits(token: &str) -> bool {
    !token.is_empty() && token.bytes().all(|byte| byte.is_ascii_digit())
}

/// Accepts exactly `-?[0-9]+` within `i32`.
fn parse_i32(number: usize, token: &str) -> Result<i32, String> {
    if !is_digits(token.strip_prefix('-').unwrap_or(token)) {
        return Err(line_error(number, &format!("`{token}` is not an integer")));
    }
    token
        .parse()
        .map_err(|_| line_error(number, &format!("`{token}` is out of range")))
}

/// Accepts exactly `[0-9]+` within `u32`.
fn parse_u32(number: usize, token: &str) -> Result<u32, String> {
    if !is_digits(token) {
        return Err(line_error(
            number,
            &format!("`{token}` is not a non-negative integer"),
        ));
    }
    token
        .parse()
        .map_err(|_| line_error(number, &format!("`{token}` is out of range")))
}

/// Flattens placements into the `newPuzzle` lanes in fixture order;
/// canonical sorting is the bridge's job.
pub(super) fn lanes(fixture: &Fixture) -> (Vec<u8>, Vec<i32>) {
    let kinds = fixture
        .placements
        .iter()
        .map(|placement| placement.kind.code())
        .collect();
    let poses = fixture
        .placements
        .iter()
        .flat_map(|placement| [placement.x, placement.y, placement.angle])
        .collect();
    (kinds, poses)
}

/// What one fixture run produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ReplayTrace {
    pub(super) outcome: FixtureOutcome,
    pub(super) outcome_step: u32,
    pub(super) final_count: u32,
    /// `counts[i]` is the goal count after step `i + 1`, through the latch and
    /// on to `step_limit - MARGIN_WIN_SLACK_STEPS`.
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
/// spread is 0 and the floor of 5 applies.
pub(super) const DEBUG_OUTCOME_STEP_TOLERANCE: u32 = 5;

/// Puzzle scalars read from one checked frame.
struct FrameScalars {
    goal_count: u32,
    outcome: u8,
    maybe_outcome_step: Option<u32>,
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
    }
}

fn assert_latch_unchanged(scalars: &FrameScalars, outcome: u8, outcome_step: u32, step: u32) {
    assert_eq!(
        (scalars.outcome, scalars.maybe_outcome_step),
        (outcome, Some(outcome_step)),
        "latched outcome changed at step {step}"
    );
}

/// Replays a fixture through the wasm-facing helper and frame getters.
///
/// Steps with `advance(1)` until the outcome latches, keeps recording counts
/// on the same session to `step_limit - MARGIN_WIN_SLACK_STEPS`, then runs
/// `POST_LATCH_STEPS` more steps, requiring the latch to hold on every frame.
///
/// # Panics
///
/// Panics (test-only harness) when the fixture does not build, a puzzle
/// scalar is missing or inconsistent, the run passes the step limit without
/// latching, or the latched outcome changes.
pub(super) fn replay(fixture: &Fixture) -> ReplayTrace {
    let (kinds, poses) = lanes(fixture);
    let mut core = crate::build_puzzle_core(fixture.level.as_str(), &kinds, &poses)
        .expect("fixture level and placements build");
    let info = core
        .level_info()
        .expect("puzzle sessions report level info");
    let mut counts = Vec::new();
    let (outcome, outcome_step, final_count) = loop {
        core.advance(1).expect("advance(1) succeeds");
        let scalars = capture(&core, &info);
        counts.push(scalars.goal_count);
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
    while core.step_index() < window_end {
        core.advance(1).expect("advance(1) succeeds");
        let scalars = capture(&core, &info);
        assert_latch_unchanged(&scalars, outcome, outcome_step, core.step_index());
        counts.push(scalars.goal_count);
    }
    for _ in 0..POST_LATCH_STEPS / POST_LATCH_ADVANCE {
        core.advance(POST_LATCH_ADVANCE)
            .expect("post-latch advance succeeds");
        let scalars = capture(&core, &info);
        assert_latch_unchanged(&scalars, outcome, outcome_step, core.step_index());
    }
    ReplayTrace {
        outcome: FixtureOutcome::from_frame_code(outcome).expect("latched outcome is won or lost"),
        outcome_step,
        final_count,
        counts,
    }
}

/// Checks a replay against its fixture's recorded expectation.
///
/// Both forms require the recorded outcome and the D-03 margin: a win must
/// still win with the target raised by `fill_margin_count(full_count)`
/// (`maybe_margin_win_step`), and a loss must keep every count at or below
/// `target - margin`. `exact` also requires the recorded outcome step and
/// final count; otherwise the outcome step may differ by up to
/// `DEBUG_OUTCOME_STEP_TOLERANCE`.
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
    if exact {
        return check_exact(trace, expected);
    }
    let spread = trace.outcome_step.abs_diff(expected.outcome_step);
    if spread > DEBUG_OUTCOME_STEP_TOLERANCE {
        return Err(format!(
            "outcome step {} is {spread} steps from recorded {} (tolerance {DEBUG_OUTCOME_STEP_TOLERANCE})",
            trace.outcome_step, expected.outcome_step
        ));
    }
    Ok(())
}

fn check_won_margin(trace: &ReplayTrace, info: &PuzzleLevelInfo) -> Result<(), String> {
    maybe_margin_win_step(
        &trace.counts,
        info.target_count(),
        info.full_count(),
        info.settle_steps(),
        info.step_limit(),
    )
    .map(|_| ())
    .ok_or_else(|| {
        format!(
            "win does not hold at the raised target {} by step {}",
            info.target_count() + fill_margin_count(info.full_count()),
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
    Ok(())
}

/// The `RECORD` line printed by record mode; never written to a file.
pub(super) fn record_line(file: &str, trace: &ReplayTrace) -> String {
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    format!(
        "RECORD {file} outcome {} outcome_step {} final_count {} profile {profile}",
        trace.outcome.token(),
        trace.outcome_step,
        trace.final_count
    )
}

#[cfg(test)]
mod tests;

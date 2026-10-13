//! Replay fixture grammar (Phase 37 D-18, Phase 39 D-12).
//!
//! The grammar is shared byte-for-byte with
//! `web/scripts/puzzle-replay/fixture.ts`: both runtimes accept and reject
//! exactly the same text with the same `fixture line N: ...` messages.
//!
//! Placement lines are `placement plank <x> <y> <angle>` (5 tokens) or
//! `placement valve <x> <y> <angle> <open_tick>` (6 tokens); the arity depends
//! on the kind, so a plank line with an open tick is rejected (D-07, D-27).

use crate::puzzle::tools::ToolKind;
use crate::puzzle::{LevelId, parse_level_id};

/// Recorded outcome of a fixture run; tokens `won` / `lost`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::puzzle) enum FixtureOutcome {
    /// Frame outcome code 1.
    Won,
    /// Frame outcome code 2.
    Lost,
}

impl FixtureOutcome {
    pub(in crate::puzzle) const fn token(self) -> &'static str {
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

    pub(in crate::puzzle) fn from_frame_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Won),
            2 => Some(Self::Lost),
            _ => None,
        }
    }
}

/// One `placement <kind> <x> <y> <angle> [<open_tick>]` line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::puzzle) struct FixturePlacement {
    pub(in crate::puzzle) kind: ToolKind,
    pub(in crate::puzzle) x: i32,
    pub(in crate::puzzle) y: i32,
    pub(in crate::puzzle) angle: i32,
    /// Valve open tick; 0 for 5-token plank lines.
    pub(in crate::puzzle) open_tick: i32,
}

/// The recorded `outcome`, `outcome_step`, `final_count`, `peak_held` and
/// `brim_step` lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::puzzle) struct FixtureExpectation {
    pub(in crate::puzzle) outcome: FixtureOutcome,
    pub(in crate::puzzle) outcome_step: u32,
    /// Frame goal count at `outcome_step`.
    pub(in crate::puzzle) final_count: u32,
    /// Frame peak held count at the decision step (D-12).
    pub(in crate::puzzle) peak_held: u32,
    /// Frame Brim step, or `None` for the `none` token (D-12).
    pub(in crate::puzzle) maybe_brim_step: Option<u32>,
}

/// A parsed fixture; input-only fixtures have no expectation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::puzzle) struct Fixture {
    pub(in crate::puzzle) level: LevelId,
    pub(in crate::puzzle) placements: Vec<FixturePlacement>,
    pub(in crate::puzzle) maybe_expectation: Option<FixtureExpectation>,
}

const KNOWN_KEYS: [&str; 7] = [
    "level",
    "placement",
    "outcome",
    "outcome_step",
    "final_count",
    "peak_held",
    "brim_step",
];

/// The `brim_step` token for a run that never reached Brim.
pub(in crate::puzzle) const NO_BRIM_TOKEN: &str = "none";

type Row<'a> = (usize, Vec<&'a str>);

fn line_error(number: usize, reason: &str) -> String {
    format!("fixture line {number}: {reason}")
}

/// Parses the strict D-18 line format with the D-12 expectation lines.
///
/// # Errors
///
/// Returns `fixture line N: ...` (1-based) for any deviation from the grammar.
pub(in crate::puzzle) fn parse_fixture(text: &str) -> Result<Fixture, String> {
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
        Some(first) => Some(parse_expectation(first, &mut rows, next_number)?),
    };
    Ok(Fixture {
        level,
        placements,
        maybe_expectation,
    })
}

fn parse_expectation<'a>(
    first: Row<'a>,
    rows: &mut impl Iterator<Item = Row<'a>>,
    next_number: usize,
) -> Result<FixtureExpectation, String> {
    let (number, outcome_token) = expect_field(Some(first), "outcome", next_number)?;
    let outcome = FixtureOutcome::from_token(outcome_token)
        .ok_or_else(|| line_error(number, "outcome must be `won` or `lost`"))?;
    let (number, step_token) = expect_field(rows.next(), "outcome_step", next_number)?;
    let outcome_step = parse_u32(number, step_token)?;
    let (number, count_token) = expect_field(rows.next(), "final_count", next_number)?;
    let final_count = parse_u32(number, count_token)?;
    let (number, peak_token) = expect_field(rows.next(), "peak_held", next_number)?;
    let peak_held = parse_u32(number, peak_token)?;
    let (number, brim_token) = expect_field(rows.next(), "brim_step", next_number)?;
    let maybe_brim_step = if brim_token == NO_BRIM_TOKEN {
        None
    } else {
        Some(parse_u32(number, brim_token)?)
    };
    if let Some((number, _)) = rows.next() {
        return Err(line_error(number, "unexpected line after brim_step"));
    }
    Ok(FixtureExpectation {
        outcome,
        outcome_step,
        final_count,
        peak_held,
        maybe_brim_step,
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
    let Some(kind_token) = tokens.get(1) else {
        return Err(line_error(number, "expected 5 space-separated tokens"));
    };
    let kind =
        ToolKind::from_token(kind_token).ok_or_else(|| line_error(number, "unknown tool"))?;
    let arity = match kind {
        ToolKind::Plank => 5,
        ToolKind::Valve => 6,
    };
    check_arity(number, tokens, arity)?;
    let open_tick = match kind {
        ToolKind::Plank => 0,
        ToolKind::Valve => parse_i32(number, tokens[5])?,
    };
    Ok(FixturePlacement {
        kind,
        x: parse_i32(number, tokens[2])?,
        y: parse_i32(number, tokens[3])?,
        angle: parse_i32(number, tokens[4])?,
        open_tick,
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
pub(in crate::puzzle) fn lanes(fixture: &Fixture) -> (Vec<u8>, Vec<i32>) {
    let kinds = fixture
        .placements
        .iter()
        .map(|placement| placement.kind.code())
        .collect();
    let poses = fixture
        .placements
        .iter()
        .flat_map(|placement| {
            [
                placement.x,
                placement.y,
                placement.angle,
                placement.open_tick,
            ]
        })
        .collect();
    (kinds, poses)
}

#[cfg(test)]
mod tests;

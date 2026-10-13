//! Tuning probe (D-18). Output is for authoring only; fixtures and assertions
//! are the evidence.
//!
//! The probes build through `validate_for_build` and never through
//! `parse_level_id`, so a draft `LevelId` that is not yet in `LEVEL_IDS` can
//! be probed while it is being tuned.

use super::levels_tests::{NudgeTally, open_tick_sweep, tally_nudges};
use super::replay::fixture::{FixtureOutcome, FixturePlacement};
use super::spec::LevelSpec;
use super::tools::{Placements, ToolKind};
use super::validate::validate_for_build;
use super::{LevelId, level_spec};
use crate::session::{SessionCore, SessionError};

/// Steps between two goal counts in `ProbeReport::curve`.
const CURVE_INTERVAL: u32 = 30;

/// What one probe run to the step limit observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProbeReport {
    /// Frame outcome code at the step limit: 0 running, 1 won, 2 lost.
    pub(super) outcome: u8,
    /// Latched outcome step.
    pub(super) maybe_outcome_step: Option<u32>,
    /// Peak held count at the step limit.
    pub(super) peak_held: u32,
    /// Step at which the peak held count reached Brim.
    pub(super) maybe_brim_step: Option<u32>,
    /// Goal count after every `CURVE_INTERVAL` steps.
    pub(super) curve: Vec<u32>,
    /// Particle count at step 0.
    pub(super) particles: usize,
}

/// `[kind_code, x, y, angle, open_tick]` rows as bridge lanes. A kind code
/// outside `u8` becomes 255, which the validator rejects as unknown.
fn probe_lanes(placements: &[[i32; 5]]) -> (Vec<u8>, Vec<i32>) {
    let kinds = placements
        .iter()
        .map(|[kind, ..]| u8::try_from(*kind).unwrap_or(u8::MAX))
        .collect();
    let poses = placements.iter().flat_map(|[_, pose @ ..]| *pose).collect();
    (kinds, poses)
}

/// `[kind_code, x, y, angle, open_tick]` rows as fixture placements.
///
/// # Panics
///
/// Panics when a kind code is not a known tool.
fn fixture_placements(placements: &[[i32; 5]]) -> Vec<FixturePlacement> {
    placements
        .iter()
        .map(|&[kind, x, y, angle, open_tick]| FixturePlacement {
            kind: u8::try_from(kind)
                .ok()
                .and_then(ToolKind::from_code)
                .unwrap_or_else(|| panic!("probe kind code {kind} is not a tool")),
            x,
            y,
            angle,
            open_tick,
        })
        .collect()
}

fn validated(spec: &LevelSpec, placements: &[[i32; 5]]) -> Result<Placements, SessionError> {
    let (kinds, poses) = probe_lanes(placements);
    validate_for_build(spec, &kinds, &poses).map_err(SessionError::InvalidPlacement)
}

/// Runs `core` with `advance(1)` through `step_limit`.
fn run_probe(mut core: SessionCore, step_limit: u32) -> Result<ProbeReport, SessionError> {
    let particles = core.particle_count();
    let mut curve = Vec::new();
    for step in 1..=step_limit {
        core.advance(1)?;
        if step % CURVE_INTERVAL == 0 {
            let status = core.maybe_puzzle_status().ok_or(SessionError::NotAPuzzle)?;
            curve.push(status.goal_count);
        }
    }
    let status = core.maybe_puzzle_status().ok_or(SessionError::NotAPuzzle)?;
    Ok(ProbeReport {
        outcome: status.outcome.code(),
        maybe_outcome_step: status.outcome.maybe_step(),
        peak_held: status.peak_held_count,
        maybe_brim_step: status.maybe_brim_step,
        curve,
        particles,
    })
}

/// Placements as `[kind_code, x, y, angle, open_tick]`; runs to `step_limit`
/// with `advance(1)`.
///
/// # Errors
///
/// Returns `InvalidPlacement` for placements the level rejects, or the
/// build or step error.
pub(super) fn probe(level: LevelId, placements: &[[i32; 5]]) -> Result<ProbeReport, SessionError> {
    let spec = level_spec(level);
    let core = SessionCore::create_puzzle(level, validated(spec, placements)?)?;
    run_probe(core, spec.step_limit)
}

/// Same, for an unregistered draft spec (Plan 01 `create_puzzle_from_spec`).
///
/// # Errors
///
/// Returns `InvalidPlacement` for placements the spec rejects, or the build
/// or step error.
pub(super) fn probe_spec(
    spec: &'static LevelSpec,
    placements: &[[i32; 5]],
) -> Result<ProbeReport, SessionError> {
    let core = SessionCore::create_puzzle_from_spec(spec, validated(spec, placements)?)?;
    run_probe(core, spec.step_limit)
}

fn step_token(maybe_step: Option<u32>) -> String {
    maybe_step.map_or_else(|| "none".to_owned(), |step| step.to_string())
}

/// The `PROBE` line for `report`.
fn report_line(label: &str, report: &ProbeReport) -> String {
    let curve = report
        .curve
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "PROBE {label} outcome {} step {} peak {} brim {} particles {} curve {curve}",
        report.outcome,
        step_token(report.maybe_outcome_step),
        report.peak_held,
        step_token(report.maybe_brim_step),
        report.particles
    )
}

/// One `PROBE <label> outcome .. step .. peak .. brim .. particles .. curve
/// c30,c60,…` line.
pub(super) fn print_report(label: &str, report: &ProbeReport) {
    println!("{}", report_line(label, report));
}

/// Draft-time nudge check (same variants as the release sweep); skips
/// invalid sets.
pub(super) fn probe_nudges(level: LevelId, placements: &[[i32; 5]]) -> NudgeTally {
    tally_nudges(level, &fixture_placements(placements))
}

/// Draft-time open-tick sweep for the valve at `valve_index`: one outcome per
/// tick `1..=max_open_tick`.
pub(super) fn probe_open_ticks(
    level: LevelId,
    placements: &[[i32; 5]],
    valve_index: usize,
) -> Vec<Option<FixtureOutcome>> {
    open_tick_sweep(level, &fixture_placements(placements), valve_index)
}

/// A first-pour plank outside the build area.
const OUTSIDE_PLANK: [i32; 5] = [1, 200, 64, 0, 0];

#[test]
fn probe_rejects_placements_the_level_rejects() {
    // Arrange
    let placements = [OUTSIDE_PLANK];

    // Act
    let result = probe(LevelId::FirstPour, &placements);

    // Assert
    assert!(matches!(result, Err(SessionError::InvalidPlacement(_))));
}

#[test]
fn probe_spec_rejects_placements_the_spec_rejects() {
    // Arrange
    let placements = [OUTSIDE_PLANK];

    // Act
    let result = probe_spec(level_spec(LevelId::FirstPour), &placements);

    // Assert
    assert!(matches!(result, Err(SessionError::InvalidPlacement(_))));
}

#[test]
fn probe_nudges_skip_invalid_sets() {
    // Arrange
    let placements = [OUTSIDE_PLANK];

    // Act
    let tally = probe_nudges(LevelId::FirstPour, &placements);

    // Assert
    assert_eq!((tally.generated, tally.skipped, tally.valid), (8, 8, 0));
}

#[test]
fn probe_open_ticks_cover_every_tick() {
    // Arrange
    let disallowed_valve = [[2, -16, 87, 0, 4]];

    // Act
    let outcomes = probe_open_ticks(LevelId::FirstPour, &disallowed_valve, 0);

    // Assert
    assert_eq!(outcomes.len(), 24);
    assert!(outcomes.iter().all(Option::is_none));
}

#[test]
fn report_line_lists_scalars_then_the_curve() {
    // Arrange
    let report = ProbeReport {
        outcome: 1,
        maybe_outcome_step: Some(201),
        peak_held: 811,
        maybe_brim_step: None,
        curve: vec![0, 12, 340],
        particles: 1972,
    };

    // Act
    let line = report_line("first-pour reference", &report);

    // Assert
    assert_eq!(
        line,
        "PROBE first-pour reference outcome 1 step 201 peak 811 brim none particles 1972 curve 0,12,340"
    );
}

#[test]
#[ignore = "authoring probe (D-18); cargo test -p liquidfun-wasm --release --lib puzzle::probe -- --ignored --nocapture"]
fn probe_first_pour_reference_solution() {
    // Arrange
    let placements = [[1, 8, 64, 23, 0]];

    // Act
    let report = probe(LevelId::FirstPour, &placements).expect("reference solution probes");

    // Assert
    print_report("first-pour reference", &report);
    assert_eq!(report.outcome, 1);
}

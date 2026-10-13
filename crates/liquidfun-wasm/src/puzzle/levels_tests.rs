//! Table-driven level proofs (Phase 40 D-14 to D-16, D-23): every registered
//! level has a three-star solution fixture and a losing empty fixture, fits
//! the particle budget, and (release only) mostly still wins when its
//! reference placements are nudged.
//!
//! A level plan adds its fixture pair to `FIXTURES` and, where it needs them,
//! one-line tests calling `replay_fixture_file`, `assert_nudges_mostly_win`,
//! `open_tick_sweep` or `run_spec_outcome`.

use std::time::Instant;

use super::goal::Outcome;
use super::replay::fixture::{
    Fixture, FixtureOutcome, FixturePlacement, parse_fixture, placement_lanes,
};
use super::spec::LevelSpec;
use super::tools::{Placements, ToolKind};
use super::validate::validate_for_build;
use super::{LEVEL_IDS, LevelId, level_spec};
use crate::session::SessionCore;

/// Most particles a level may hold at step 0, jelly included (D-23).
pub(super) const PARTICLE_BUDGET: usize = 3000;

/// A three-star win latches at least this many steps (1 s) before the limit (D-15).
const WIN_MARGIN_STEPS: u32 = 60;

/// Grid-cell nudges per placement and axis (D-16).
const NUDGE_CELLS: [i32; 4] = [-2, -1, 1, 2];

/// Open-tick nudges per valve (D-16).
const OPEN_TICK_NUDGES: [i32; 2] = [-1, 1];

/// Steps timed per level by `level_step_costs`.
const TIMED_STEPS: u32 = 720;

/// Every checked-in fixture by file name.
///
/// Level plans add their pair here; extras such as timed-gate early/late
/// losses go in too.
const FIXTURES: &[(&str, &str)] = &[
    (
        "first-pour-solution.txt",
        include_str!("fixtures/first-pour-solution.txt"),
    ),
    (
        "first-pour-empty.txt",
        include_str!("fixtures/first-pour-empty.txt"),
    ),
];

/// The text of a fixture in `FIXTURES`.
///
/// # Panics
///
/// Panics naming `file` when it is not in the table.
pub(super) fn fixture_text(file: &str) -> &'static str {
    FIXTURES
        .iter()
        .find_map(|&(name, text)| (name == file).then_some(text))
        .unwrap_or_else(|| panic!("fixture {file} is not in FIXTURES"))
}

fn solution_file(level: LevelId) -> String {
    format!("{}-solution.txt", level.as_str())
}

fn empty_file(level: LevelId) -> String {
    format!("{}-empty.txt", level.as_str())
}

fn parsed_fixture(file: &str) -> Fixture {
    parse_fixture(fixture_text(file)).unwrap_or_else(|error| panic!("{file}: {error}"))
}

/// Steps until the outcome latches; a loss latches at the step limit.
fn run_to_outcome(mut core: SessionCore, step_limit: u32) -> FixtureOutcome {
    loop {
        core.advance(1).expect("level steps");
        let status = core
            .maybe_puzzle_status()
            .expect("puzzle sessions report status");
        match status.outcome {
            Outcome::Won { .. } => return FixtureOutcome::Won,
            Outcome::Lost { .. } => return FixtureOutcome::Lost,
            Outcome::Running => assert!(
                core.step_index() < step_limit,
                "no outcome by step limit {step_limit}"
            ),
        }
    }
}

/// Validates `placements` against `spec` and runs the build to its outcome;
/// `None` when the bridge rejects the placements.
fn outcome_with(
    spec: &LevelSpec,
    placements: &[FixturePlacement],
    build: impl FnOnce(Placements) -> SessionCore,
) -> Option<FixtureOutcome> {
    let (kinds, poses) = placement_lanes(placements);
    let validated = validate_for_build(spec, &kinds, &poses).ok()?;
    Some(run_to_outcome(build(validated), spec.step_limit))
}

/// Outcome of `level` with `placements`, or `None` when they are invalid.
///
/// Uses the same validation as `build_puzzle_core` but skips the id
/// allowlist, so a draft level not yet in `LEVEL_IDS` can be checked.
pub(super) fn run_outcome(
    level: LevelId,
    placements: &[FixturePlacement],
) -> Option<FixtureOutcome> {
    outcome_with(level_spec(level), placements, |built| {
        SessionCore::create_puzzle(level, built).expect("valid placements build")
    })
}

/// Outcome of an unregistered spec, for idea-necessity controls: a test-only
/// `const` copy of a level with one idea removed or changed.
pub(super) fn run_spec_outcome(
    spec: &'static LevelSpec,
    placements: &[FixturePlacement],
) -> Option<FixtureOutcome> {
    outcome_with(spec, placements, |built| {
        SessionCore::create_puzzle_from_spec(spec, built).expect("valid placements build")
    })
}

/// D-16 variants: per placement, x then y by each of `NUDGE_CELLS`, then a
/// valve's open tick by each of `OPEN_TICK_NUDGES`; one change per variant.
pub(super) fn nudge_variants(placements: &[FixturePlacement]) -> Vec<Vec<FixturePlacement>> {
    let mut variants = Vec::new();
    for (index, &placement) in placements.iter().enumerate() {
        let along_x = NUDGE_CELLS.map(|delta| FixturePlacement {
            x: placement.x.saturating_add(delta),
            ..placement
        });
        let along_y = NUDGE_CELLS.map(|delta| FixturePlacement {
            y: placement.y.saturating_add(delta),
            ..placement
        });
        let ticks = OPEN_TICK_NUDGES
            .map(|delta| FixturePlacement {
                open_tick: placement.open_tick.saturating_add(delta),
                ..placement
            })
            .into_iter()
            .filter(|_| placement.kind == ToolKind::Valve);
        for changed in along_x.into_iter().chain(along_y).chain(ticks) {
            let mut variant = placements.to_vec();
            variant[index] = changed;
            variants.push(variant);
        }
    }
    variants
}

/// How a nudge sweep went; invalid sets are skipped, not lost.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct NudgeTally {
    pub(super) generated: usize,
    pub(super) valid: usize,
    pub(super) wins: usize,
    pub(super) skipped: usize,
}

/// Runs every nudge variant of `solution` on `level`.
pub(super) fn tally_nudges(level: LevelId, solution: &[FixturePlacement]) -> NudgeTally {
    nudge_variants(solution)
        .iter()
        .map(|variant| run_outcome(level, variant))
        .fold(NudgeTally::default(), |tally, maybe_outcome| {
            let generated = tally.generated + 1;
            match maybe_outcome {
                None => NudgeTally {
                    generated,
                    skipped: tally.skipped + 1,
                    ..tally
                },
                Some(outcome) => NudgeTally {
                    generated,
                    valid: tally.valid + 1,
                    wins: tally.wins + usize::from(outcome == FixtureOutcome::Won),
                    ..tally
                },
            }
        })
}

/// At least 70% of valid variants win, with a non-vacuity floor of four valid
/// variants and half of those generated (research Pitfall 9).
fn nudges_mostly_win(tally: &NudgeTally) -> bool {
    tally.valid >= 4 && 2 * tally.valid >= tally.generated && 10 * tally.wins >= 7 * tally.valid
}

/// Asserts the D-16 sweep for a solution fixture in `FIXTURES`.
///
/// # Panics
///
/// Panics with the tally when the sweep does not mostly win.
pub(super) fn assert_nudges_mostly_win(solution_file: &str) {
    let fixture = parsed_fixture(solution_file);
    let tally = tally_nudges(fixture.level, &fixture.placements);
    let NudgeTally {
        generated,
        valid,
        wins,
        skipped,
    } = tally;
    println!(
        "NUDGES {solution_file} wins {wins} valid {valid} skipped {skipped} generated {generated}"
    );
    assert!(
        nudges_mostly_win(&tally),
        "{solution_file}: wins {wins}/valid {valid}, skipped {skipped}, generated {generated}"
    );
}

/// One outcome per open tick `1..=max_open_tick` for the valve at
/// `valve_index` (fixture order); `None` marks an invalid set.
///
/// # Panics
///
/// Panics when the placement at `valve_index` is not a valve.
pub(super) fn open_tick_sweep(
    level: LevelId,
    placements: &[FixturePlacement],
    valve_index: usize,
) -> Vec<Option<FixtureOutcome>> {
    assert_eq!(
        placements[valve_index].kind,
        ToolKind::Valve,
        "placement {valve_index} is not a valve"
    );
    (1..=i32::from(level_spec(level).max_open_tick()))
        .map(|tick| {
            let mut variant = placements.to_vec();
            variant[valve_index].open_tick = tick;
            run_outcome(level, &variant)
        })
        .collect()
}

/// D-13 window: the first and last ticks do not win, and the winning ticks
/// form one contiguous band of at least two. Invalid ticks count as not won.
pub(super) fn timing_window_ok(outcomes: &[Option<FixtureOutcome>]) -> bool {
    let won: Vec<bool> = outcomes
        .iter()
        .map(|maybe_outcome| *maybe_outcome == Some(FixtureOutcome::Won))
        .collect();
    let (Some(&first), Some(&last)) = (won.first(), won.last()) else {
        return false;
    };
    let Some(start) = won.iter().position(|&is_won| is_won) else {
        return false;
    };
    let band = won[start..].iter().take_while(|&&is_won| is_won).count();
    !first && !last && band >= 2 && !won[start + band..].contains(&true)
}

fn plank(x: i32, y: i32, angle: i32) -> FixturePlacement {
    FixturePlacement {
        kind: ToolKind::Plank,
        x,
        y,
        angle,
        open_tick: 0,
    }
}

fn valve(x: i32, y: i32, angle: i32, open_tick: i32) -> FixturePlacement {
    FixturePlacement {
        kind: ToolKind::Valve,
        x,
        y,
        angle,
        open_tick,
    }
}

#[test]
fn nudge_variants_of_one_plank_move_x_then_y() {
    // Arrange
    let solution = [plank(8, 64, 23)];

    // Act
    let variants = nudge_variants(&solution);

    // Assert
    assert_eq!(
        variants,
        [
            [plank(6, 64, 23)],
            [plank(7, 64, 23)],
            [plank(9, 64, 23)],
            [plank(10, 64, 23)],
            [plank(8, 62, 23)],
            [plank(8, 63, 23)],
            [plank(8, 65, 23)],
            [plank(8, 66, 23)],
        ]
    );
}

#[test]
fn nudge_variants_of_a_valve_add_open_tick_steps() {
    // Arrange
    let solution = [valve(-16, 87, 0, 4)];

    // Act
    let variants = nudge_variants(&solution);

    // Assert
    assert_eq!(variants.len(), 10);
    assert_eq!(variants[8], [valve(-16, 87, 0, 3)]);
    assert_eq!(variants[9], [valve(-16, 87, 0, 5)]);
}

#[test]
fn nudge_variants_change_one_placement_at_a_time() {
    // Arrange
    let solution = [plank(8, 64, 23), valve(-16, 87, 0, 4)];

    // Act
    let variants = nudge_variants(&solution);

    // Assert
    assert_eq!(variants.len(), 18);
    for (index, variant) in variants.iter().enumerate() {
        let changed = variant
            .iter()
            .zip(&solution)
            .filter(|(nudged, original)| nudged != original)
            .count();
        assert_eq!(changed, 1, "variant {index} changes {changed} placements");
    }
    assert!(
        variants[..8]
            .iter()
            .all(|variant| variant[1] == solution[1])
    );
    assert!(
        variants[8..]
            .iter()
            .all(|variant| variant[0] == solution[0])
    );
}

#[test]
fn tally_skips_nudged_sets_the_bridge_rejects() {
    // Arrange
    let outside_build_area = [plank(200, 64, 0)];

    // Act
    let tally = tally_nudges(LevelId::FirstPour, &outside_build_area);

    // Assert
    assert_eq!(
        tally,
        NudgeTally {
            generated: 8,
            valid: 0,
            wins: 0,
            skipped: 8,
        }
    );
}

#[test]
fn run_spec_outcome_rejects_invalid_placements() {
    // Arrange
    let outside_build_area = [plank(200, 64, 0)];

    // Act
    let maybe_outcome = run_spec_outcome(level_spec(LevelId::FirstPour), &outside_build_area);

    // Assert
    assert_eq!(maybe_outcome, None);
}

#[test]
fn nudges_mostly_win_accepts_seventy_percent_of_enough_valid_variants() {
    // Arrange
    let tally = NudgeTally {
        generated: 16,
        valid: 10,
        wins: 7,
        skipped: 6,
    };

    // Act
    let accepted = nudges_mostly_win(&tally);

    // Assert
    assert!(accepted);
}

#[test]
fn nudges_mostly_win_rejects_few_wins_or_vacuous_sweeps() {
    // Arrange
    let too_few_wins = NudgeTally {
        generated: 10,
        valid: 10,
        wins: 6,
        skipped: 0,
    };
    let too_few_valid = NudgeTally {
        generated: 3,
        valid: 3,
        wins: 3,
        skipped: 0,
    };
    let mostly_skipped = NudgeTally {
        generated: 10,
        valid: 4,
        wins: 4,
        skipped: 6,
    };

    // Act
    let accepted =
        [too_few_wins, too_few_valid, mostly_skipped].map(|tally| nudges_mostly_win(&tally));

    // Assert
    assert_eq!(accepted, [false, false, false]);
}

#[test]
fn timing_window_accepts_one_band_between_losses() {
    // Arrange
    let (won, lost) = (Some(FixtureOutcome::Won), Some(FixtureOutcome::Lost));
    let outcomes = [lost, lost, won, won, won, lost];

    // Act
    let accepted = timing_window_ok(&outcomes);

    // Assert
    assert!(accepted);
}

#[test]
fn timing_window_rejects_edge_wins_split_bands_and_single_ticks() {
    // Arrange
    let (won, lost) = (Some(FixtureOutcome::Won), Some(FixtureOutcome::Lost));
    let early_win = vec![won, won, lost];
    let split = vec![lost, won, lost, won, lost];
    let single = vec![lost, won, lost];
    let invalid_edges = vec![None, won, won, None];
    let empty = Vec::new();

    // Act
    let accepted = [early_win, split, single, invalid_edges.clone(), empty]
        .map(|outcomes| timing_window_ok(&outcomes));

    // Assert
    assert_eq!(accepted, [false, false, false, true, false]);
}

#[test]
fn open_tick_sweep_reports_every_tick_of_a_rejected_valve_as_invalid() {
    // Arrange
    let disallowed_valve = [valve(-16, 87, 0, 4)];
    let max_tick = usize::from(level_spec(LevelId::FirstPour).max_open_tick());

    // Act
    let outcomes = open_tick_sweep(LevelId::FirstPour, &disallowed_valve, 0);

    // Assert
    assert_eq!(outcomes, vec![None; max_tick]);
    assert!(!timing_window_ok(&outcomes));
}

#[test]
fn every_level_has_solution_and_empty_fixtures() {
    for level in LEVEL_IDS {
        // Arrange
        let files = [solution_file(level), empty_file(level)];

        // Act
        let missing: Vec<&String> = files
            .iter()
            .filter(|file| !FIXTURES.iter().any(|(name, _)| name == file))
            .collect();

        // Assert
        assert!(missing.is_empty(), "missing fixtures {missing:?}");
    }
}

#[test]
fn solution_fixtures_record_three_stars_with_margin() {
    for level in LEVEL_IDS {
        // Arrange
        let spec = level_spec(level);
        let file = solution_file(level);

        // Act
        let fixture = parsed_fixture(&file);

        // Assert
        let expectation = fixture
            .maybe_expectation
            .unwrap_or_else(|| panic!("{file} has no recorded outcome"));
        assert_eq!(fixture.level, level, "{file}");
        assert_eq!(expectation.outcome, FixtureOutcome::Won, "{file}");
        assert!(
            expectation.maybe_brim_step.is_some(),
            "{file}: no Brim step"
        );
        let placements = u32::try_from(fixture.placements.len()).expect("placement count fits");
        assert!(placements <= spec.par_tools, "{file}: {placements} > par");
        assert!(
            expectation.outcome_step + WIN_MARGIN_STEPS <= spec.step_limit,
            "{file}: win at {} is within {WIN_MARGIN_STEPS} steps of the limit",
            expectation.outcome_step
        );
    }
}

#[test]
fn brim_counts_clear_the_target_by_ten_points_within_full() {
    for level in LEVEL_IDS {
        // Arrange
        let spec = level_spec(level);

        // Act
        let (brim, target, full) = (spec.brim_count, spec.target_count, spec.full_count);

        // Assert
        assert!(
            brim * 10 >= target * 10 + full,
            "{}: Brim {brim} is below target {target} + 10% of {full}",
            spec.title
        );
        assert!(brim <= full, "{}: Brim {brim} > full {full}", spec.title);
    }
}

#[test]
fn empty_fixtures_have_no_placements_and_lose() {
    for level in LEVEL_IDS {
        // Arrange
        let file = empty_file(level);

        // Act
        let fixture = parsed_fixture(&file);

        // Assert
        assert_eq!(fixture.level, level, "{file}");
        assert!(fixture.placements.is_empty(), "{file} has placements");
        let outcome = fixture.maybe_expectation.map(|expected| expected.outcome);
        assert_eq!(outcome, Some(FixtureOutcome::Lost), "{file}");
    }
}

#[test]
fn step_zero_particles_fit_the_budget() {
    for level in LEVEL_IDS {
        // Arrange
        let spec = level_spec(level);

        // Act
        let core = crate::build_puzzle_core(level.as_str(), &[], &[]).expect("empty level builds");

        // Assert
        let particles = core.particle_count();
        assert!(
            particles <= PARTICLE_BUDGET,
            "{}: {particles} particles > {PARTICLE_BUDGET}",
            spec.title
        );
        assert!(
            particles <= spec.maximum_particle_count,
            "{}: {particles} particles > cap {}",
            spec.title,
            spec.maximum_particle_count
        );
    }
}

#[test]
#[cfg_attr(
    debug_assertions,
    ignore = "release-only nudge sweep (D-16); run with --release"
)]
fn first_pour_nudges_mostly_win() {
    assert_nudges_mostly_win("first-pour-solution.txt");
}

#[test]
#[ignore = "informational D-23 timing; cargo test -p liquidfun-wasm --release --lib puzzle::levels_tests::level_step_costs -- --ignored --nocapture"]
fn level_step_costs() {
    for level in LEVEL_IDS {
        // Arrange
        let fixture = parsed_fixture(&solution_file(level));
        let (kinds, poses) = placement_lanes(&fixture.placements);
        let mut core =
            crate::build_puzzle_core(level.as_str(), &kinds, &poses).expect("solution builds");
        let particles = core.particle_count();

        // Act
        let start = Instant::now();
        for _ in 0..TIMED_STEPS {
            core.advance(1).expect("level steps");
        }
        let elapsed = start.elapsed();

        // Assert
        let ms_per_step = elapsed.as_secs_f64() * 1000.0 / f64::from(TIMED_STEPS);
        println!(
            "STEPCOST {} particles {particles} ms_per_step {ms_per_step:.2}",
            level.as_str()
        );
        assert!(ms_per_step.is_finite());
    }
}

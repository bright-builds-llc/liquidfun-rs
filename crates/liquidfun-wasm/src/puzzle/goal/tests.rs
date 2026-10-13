use liquidfun::math::Vec2;

use super::{
    GoalRect, GoalTracker, MARGIN_WIN_SLACK_STEPS, Outcome, PuzzleStatus, count_in_goal,
    fill_margin_count, fill_ratio, maybe_margin_win_step,
};

const TARGET: u32 = 10;
const SETTLE: u32 = 30;
const LIMIT: u32 = 600;

fn interior() -> GoalRect {
    GoalRect::new(Vec2::new(1.0, 0.0), Vec2::new(3.0, 2.0))
}

fn count_group_one(positions: &[Vec2]) -> u32 {
    let groups = vec![Some(1_u8); positions.len()];
    count_in_goal(positions, &groups, &[1], interior())
}

fn tracker() -> GoalTracker {
    GoalTracker::new(TARGET, SETTLE, LIMIT).expect("default tracker configuration is valid")
}

/// Observes `count` for every step in `steps` and returns every outcome in order.
fn observe_steps(
    tracker: &mut GoalTracker,
    count: u32,
    steps: std::ops::RangeInclusive<u32>,
) -> Vec<Outcome> {
    steps.map(|step| tracker.observe(count, step)).collect()
}

fn all_running(outcomes: &[Outcome]) -> bool {
    outcomes.iter().all(|outcome| *outcome == Outcome::Running)
}

#[test]
fn counts_a_counted_particle_strictly_inside() {
    // Arrange
    let positions = [Vec2::new(2.0, 1.0)];

    // Act
    let count = count_group_one(&positions);

    // Assert
    assert_eq!(count, 1);
}

#[test]
fn excludes_particles_on_each_interior_edge() {
    // Arrange
    let positions = [
        Vec2::new(1.0, 1.0),
        Vec2::new(3.0, 1.0),
        Vec2::new(2.0, 0.0),
        Vec2::new(2.0, 2.0),
    ];

    // Act
    let count = count_group_one(&positions);

    // Assert
    assert_eq!(count, 0);
}

#[test]
fn excludes_a_particle_just_above_the_rim() {
    // Arrange
    let positions = [Vec2::new(2.0, 2.000_001)];

    // Act
    let count = count_group_one(&positions);

    // Assert
    assert_eq!(count, 0);
}

#[test]
fn excludes_a_particle_outside_the_wall() {
    // Arrange
    let positions = [Vec2::new(0.9, 1.0)];

    // Act
    let count = count_group_one(&positions);

    // Assert
    assert_eq!(count, 0);
}

#[test]
fn empty_lanes_count_zero() {
    // Arrange
    let positions: [Vec2; 0] = [];

    // Act
    let count = count_group_one(&positions);

    // Assert
    assert_eq!(count, 0);
}

#[test]
fn excludes_non_counted_groups() {
    // Arrange
    let positions = [Vec2::new(2.0, 1.0)];
    let groups = [Some(2_u8)];

    // Act
    let count = count_in_goal(&positions, &groups, &[1], interior());

    // Assert
    assert_eq!(count, 0);
}

#[test]
fn excludes_ungrouped_particles() {
    // Arrange
    let positions = [Vec2::new(2.0, 1.0)];
    let groups: [Option<u8>; 1] = [None];

    // Act
    let count = count_in_goal(&positions, &groups, &[1], interior());

    // Assert
    assert_eq!(count, 0);
}

#[test]
fn excludes_non_finite_positions() {
    // Arrange
    let positions = [Vec2::new(f32::NAN, 1.0)];

    // Act
    let count = count_group_one(&positions);

    // Assert
    assert_eq!(count, 0);
}

#[test]
fn evicted_particles_lower_the_count_but_not_full() {
    // Arrange
    let full_count = 4;
    let before = [
        Vec2::new(1.5, 1.0),
        Vec2::new(2.0, 1.0),
        Vec2::new(2.5, 1.0),
    ];
    let after = [Vec2::new(1.5, 1.0), Vec2::new(2.5, 1.0)];

    // Act
    let count_before = count_group_one(&before);
    let count_after = count_group_one(&after);
    let fill_after = fill_ratio(count_after, full_count);

    // Assert
    assert_eq!((count_before, count_after), (3, 2));
    assert_eq!(fill_after.to_bits(), 0.5_f64.to_bits());
}

#[test]
fn fill_ratio_is_count_over_full() {
    // Arrange
    let (count, full_count) = (1, 4);

    // Act
    let fill = fill_ratio(count, full_count);

    // Assert
    assert_eq!(fill.to_bits(), 0.25_f64.to_bits());
}

#[test]
fn fill_ratio_clamps_to_one() {
    // Arrange
    let (count, full_count) = (5, 4);

    // Act
    let fill = fill_ratio(count, full_count);

    // Assert
    assert_eq!(fill.to_bits(), 1.0_f64.to_bits());
}

#[test]
fn fill_ratio_of_zero_full_is_zero() {
    // Arrange
    let (count, full_count) = (3, 0);

    // Act
    let fill = fill_ratio(count, full_count);

    // Assert
    assert_eq!(fill.to_bits(), 0.0_f64.to_bits());
}

#[test]
fn fill_margin_is_five_percent_rounded_up() {
    // Arrange
    let full_counts = [900, 901, 1, 0];

    // Act
    let margins = full_counts.map(fill_margin_count);

    // Assert
    assert_eq!(margins, [45, 46, 1, 0]);
}

#[test]
fn goal_rect_rejects_inverted_or_non_finite_corners() {
    // Arrange
    let degenerate = GoalRect::new(Vec2::new(1.0, 1.0), Vec2::new(1.0, 1.0));
    let inverted = GoalRect::new(Vec2::new(3.0, 0.0), Vec2::new(1.0, 2.0));
    let non_finite = GoalRect::new(Vec2::new(f32::NAN, 0.0), Vec2::new(3.0, 2.0));

    // Act
    let validity = [degenerate, inverted, non_finite, interior()].map(GoalRect::is_valid);

    // Assert
    assert_eq!(validity, [false, false, false, true]);
}

#[test]
fn tracker_rejects_invalid_configuration() {
    // Arrange
    let configurations = [(0, 30, 600), (10, 0, 600), (10, 601, 600), (10, 30, 600)];

    // Act
    let accepted = configurations
        .map(|(target, settle, limit)| GoalTracker::new(target, settle, limit).is_some());

    // Assert
    assert_eq!(accepted, [false, false, false, true]);
}

#[test]
fn holding_from_step_s_wins_at_s_plus_settle_minus_one() {
    // Arrange
    let mut tracker = tracker();
    let before_hold = observe_steps(&mut tracker, 0, 1..=99);

    // Act
    let during_hold = observe_steps(&mut tracker, TARGET, 100..=128);
    let completing = tracker.observe(TARGET, 129);

    // Assert
    assert!(all_running(&before_hold) && all_running(&during_hold));
    assert_eq!(completing, Outcome::Won { step: 129 });
}

#[test]
fn splash_shorter_than_settle_does_not_win() {
    // Arrange
    let mut tracker = tracker();

    // Act
    let splash = observe_steps(&mut tracker, TARGET, 1..=29);
    let drained = observe_steps(&mut tracker, 0, 30..=100);

    // Assert
    assert!(all_running(&splash) && all_running(&drained));
}

#[test]
fn dip_resets_the_hold() {
    // Arrange
    let mut tracker = tracker();
    let first_hold = observe_steps(&mut tracker, TARGET, 1..=20);
    let dip = tracker.observe(TARGET - 1, 21);

    // Act
    let second_hold = observe_steps(&mut tracker, TARGET, 22..=50);
    let completing = tracker.observe(TARGET, 51);

    // Assert
    assert!(all_running(&first_hold) && all_running(&second_hold));
    assert_eq!(dip, Outcome::Running);
    assert_eq!(completing, Outcome::Won { step: 51 });
}

#[test]
fn win_completing_at_the_step_limit_wins() {
    // Arrange
    let mut tracker = tracker();
    let empty = observe_steps(&mut tracker, 0, 1..=570);
    let hold = observe_steps(&mut tracker, TARGET, 571..=599);

    // Act
    let completing = tracker.observe(TARGET, 600);

    // Assert
    assert!(all_running(&empty) && all_running(&hold));
    assert_eq!(completing, Outcome::Won { step: 600 });
}

#[test]
fn never_holding_loses_at_the_step_limit() {
    // Arrange
    let mut tracker = tracker();

    // Act
    let before_limit = observe_steps(&mut tracker, 0, 1..=599);
    let at_limit = tracker.observe(0, 600);

    // Assert
    assert!(all_running(&before_limit));
    assert_eq!(at_limit, Outcome::Lost { step: 600 });
}

#[test]
fn observing_past_the_limit_first_time_loses_at_the_limit() {
    // Arrange
    let mut tracker = tracker();

    // Act
    let outcome = tracker.observe(0, 650);

    // Assert
    assert_eq!(outcome, Outcome::Lost { step: 600 });
}

#[test]
fn win_survives_a_later_slosh() {
    // Arrange
    let mut tracker = tracker();
    observe_steps(&mut tracker, 0, 1..=99);
    observe_steps(&mut tracker, TARGET, 100..=129);

    // Act
    let after_slosh = tracker.observe(0, 130);

    // Assert
    assert_eq!(after_slosh, Outcome::Won { step: 129 });
    assert_eq!(tracker.outcome(), Outcome::Won { step: 129 });
}

#[test]
fn loss_survives_a_later_hold() {
    // Arrange
    let mut tracker = tracker();
    observe_steps(&mut tracker, 0, 1..=600);

    // Act
    let later = observe_steps(&mut tracker, TARGET, 601..=640);

    // Assert
    assert!(
        later
            .iter()
            .all(|outcome| *outcome == Outcome::Lost { step: 600 })
    );
}

#[test]
fn outcome_codes_and_steps_follow_the_frame_contract() {
    // Arrange
    let outcomes = [
        Outcome::Running,
        Outcome::Won { step: 7 },
        Outcome::Lost { step: 600 },
    ];

    // Act
    let encoded = outcomes.map(|outcome| (outcome.code(), outcome.maybe_step()));

    // Assert
    assert_eq!(encoded, [(0, None), (1, Some(7)), (2, Some(600))]);
}

#[test]
fn goal_rect_exposes_its_corners() {
    // Arrange
    let rect = interior();

    // Act
    let corners = (rect.min(), rect.max());

    // Assert
    assert_eq!(corners, (Vec2::new(1.0, 0.0), Vec2::new(3.0, 2.0)));
}

#[test]
fn puzzle_status_carries_the_latched_outcome() {
    // Arrange
    let mut tracker = tracker();
    observe_steps(&mut tracker, 0, 1..=600);

    // Act
    let status = PuzzleStatus {
        goal_count: 0,
        target_count: TARGET,
        full_count: 12,
        outcome: tracker.outcome(),
        observed_step: 600,
        step_limit: LIMIT,
        peak_held_count: 0,
        brim_count: 11,
        maybe_brim_step: None,
        maybe_decided_step: Some(LIMIT),
    };

    // Assert
    assert_eq!(status.outcome.maybe_step(), Some(status.step_limit));
}

/// Full count whose 5% margin is 5, so the raised target is `TARGET + 5`.
const MARGIN_FULL: u32 = 100;
const RAISED_TARGET: u32 = TARGET + 5;

/// `quiet` zero counts followed by `held` counts at `count`.
fn counts_after_quiet_steps(quiet: u32, held: u32, count: u32) -> Vec<u32> {
    let mut counts = vec![0; quiet as usize];
    counts.resize((quiet + held) as usize, count);
    counts
}

#[test]
fn margin_win_completes_when_the_raised_target_holds_for_the_settle_window() {
    // Arrange
    let counts = counts_after_quiet_steps(5, SETTLE, RAISED_TARGET);

    // Act
    let maybe_step = maybe_margin_win_step(&counts, TARGET, MARGIN_FULL, SETTLE, LIMIT);

    // Assert
    assert_eq!(maybe_step, Some(5 + SETTLE));
}

#[test]
fn margin_win_needs_counts_at_or_above_the_raised_target() {
    // Arrange
    let counts = counts_after_quiet_steps(5, 200, RAISED_TARGET - 1);

    // Act
    let maybe_step = maybe_margin_win_step(&counts, TARGET, MARGIN_FULL, SETTLE, LIMIT);

    // Assert
    assert_eq!(maybe_step, None);
}

#[test]
fn margin_win_must_complete_by_the_slack_deadline() {
    // Arrange
    let deadline = LIMIT - MARGIN_WIN_SLACK_STEPS;
    let on_time = counts_after_quiet_steps(deadline - SETTLE, SETTLE, RAISED_TARGET);
    let late = counts_after_quiet_steps(deadline - SETTLE + 1, SETTLE, RAISED_TARGET);

    // Act
    let maybe_on_time = maybe_margin_win_step(&on_time, TARGET, MARGIN_FULL, SETTLE, LIMIT);
    let maybe_late = maybe_margin_win_step(&late, TARGET, MARGIN_FULL, SETTLE, LIMIT);

    // Assert
    assert_eq!((maybe_on_time, maybe_late), (Some(deadline), None));
}

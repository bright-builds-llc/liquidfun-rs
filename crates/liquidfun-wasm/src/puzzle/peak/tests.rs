use super::{PeakHeldTracker, decided_step};
use crate::puzzle::goal::{GoalTracker, Outcome};

const WINDOW: u32 = 3;
const LIMIT: u32 = 100;
const HIGH_BRIM: u32 = 100;

fn tracker(step_limit: u32, brim_count: u32) -> PeakHeldTracker {
    PeakHeldTracker::new(WINDOW, step_limit, brim_count).expect("valid peak tracker")
}

/// Observes `counts[i]` as the count after step `i + 1`.
fn observe_all(tracker: &mut PeakHeldTracker, counts: &[u32]) {
    for (step, &count) in (1_u32..).zip(counts) {
        tracker.observe(count, step);
    }
}

#[test]
fn new_rejects_invalid_configurations() {
    // Arrange
    let configurations = [(0, LIMIT, 5), (LIMIT + 1, LIMIT, 5), (WINDOW, LIMIT, 0)];

    // Act
    let results = configurations
        .map(|(settle, limit, brim)| PeakHeldTracker::new(settle, limit, brim).is_none());

    // Assert
    assert_eq!(results, [true; 3]);
}

#[test]
fn a_splash_shorter_than_the_window_does_not_raise_the_peak() {
    // Arrange
    let mut tracker = tracker(LIMIT, HIGH_BRIM);

    // Act
    observe_all(&mut tracker, &[5, 9, 5, 5, 5]);

    // Assert
    assert_eq!(tracker.peak(), 5);
}

#[test]
fn the_peak_never_decreases() {
    // Arrange
    let mut tracker = tracker(LIMIT, HIGH_BRIM);

    // Act
    observe_all(&mut tracker, &[6, 6, 6, 2, 2, 2]);

    // Assert
    assert_eq!(tracker.peak(), 6);
}

#[test]
fn the_peak_and_brim_freeze_after_the_step_limit() {
    // Arrange
    let mut tracker = tracker(4, 8);

    // Act
    observe_all(&mut tracker, &[7, 7, 7, 7, 9, 9, 9, 9]);

    // Assert
    assert_eq!((tracker.peak(), tracker.maybe_brim_step()), (7, None));
}

#[test]
fn brim_latches_at_the_first_step_the_peak_reaches_it() {
    // Arrange
    let mut tracker = tracker(LIMIT, 7);
    observe_all(&mut tracker, &[7, 7, 7]);
    let first_latch = tracker.maybe_brim_step();

    // Act
    observe_all(&mut tracker, &[9, 9, 9, 9]);

    // Assert
    assert_eq!(first_latch, Some(3));
    assert_eq!(tracker.maybe_brim_step(), Some(3));
}

#[test]
fn running_and_lost_runs_decide_at_none_and_the_loss_latch() {
    // Arrange
    let cases = [
        (Outcome::Running, None, 300),
        (Outcome::Lost { step: 720 }, None, 720),
    ];

    // Act
    let decided = cases
        .map(|(outcome, maybe_brim, observed)| decided_step(outcome, maybe_brim, observed, 720));

    // Assert
    assert_eq!(decided, [None, Some(720)]);
}

#[test]
fn a_won_run_with_brim_decides_at_the_brim_step() {
    // Arrange
    let outcome = Outcome::Won { step: 201 };

    // Act
    let decided = decided_step(outcome, Some(262), 300, 720);

    // Assert
    assert_eq!(decided, Some(262));
}

#[test]
fn a_won_run_without_brim_decides_at_the_step_limit() {
    // Arrange
    let outcome = Outcome::Won { step: 201 };

    // Act
    let decided = [700, 720, 730].map(|observed| decided_step(outcome, None, observed, 720));

    // Assert
    assert_eq!(decided, [None, Some(720), Some(720)]);
}

/// Deterministic xorshift32 counts in `0..=12`.
fn pseudo_random_counts(seed: u32, len: usize) -> Vec<u32> {
    let mut state = seed;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state % 13
        })
        .collect()
}

fn goal_win_step(counts: &[u32], target: u32, settle: u32, limit: u32) -> Option<u32> {
    let mut goal = GoalTracker::new(target, settle, limit).expect("valid goal tracker");
    (1_u32..)
        .zip(counts)
        .find_map(|(step, &count)| match goal.observe(count, step) {
            Outcome::Won { step } => Some(step),
            Outcome::Running | Outcome::Lost { .. } => None,
        })
}

fn first_peak_step(counts: &[u32], target: u32, settle: u32, limit: u32) -> Option<u32> {
    let mut peak = PeakHeldTracker::new(settle, limit, target).expect("valid peak tracker");
    (1_u32..).zip(counts).find_map(|(step, &count)| {
        peak.observe(count, step);
        (peak.peak() >= target).then_some(step)
    })
}

#[test]
fn the_first_peak_at_target_matches_the_goal_tracker_win_step() {
    // Arrange
    let (target, settle, limit) = (6, 4, 60);
    let sequences: Vec<Vec<u32>> = (1_u32..=200)
        .map(|seed| pseudo_random_counts(seed.wrapping_mul(2_654_435_761), 70))
        .collect();

    // Act
    let pairs: Vec<(Option<u32>, Option<u32>)> = sequences
        .iter()
        .map(|counts| {
            (
                goal_win_step(counts, target, settle, limit),
                first_peak_step(counts, target, settle, limit),
            )
        })
        .collect();

    // Assert
    for (index, (goal, peak)) in pairs.iter().enumerate() {
        assert_eq!(goal, peak, "sequence {index} disagrees");
    }
    assert!(pairs.iter().any(|(goal, _)| goal.is_some()));
    assert!(pairs.iter().any(|(goal, _)| goal.is_none()));
}

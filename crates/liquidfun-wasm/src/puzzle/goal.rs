//! Pure goal counting and the step-latched puzzle outcome (Phase 37).
//!
//! Nothing here reads time, frames or the world: callers pass particle lanes
//! and the session step index, so every edge case is a plain unit test.

use liquidfun::math::Vec2;

/// Axis-aligned cup interior used for goal counting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GoalRect {
    min: Vec2,
    max: Vec2,
}

impl GoalRect {
    /// Creates a rectangle from its lower-left and upper-right corners.
    pub(crate) const fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Lower-left corner.
    pub(crate) const fn min(self) -> Vec2 {
        self.min
    }

    /// Upper-right corner.
    pub(crate) const fn max(self) -> Vec2 {
        self.max
    }

    /// Finite corners with min strictly below max on both axes.
    pub(crate) fn is_valid(self) -> bool {
        self.min.is_valid()
            && self.max.is_valid()
            && self.min.x < self.max.x
            && self.min.y < self.max.y
    }

    /// Strict containment: a centre on any edge is outside; NaN is outside.
    pub(crate) fn contains_strictly(self, point: Vec2) -> bool {
        self.min.x < point.x && point.x < self.max.x && self.min.y < point.y && point.y < self.max.y
    }
}

/// Counts particles whose group is in `counted` and whose centre lies
/// strictly inside `interior`.
///
/// Production lanes come from `ParticleSystemView::positions()` and
/// `group_ids()`, which are aligned per particle; the zip stops at the
/// shorter lane. Ungrouped particles never count. The count saturates and
/// nothing is allocated.
pub(crate) fn count_in_goal<G: PartialEq>(
    positions: &[Vec2],
    groups: &[Option<G>],
    counted: &[G],
    interior: GoalRect,
) -> u32 {
    positions
        .iter()
        .zip(groups)
        .filter(|(_, maybe_group)| {
            maybe_group
                .as_ref()
                .is_some_and(|group| counted.contains(group))
        })
        .filter(|(position, _)| interior.contains_strictly(**position))
        .fold(0_u32, |count, _| count.saturating_add(1))
}

/// Fill as `count / full_count`, clamped to 1.0; 0.0 when `full_count` is 0.
///
/// `full_count` is authored and fixed, so eviction lowers only `count`.
pub(crate) fn fill_ratio(count: u32, full_count: u32) -> f64 {
    if full_count == 0 {
        return 0.0;
    }
    (f64::from(count) / f64::from(full_count)).min(1.0)
}

/// The +5% fill margin (D-03 fallback) as a particle count, rounded up.
pub(crate) const fn fill_margin_count(full_count: u32) -> u32 {
    full_count.div_ceil(20)
}

/// Steps before the step limit by which the raised-target win must complete.
#[cfg(test)]
pub(crate) const MARGIN_WIN_SLACK_STEPS: u32 = 120;

/// Step at which a run would still win with its target raised by the fill
/// margin (the D-03 fallback margin rule).
///
/// `counts[i]` is the goal count after step `i + 1`. The counts are replayed
/// through a second `GoalTracker` with target
/// `target_count + fill_margin_count(full_count)`, the same `settle_steps`,
/// and a step limit `MARGIN_WIN_SLACK_STEPS` before `step_limit`. A rule over
/// the latched settle window alone would need a one-step inflow larger than
/// the margin, which a pour never produces. Returns `None` when that tracker
/// does not win within `counts` or the configuration is invalid.
#[cfg(test)]
pub(crate) fn maybe_margin_win_step(
    counts: &[u32],
    target_count: u32,
    full_count: u32,
    settle_steps: u32,
    step_limit: u32,
) -> Option<u32> {
    let raised_target = target_count.checked_add(fill_margin_count(full_count))?;
    let deadline = step_limit.checked_sub(MARGIN_WIN_SLACK_STEPS)?;
    let mut tracker = GoalTracker::new(raised_target, settle_steps, deadline)?;
    for (step, &count) in (1_u32..).zip(counts) {
        match tracker.observe(count, step) {
            Outcome::Running => {}
            Outcome::Won { step } => return Some(step),
            Outcome::Lost { .. } => return None,
        }
    }
    None
}

/// Puzzle outcome decided per fixed step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// No win yet and the step limit has not passed.
    Running,
    /// The settle hold completed at `step`, at or before the step limit.
    Won {
        /// Step at which the hold completed.
        step: u32,
    },
    /// The step limit passed without a win; `step` is the step limit.
    Lost {
        /// The level's step limit.
        step: u32,
    },
}

impl Outcome {
    /// Frame code: 0 running, 1 won, 2 lost (D-17).
    pub(crate) const fn code(self) -> u8 {
        match self {
            Self::Running => 0,
            Self::Won { .. } => 1,
            Self::Lost { .. } => 2,
        }
    }

    /// Latched step, or `None` while running.
    pub(crate) const fn maybe_step(self) -> Option<u32> {
        match self {
            Self::Running => None,
            Self::Won { step } | Self::Lost { step } => Some(step),
        }
    }
}

/// Step-latched win/loss tracker for one puzzle run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GoalTracker {
    target: u32,
    settle_steps: u32,
    step_limit: u32,
    held: u32,
    outcome: Outcome,
}

impl GoalTracker {
    /// None unless 1 <= target, 1 <= `settle_steps` <= `step_limit`.
    pub(crate) const fn new(target: u32, settle_steps: u32, step_limit: u32) -> Option<Self> {
        if target == 0 || settle_steps == 0 || settle_steps > step_limit {
            return None;
        }
        Some(Self {
            target,
            settle_steps,
            step_limit,
            held: 0,
            outcome: Outcome::Running,
        })
    }

    /// Records the goal count after session step `step` and returns the outcome.
    ///
    /// A win needs `count >= target` for `settle_steps` consecutive
    /// observations, completing at or before the step limit; the win is
    /// checked before the loss. Once latched, the outcome never changes.
    pub(crate) fn observe(&mut self, count: u32, step: u32) -> Outcome {
        if self.outcome != Outcome::Running {
            return self.outcome;
        }
        self.held = if count >= self.target {
            self.held.saturating_add(1)
        } else {
            0
        };
        if self.held >= self.settle_steps && step <= self.step_limit {
            self.outcome = Outcome::Won { step };
        } else if step >= self.step_limit {
            self.outcome = Outcome::Lost {
                step: self.step_limit,
            };
        }
        self.outcome
    }

    /// Current outcome without observing a step.
    pub(crate) const fn outcome(&self) -> Outcome {
        self.outcome
    }
}

/// What a puzzle session reports after each fixed step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PuzzleStatus {
    /// Counted particles strictly inside the cup interior.
    pub(crate) goal_count: u32,
    /// Count that must hold for the settle window.
    pub(crate) target_count: u32,
    /// Authored count for 100% fill.
    pub(crate) full_count: u32,
    /// Latched or running outcome.
    pub(crate) outcome: Outcome,
    /// Session step this status was observed at.
    pub(crate) observed_step: u32,
    /// Level step limit.
    pub(crate) step_limit: u32,
}

#[cfg(test)]
mod tests;

//! Peak held fill, the Brim latch and the decision step (D-08 to D-10).
//!
//! Integer-only and step-driven like `GoalTracker`: callers pass the goal
//! count and the session step, so one Rust quantity drives Win, Brim, the
//! decision step and the win-card percentage at every playback speed.

use std::collections::VecDeque;

use super::goal::Outcome;

/// Running maximum of the trailing `settle_steps` minimum, frozen after
/// `step_limit` (D-08), plus the Brim latch (D-09).
///
/// The peak is the largest `c` such that `count >= c` held for `settle_steps`
/// consecutive steps at or before the step limit, so a splash shorter than
/// the window never raises it and it never decreases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PeakHeldTracker {
    window: VecDeque<u32>,
    settle_steps: usize,
    step_limit: u32,
    brim_count: u32,
    peak: u32,
    maybe_brim_step: Option<u32>,
}

impl PeakHeldTracker {
    /// None unless 1 <= `settle_steps` <= `step_limit` and `brim_count` >= 1.
    pub(crate) fn new(settle_steps: u32, step_limit: u32, brim_count: u32) -> Option<Self> {
        if settle_steps == 0 || settle_steps > step_limit || brim_count == 0 {
            return None;
        }
        let window_len = usize::try_from(settle_steps).ok()?;
        Some(Self {
            window: VecDeque::with_capacity(window_len),
            settle_steps: window_len,
            step_limit,
            brim_count,
            peak: 0,
            maybe_brim_step: None,
        })
    }

    /// Records the goal count after session step `step`; ignores steps after
    /// the limit.
    ///
    /// The window holds the last `settle_steps` counts. Once it is full, its
    /// minimum is a held count and raises the peak; Brim latches at the first
    /// step whose peak reaches `brim_count` and never moves afterwards.
    pub(crate) fn observe(&mut self, count: u32, step: u32) {
        if step > self.step_limit {
            return;
        }
        if self.window.len() == self.settle_steps {
            self.window.pop_front();
        }
        self.window.push_back(count);
        if self.window.len() < self.settle_steps {
            return;
        }
        let Some(&held) = self.window.iter().min() else {
            return;
        };
        self.peak = self.peak.max(held);
        if self.maybe_brim_step.is_none() && self.peak >= self.brim_count {
            self.maybe_brim_step = Some(step);
        }
    }

    /// Largest count held for the full settle window so far.
    pub(crate) const fn peak(&self) -> u32 {
        self.peak
    }

    /// Step at which the peak first reached `brim_count`, if it has.
    pub(crate) const fn maybe_brim_step(&self) -> Option<u32> {
        self.maybe_brim_step
    }
}

/// When the run is decided (D-10): at the loss latch, at the Brim latch, or
/// at the step limit for a win without Brim. `None` while undecided.
pub(crate) const fn decided_step(
    outcome: Outcome,
    maybe_brim_step: Option<u32>,
    observed_step: u32,
    step_limit: u32,
) -> Option<u32> {
    match (outcome, maybe_brim_step) {
        (Outcome::Lost { step }, _) | (Outcome::Won { .. }, Some(step)) => Some(step),
        (Outcome::Won { .. }, None) if observed_step >= step_limit => Some(step_limit),
        (Outcome::Running, _) | (Outcome::Won { .. }, None) => None,
    }
}

#[cfg(test)]
mod tests;

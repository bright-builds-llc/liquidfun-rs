//! Optional puzzle scalars on `ProofFrame` (D-17, Phase 39 D-11); undefined
//! in JS for scene frames.
//!
//! `status_is_consistent` is the one Rust rule set that a puzzle status must
//! satisfy before it rides on a frame. `web/src/physics/frame-puzzle.ts`
//! mirrors the same rules when it parses these scalars.

use wasm_bindgen::prelude::*;

use super::ProofFrame;
use crate::puzzle::goal::{Outcome, PuzzleStatus};
use crate::puzzle::peak;

#[wasm_bindgen]
impl ProofFrame {
    /// Returns the counted particles strictly inside the cup, or undefined for scenes.
    #[must_use]
    #[wasm_bindgen(js_name = goalCount)]
    pub fn goal_count(&self) -> Option<u32> {
        self.data.maybe_puzzle.map(|status| status.goal_count)
    }

    /// Returns the count that must hold for the settle window, or undefined for scenes.
    #[must_use]
    #[wasm_bindgen(js_name = goalTarget)]
    pub fn goal_target(&self) -> Option<u32> {
        self.data.maybe_puzzle.map(|status| status.target_count)
    }

    /// Returns the authored count for 100% fill, or undefined for scenes.
    #[must_use]
    #[wasm_bindgen(js_name = goalFull)]
    pub fn goal_full(&self) -> Option<u32> {
        self.data.maybe_puzzle.map(|status| status.full_count)
    }

    /// Returns 0 while running, 1 once won and 2 once lost, or undefined for scenes.
    #[must_use]
    #[wasm_bindgen(js_name = outcome)]
    pub fn outcome(&self) -> Option<u8> {
        self.data.maybe_puzzle.map(|status| status.outcome.code())
    }

    /// Returns the latched outcome step, or undefined while running and for scenes.
    #[must_use]
    #[wasm_bindgen(js_name = outcomeStep)]
    pub fn outcome_step(&self) -> Option<u32> {
        self.data
            .maybe_puzzle
            .and_then(|status| status.outcome.maybe_step())
    }

    /// Returns the level step limit, or undefined for scenes.
    #[must_use]
    #[wasm_bindgen(js_name = stepLimit)]
    pub fn step_limit(&self) -> Option<u32> {
        self.data.maybe_puzzle.map(|status| status.step_limit)
    }

    /// Returns the largest cup count held for the settle window, or undefined for scenes.
    #[must_use]
    #[wasm_bindgen(js_name = peakHeldCount)]
    pub fn peak_held_count(&self) -> Option<u32> {
        self.data.maybe_puzzle.map(|status| status.peak_held_count)
    }

    /// Returns the Brim star count, or undefined for scenes.
    #[must_use]
    #[wasm_bindgen(js_name = brimCount)]
    pub fn brim_count(&self) -> Option<u32> {
        self.data.maybe_puzzle.map(|status| status.brim_count)
    }

    /// Returns the step at which Brim latched, or undefined before that and for scenes.
    #[must_use]
    #[wasm_bindgen(js_name = brimStep)]
    pub fn brim_step(&self) -> Option<u32> {
        self.data
            .maybe_puzzle
            .and_then(|status| status.maybe_brim_step)
    }

    /// Returns the step at which the run was decided, or undefined before that and for scenes.
    #[must_use]
    #[wasm_bindgen(js_name = decidedStep)]
    pub fn decided_step(&self) -> Option<u32> {
        self.data
            .maybe_puzzle
            .and_then(|status| status.maybe_decided_step)
    }
}

/// Every rule a puzzle status must satisfy to attach to a frame at
/// `step_index` holding `particle_count` particles.
///
/// The peak held count is never compared with `particle_count`, because
/// eviction can remove particles after the peak was held (Pitfall 4).
pub(super) fn status_is_consistent(
    status: &PuzzleStatus,
    particle_count: u32,
    step_index: u32,
) -> bool {
    counts_are_consistent(status, particle_count)
        && steps_are_consistent(status, step_index)
        && stars_are_consistent(status)
}

/// `0 < target < brim <= full`, a non-zero limit and a live cup count.
const fn counts_are_consistent(status: &PuzzleStatus, particle_count: u32) -> bool {
    status.target_count != 0
        && status.target_count < status.brim_count
        && status.brim_count <= status.full_count
        && status.step_limit != 0
        && status.goal_count <= particle_count
}

/// The hooks' counter matches the frame (D-14); a latch is never in the
/// future or past the limit, and a loss latches exactly at the limit.
fn steps_are_consistent(status: &PuzzleStatus, step_index: u32) -> bool {
    let latch_is_in_range = status
        .outcome
        .maybe_step()
        .is_none_or(|step| step <= status.step_limit && step <= status.observed_step);
    let loss_is_at_limit = match status.outcome {
        Outcome::Lost { step } => step == status.step_limit,
        Outcome::Running | Outcome::Won { .. } => true,
    };
    status.observed_step == step_index && latch_is_in_range && loss_is_at_limit
}

/// Win iff peak reaches target; Brim iff peak reaches brim, only on a won
/// run between the win and the current step; decision per D-10.
fn stars_are_consistent(status: &PuzzleStatus) -> bool {
    let is_won = matches!(status.outcome, Outcome::Won { .. });
    let win_matches_peak = is_won == (status.peak_held_count >= status.target_count);
    let brim_matches_peak =
        status.maybe_brim_step.is_some() == (status.peak_held_count >= status.brim_count);
    let latest_brim_step = status.observed_step.min(status.step_limit);
    let brim_step_is_in_range = match (status.maybe_brim_step, status.outcome) {
        (None, _) => true,
        (Some(brim_step), Outcome::Won { step }) => {
            step <= brim_step && brim_step <= latest_brim_step
        }
        (Some(_), Outcome::Running | Outcome::Lost { .. }) => false,
    };
    let expected_decided_step = peak::decided_step(
        status.outcome,
        status.maybe_brim_step,
        status.observed_step,
        status.step_limit,
    );
    win_matches_peak
        && brim_matches_peak
        && brim_step_is_in_range
        && status.maybe_decided_step == expected_decided_step
}

#[cfg(test)]
mod tests;

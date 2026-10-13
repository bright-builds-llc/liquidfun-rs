//! Scene hooks for a built puzzle level: per-step cup counting, the
//! outcome latch (D-11 to D-15), the peak held fill (Phase 39 D-08 to D-10)
//! and the timed valves (Phase 40 D-01, D-03).

use liquidfun::math::Vec2;
use liquidfun::{BodyId, ContactTransition, ParticleGroupId, ParticleSystemId, World};

use super::goal::{GoalTracker, PuzzleStatus, count_in_goal};
use super::peak::{PeakHeldTracker, decided_step};
use super::spec::LevelSpec;
use super::valve::ValveGate;
use crate::scene::{ControlEffect, PointerKind, RigidSegment, SceneHooks};
use crate::session::SessionError;

/// The level's rigid parts the hooks draw or open after the build.
pub(crate) struct LevelBodies {
    /// Walls, cup walls and non-valve tools; closed valves add their own.
    pub(crate) segments: Vec<RigidSegment>,
    /// Gates in canonical placement order.
    pub(crate) valves: Vec<ValveGate>,
    /// Level float bodies with their radii, in spec order (D-12).
    pub(crate) floats: Vec<(BodyId, f32)>,
}

/// Hooks that count the cup once per fixed step and reject scene input.
pub(crate) struct PuzzleHooks {
    spec: &'static LevelSpec,
    counted_groups: Vec<ParticleGroupId>,
    tracker: GoalTracker,
    peak: PeakHeldTracker,
    goal_count: u32,
    observed_step: u32,
    /// Walls, cup walls and non-valve tools; closed valves add their own.
    segments: Vec<RigidSegment>,
    /// Gates in canonical placement order.
    valves: Vec<ValveGate>,
    /// Level float bodies with their radii, in spec order.
    floats: Vec<(BodyId, f32)>,
}

impl PuzzleHooks {
    /// Hooks for a freshly built level at step 0 with its initial cup count.
    pub(crate) fn new(
        spec: &'static LevelSpec,
        counted_groups: Vec<ParticleGroupId>,
        tracker: GoalTracker,
        peak: PeakHeldTracker,
        initial_count: u32,
        bodies: LevelBodies,
    ) -> Self {
        let LevelBodies {
            segments,
            valves,
            floats,
        } = bodies;
        Self {
            spec,
            counted_groups,
            tracker,
            peak,
            goal_count: initial_count,
            observed_step: 0,
            segments,
            valves,
            floats,
        }
    }
}

impl SceneHooks for PuzzleHooks {
    /// Opens every valve due for the step about to run by destroying its body.
    ///
    /// Runs before eviction and `world.step` (`session.rs`), so step
    /// `open_tick × 30` is the first step without the gate (D-03). Uses only
    /// the own step counter, never wall-clock time.
    fn on_advance(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        let next_step = self
            .observed_step
            .checked_add(1)
            .ok_or(SessionError::StepIndexExhausted)?;
        for valve in self
            .valves
            .iter_mut()
            .filter(|valve| valve.is_due(next_step))
        {
            let Some(body) = valve.take_body() else {
                continue;
            };
            world
                .destroy_body(body)
                .map_err(|_error| SessionError::StepFailed)?;
        }
        Ok(())
    }

    /// Counts the cup and feeds the outcome and peak trackers.
    ///
    /// `SessionCore::advance` calls this once per fixed step, after
    /// `world.step` and eviction. The hooks are rebuilt at step 0 (D-10), so
    /// the own counter equals the session step index after every step (D-14).
    /// Nothing here reads wall-clock time.
    fn on_after_step(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
        _transitions: &[ContactTransition],
    ) -> Result<(), SessionError> {
        self.observed_step = self
            .observed_step
            .checked_add(1)
            .ok_or(SessionError::StepIndexExhausted)?;
        let view = world
            .particle_system_view(system)
            .map_err(|_error| SessionError::StepFailed)?;
        self.goal_count = count_in_goal(
            view.positions(),
            view.group_ids(),
            &self.counted_groups,
            self.spec.cup_interior,
        );
        self.tracker.observe(self.goal_count, self.observed_step);
        self.peak.observe(self.goal_count, self.observed_step);
        Ok(())
    }

    fn maybe_puzzle_status(&self) -> Option<PuzzleStatus> {
        let outcome = self.tracker.outcome();
        let maybe_brim_step = self.peak.maybe_brim_step();
        Some(PuzzleStatus {
            goal_count: self.goal_count,
            target_count: self.spec.target_count,
            full_count: self.spec.full_count,
            outcome,
            observed_step: self.observed_step,
            step_limit: self.spec.step_limit,
            peak_held_count: self.peak.peak(),
            brim_count: self.spec.brim_count,
            maybe_brim_step,
            maybe_decided_step: decided_step(
                outcome,
                maybe_brim_step,
                self.observed_step,
                self.spec.step_limit,
            ),
        })
    }

    /// Puzzles take no scene controls (D-15).
    fn apply_control(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
        _name: &str,
        _value: &str,
    ) -> Result<ControlEffect, SessionError> {
        Err(SessionError::UnknownControl)
    }

    /// Puzzles take no scene actions (D-15).
    fn apply_action(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
        _name: &str,
    ) -> Result<(), SessionError> {
        Err(SessionError::UnknownControl)
    }

    /// The bridge never edits a live puzzle world (D-10, D-15).
    fn apply_pointer(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
        _kind: PointerKind,
        _world_x: f32,
        _world_y: f32,
    ) -> Result<(), SessionError> {
        Err(SessionError::UnknownControl)
    }

    /// Static segments, then each closed valve's edges in canonical order.
    fn collect_segments(&self, _world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        let closed_valves = self.valves.iter().flat_map(ValveGate::closed_segments);
        Ok(self.segments.iter().chain(closed_valves).copied().collect())
    }

    /// Each level float's live centre and radius, in spec order.
    fn collect_circles(&self, world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        self.floats
            .iter()
            .map(|&(body, radius)| {
                let position = world
                    .body_snapshot(body)
                    .map_err(|_error| SessionError::FrameCaptureFailed)?
                    .position();
                Ok((position, radius))
            })
            .collect()
    }
}

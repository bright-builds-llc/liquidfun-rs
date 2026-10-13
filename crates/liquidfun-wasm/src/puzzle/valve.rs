//! Timed valve scheduling and the built gate record (Phase 40 D-01 to D-03).
//!
//! A valve is a static body from step 0. `PuzzleHooks::on_advance` destroys
//! it before `world.step` runs step [`open_step`], keyed only to the hooks'
//! own step counter.

use liquidfun::BodyId;

use crate::scene::RigidSegment;

/// Fixed steps per half-second open tick (D-02).
pub(crate) const STEPS_PER_OPEN_TICK: u32 = 30;

/// Session step whose `world.step` runs without the gate (D-03).
pub(crate) fn open_step(open_tick: u8) -> u32 {
    u32::from(open_tick) * STEPS_PER_OPEN_TICK
}

/// One built valve: its static body until opened, and its four outline edges.
#[derive(Debug)]
pub(crate) struct ValveGate {
    open_step: u32,
    maybe_body: Option<BodyId>,
    segments: [RigidSegment; 4],
}

impl ValveGate {
    /// A closed gate that opens before step `open_tick × 30`.
    pub(crate) fn new(open_tick: u8, body: BodyId, segments: [RigidSegment; 4]) -> Self {
        Self {
            open_step: open_step(open_tick),
            maybe_body: Some(body),
            segments,
        }
    }

    /// Whether the gate is still closed and must open before `next_step` runs.
    pub(crate) fn is_due(&self, next_step: u32) -> bool {
        self.maybe_body.is_some() && next_step >= self.open_step
    }

    /// Opens the gate, returning its body for destruction; `None` once open.
    pub(crate) fn take_body(&mut self) -> Option<BodyId> {
        self.maybe_body.take()
    }

    /// The four outline edges while closed; empty once opened.
    pub(crate) fn closed_segments(&self) -> &[RigidSegment] {
        if self.maybe_body.is_some() {
            &self.segments
        } else {
            &[]
        }
    }

    /// Whether the gate body still exists.
    #[cfg(test)]
    pub(crate) const fn is_closed(&self) -> bool {
        self.maybe_body.is_some()
    }
}

/// First-pour with one valve allowed and a build area reaching under the
/// reservoir outlet, for valve tests only.
#[cfg(test)]
pub(crate) const VALVE_TEST_SPEC: super::spec::LevelSpec = super::spec::LevelSpec {
    tool_budget: &[
        super::tools::ToolAllowance {
            kind: super::tools::ToolKind::Plank,
            count: 2,
        },
        super::tools::ToolAllowance {
            kind: super::tools::ToolKind::Valve,
            count: 1,
        },
    ],
    build_area: super::tools::GridRect::new(-40, 40, 40, 90),
    ..super::first_pour::FIRST_POUR
};

/// A valve centred 0.125 m under the first-pour outlet (x −0.8 m, y 4.35 m),
/// opening at tick 4 (step 120).
#[cfg(test)]
pub(crate) const REFERENCE_VALVE_POSE: [i32; 4] = [-16, 87, 0, 4];

#[cfg(test)]
mod tests;

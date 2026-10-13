//! Read-only audio aggregates for puzzle frames (Phase 42 D-07).
//!
//! Every value is computed from data each fixed step already produces: the
//! particle system's body contacts and the step's own `StepReport`. The tally
//! only borrows the world and the report, so it never writes to the
//! simulation and cannot change stepping, contact order or any float result
//! (D-12).

use std::collections::HashSet;
use std::hash::{BuildHasherDefault, DefaultHasher};

use liquidfun::{
    ContactTransitionKind, FixtureId, ManagedContactSnapshot, ParticleId, ParticleSystemId,
    StepReport, World,
};

use crate::session::SessionError;

/// One particle touching one fixture.
type ContactKey = (ParticleId, FixtureId);

/// Fixed-key hasher, so membership is reproducible. The set is only queried
/// for membership and never iterated to produce output.
type ContactSet = HashSet<ContactKey, BuildHasherDefault<DefaultHasher>>;

/// Read-only audio aggregates for one fixed step (Phase 42 D-07).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct SoundSignals {
    /// Particle-fixture contacts present this step that were absent the
    /// previous step.
    pub(crate) splash_contacts: u32,
    /// Non-sensor rigid contacts with a `ContactTransitionKind::Begin`
    /// transition this step.
    pub(crate) impact_count: u32,
    /// Largest summed normal impulse (each point clamped to `>= 0`) among
    /// those contacts, matched in `contact_solves()` chained with
    /// `continuous_contact_solves()` by fixtures and child indices; 0.0 when
    /// there is none.
    pub(crate) impact_impulse: f32,
}

/// Per-session tally that observes the world after each step and never
/// writes to it.
#[derive(Default)]
pub(crate) struct SoundTally {
    /// Body contacts seen after the previous step.
    previous: ContactSet,
    /// Reused buffer for the current step's body contacts.
    scratch: ContactSet,
    /// Aggregates for the last observed step.
    signals: SoundSignals,
}

impl SoundTally {
    /// Records the aggregates for the step that produced `report`.
    ///
    /// # Errors
    ///
    /// Returns `SessionError::StepFailed` when the particle system view is
    /// unavailable. The session records the failure detail.
    pub(crate) fn observe(
        &mut self,
        world: &World,
        system: ParticleSystemId,
        report: &StepReport,
    ) -> Result<(), SessionError> {
        let view = world
            .particle_system_view(system)
            .map_err(|_error| SessionError::StepFailed)?;
        self.scratch.clear();
        let mut fresh = 0_usize;
        // Count in the view's deterministic order; the sets are membership only.
        for contact in view.body_contacts() {
            let key = (contact.particle(), contact.fixture());
            if !self.previous.contains(&key) {
                fresh += 1;
            }
            self.scratch.insert(key);
        }
        std::mem::swap(&mut self.previous, &mut self.scratch);
        let (impact_count, impact_impulse) = rigid_impacts(report);
        self.signals = SoundSignals {
            splash_contacts: u32::try_from(fresh).unwrap_or(u32::MAX),
            impact_count,
            impact_impulse,
        };
        Ok(())
    }

    /// Aggregates for the last observed step; all zero before the first step.
    pub(crate) const fn signals(&self) -> SoundSignals {
        self.signals
    }
}

/// Count and strongest impulse of non-sensor rigid contacts that began
/// touching in the step that produced `report`.
fn rigid_impacts(report: &StepReport) -> (u32, f32) {
    let mut count = 0_u32;
    let mut strongest = 0.0_f32;
    for transition in report.contact_transitions() {
        let contact = transition.contact();
        if transition.kind() != ContactTransitionKind::Begin || contact.is_sensor() {
            continue;
        }
        count = count.saturating_add(1);
        strongest = strongest.max(begin_impulse(report, contact));
    }
    (count, strongest)
}

/// Largest finite summed normal impulse among the step's solves of `contact`.
fn begin_impulse(report: &StepReport, contact: &ManagedContactSnapshot) -> f32 {
    strongest_finite_impulse(
        report
            .contact_solves()
            .iter()
            .chain(report.continuous_contact_solves())
            .map(liquidfun::ContactSolve::contact)
            .filter(|solved| is_same_contact(solved, contact))
            .map(summed_normal_impulse),
    )
}

/// Largest finite impulse; 0.0 when there is none.
///
/// A non-finite impulse is sound-only data, so it is dropped rather than
/// rejected by `FrameData::set_puzzle_sound`, which would fail the whole run
/// (D-08). The impact itself still counts.
fn strongest_finite_impulse(impulses: impl IntoIterator<Item = f32>) -> f32 {
    impulses
        .into_iter()
        .filter(|impulse| impulse.is_finite())
        .fold(0.0_f32, f32::max)
}

/// Whether two snapshots describe the same fixture pair and shape children.
fn is_same_contact(left: &ManagedContactSnapshot, right: &ManagedContactSnapshot) -> bool {
    left.fixtures() == right.fixtures() && left.child_indices() == right.child_indices()
}

/// Sum of the contact's point normal impulses, each clamped to `>= 0`.
fn summed_normal_impulse(contact: &ManagedContactSnapshot) -> f32 {
    contact
        .points()
        .iter()
        .map(|point| point.normal_impulse().max(0.0))
        .sum()
}

#[cfg(test)]
mod tests;

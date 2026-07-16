//! Source-timed particle lifetime and zombie effects inside the World transaction.

use crate::particle::lifetime::ParticleLifecycleError;
use crate::{DestructionCause, DestructionRecord, ParticleId, World};

use super::step::{ContactHookRun, StepError};

impl World {
    #[allow(
        clippy::needless_continue,
        reason = "the paused early-return seam is intentional and receives active contact work in the next Phase 9 plans"
    )]
    pub(super) fn run_particle_lifecycle_step<H: super::step::CollisionDecisionHook>(
        &mut self,
        time_step: f32,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        let mut candidate = self.particle_systems.clone();
        let mut requested_records = Vec::new();

        for system_id in self.particle_system_order.iter().copied() {
            let system = candidate
                .get_mut(system_id)
                .expect("world particle-system order contains only live systems");
            system
                .storage
                .synchronize_zombie_flags()
                .map_err(|_error| StepError::ParticleLifecycleInvariant)?;
            let expired = system
                .lifetime
                .solve_lifetimes(&mut system.storage, time_step)
                .map_err(particle_step_error)?;
            let outcome =
                crate::particle::lifetime::compact_pending_with_occurrences(&mut system.storage)
                    .map_err(|_error| StepError::ParticleLifecycleInvariant)?;

            append_requested_records(&mut requested_records, outcome, &expired);

            if system.definition.is_paused() {
                continue;
            }

            // Contact generation and rigid coupling are integrated by the
            // subsequent Phase 9 plans at this source-timed active-system seam.
        }

        hook_run.ensure_lifecycle_capacity(requested_records.len())?;
        for record in requested_records {
            hook_run.record_particle_destruction(record)?;
        }
        self.particle_systems = candidate;
        Ok(())
    }
}

fn append_requested_records(
    records: &mut Vec<DestructionRecord>,
    outcome: crate::particle::lifetime::ParticleCompactionOutcome,
    expired: &[crate::particle::storage::ParticleSnapshot],
) {
    for occurrence in outcome.requested_listener_occurrences {
        let particle = occurrence.particle();
        let snapshot = outcome
            .destroyed
            .iter()
            .find(|snapshot| snapshot.id == particle)
            .copied()
            .expect("a listener occurrence always names a destroyed snapshot");
        let cause = if contains_particle(expired, particle) {
            DestructionCause::ParticleExpiration
        } else {
            DestructionCause::Explicit
        };
        records.push(World::particle_destruction_record(snapshot, cause));
    }
}

fn contains_particle(
    snapshots: &[crate::particle::storage::ParticleSnapshot],
    particle: ParticleId,
) -> bool {
    snapshots.iter().any(|snapshot| snapshot.id == particle)
}

fn particle_step_error(error: ParticleLifecycleError) -> StepError {
    match error {
        ParticleLifecycleError::Lifetime(error) => StepError::ParticleLifetime(error),
        ParticleLifecycleError::Storage(_)
        | ParticleLifecycleError::CapacityExceeded { .. }
        | ParticleLifecycleError::OldestRankOutOfRange => StepError::ParticleLifecycleInvariant,
    }
}

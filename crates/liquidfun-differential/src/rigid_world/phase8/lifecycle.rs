//! Ordered Phase 8 lifecycle projection.

use liquidfun::{DestroyedId, LifecycleEvent, StepLifecycleEvent, StepReport};
use liquidfun_test_protocol::{
    RigidLifecycleObservation, RigidLifecycleObservationKind, RigidWorldObservation,
    RigidWorldWitnessFamily, ScenarioId,
};

use super::super::{NativeRigidWorldError, TimelineExecutor};

pub(super) fn collect_step_lifecycle(
    executor: &mut TimelineExecutor,
    report: &StepReport,
) -> Result<(), NativeRigidWorldError> {
    if !captures_lifecycle_evidence(executor.family) {
        return Ok(());
    }
    collect_mutation_lifecycle(executor, report.lifecycle())
}

pub(super) fn collect_mutation_lifecycle(
    executor: &mut TimelineExecutor,
    lifecycle: &[LifecycleEvent],
) -> Result<(), NativeRigidWorldError> {
    if !captures_lifecycle_evidence(executor.family) {
        return Ok(());
    }
    for event in lifecycle {
        match event {
            StepLifecycleEvent::Filter(filter) => {
                let fixture = executor.semantic_fixture(filter.fixtures()[0])?;
                push_lifecycle(
                    executor,
                    RigidLifecycleObservationKind::FilterDecision,
                    None,
                    Some(fixture),
                )?;
            }
            StepLifecycleEvent::Contact(transition) => {
                collect_contact_lifecycle(executor, transition)?;
            }
            StepLifecycleEvent::ContactDestruction(transition) => {
                let contact = executor.contact_identity(transition.contact())?;
                push_lifecycle(
                    executor,
                    RigidLifecycleObservationKind::EndContact,
                    Some(contact.clone()),
                    None,
                )?;
                push_lifecycle(
                    executor,
                    RigidLifecycleObservationKind::ContactDestroyed,
                    Some(contact),
                    None,
                )?;
            }
            StepLifecycleEvent::Hook(event) if event.maybe_pre_solve().is_some() => {
                let contact = executor.contact_identity(event.contact())?;
                push_lifecycle(
                    executor,
                    RigidLifecycleObservationKind::PreSolve,
                    Some(contact),
                    None,
                )?;
            }
            StepLifecycleEvent::Solve(solve) | StepLifecycleEvent::ContinuousSolve(solve) => {
                let contact = executor.contact_identity(solve.contact())?;
                push_lifecycle(
                    executor,
                    RigidLifecycleObservationKind::PostSolve,
                    Some(contact),
                    None,
                )?;
            }
            StepLifecycleEvent::JointGoodbye(record) => {
                let entity = semantic_destroyed(executor, record.destroyed())?;
                push_lifecycle(
                    executor,
                    RigidLifecycleObservationKind::JointGoodbye,
                    None,
                    Some(entity),
                )?;
            }
            StepLifecycleEvent::FixtureGoodbye(record) => {
                let entity = semantic_destroyed(executor, record.destroyed())?;
                push_lifecycle(
                    executor,
                    RigidLifecycleObservationKind::FixtureGoodbye,
                    None,
                    Some(entity),
                )?;
            }
            StepLifecycleEvent::Destruction(record)
                if matches!(record.destroyed(), DestroyedId::Body(_)) =>
            {
                let entity = semantic_destroyed(executor, record.destroyed())?;
                push_lifecycle(
                    executor,
                    RigidLifecycleObservationKind::BodyDestroyed,
                    None,
                    Some(entity),
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn collect_contact_lifecycle(
    executor: &mut TimelineExecutor,
    transition: &liquidfun::ContactTransition,
) -> Result<(), NativeRigidWorldError> {
    let maybe_kind = match transition.kind() {
        liquidfun::ContactTransitionKind::Begin => {
            Some(RigidLifecycleObservationKind::BeginContact)
        }
        liquidfun::ContactTransitionKind::End => Some(RigidLifecycleObservationKind::EndContact),
        _ => None,
    };
    let Some(kind) = maybe_kind else {
        return Ok(());
    };
    let occurrence = transition.contact().differential_occurrence();
    let contact = executor.contact_identity(transition.contact())?;
    if kind == RigidLifecycleObservationKind::BeginContact
        && !executor.seen_lifecycle_occurrences.contains(&occurrence)
    {
        executor.seen_lifecycle_occurrences.push(occurrence);
        push_lifecycle(
            executor,
            RigidLifecycleObservationKind::ContactCreated,
            Some(contact.clone()),
            None,
        )?;
    }
    push_lifecycle(executor, kind, Some(contact), None)
}

fn captures_lifecycle_evidence(family: RigidWorldWitnessFamily) -> bool {
    matches!(
        family,
        RigidWorldWitnessFamily::ContactFilterListenerAndPreSolveTiming
            | RigidWorldWitnessFamily::DestructionListenerAndDependencyCascades
    )
}

fn push_lifecycle(
    executor: &mut TimelineExecutor,
    kind: RigidLifecycleObservationKind,
    maybe_contact: Option<liquidfun_test_protocol::RigidContactIdentity>,
    maybe_entity_id: Option<ScenarioId>,
) -> Result<(), NativeRigidWorldError> {
    let ordinal = executor.next_lifecycle_ordinal;
    executor.next_lifecycle_ordinal =
        ordinal
            .checked_add(1)
            .ok_or_else(|| NativeRigidWorldError::Declaration {
                checkpoint_id: "lifecycle-ordinal".into(),
                message: "lifecycle ordinal exceeded the protocol representation".into(),
            })?;
    executor
        .semantic_observations
        .push(RigidWorldObservation::Lifecycle {
            event: RigidLifecycleObservation {
                ordinal,
                kind,
                maybe_contact,
                maybe_entity_id,
            },
        });
    Ok(())
}

pub(super) fn remove_destroyed_mapping(executor: &mut TimelineExecutor, destroyed: DestroyedId) {
    if let DestroyedId::Joint(joint) = destroyed {
        executor.joints.retain(|(_, value)| *value != joint);
    }
}

fn semantic_destroyed(
    executor: &TimelineExecutor,
    destroyed: DestroyedId,
) -> Result<ScenarioId, NativeRigidWorldError> {
    match destroyed {
        DestroyedId::Body(id) => executor.semantic_body(id),
        DestroyedId::Fixture(id) => executor.semantic_fixture(id),
        DestroyedId::Joint(id) => executor.semantic_joint(id),
        _ => Err(NativeRigidWorldError::Declaration {
            checkpoint_id: "lifecycle-map".into(),
            message: "unsupported lifecycle entity".into(),
        }),
    }
}

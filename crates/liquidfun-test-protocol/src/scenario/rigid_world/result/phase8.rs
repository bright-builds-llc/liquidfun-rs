use std::collections::HashSet;

use super::{
    RIGID_RAY_INITIAL_MAX_FRACTION_BITS, RigidCheckpointLiveIdentities, RigidJointSnapshot,
    RigidLifecycleObservation, RigidLifecycleObservationKind, RigidQueryCompletion,
    RigidQueryObservation, RigidRayCompletion, RigidRayHitObservation, RigidRayObservation,
    RigidReconstructionKind, RigidReconstructionSupport, RigidWorldAction, RigidWorldDecodeError,
    RigidWorldErrorKind, RigidWorldObservation, RigidWorldWitnessFamily, validation,
};
use crate::{
    RigidFixtureShape, RigidQueryDirective, RigidQueryDirectiveRule, RigidRayDirective,
    RigidRayDirectiveRule, RigidWorldActionRecord, ScenarioId, Vec2Bits,
};
pub(super) fn validate_phase8_observation_contract(
    family: RigidWorldWitnessFamily,
    actions: &[RigidWorldActionRecord],
    observations: &[RigidWorldObservation],
) -> Result<(), RigidWorldDecodeError> {
    if !RigidWorldWitnessFamily::PHASE8_REQUIRED.contains(&family) {
        return Ok(());
    }
    let lifecycle = observations
        .iter()
        .filter_map(|observation| match observation {
            RigidWorldObservation::Lifecycle { event } => Some(event),
            _ => None,
        })
        .collect::<Vec<_>>();
    if lifecycle.iter().enumerate().any(|(ordinal, event)| {
        event.ordinal != ordinal as u32 || !lifecycle_identity_shape_is_valid(event)
    }) {
        return Err(validation(RigidWorldErrorKind::ResultObservationMismatch));
    }
    if observations.iter().any(|observation| match observation {
        RigidWorldObservation::Joint { snapshot } => !joint_snapshot_is_finite(snapshot),
        RigidWorldObservation::Rope { snapshot } => snapshot.vertices.iter().any(|vertex| {
            !vertex.x_bits.to_f32().is_finite() || !vertex.y_bits.to_f32().is_finite()
        }),
        RigidWorldObservation::Diagnostics { snapshot } => {
            !snapshot.tree_quality_bits.to_f32().is_finite()
        }
        _ => false,
    }) {
        return Err(validation(RigidWorldErrorKind::ResultObservationMismatch));
    }

    match family {
        RigidWorldWitnessFamily::ContactFilterListenerAndPreSolveTiming => {
            let kinds = lifecycle.iter().map(|event| event.kind).collect::<Vec<_>>();
            let required = [
                RigidLifecycleObservationKind::FilterDecision,
                RigidLifecycleObservationKind::ContactCreated,
                RigidLifecycleObservationKind::BeginContact,
                RigidLifecycleObservationKind::PreSolve,
                RigidLifecycleObservationKind::PostSolve,
            ];
            if !ordered_kinds_include(&kinds, &required)
                || kinds
                    .iter()
                    .filter(|kind| **kind == RigidLifecycleObservationKind::FilterDecision)
                    .count()
                    < 2
                || kinds
                    .iter()
                    .filter(|kind| **kind == RigidLifecycleObservationKind::PreSolve)
                    .count()
                    < 2
            {
                return Err(validation(RigidWorldErrorKind::ResultObservationMismatch));
            }
        }
        RigidWorldWitnessFamily::DestructionListenerAndDependencyCascades => {
            let kinds = lifecycle.iter().map(|event| event.kind).collect::<Vec<_>>();
            let required = [
                RigidLifecycleObservationKind::JointGoodbye,
                RigidLifecycleObservationKind::EndContact,
                RigidLifecycleObservationKind::ContactDestroyed,
                RigidLifecycleObservationKind::FixtureGoodbye,
                RigidLifecycleObservationKind::BodyDestroyed,
            ];
            let explicit_joint_ids = actions
                .iter()
                .filter_map(|record| match record.action() {
                    RigidWorldAction::DestroyJoint { joint_id } => Some(joint_id),
                    _ => None,
                })
                .collect::<HashSet<_>>();
            let explicit_fixture_ids = actions
                .iter()
                .filter_map(|record| match record.action() {
                    RigidWorldAction::DestroyFixture { fixture_id } => Some(fixture_id),
                    _ => None,
                })
                .collect::<HashSet<_>>();
            let fabricated_goodbye = lifecycle.iter().any(|event| match event.kind {
                RigidLifecycleObservationKind::JointGoodbye => event
                    .maybe_entity_id
                    .as_ref()
                    .is_some_and(|id| explicit_joint_ids.contains(id)),
                RigidLifecycleObservationKind::FixtureGoodbye => event
                    .maybe_entity_id
                    .as_ref()
                    .is_some_and(|id| explicit_fixture_ids.contains(id)),
                _ => false,
            });
            if !ordered_kinds_include(&kinds, &required) || fabricated_goodbye {
                return Err(validation(RigidWorldErrorKind::ResultObservationMismatch));
            }
        }
        RigidWorldWitnessFamily::DiagnosticReconstructionAndDumpOrder => {
            let records = observations
                .iter()
                .filter_map(|observation| match observation {
                    RigidWorldObservation::Reconstruction { record } => Some(record),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let has_unsupported_mouse = records
                .iter()
                .any(|record| record.support == RigidReconstructionSupport::UnsupportedMouseJoint);
            let gear_after_non_gear = records
                .iter()
                .position(|record| !record.dependency_ids.is_empty())
                .is_some_and(|gear_index| {
                    records[..gear_index]
                        .iter()
                        .any(|record| record.kind == RigidReconstructionKind::Joint)
                });
            if !has_unsupported_mouse || !gear_after_non_gear {
                return Err(validation(RigidWorldErrorKind::ResultObservationMismatch));
            }
        }
        _ => {}
    }
    Ok(())
}

fn lifecycle_identity_shape_is_valid(event: &RigidLifecycleObservation) -> bool {
    match event.kind {
        RigidLifecycleObservationKind::FilterDecision => {
            event.maybe_contact.is_some() != event.maybe_entity_id.is_some()
        }
        RigidLifecycleObservationKind::ContactCreated
        | RigidLifecycleObservationKind::BeginContact
        | RigidLifecycleObservationKind::PreSolve
        | RigidLifecycleObservationKind::PostSolve
        | RigidLifecycleObservationKind::EndContact
        | RigidLifecycleObservationKind::ContactDestroyed => {
            event.maybe_contact.is_some() && event.maybe_entity_id.is_none()
        }
        RigidLifecycleObservationKind::JointGoodbye
        | RigidLifecycleObservationKind::FixtureGoodbye
        | RigidLifecycleObservationKind::BodyDestroyed => {
            event.maybe_contact.is_none() && event.maybe_entity_id.is_some()
        }
    }
}

fn joint_snapshot_is_finite(snapshot: &RigidJointSnapshot) -> bool {
    [
        snapshot.coordinate_bits,
        snapshot.speed_bits,
        snapshot.reaction_force.x_bits,
        snapshot.reaction_force.y_bits,
        snapshot.reaction_torque_bits,
    ]
    .into_iter()
    .all(|bits| bits.to_f32().is_finite())
}

fn ordered_kinds_include(
    actual: &[RigidLifecycleObservationKind],
    required: &[RigidLifecycleObservationKind],
) -> bool {
    let mut required = required.iter();
    let mut maybe_next = required.next();
    for kind in actual {
        if maybe_next == Some(kind) {
            maybe_next = required.next();
        }
    }
    maybe_next.is_none()
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ExpectedObservation<'a> {
    BodyState(&'a ScenarioId),
    Step,
    Query(&'a [RigidQueryDirectiveRule]),
    RayCast(&'a [RigidRayDirectiveRule]),
    OriginShift(Vec2Bits),
    Joint(&'a ScenarioId),
    Rope(&'a ScenarioId),
    Reconstruction,
    Diagnostics,
}

impl ExpectedObservation<'_> {
    pub(super) fn matches(
        self,
        live_identities: &RigidCheckpointLiveIdentities<'_>,
        actual: &RigidWorldObservation,
    ) -> bool {
        match (self, actual) {
            (Self::BodyState(expected), RigidWorldObservation::BodyState { state }) => {
                expected == &state.body_id
            }
            (Self::Step, RigidWorldObservation::Step { .. }) => true,
            (Self::Query(rules), RigidWorldObservation::Query { observation }) => {
                query_observation_matches(live_identities, rules, observation)
            }
            (Self::RayCast(rules), RigidWorldObservation::RayCast { observation }) => {
                ray_observation_matches(live_identities, rules, observation)
            }
            (Self::OriginShift(expected), RigidWorldObservation::OriginShift { shift }) => {
                expected == *shift
            }
            (Self::Joint(expected), RigidWorldObservation::Joint { snapshot }) => {
                expected == &snapshot.joint_id
            }
            (Self::Rope(expected), RigidWorldObservation::Rope { snapshot }) => {
                expected == &snapshot.rope_id
            }
            (Self::Reconstruction, RigidWorldObservation::Reconstruction { .. })
            | (Self::Diagnostics, RigidWorldObservation::Diagnostics { .. }) => true,
            _ => false,
        }
    }
}

fn query_observation_matches(
    live_identities: &RigidCheckpointLiveIdentities<'_>,
    rules: &[RigidQueryDirectiveRule],
    observation: &RigidQueryObservation,
) -> bool {
    let mut terminated = false;
    for occurrence in &observation.occurrences {
        if terminated
            || !fixture_child_is_live(
                live_identities,
                &occurrence.fixture_id,
                occurrence.child_index,
            )
        {
            return false;
        }
        terminated = rules.iter().any(|rule| {
            rule.target.fixture_id == occurrence.fixture_id
                && rule.target.child_index == occurrence.child_index
                && rule.directive == RigidQueryDirective::Terminate
        });
    }

    observation.completion
        == if terminated {
            RigidQueryCompletion::Terminated
        } else {
            RigidQueryCompletion::Exhausted
        }
}

fn ray_observation_matches(
    live_identities: &RigidCheckpointLiveIdentities<'_>,
    rules: &[RigidRayDirectiveRule],
    observation: &RigidRayObservation,
) -> bool {
    let mut current_max_fraction_bits = RIGID_RAY_INITIAL_MAX_FRACTION_BITS;
    let mut current_max_fraction = current_max_fraction_bits.to_f32();
    let mut terminated = false;

    for hit in &observation.hits {
        if terminated || !fixture_child_is_live(live_identities, &hit.fixture_id, hit.child_index) {
            return false;
        }
        if !ray_hit_geometry_is_finite(hit) {
            return false;
        }
        let hit_fraction = hit.fraction_bits.to_f32();
        if !hit_fraction.is_finite() || hit_fraction < 0.0 || hit_fraction > current_max_fraction {
            return false;
        }
        let directive = rules
            .iter()
            .find(|rule| {
                rule.target.fixture_id == hit.fixture_id
                    && rule.target.child_index == hit.child_index
            })
            .map_or(RigidRayDirective::Continue, |rule| rule.directive);
        match directive {
            RigidRayDirective::Ignore | RigidRayDirective::Continue => {}
            RigidRayDirective::Terminate => terminated = true,
            RigidRayDirective::Clip { fraction_bits } => {
                let fraction = fraction_bits.to_f32();
                if !fraction.is_finite() || fraction < 0.0 || fraction > current_max_fraction {
                    return false;
                }
                if fraction < current_max_fraction {
                    current_max_fraction = fraction;
                    current_max_fraction_bits = fraction_bits;
                }
            }
        }
    }

    let expected_completion = if terminated {
        RigidRayCompletion::Terminated
    } else {
        RigidRayCompletion::Exhausted
    };
    observation.completion == expected_completion
        && observation.final_max_fraction_bits == current_max_fraction_bits
}

fn ray_hit_geometry_is_finite(hit: &RigidRayHitObservation) -> bool {
    [
        hit.point.x_bits,
        hit.point.y_bits,
        hit.normal.x_bits,
        hit.normal.y_bits,
    ]
    .into_iter()
    .all(|bits| bits.to_f32().is_finite())
}

fn fixture_child_is_live(
    live_identities: &RigidCheckpointLiveIdentities<'_>,
    fixture_id: &ScenarioId,
    child_index: u32,
) -> bool {
    live_identities.fixtures.iter().any(|fixture| {
        fixture.fixture_id() == fixture_id
            && child_index
                < match fixture.shape() {
                    RigidFixtureShape::Circle { .. } | RigidFixtureShape::Polygon { .. } => 1,
                }
    })
}

pub(super) fn expected_observation(action: &RigidWorldAction) -> Option<ExpectedObservation<'_>> {
    match action {
        RigidWorldAction::SetLinearVelocity { body_id, .. }
        | RigidWorldAction::SetAngularVelocity { body_id, .. }
        | RigidWorldAction::ApplyForce { body_id, .. }
        | RigidWorldAction::ApplyTorque { body_id, .. }
        | RigidWorldAction::ApplyLinearImpulse { body_id, .. }
        | RigidWorldAction::ApplyAngularImpulse { body_id, .. }
        | RigidWorldAction::SetBodyDamping { body_id, .. }
        | RigidWorldAction::SetGravityScale { body_id, .. }
        | RigidWorldAction::SetFixedRotation { body_id, .. }
        | RigidWorldAction::SetSleepingAllowed { body_id, .. }
        | RigidWorldAction::SetAwake { body_id, .. }
        | RigidWorldAction::SetBullet { body_id, .. } => {
            Some(ExpectedObservation::BodyState(body_id))
        }
        RigidWorldAction::ConfiguredStep { .. } => Some(ExpectedObservation::Step),
        RigidWorldAction::QueryAabb {
            directive_rules, ..
        } => Some(ExpectedObservation::Query(directive_rules)),
        RigidWorldAction::RayCast {
            directive_rules, ..
        } => Some(ExpectedObservation::RayCast(directive_rules)),
        RigidWorldAction::ShiftOrigin { shift } => Some(ExpectedObservation::OriginShift(*shift)),
        RigidWorldAction::CreateJoint { joint_id, .. }
        | RigidWorldAction::InspectJoint { joint_id, .. }
        | RigidWorldAction::MutateJoint { joint_id, .. } => {
            Some(ExpectedObservation::Joint(joint_id))
        }
        RigidWorldAction::InspectRope { rope_id, .. }
        | RigidWorldAction::SetRopeAngle { rope_id, .. }
        | RigidWorldAction::StepRope { rope_id, .. } => Some(ExpectedObservation::Rope(rope_id)),
        RigidWorldAction::RequestReconstruction => Some(ExpectedObservation::Reconstruction),
        RigidWorldAction::RequestDiagnostics => Some(ExpectedObservation::Diagnostics),
        _ => None,
    }
}

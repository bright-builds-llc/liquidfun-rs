//! Phase 8 diagnostics and semantic reconstruction projections.

use liquidfun_test_protocol::{
    FloatBits, RigidDiagnosticsObservation, RigidReconstructionKind,
    RigidReconstructionObservation, RigidReconstructionSupport, RigidWorldActionRecord,
    RigidWorldObservation, ScenarioId,
};

use super::super::{NativeRigidWorldError, TimelineExecutor, action_error, checked_u32};

pub(super) fn observe_diagnostics(
    executor: &mut TimelineExecutor,
) -> Result<(), NativeRigidWorldError> {
    let diagnostics = executor.world.world_diagnostics();
    executor
        .semantic_observations
        .push(RigidWorldObservation::Diagnostics {
            snapshot: RigidDiagnosticsObservation {
                body_count: checked_u32(diagnostics.body_count(), "diagnostic-body-count")?,
                fixture_count: checked_u32(
                    diagnostics.fixture_count(),
                    "diagnostic-fixture-count",
                )?,
                joint_count: checked_u32(diagnostics.joint_count(), "diagnostic-joint-count")?,
                contact_count: checked_u32(
                    diagnostics.contact_count(),
                    "diagnostic-contact-count",
                )?,
                tree_height: u32::try_from(diagnostics.tree_height()).map_err(|error| {
                    NativeRigidWorldError::Declaration {
                        checkpoint_id: "diagnostic-tree-height".into(),
                        message: error.to_string().into(),
                    }
                })?,
                tree_max_balance: u32::try_from(diagnostics.tree_balance()).map_err(|error| {
                    NativeRigidWorldError::Declaration {
                        checkpoint_id: "diagnostic-tree-balance".into(),
                        message: error.to_string().into(),
                    }
                })?,
                tree_quality_bits: FloatBits::from_f32(diagnostics.tree_quality()),
            },
        });
    Ok(())
}

pub(super) fn observe_reconstruction(
    executor: &mut TimelineExecutor,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let reconstruction = executor
        .world
        .semantic_reconstruction()
        .map_err(|error| action_error(action, error))?;
    let body_ids = executor
        .bodies
        .iter()
        .rev()
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    let mut body_ids = body_ids.into_iter();
    let mut ordinal = 0_u32;
    for body in reconstruction.bodies() {
        let body_id = body_ids
            .next()
            .ok_or_else(|| action_error(action, "reconstruction body mapping exhausted"))?;
        let native_body = executor.body(&body_id, action)?;
        push_reconstruction(
            executor,
            &mut ordinal,
            RigidReconstructionKind::Body,
            body_id,
            RigidReconstructionSupport::Supported,
            Vec::new(),
        )?;
        let fixture_ids = executor
            .fixtures
            .iter()
            .rev()
            .filter_map(|(id, fixture)| {
                executor
                    .fixture_owners
                    .iter()
                    .find_map(|(candidate, owner)| {
                        (*candidate == *fixture && *owner == native_body).then(|| id.clone())
                    })
            })
            .collect::<Vec<_>>();
        let mut fixture_ids = fixture_ids.into_iter();
        for _fixture in body.fixtures() {
            let fixture_id = fixture_ids
                .next()
                .ok_or_else(|| action_error(action, "reconstruction fixture mapping exhausted"))?;
            push_reconstruction(
                executor,
                &mut ordinal,
                RigidReconstructionKind::Fixture,
                fixture_id,
                RigidReconstructionSupport::Supported,
                Vec::new(),
            )?;
        }
    }
    let ordered_joints = reconstruction.joints();
    let joint_by_index = executor
        .joints
        .iter()
        .rev()
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    for joint in ordered_joints {
        let position =
            usize::try_from(joint.index().get()).map_err(|error| action_error(action, error))?;
        let entity_id = joint_by_index
            .get(position)
            .cloned()
            .ok_or_else(|| action_error(action, "reconstruction joint index was unmapped"))?;
        let dependencies = joint
            .maybe_source_joint_indices()
            .map_or_else(Vec::new, |indices| {
                indices
                    .into_iter()
                    .filter_map(|index| {
                        usize::try_from(index.get())
                            .ok()
                            .and_then(|value| joint_by_index.get(value).cloned())
                    })
                    .collect()
            });
        let support = match joint.support() {
            liquidfun::ReconstructionSupport::Supported(_) => RigidReconstructionSupport::Supported,
            liquidfun::ReconstructionSupport::Unsupported(
                liquidfun::ReconstructionUnsupported::MouseJoint,
            ) => RigidReconstructionSupport::UnsupportedMouseJoint,
        };
        push_reconstruction(
            executor,
            &mut ordinal,
            RigidReconstructionKind::Joint,
            entity_id,
            support,
            dependencies,
        )?;
    }
    Ok(())
}

fn push_reconstruction(
    executor: &mut TimelineExecutor,
    ordinal: &mut u32,
    kind: RigidReconstructionKind,
    entity_id: ScenarioId,
    support: RigidReconstructionSupport,
    dependencies: Vec<ScenarioId>,
) -> Result<(), NativeRigidWorldError> {
    executor
        .semantic_observations
        .push(RigidWorldObservation::Reconstruction {
            record: RigidReconstructionObservation {
                ordinal: *ordinal,
                kind,
                entity_id,
                support,
                dependency_ids: dependencies.into_boxed_slice(),
            },
        });
    *ordinal = ordinal
        .checked_add(1)
        .ok_or_else(|| NativeRigidWorldError::Declaration {
            checkpoint_id: "reconstruction-ordinal".into(),
            message: "reconstruction ordinal exceeded the protocol representation".into(),
        })?;
    Ok(())
}

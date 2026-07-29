//! Phase 8 rope lifecycle and semantic snapshots.

use liquidfun::rope::{Rope, RopeDef};
use liquidfun_test_protocol::{
    RigidRopeDeclaration, RigidRopeSnapshot, RigidWorldActionRecord, RigidWorldObservation,
    RigidWorldTimeline, ScenarioId,
};

use super::super::{NativeRigidWorldError, TimelineExecutor, action_error, vec2, vec2_bits};

pub(super) fn create_rope(
    executor: &mut TimelineExecutor,
    timeline: &RigidWorldTimeline,
    rope_id: &ScenarioId,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let declaration = timeline
        .ropes()
        .iter()
        .find(|value| &value.rope_id == rope_id)
        .ok_or_else(|| action_error(action, format!("missing declaration for rope `{rope_id}`")))?;
    let rope =
        Rope::new(rope_definition(declaration).map_err(|error| action_error(action, error))?)
            .map_err(|error| action_error(action, error))?;
    executor.ropes.push((rope_id.clone(), rope));
    Ok(())
}

fn rope_definition(
    declaration: &RigidRopeDeclaration,
) -> Result<RopeDef, liquidfun::rope::RopeError> {
    RopeDef::new(
        declaration.vertices.iter().copied().map(vec2).collect(),
        declaration
            .masses_bits
            .iter()
            .map(|bits| bits.to_f32())
            .collect(),
        vec2(declaration.gravity),
        declaration.damping_bits.to_f32(),
        declaration.stretch_stiffness_bits.to_f32(),
        declaration.bend_stiffness_bits.to_f32(),
    )
}

pub(super) fn observe_rope(
    executor: &mut TimelineExecutor,
    rope_id: &ScenarioId,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let rope = executor
        .ropes
        .iter()
        .find_map(|(id, rope)| (id == rope_id).then_some(rope))
        .ok_or_else(|| action_error(action, format!("unknown rope `{rope_id}`")))?;
    let snapshot = RigidRopeSnapshot {
        rope_id: rope_id.clone(),
        vertices: rope.vertices().iter().copied().map(vec2_bits).collect(),
    };
    executor
        .semantic_observations
        .push(RigidWorldObservation::Rope { snapshot });
    Ok(())
}

pub(super) fn rope_mut<'a>(
    executor: &'a mut TimelineExecutor,
    rope_id: &ScenarioId,
    action: &RigidWorldActionRecord,
) -> Result<&'a mut Rope, NativeRigidWorldError> {
    executor
        .ropes
        .iter_mut()
        .find_map(|(id, rope)| (id == rope_id).then_some(rope))
        .ok_or_else(|| action_error(action, format!("unknown rope `{rope_id}`")))
}

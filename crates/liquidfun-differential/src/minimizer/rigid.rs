use liquidfun_test_protocol::{
    HarnessLimits, RigidWorldRequestRecord, decode_rigid_world_request_jsonl,
};

use crate::RigidFailureSignature;

use super::{RigidScenarioTransform, removal_ranges};

pub(super) fn rigid_candidate_transforms(
    request: &RigidWorldRequestRecord,
    target: &RigidFailureSignature,
) -> Vec<RigidScenarioTransform> {
    request
        .scenario()
        .timelines()
        .iter()
        .enumerate()
        .flat_map(|(timeline_index, timeline)| {
            let maybe_protected_prefix_end = timeline
                .actions()
                .iter()
                .position(|action| action.action_id().as_str() == target.action_id());
            removal_ranges(timeline.actions().len())
                .into_iter()
                .filter(move |(start, _end)| {
                    maybe_protected_prefix_end.is_none_or(|protected| *start > protected)
                })
                .map(move |(start, end)| RigidScenarioTransform::RemoveActions {
                    timeline_index,
                    start,
                    end,
                })
        })
        .collect()
}

pub(crate) fn apply_rigid_scenario_transform(
    request: &RigidWorldRequestRecord,
    transform: RigidScenarioTransform,
    limits: &HarnessLimits,
) -> Option<RigidWorldRequestRecord> {
    let mut value = serde_json::to_value(request).ok()?;
    let RigidScenarioTransform::RemoveActions {
        timeline_index,
        start,
        end,
    } = transform;
    let actions = value
        .get_mut("scenario")?
        .get_mut("timelines")?
        .as_array_mut()?
        .get_mut(timeline_index)?
        .get_mut("actions")?
        .as_array_mut()?;
    if start >= end || end > actions.len() {
        return None;
    }
    actions.drain(start..end);
    let mut bytes = serde_json::to_vec(&value).ok()?;
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, limits).ok()
}

pub(crate) fn reconstruct_complete_rigid_minimization(
    source: &RigidWorldRequestRecord,
    target: &RigidFailureSignature,
    attempted: &[RigidScenarioTransform],
    accepted: &[RigidScenarioTransform],
    limits: &HarnessLimits,
) -> Option<RigidWorldRequestRecord> {
    let mut reconstructed = source.clone();
    let mut attempted_offset = 0_usize;
    for accepted_transform in accepted {
        let candidates = rigid_candidate_transforms(&reconstructed, target);
        let accepted_index = candidates
            .iter()
            .position(|candidate| candidate == accepted_transform)?;
        let attempted_end = attempted_offset.checked_add(accepted_index + 1)?;
        if attempted.get(attempted_offset..attempted_end)? != candidates.get(..=accepted_index)? {
            return None;
        }
        reconstructed =
            apply_rigid_scenario_transform(&reconstructed, *accepted_transform, limits)?;
        attempted_offset = attempted_end;
    }

    let final_candidates = rigid_candidate_transforms(&reconstructed, target);
    if attempted.get(attempted_offset..)? != final_candidates.as_slice() {
        return None;
    }
    Some(reconstructed)
}

pub(crate) fn canonical_rigid_request_bytes(
    request: &RigidWorldRequestRecord,
) -> Result<Vec<u8>, serde_json::Error> {
    let mut bytes = serde_json::to_vec(request)?;
    bytes.push(b'\n');
    Ok(bytes)
}

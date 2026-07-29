use crate::arena::Arena;
use crate::{BodyId, JointId};

use super::{Island, IslandBuildError, IslandLimits, IslandPosition, IslandVelocity};
use crate::world::body::{BodyState, BodyType};
use crate::world::contact_manager::ContactManager;
use crate::world::joint::JointRecord;
use crate::world::object::Body;

impl Island {
    fn new() -> Self {
        Self {
            body_ids: Vec::new(),
            body_states: Vec::new(),
            contact_indices: Vec::new(),
            joint_ids: Vec::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
        }
    }

    fn add_body(
        &mut self,
        body: BodyId,
        state: BodyState,
        limit: usize,
    ) -> Result<(), IslandBuildError> {
        if self.body_ids.len() >= limit {
            return Err(IslandBuildError::CapacityExceeded {
                resource: "island bodies",
                limit,
            });
        }
        reserve_one(&mut self.body_ids, "island bodies", limit)?;
        reserve_one(&mut self.body_states, "island body states", limit)?;
        reserve_one(&mut self.positions, "island positions", limit)?;
        reserve_one(&mut self.velocities, "island velocities", limit)?;
        let snapshot = state.snapshot();
        self.body_ids.push(body);
        self.body_states.push(state);
        self.positions.push(IslandPosition {
            position: snapshot.position(),
            angle: snapshot.angle(),
        });
        self.velocities.push(IslandVelocity {
            linear: state.solver_linear(),
            angular: state.solver_angular(),
        });
        Ok(())
    }

    fn add_contact(&mut self, index: usize, limit: usize) -> Result<(), IslandBuildError> {
        if self.contact_indices.len() >= limit {
            return Err(IslandBuildError::CapacityExceeded {
                resource: "island contacts",
                limit,
            });
        }
        reserve_one(&mut self.contact_indices, "island contacts", limit)?;
        self.contact_indices.push(index);
        Ok(())
    }

    fn add_joint(&mut self, joint: JointId, limit: usize) -> Result<(), IslandBuildError> {
        if self.joint_ids.len() >= limit {
            return Err(IslandBuildError::CapacityExceeded {
                resource: "island joints",
                limit,
            });
        }
        reserve_one(&mut self.joint_ids, "island joints", limit)?;
        self.joint_ids.push(joint);
        Ok(())
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "source-ordered contact and joint DFS is clearer as one auditable traversal"
)]
pub(in crate::world) fn build_islands(
    body_order: &[BodyId],
    bodies: &Arena<Body, BodyId>,
    joints: &Arena<JointRecord, JointId>,
    contact_manager: &ContactManager,
    limits: IslandLimits,
) -> Result<Vec<Island>, IslandBuildError> {
    preflight_graph(body_order, bodies, joints, contact_manager, limits)?;
    let mut body_visited = bounded_false_lanes(body_order.len(), "body visitation")?;
    let mut contact_visited = bounded_false_lanes(contact_manager.len(), "contact visitation")?;
    let joint_order = joints
        .iter()
        .map(|(joint, _record)| joint)
        .collect::<Vec<_>>();
    let mut joint_visited = bounded_false_lanes(joint_order.len(), "joint visitation")?;
    let mut stack = Vec::new();
    stack
        .try_reserve_exact(body_order.len())
        .map_err(|_| IslandBuildError::CapacityExceeded {
            resource: "island DFS stack",
            limit: limits.bodies,
        })?;
    let mut islands = Vec::new();

    for seed_index in 0..body_order.len() {
        if body_visited[seed_index] {
            continue;
        }
        let seed = bodies
            .get(body_order[seed_index])
            .map_err(|_| IslandBuildError::InvalidGraph)?;
        let snapshot = seed.state.snapshot();
        if !snapshot.is_awake() || !snapshot.is_active() || snapshot.body_type() == BodyType::Static
        {
            continue;
        }

        let mut island = Island::new();
        stack.clear();
        stack.push(seed_index);
        body_visited[seed_index] = true;
        while let Some(body_index) = stack.pop() {
            let body_id = body_order[body_index];
            let body = bodies
                .get(body_id)
                .map_err(|_| IslandBuildError::InvalidGraph)?;
            let candidate = body.state.candidate_set_awake(true);
            island.add_body(body_id, candidate, limits.bodies)?;
            if candidate.snapshot().body_type() == BodyType::Static {
                continue;
            }

            for ordinal in &body.contacts {
                let contact_index = contact_manager
                    .contact_index_for_ordinal(*ordinal)
                    .ok_or(IslandBuildError::InvalidGraph)?;
                if contact_visited[contact_index] {
                    continue;
                }
                let contact = &contact_manager.contacts()[contact_index];
                if !contact.is_enabled() || !contact.is_touching() || contact.is_sensor() {
                    continue;
                }
                island.add_contact(contact_index, limits.contacts)?;
                contact_visited[contact_index] = true;

                let other = contact
                    .other_body(body_id)
                    .ok_or(IslandBuildError::InvalidGraph)?;
                let other_index = body_order
                    .iter()
                    .position(|candidate| *candidate == other)
                    .ok_or(IslandBuildError::InvalidGraph)?;
                if body_visited[other_index] {
                    continue;
                }
                body_visited[other_index] = true;
                stack.push(other_index);
            }

            for joint_id in &body.joints {
                let joint_index = joint_order
                    .iter()
                    .position(|candidate| candidate == joint_id)
                    .ok_or(IslandBuildError::InvalidGraph)?;
                if joint_visited[joint_index] {
                    continue;
                }
                let joint = joints
                    .get(*joint_id)
                    .map_err(|_| IslandBuildError::InvalidGraph)?;
                let other = if joint.bodies[0] == body_id {
                    joint.bodies[1]
                } else if joint.bodies[1] == body_id {
                    joint.bodies[0]
                } else {
                    return Err(IslandBuildError::InvalidGraph);
                };
                let other_index = body_order
                    .iter()
                    .position(|candidate| *candidate == other)
                    .ok_or(IslandBuildError::InvalidGraph)?;
                let other_body = bodies
                    .get(other)
                    .map_err(|_| IslandBuildError::InvalidGraph)?;
                if !other_body.state.snapshot().is_active() {
                    continue;
                }
                island.add_joint(*joint_id, limits.joints)?;
                joint_visited[joint_index] = true;
                if body_visited[other_index] {
                    continue;
                }
                body_visited[other_index] = true;
                stack.push(other_index);
            }
        }

        for body_id in &island.body_ids {
            let body = bodies
                .get(*body_id)
                .map_err(|_| IslandBuildError::InvalidGraph)?;
            if body.state.snapshot().body_type() != BodyType::Static {
                continue;
            }
            let static_index = body_order
                .iter()
                .position(|candidate| candidate == body_id)
                .ok_or(IslandBuildError::InvalidGraph)?;
            body_visited[static_index] = false;
        }
        islands
            .try_reserve(1)
            .map_err(|_| IslandBuildError::CapacityExceeded {
                resource: "island collection",
                limit: limits.bodies,
            })?;
        islands.push(island);
    }
    Ok(islands)
}

fn preflight_graph(
    body_order: &[BodyId],
    bodies: &Arena<Body, BodyId>,
    joints: &Arena<JointRecord, JointId>,
    contact_manager: &ContactManager,
    limits: IslandLimits,
) -> Result<(), IslandBuildError> {
    if body_order.len() > limits.bodies {
        return Err(IslandBuildError::CapacityExceeded {
            resource: "island bodies",
            limit: limits.bodies,
        });
    }
    if contact_manager.len() > limits.contacts {
        return Err(IslandBuildError::CapacityExceeded {
            resource: "island contacts",
            limit: limits.contacts,
        });
    }
    if joints.iter().count() > limits.joints {
        return Err(IslandBuildError::CapacityExceeded {
            resource: "island joints",
            limit: limits.joints,
        });
    }
    if body_order.len() != bodies.iter().count() {
        return Err(IslandBuildError::InvalidGraph);
    }
    for (index, body_id) in body_order.iter().copied().enumerate() {
        let body = bodies
            .get(body_id)
            .map_err(|_| IslandBuildError::InvalidGraph)?;
        if body_order[index + 1..].contains(&body_id) {
            return Err(IslandBuildError::InvalidGraph);
        }
        for ordinal in &body.contacts {
            let contact_index = contact_manager
                .contact_index_for_ordinal(*ordinal)
                .ok_or(IslandBuildError::InvalidGraph)?;
            if contact_manager.contacts()[contact_index]
                .other_body(body_id)
                .is_none()
            {
                return Err(IslandBuildError::InvalidGraph);
            }
        }
        for joint_id in &body.joints {
            let joint = joints
                .get(*joint_id)
                .map_err(|_| IslandBuildError::InvalidGraph)?;
            if joint.bodies[0] != body_id && joint.bodies[1] != body_id {
                return Err(IslandBuildError::InvalidGraph);
            }
        }
    }
    for (index, contact) in contact_manager.contacts().iter().enumerate() {
        if contact_manager.contacts()[index + 1..]
            .iter()
            .any(|candidate| candidate.ordinal == contact.ordinal)
        {
            return Err(IslandBuildError::InvalidGraph);
        }
        for body_id in [contact.key.first.body, contact.key.second.body] {
            let body = bodies
                .get(body_id)
                .map_err(|_| IslandBuildError::InvalidGraph)?;
            if body
                .contacts
                .iter()
                .filter(|ordinal| **ordinal == contact.ordinal)
                .count()
                != 1
            {
                return Err(IslandBuildError::InvalidGraph);
            }
        }
    }
    for (joint_id, joint) in joints.iter() {
        for body_id in joint.bodies {
            let body = bodies
                .get(body_id)
                .map_err(|_| IslandBuildError::InvalidGraph)?;
            if body
                .joints
                .iter()
                .filter(|candidate| **candidate == joint_id)
                .count()
                != 1
            {
                return Err(IslandBuildError::InvalidGraph);
            }
        }
    }
    Ok(())
}

fn bounded_false_lanes(len: usize, resource: &'static str) -> Result<Vec<bool>, IslandBuildError> {
    let mut lanes = Vec::new();
    lanes
        .try_reserve_exact(len)
        .map_err(|_| IslandBuildError::CapacityExceeded {
            resource,
            limit: len,
        })?;
    lanes.resize(len, false);
    Ok(lanes)
}

pub(super) fn reserve_one<T>(
    values: &mut Vec<T>,
    resource: &'static str,
    limit: usize,
) -> Result<(), IslandBuildError> {
    values
        .try_reserve(1)
        .map_err(|_| IslandBuildError::CapacityExceeded { resource, limit })
}

use crate::arena::Arena;
use crate::math::Vec2;
use crate::{BodyId, FixtureId, JointId};

use super::body::{BodyState, BodyType};
use super::config::StepConfiguration;
use super::contact_manager::ContactManager;
use super::contact_solver::{
    ContactConstraintInput, ContactImpulseSolution, ContactSolveFailure, solve_island_constraints,
};
use super::object::{Body, Fixture};

const REVIEWED_MAX_ISLAND_BODIES: usize = 4_096;
const REVIEWED_MAX_ISLAND_CONTACTS: usize = 8_192;

#[derive(Debug, Clone, Copy)]
pub(super) struct IslandLimits {
    max_bodies: usize,
    max_contacts: usize,
}

impl IslandLimits {
    pub(super) const REVIEWED: Self = Self {
        max_bodies: REVIEWED_MAX_ISLAND_BODIES,
        max_contacts: REVIEWED_MAX_ISLAND_CONTACTS,
    };

    #[cfg(feature = "differential-internals")]
    pub(super) const fn diagnostic(max_bodies: usize, max_contacts: usize) -> Self {
        Self {
            max_bodies,
            max_contacts,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IslandBuildError {
    CapacityExceeded {
        resource: &'static str,
        limit: usize,
    },
    InvalidGraph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SolveFailureInjection {
    LateIsland { solved_islands: usize },
    ProxyBounds { fixture: FixtureId },
}

#[derive(Debug, Clone, Copy)]
pub(super) struct IslandSolveParameters {
    gravity: Vec2,
    configuration: StepConfiguration,
    time_step_ratio: f32,
    warm_starting: bool,
    maybe_failure_injection: Option<SolveFailureInjection>,
}

impl IslandSolveParameters {
    pub(super) const fn new(
        gravity: Vec2,
        configuration: StepConfiguration,
        time_step_ratio: f32,
        warm_starting: bool,
        maybe_failure_injection: Option<SolveFailureInjection>,
    ) -> Self {
        Self {
            gravity,
            configuration,
            time_step_ratio,
            warm_starting,
            maybe_failure_injection,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct IslandPosition {
    pub(super) position: Vec2,
    pub(super) angle: f32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct IslandVelocity {
    pub(super) linear: Vec2,
    pub(super) angular: f32,
}

#[derive(Debug)]
pub(super) struct Island {
    pub(super) body_ids: Vec<BodyId>,
    pub(super) body_states: Vec<BodyState>,
    pub(super) contact_indices: Vec<usize>,
    pub(super) joint_ids: Vec<JointId>,
    pub(super) positions: Vec<IslandPosition>,
    pub(super) velocities: Vec<IslandVelocity>,
}

#[derive(Debug)]
pub(super) struct IslandSolution {
    pub(super) body_ids: Vec<BodyId>,
    pub(super) body_states: Vec<BodyState>,
    pub(super) contact_impulses: Vec<ContactImpulseSolution>,
}

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
}

pub(super) fn build_islands(
    body_order: &[BodyId],
    bodies: &Arena<Body, BodyId>,
    contact_manager: &ContactManager,
    limits: IslandLimits,
) -> Result<Vec<Island>, IslandBuildError> {
    preflight_graph(body_order, bodies, contact_manager, limits)?;
    let mut body_visited = bounded_false_lanes(body_order.len(), "body visitation")?;
    let mut contact_visited = bounded_false_lanes(contact_manager.len(), "contact visitation")?;
    let mut stack = Vec::new();
    stack
        .try_reserve_exact(body_order.len())
        .map_err(|_| IslandBuildError::CapacityExceeded {
            resource: "island DFS stack",
            limit: limits.max_bodies,
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
            island.add_body(body_id, candidate, limits.max_bodies)?;
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
                island.add_contact(contact_index, limits.max_contacts)?;
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
                limit: limits.max_bodies,
            })?;
        islands.push(island);
    }
    Ok(islands)
}

pub(super) fn solve_islands(
    islands: &[Island],
    contact_manager: &ContactManager,
    fixtures: &Arena<Fixture, FixtureId>,
    parameters: IslandSolveParameters,
) -> Result<Vec<IslandSolution>, ContactSolveFailure> {
    let mut solutions = Vec::new();
    solutions.try_reserve_exact(islands.len()).map_err(|_| {
        ContactSolveFailure::CapacityExceeded {
            resource: "island solutions",
            limit: islands.len(),
        }
    })?;
    for island in islands {
        if island.body_ids.len() != island.body_states.len()
            || island.body_ids.len() != island.positions.len()
            || island.body_ids.len() != island.velocities.len()
            || island
                .positions
                .iter()
                .any(|position| !position.position.is_valid() || !position.angle.is_finite())
            || island
                .velocities
                .iter()
                .any(|velocity| !velocity.linear.is_valid() || !velocity.angular.is_finite())
        {
            return Err(ContactSolveFailure::UnsupportedTopology);
        }
        let mut inputs = Vec::new();
        inputs
            .try_reserve_exact(island.contact_indices.len())
            .map_err(|_| ContactSolveFailure::CapacityExceeded {
                resource: "island contact inputs",
                limit: island.contact_indices.len(),
            })?;
        for contact_index in &island.contact_indices {
            let contact = contact_manager
                .contacts()
                .get(*contact_index)
                .ok_or(ContactSolveFailure::UnsupportedTopology)?;
            let first_body_index = island
                .body_ids
                .iter()
                .position(|body| *body == contact.key.first.body)
                .ok_or(ContactSolveFailure::UnsupportedTopology)?;
            let second_body_index = island
                .body_ids
                .iter()
                .position(|body| *body == contact.key.second.body)
                .ok_or(ContactSolveFailure::UnsupportedTopology)?;
            let first_shape = fixtures
                .get(contact.key.first.fixture)
                .map_err(|_| ContactSolveFailure::UnsupportedTopology)?
                .definition
                .shape();
            let second_shape = fixtures
                .get(contact.key.second.fixture)
                .map_err(|_| ContactSolveFailure::UnsupportedTopology)?
                .definition
                .shape();
            inputs.push(ContactConstraintInput {
                contact_index: *contact_index,
                first_body_index,
                second_body_index,
                contact,
                first_shape,
                second_shape,
            });
        }

        let solved = solve_island_constraints(
            &island.body_states,
            &inputs,
            parameters.gravity,
            parameters.configuration,
            parameters.time_step_ratio,
            parameters.warm_starting,
        )?;
        let mut body_states = island.body_states.clone();
        if body_states.len() != solved.motions.len() {
            return Err(ContactSolveFailure::UnsupportedTopology);
        }
        for (state, motion) in body_states.iter_mut().zip(solved.motions) {
            *state = state
                .candidate_set_solver_state(
                    motion.position,
                    motion.angle,
                    motion.linear,
                    motion.angular,
                )
                .map_err(|_error| ContactSolveFailure::NonFinite)?;
        }
        solutions.push(IslandSolution {
            body_ids: island.body_ids.clone(),
            body_states,
            contact_impulses: solved.contact_impulses,
        });
        if matches!(
            parameters.maybe_failure_injection,
            Some(SolveFailureInjection::LateIsland { solved_islands })
                if solutions.len() == solved_islands
        ) {
            return Err(ContactSolveFailure::NonFinite);
        }
    }
    Ok(solutions)
}

fn preflight_graph(
    body_order: &[BodyId],
    bodies: &Arena<Body, BodyId>,
    contact_manager: &ContactManager,
    limits: IslandLimits,
) -> Result<(), IslandBuildError> {
    if body_order.len() > limits.max_bodies {
        return Err(IslandBuildError::CapacityExceeded {
            resource: "island bodies",
            limit: limits.max_bodies,
        });
    }
    if contact_manager.len() > limits.max_contacts {
        return Err(IslandBuildError::CapacityExceeded {
            resource: "island contacts",
            limit: limits.max_contacts,
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

fn reserve_one<T>(
    values: &mut Vec<T>,
    resource: &'static str,
    limit: usize,
) -> Result<(), IslandBuildError> {
    values
        .try_reserve(1)
        .map_err(|_| IslandBuildError::CapacityExceeded { resource, limit })
}

use crate::arena::Arena;
use crate::math::Vec2;
use crate::math::settings::{ANGULAR_SLEEP_TOLERANCE, LINEAR_SLEEP_TOLERANCE, TIME_TO_SLEEP};
use crate::{BodyId, FixtureId, JointDef, JointId};

use super::body::{BodyState, BodyType};
use super::config::StepConfiguration;
use super::contact_manager::ContactManager;
use super::contact_solver::{
    ContactConstraintInput, ContactImpulseSolution, ContactSolveFailure, solve_island_constraints,
    solve_toi_constraints,
};
use super::joint::solver::{
    GearSolverLanes, JointConstraintInput, JointImpulseSolution, OrdinarySolverLanes,
    SolverBodyLane,
};
use super::joint::{JointRecord, JointRuntime};
use super::object::Fixture;

const REVIEWED_MAX_ISLAND_BODIES: usize = 4_096;
const REVIEWED_MAX_ISLAND_CONTACTS: usize = 8_192;
const REVIEWED_MAX_ISLAND_JOINTS: usize = 8_192;

mod graph;
mod toi;
pub(super) use graph::build_islands;
use graph::reserve_one;
pub(super) use toi::{ToiIsland, ToiIslandLimits, ToiIslandSolution, solve_toi_island};

#[derive(Debug, Clone, Copy)]
pub(super) struct IslandLimits {
    bodies: usize,
    contacts: usize,
    joints: usize,
}

impl IslandLimits {
    pub(super) const REVIEWED: Self = Self {
        bodies: REVIEWED_MAX_ISLAND_BODIES,
        contacts: REVIEWED_MAX_ISLAND_CONTACTS,
        joints: REVIEWED_MAX_ISLAND_JOINTS,
    };

    #[cfg(feature = "differential-internals")]
    pub(super) const fn diagnostic(max_bodies: usize, max_contacts: usize) -> Self {
        Self {
            bodies: max_bodies,
            contacts: max_contacts,
            joints: REVIEWED_MAX_ISLAND_JOINTS,
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
    #[cfg(feature = "differential-internals")]
    LateIsland { solved_islands: usize },
    #[cfg(feature = "differential-internals")]
    ProxyBounds { fixture: FixtureId },
}

#[derive(Debug, Clone, Copy)]
pub(super) struct IslandSolveParameters {
    gravity: Vec2,
    configuration: StepConfiguration,
    time_step_ratio: f32,
    warm_starting: bool,
    #[cfg(feature = "differential-internals")]
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
        #[cfg(not(feature = "differential-internals"))]
        let _ = maybe_failure_injection;
        Self {
            gravity,
            configuration,
            time_step_ratio,
            warm_starting,
            #[cfg(feature = "differential-internals")]
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
    pub(super) joint_impulses: Vec<JointImpulseSolution>,
}

#[allow(
    clippy::too_many_lines,
    reason = "transactional contact and joint input staging stays together for auditability"
)]
pub(super) fn solve_islands(
    islands: &[Island],
    contact_manager: &ContactManager,
    fixtures: &Arena<Fixture, FixtureId>,
    joints: &Arena<JointRecord, JointId>,
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
        validate_island_lanes(island)?;
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

        let mut joint_inputs = Vec::new();
        joint_inputs
            .try_reserve_exact(island.joint_ids.len())
            .map_err(|_| ContactSolveFailure::CapacityExceeded {
                resource: "island joint inputs",
                limit: island.joint_ids.len(),
            })?;
        for joint_id in &island.joint_ids {
            let record = joints
                .get(*joint_id)
                .map_err(|_| ContactSolveFailure::UnsupportedTopology)?;
            if let (JointDef::Gear(definition), JointRuntime::Gear(runtime)) =
                (record.definition, record.runtime)
            {
                let source_joints = definition.source_joints();
                let source_a = joints
                    .get(source_joints[0])
                    .map_err(|_| ContactSolveFailure::UnsupportedTopology)?;
                let source_b = joints
                    .get(source_joints[1])
                    .map_err(|_| ContactSolveFailure::UnsupportedTopology)?;
                let lanes = GearSolverLanes::new(
                    solver_body_lane(island, source_a.bodies[1]),
                    solver_body_lane(island, source_b.bodies[1]),
                    solver_body_lane(island, source_a.bodies[0]),
                    solver_body_lane(island, source_b.bodies[0]),
                );
                joint_inputs.push(JointConstraintInput::gear(
                    *joint_id, &lanes, definition, runtime,
                ));
                continue;
            }
            joint_inputs.push(JointConstraintInput::ordinary(
                *joint_id,
                OrdinarySolverLanes::new(
                    solver_body_lane(island, record.bodies[0]),
                    solver_body_lane(island, record.bodies[1]),
                ),
                record.definition,
                record.runtime,
            ));
        }

        let solved = solve_island_constraints(
            &island.body_states,
            &inputs,
            &joint_inputs,
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
        evaluate_sleep_candidates(
            &mut body_states,
            parameters.configuration.time_step(),
            solved.position_solved,
        )?;
        solutions.push(IslandSolution {
            body_ids: island.body_ids.clone(),
            body_states,
            contact_impulses: solved.contact_impulses,
            joint_impulses: solved.joint_impulses,
        });
        #[cfg(feature = "differential-internals")]
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

fn solver_body_lane(island: &Island, body_id: BodyId) -> SolverBodyLane {
    let maybe_solver_index = island
        .body_ids
        .iter()
        .position(|candidate| *candidate == body_id);
    maybe_solver_index.map_or_else(
        || SolverBodyLane::unresolved(body_id),
        |solver_index| SolverBodyLane::resolved(body_id, solver_index),
    )
}

fn validate_island_lanes(island: &Island) -> Result<(), ContactSolveFailure> {
    let body_count = island.body_ids.len();
    if body_count != island.body_states.len()
        || body_count != island.positions.len()
        || body_count != island.velocities.len()
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
    Ok(())
}

fn evaluate_sleep_candidates(
    body_states: &mut [BodyState],
    time_step: f32,
    position_solved: bool,
) -> Result<(), ContactSolveFailure> {
    let linear_tolerance_squared = LINEAR_SLEEP_TOLERANCE * LINEAR_SLEEP_TOLERANCE;
    let angular_tolerance_squared = ANGULAR_SLEEP_TOLERANCE * ANGULAR_SLEEP_TOLERANCE;
    let mut minimum_sleep_time = f32::MAX;

    for state in body_states.iter_mut() {
        let snapshot = state.snapshot();
        if snapshot.body_type() == BodyType::Static {
            continue;
        }
        if !snapshot.is_sleeping_allowed()
            || state.solver_angular() * state.solver_angular() > angular_tolerance_squared
            || state.solver_linear().length_squared() > linear_tolerance_squared
        {
            *state = state.candidate_set_sleep_time(0.0);
            minimum_sleep_time = 0.0;
            continue;
        }
        let sleep_time = state.sleep_time() + time_step;
        if !sleep_time.is_finite() {
            return Err(ContactSolveFailure::NonFinite);
        }
        *state = state.candidate_set_sleep_time(sleep_time);
        minimum_sleep_time = minimum_sleep_time.min(sleep_time);
    }

    if minimum_sleep_time >= TIME_TO_SLEEP && position_solved {
        for state in body_states.iter_mut() {
            *state = state.candidate_set_awake(false);
        }
    }
    Ok(())
}

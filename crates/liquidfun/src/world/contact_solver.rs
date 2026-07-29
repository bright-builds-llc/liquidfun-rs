use crate::collision::{ContactFeatureId, Manifold, Shape, world_manifold};
use crate::math::settings::{
    BAUMGARTE, LINEAR_SLOP, MAX_LINEAR_CORRECTION, MAX_ROTATION, MAX_ROTATION_SQUARED,
    MAX_TRANSLATION, MAX_TRANSLATION_SQUARED, TOI_BAUMGARTE, VELOCITY_THRESHOLD,
};
use crate::math::{Transform, Vec2, clamp, max};

use super::body::BodyState;
use super::config::StepConfiguration;
use super::contact::{Contact, ContactPoint, ManagedContactSnapshot};
use super::joint::solver::{
    JointConstraintInput, JointImpulseSolution, build_constraints as build_joint_constraints,
    solve_position as solve_joint_position, solve_velocity as solve_joint_velocity,
    transient_impulses as transient_joint_impulses, warm_start as warm_start_joints,
};

const MAX_CONDITION_NUMBER: f32 = 1_000.0;

mod position;
mod toi;
mod velocity;
use position::{build_constraint, solve_position_constraints};
pub(super) use toi::solve_toi_constraints;
use velocity::{
    constraint_bodies, shape_radius, solve_velocity_constraints, store_constraint_bodies,
    validate_solution, warm_start,
};

/// Owned post-solve evidence for one private manager contact occurrence.
#[derive(Debug, Clone, PartialEq)]
pub struct ContactSolve {
    contact: ManagedContactSnapshot,
}

impl ContactSolve {
    pub(super) const fn new(contact: ManagedContactSnapshot) -> Self {
        Self { contact }
    }

    /// Returns owned post-solve manifold, material, and warm-start state.
    #[must_use]
    pub const fn contact(&self) -> &ManagedContactSnapshot {
        &self.contact
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ContactSolveFailure {
    UnsupportedTopology,
    NonFinite,
    InvalidProxyBounds,
    CapacityExceeded {
        resource: &'static str,
        limit: usize,
    },
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SolvedBodyMotion {
    pub(super) position: Vec2,
    pub(super) angle: f32,
    pub(super) linear: Vec2,
    pub(super) angular: f32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SolverBody {
    pub(super) center: Vec2,
    pub(super) local_center: Vec2,
    pub(super) angle: f32,
    pub(super) transform: Transform,
    pub(super) linear_velocity: Vec2,
    pub(super) angular_velocity: f32,
    pub(super) inverse_mass: f32,
    pub(super) inverse_inertia: f32,
}

impl SolverBody {
    fn from_state(state: BodyState) -> Result<Self, ContactSolveFailure> {
        let body = Self {
            center: state.sweep().center(),
            local_center: state.sweep().local_center(),
            angle: state.snapshot().angle(),
            transform: state.transform(),
            linear_velocity: state.solver_linear(),
            angular_velocity: state.solver_angular(),
            inverse_mass: state.inverse_mass(),
            inverse_inertia: state.inverse_inertia(),
        };
        if !body.is_finite() {
            return Err(ContactSolveFailure::NonFinite);
        }
        Ok(body)
    }

    pub(super) fn is_finite(self) -> bool {
        self.center.is_valid()
            && self.transform.position().is_valid()
            && self.transform.rotation().sine().is_finite()
            && self.transform.rotation().cosine().is_finite()
            && self.angle.is_finite()
            && self.linear_velocity.is_valid()
            && self.angular_velocity.is_finite()
            && self.inverse_mass.is_finite()
            && self.inverse_inertia.is_finite()
    }
}

#[derive(Debug, Clone, Copy)]
struct ConstraintPoint {
    feature_id: ContactFeatureId,
    r_a: Vec2,
    r_b: Vec2,
    normal_impulse: f32,
    tangent_impulse: f32,
    normal_mass: f32,
    tangent_mass: f32,
    velocity_bias: f32,
}

impl ConstraintPoint {
    const fn cold(feature_id: ContactFeatureId) -> Self {
        Self {
            feature_id,
            r_a: Vec2::ZERO,
            r_b: Vec2::ZERO,
            normal_impulse: 0.0,
            tangent_impulse: 0.0,
            normal_mass: 0.0,
            tangent_mass: 0.0,
            velocity_bias: 0.0,
        }
    }

    fn is_finite(self) -> bool {
        self.r_a.is_valid()
            && self.r_b.is_valid()
            && self.normal_impulse.is_finite()
            && self.tangent_impulse.is_finite()
            && self.normal_mass.is_finite()
            && self.tangent_mass.is_finite()
            && self.velocity_bias.is_finite()
    }
}

#[derive(Debug)]
struct VelocityConstraint {
    contact_index: usize,
    first_body_index: usize,
    second_body_index: usize,
    manifold: Manifold,
    first_radius: f32,
    second_radius: f32,
    points: [ConstraintPoint; 2],
    point_count: usize,
    normal: Vec2,
    friction: f32,
    tangent_speed: f32,
    k: [[f32; 2]; 2],
    normal_mass: [[f32; 2]; 2],
}

pub(super) struct ContactConstraintInput<'a> {
    pub(super) contact_index: usize,
    pub(super) first_body_index: usize,
    pub(super) second_body_index: usize,
    pub(super) contact: &'a Contact,
    pub(super) first_shape: &'a Shape,
    pub(super) second_shape: &'a Shape,
}

#[derive(Debug)]
pub(super) struct ContactImpulseSolution {
    pub(super) contact_index: usize,
    pub(super) impulses: Vec<(ContactFeatureId, f32, f32)>,
}

#[derive(Debug)]
pub(super) struct IslandConstraintSolution {
    pub(super) motions: Vec<SolvedBodyMotion>,
    pub(super) contact_impulses: Vec<ContactImpulseSolution>,
    pub(super) joint_impulses: Vec<JointImpulseSolution>,
    pub(super) position_solved: bool,
}

fn build_constraints(
    inputs: &[ContactConstraintInput<'_>],
    bodies: &[SolverBody],
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<Vec<VelocityConstraint>, ContactSolveFailure> {
    let mut constraints = Vec::new();
    constraints.try_reserve_exact(inputs.len()).map_err(|_| {
        ContactSolveFailure::CapacityExceeded {
            resource: "island contact constraints",
            limit: inputs.len(),
        }
    })?;
    for input in inputs {
        constraints.push(build_constraint(
            input,
            bodies,
            time_step_ratio,
            warm_starting,
        )?);
    }
    Ok(constraints)
}

fn transient_impulses(
    constraints: &[VelocityConstraint],
) -> Result<Vec<ContactImpulseSolution>, ContactSolveFailure> {
    let mut contact_impulses = Vec::new();
    contact_impulses
        .try_reserve_exact(constraints.len())
        .map_err(|_| ContactSolveFailure::CapacityExceeded {
            resource: "island contact impulses",
            limit: constraints.len(),
        })?;
    for constraint in constraints {
        contact_impulses.push(ContactImpulseSolution {
            contact_index: constraint.contact_index,
            impulses: constraint.points[..constraint.point_count]
                .iter()
                .map(|point| {
                    (
                        point.feature_id,
                        point.normal_impulse,
                        point.tangent_impulse,
                    )
                })
                .collect(),
        });
    }
    Ok(contact_impulses)
}

pub(super) fn solve_island_constraints(
    body_states: &[BodyState],
    inputs: &[ContactConstraintInput<'_>],
    joint_inputs: &[JointConstraintInput],
    gravity: Vec2,
    configuration: StepConfiguration,
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<IslandConstraintSolution, ContactSolveFailure> {
    let mut bodies = Vec::new();
    bodies.try_reserve_exact(body_states.len()).map_err(|_| {
        ContactSolveFailure::CapacityExceeded {
            resource: "island solver bodies",
            limit: body_states.len(),
        }
    })?;
    for state in body_states {
        let mut body = SolverBody::from_state(*state)?;
        integrate_velocity(&mut body, *state, gravity, configuration.time_step())?;
        bodies.push(body);
    }

    let mut constraints = build_constraints(inputs, &bodies, time_step_ratio, warm_starting)?;
    let mut joint_constraints = build_joint_constraints(
        joint_inputs,
        &bodies,
        configuration.time_step(),
        time_step_ratio,
        warm_starting,
    )?;

    if warm_starting {
        for constraint in &constraints {
            warm_start(constraint, &mut bodies)?;
        }
        warm_start_joints(&joint_constraints, &mut bodies)?;
    }
    for _iteration in 0..configuration.velocity_iterations() {
        for constraint in &mut joint_constraints {
            solve_joint_velocity(constraint, &mut bodies)?;
        }
        for constraint in &mut constraints {
            solve_velocity_constraints(constraint, &mut bodies)?;
        }
    }

    let contact_impulses = transient_impulses(&constraints)?;

    for body in &mut bodies {
        integrate_position(body, configuration.time_step());
    }
    let mut position_solved = false;
    for _iteration in 0..configuration.position_iterations() {
        position_solved = true;
        for constraint in &constraints {
            position_solved =
                solve_position_constraints(constraint, &mut bodies)? && position_solved;
        }
        for constraint in &mut joint_constraints {
            position_solved = solve_joint_position(constraint, &mut bodies)? && position_solved;
        }
        if position_solved {
            break;
        }
    }

    for body in &bodies {
        if !body.is_finite() {
            return Err(ContactSolveFailure::NonFinite);
        }
    }
    for constraint in &constraints {
        validate_solution(constraint, &bodies)?;
    }
    let joint_impulses = transient_joint_impulses(&joint_constraints);

    Ok(IslandConstraintSolution {
        motions: bodies
            .into_iter()
            .map(|body| SolvedBodyMotion {
                position: body.transform.position(),
                angle: body.angle,
                linear: body.linear_velocity,
                angular: body.angular_velocity,
            })
            .collect(),
        contact_impulses,
        joint_impulses,
        position_solved,
    })
}

fn integrate_velocity(
    body: &mut SolverBody,
    state: BodyState,
    gravity: Vec2,
    time_step: f32,
) -> Result<(), ContactSolveFailure> {
    if state.snapshot().body_type() != super::body::BodyType::Dynamic {
        return Ok(());
    }
    body.linear_velocity += time_step
        * (state.snapshot().gravity_scale() * gravity
            + state.inverse_mass() * state.accumulated_force());
    body.angular_velocity += time_step * state.inverse_inertia() * state.accumulated_torque();
    body.linear_velocity *= 1.0 / (1.0 + time_step * state.snapshot().linear_damping());
    body.angular_velocity *= 1.0 / (1.0 + time_step * state.snapshot().angular_damping());
    if !body.is_finite() {
        return Err(ContactSolveFailure::NonFinite);
    }
    Ok(())
}

fn integrate_position(body: &mut SolverBody, time_step: f32) {
    let mut translation = time_step * body.linear_velocity;
    if translation.length_squared() > MAX_TRANSLATION_SQUARED {
        body.linear_velocity *= MAX_TRANSLATION / translation.length();
        translation = time_step * body.linear_velocity;
    }
    let mut rotation = time_step * body.angular_velocity;
    if rotation * rotation > MAX_ROTATION_SQUARED {
        body.angular_velocity *= MAX_ROTATION / rotation.abs();
        rotation = time_step * body.angular_velocity;
    }
    body.center += translation;
    body.angle += rotation;
    body.synchronize_transform();
}

#[cfg(test)]
mod tests;

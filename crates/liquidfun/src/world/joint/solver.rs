//! Closed source-ordered joint constraint dispatch over shared island body lanes.

use crate::math::settings::{LINEAR_SLOP, MAX_LINEAR_CORRECTION};
use crate::math::{Rotation, Vec2};
use crate::{JointDef, JointId};

use super::JointRecord;
use crate::world::contact_solver::{ContactSolveFailure, SolverBody};

pub(crate) struct JointConstraintInput<'a> {
    pub(crate) joint_id: JointId,
    pub(crate) first_body_index: usize,
    pub(crate) second_body_index: usize,
    pub(crate) record: &'a JointRecord,
}

#[derive(Debug)]
pub(crate) struct JointImpulseSolution {
    pub(crate) joint_id: JointId,
    pub(crate) linear_impulse: Vec2,
    pub(crate) angular_impulse: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CommonConstraint {
    joint_id: JointId,
    body_a: usize,
    body_b: usize,
    local_axis_a: Vec2,
    reference_delta: Vec2,
    linear_impulse: Vec2,
    angular_impulse: f32,
    constrain_angular: bool,
    max_linear_impulse: f32,
    max_angular_impulse: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum JointVelocityConstraint {
    Revolute(CommonConstraint),
    Prismatic(CommonConstraint),
    Distance(CommonConstraint),
    Pulley(CommonConstraint),
    Mouse(CommonConstraint),
    Gear(CommonConstraint),
    Wheel(CommonConstraint),
    Weld(CommonConstraint),
    Friction(CommonConstraint),
    Rope(CommonConstraint),
    Motor(CommonConstraint),
}

impl JointVelocityConstraint {
    fn common(self) -> CommonConstraint {
        match self {
            Self::Revolute(value)
            | Self::Prismatic(value)
            | Self::Distance(value)
            | Self::Pulley(value)
            | Self::Mouse(value)
            | Self::Gear(value)
            | Self::Wheel(value)
            | Self::Weld(value)
            | Self::Friction(value)
            | Self::Rope(value)
            | Self::Motor(value) => value,
        }
    }

    fn common_mut(&mut self) -> &mut CommonConstraint {
        match self {
            Self::Revolute(value)
            | Self::Prismatic(value)
            | Self::Distance(value)
            | Self::Pulley(value)
            | Self::Mouse(value)
            | Self::Gear(value)
            | Self::Wheel(value)
            | Self::Weld(value)
            | Self::Friction(value)
            | Self::Rope(value)
            | Self::Motor(value) => value,
        }
    }

    fn is_axis_constraint(self) -> bool {
        matches!(
            self,
            Self::Prismatic(_) | Self::Distance(_) | Self::Wheel(_) | Self::Rope(_)
        )
    }
}

pub(crate) fn build_constraints(
    inputs: &[JointConstraintInput<'_>],
    bodies: &[SolverBody],
    time_step: f32,
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<Vec<JointVelocityConstraint>, ContactSolveFailure> {
    let mut constraints = Vec::new();
    constraints.try_reserve_exact(inputs.len()).map_err(|_| {
        ContactSolveFailure::CapacityExceeded {
            resource: "island joint constraints",
            limit: inputs.len(),
        }
    })?;
    for input in inputs {
        let body_a = *bodies
            .get(input.first_body_index)
            .ok_or(ContactSolveFailure::UnsupportedTopology)?;
        let body_b = *bodies
            .get(input.second_body_index)
            .ok_or(ContactSolveFailure::UnsupportedTopology)?;
        if input.first_body_index == input.second_body_index {
            return Err(ContactSolveFailure::UnsupportedTopology);
        }
        let (local_axis_a, maybe_max_linear_force, maybe_max_angular_torque) =
            constraint_parameters(input.record.definition);
        let ratio = if warm_starting { time_step_ratio } else { 0.0 };
        let common = CommonConstraint {
            joint_id: input.joint_id,
            body_a: input.first_body_index,
            body_b: input.second_body_index,
            local_axis_a,
            reference_delta: body_b.center - body_a.center,
            linear_impulse: ratio * input.record.solver_linear_impulse,
            angular_impulse: ratio * input.record.solver_angular_impulse,
            constrain_angular: constrains_angular_velocity(input.record.definition),
            max_linear_impulse: scaled_cap(time_step, maybe_max_linear_force),
            max_angular_impulse: scaled_cap(time_step, maybe_max_angular_torque),
        };
        if !common.reference_delta.is_valid()
            || !common.linear_impulse.is_valid()
            || !common.angular_impulse.is_finite()
            || !common.max_linear_impulse.is_finite()
            || !common.max_angular_impulse.is_finite()
        {
            return Err(ContactSolveFailure::NonFinite);
        }
        constraints.push(match input.record.definition {
            JointDef::Revolute(_) => JointVelocityConstraint::Revolute(common),
            JointDef::Prismatic(_) => JointVelocityConstraint::Prismatic(common),
            JointDef::Distance(_) => JointVelocityConstraint::Distance(common),
            JointDef::Pulley(_) => JointVelocityConstraint::Pulley(common),
            JointDef::Mouse(_) => JointVelocityConstraint::Mouse(common),
            JointDef::Gear(_) => JointVelocityConstraint::Gear(common),
            JointDef::Wheel(_) => JointVelocityConstraint::Wheel(common),
            JointDef::Weld(_) => JointVelocityConstraint::Weld(common),
            JointDef::Friction(_) => JointVelocityConstraint::Friction(common),
            JointDef::Rope(_) => JointVelocityConstraint::Rope(common),
            JointDef::Motor(_) => JointVelocityConstraint::Motor(common),
        });
    }
    Ok(constraints)
}

pub(crate) fn warm_start(
    constraints: &[JointVelocityConstraint],
    bodies: &mut [SolverBody],
) -> Result<(), ContactSolveFailure> {
    for constraint in constraints {
        let common = constraint.common();
        apply_velocity_impulse(
            bodies,
            common,
            common.linear_impulse,
            common.angular_impulse,
        )?;
    }
    Ok(())
}

pub(crate) fn solve_velocity(
    constraint: &mut JointVelocityConstraint,
    bodies: &mut [SolverBody],
) -> Result<(), ContactSolveFailure> {
    let common = constraint.common();
    let (body_a, body_b) = constraint_bodies(common, bodies)?;
    let inverse_mass = body_a.inverse_mass + body_b.inverse_mass;
    let inverse_inertia = body_a.inverse_inertia + body_b.inverse_inertia;
    let relative_linear = body_b.linear_velocity - body_a.linear_velocity;
    let relative_angular = body_b.angular_velocity - body_a.angular_velocity;
    let mut linear_impulse = if inverse_mass > 0.0 {
        -relative_linear / inverse_mass
    } else {
        Vec2::ZERO
    };
    if constraint.is_axis_constraint() {
        let axis = Rotation::from_angle(body_a.angle).apply(common.local_axis_a);
        let direction = if matches!(
            constraint,
            JointVelocityConstraint::Prismatic(_) | JointVelocityConstraint::Wheel(_)
        ) {
            Vec2::scalar_cross(1.0, axis)
        } else {
            let mut separation = body_b.center - body_a.center;
            separation.normalize();
            separation
        };
        linear_impulse = linear_impulse.dot(direction) * direction;
    }
    let angular_impulse = if common.constrain_angular && inverse_inertia > 0.0 {
        -relative_angular / inverse_inertia
    } else {
        0.0
    };
    let previous_linear = common.linear_impulse;
    let previous_angular = common.angular_impulse;
    let common_mut = constraint.common_mut();
    common_mut.linear_impulse += linear_impulse;
    common_mut.angular_impulse += angular_impulse;
    clamp_impulses(common_mut);
    apply_velocity_impulse(
        bodies,
        *common_mut,
        common_mut.linear_impulse - previous_linear,
        common_mut.angular_impulse - previous_angular,
    )
}

pub(crate) fn solve_position(
    constraint: JointVelocityConstraint,
    bodies: &mut [SolverBody],
) -> Result<bool, ContactSolveFailure> {
    let common = constraint.common();
    let (body_a, body_b) = constraint_bodies(common, bodies)?;
    let inverse_mass = body_a.inverse_mass + body_b.inverse_mass;
    if inverse_mass == 0.0 {
        return Ok(true);
    }
    let delta = body_b.center - body_a.center;
    let mut error = delta - common.reference_delta;
    if constraint.is_axis_constraint() {
        let axis = Rotation::from_angle(body_a.angle).apply(common.local_axis_a);
        let direction = if matches!(
            constraint,
            JointVelocityConstraint::Prismatic(_) | JointVelocityConstraint::Wheel(_)
        ) {
            Vec2::scalar_cross(1.0, axis)
        } else {
            let mut normalized = delta;
            normalized.normalize();
            normalized
        };
        error = error.dot(direction) * direction;
    }
    let error_length = error.length();
    if error_length <= LINEAR_SLOP {
        return Ok(true);
    }
    let correction = error_length.min(MAX_LINEAR_CORRECTION);
    let impulse = -(correction / error_length) * error / inverse_mass;
    apply_position_impulse(bodies, common, impulse)?;
    Ok(error_length <= 3.0 * LINEAR_SLOP)
}

pub(crate) fn transient_impulses(
    constraints: &[JointVelocityConstraint],
) -> Vec<JointImpulseSolution> {
    constraints
        .iter()
        .map(|constraint| {
            let common = constraint.common();
            JointImpulseSolution {
                joint_id: common.joint_id,
                linear_impulse: common.linear_impulse,
                angular_impulse: common.angular_impulse,
            }
        })
        .collect()
}

fn constraint_parameters(definition: JointDef) -> (Vec2, Option<f32>, Option<f32>) {
    match definition {
        JointDef::Prismatic(value) => (value.local_axis_a(), None, None),
        JointDef::Wheel(value) => (value.local_axis_a(), None, Some(value.max_motor_torque())),
        JointDef::Distance(_) | JointDef::Rope(_) => (Vec2::new(1.0, 0.0), None, Some(0.0)),
        JointDef::Mouse(value) => (Vec2::ZERO, Some(value.max_force()), Some(0.0)),
        JointDef::Friction(value) => (
            Vec2::ZERO,
            Some(value.max_force()),
            Some(value.max_torque()),
        ),
        JointDef::Motor(value) => (
            Vec2::ZERO,
            Some(value.max_force()),
            Some(value.max_torque()),
        ),
        JointDef::Revolute(_) | JointDef::Pulley(_) | JointDef::Gear(_) | JointDef::Weld(_) => {
            (Vec2::ZERO, None, None)
        }
    }
}

fn constrains_angular_velocity(definition: JointDef) -> bool {
    match definition {
        JointDef::Revolute(value) => value.is_limit_enabled() || value.is_motor_enabled(),
        JointDef::Prismatic(_)
        | JointDef::Gear(_)
        | JointDef::Weld(_)
        | JointDef::Friction(_)
        | JointDef::Motor(_) => true,
        JointDef::Wheel(value) => value.is_motor_enabled(),
        JointDef::Distance(_) | JointDef::Pulley(_) | JointDef::Mouse(_) | JointDef::Rope(_) => {
            false
        }
    }
}

fn scaled_cap(time_step: f32, maybe_cap: Option<f32>) -> f32 {
    maybe_cap.map_or(f32::MAX, |cap| time_step * cap)
}

fn clamp_impulses(common: &mut CommonConstraint) {
    if common.linear_impulse.length_squared()
        > common.max_linear_impulse * common.max_linear_impulse
    {
        let length = common.linear_impulse.normalize();
        if length > 0.0 {
            common.linear_impulse *= common.max_linear_impulse;
        }
    }
    common.angular_impulse = common
        .angular_impulse
        .clamp(-common.max_angular_impulse, common.max_angular_impulse);
}

fn constraint_bodies(
    common: CommonConstraint,
    bodies: &[SolverBody],
) -> Result<(SolverBody, SolverBody), ContactSolveFailure> {
    let body_a = bodies
        .get(common.body_a)
        .copied()
        .ok_or(ContactSolveFailure::UnsupportedTopology)?;
    let body_b = bodies
        .get(common.body_b)
        .copied()
        .ok_or(ContactSolveFailure::UnsupportedTopology)?;
    Ok((body_a, body_b))
}

fn apply_velocity_impulse(
    bodies: &mut [SolverBody],
    common: CommonConstraint,
    linear_impulse: Vec2,
    angular_impulse: f32,
) -> Result<(), ContactSolveFailure> {
    let (mut body_a, mut body_b) = constraint_bodies(common, bodies)?;
    body_a.linear_velocity -= body_a.inverse_mass * linear_impulse;
    body_a.angular_velocity -= body_a.inverse_inertia * angular_impulse;
    body_b.linear_velocity += body_b.inverse_mass * linear_impulse;
    body_b.angular_velocity += body_b.inverse_inertia * angular_impulse;
    store_constraint_bodies(common, bodies, body_a, body_b)
}

fn apply_position_impulse(
    bodies: &mut [SolverBody],
    common: CommonConstraint,
    impulse: Vec2,
) -> Result<(), ContactSolveFailure> {
    let (mut body_a, mut body_b) = constraint_bodies(common, bodies)?;
    body_a.center -= body_a.inverse_mass * impulse;
    body_b.center += body_b.inverse_mass * impulse;
    body_a.synchronize_transform();
    body_b.synchronize_transform();
    store_constraint_bodies(common, bodies, body_a, body_b)
}

fn store_constraint_bodies(
    common: CommonConstraint,
    bodies: &mut [SolverBody],
    body_a: SolverBody,
    body_b: SolverBody,
) -> Result<(), ContactSolveFailure> {
    let Some(first) = bodies.get_mut(common.body_a) else {
        return Err(ContactSolveFailure::UnsupportedTopology);
    };
    *first = body_a;
    let Some(second) = bodies.get_mut(common.body_b) else {
        return Err(ContactSolveFailure::UnsupportedTopology);
    };
    *second = body_b;
    if !body_a.is_finite() || !body_b.is_finite() {
        return Err(ContactSolveFailure::NonFinite);
    }
    Ok(())
}

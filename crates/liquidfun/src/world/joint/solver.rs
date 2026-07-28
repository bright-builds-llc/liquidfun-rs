//! Closed source-ordered joint constraint dispatch over shared island body lanes.

use crate::math::settings::{
    ANGULAR_SLOP, LINEAR_SLOP, MAX_ANGULAR_CORRECTION, MAX_LINEAR_CORRECTION,
};
use crate::math::{Mat22, Mat33, Rotation, Vec2, Vec3};
use crate::{
    BodyId, DistanceJointDef, FrictionJointDef, GearJointDef, JointDef, JointId, MotorJointDef,
    MouseJointDef, PrismaticJointDef, PulleyJointDef, RevoluteJointDef, RopeJointDef, WeldJointDef,
    WheelJointDef,
};

use super::{
    JointRuntime, distance::DistanceRuntime, friction::FrictionRuntime, gear::GearRuntime,
    motor::MotorRuntime, mouse::MouseRuntime, prismatic::PrismaticRuntime, pulley::PulleyRuntime,
    revolute::RevoluteRuntime, rope::RopeJointRuntime, weld::WeldRuntime, wheel::WheelRuntime,
};
use crate::world::contact_solver::{ContactSolveFailure, SolverBody};

/// One semantic body lane and its optional position in the current island scratch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SolverBodyLane {
    body_id: BodyId,
    maybe_solver_index: Option<usize>,
}

impl SolverBodyLane {
    pub(crate) const fn resolved(body_id: BodyId, solver_index: usize) -> Self {
        Self {
            body_id,
            maybe_solver_index: Some(solver_index),
        }
    }

    pub(crate) const fn unresolved(body_id: BodyId) -> Self {
        Self {
            body_id,
            maybe_solver_index: None,
        }
    }

    #[allow(
        dead_code,
        reason = "family plans consume semantic lane identity during activation"
    )]
    pub(crate) const fn body_id(self) -> BodyId {
        self.body_id
    }

    #[allow(
        dead_code,
        reason = "family plans consume resolved lane indices during activation"
    )]
    pub(crate) const fn maybe_solver_index(self) -> Option<usize> {
        self.maybe_solver_index
    }

    fn solver_index(self, bodies: &[SolverBody]) -> Result<usize, ContactSolveFailure> {
        let Some(index) = self.maybe_solver_index else {
            return Err(ContactSolveFailure::UnsupportedTopology);
        };
        if bodies.get(index).is_none() {
            return Err(ContactSolveFailure::UnsupportedTopology);
        }
        Ok(index)
    }
}

/// The exact A/B lanes used by every ordinary joint family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OrdinarySolverLanes {
    body_a: SolverBodyLane,
    body_b: SolverBodyLane,
}

impl OrdinarySolverLanes {
    pub(crate) const fn new(body_a: SolverBodyLane, body_b: SolverBodyLane) -> Self {
        Self { body_a, body_b }
    }

    #[allow(
        dead_code,
        reason = "family plans construct typed ordinary lanes during activation"
    )]
    pub(crate) const fn resolved(
        body_a: BodyId,
        index_a: usize,
        body_b: BodyId,
        index_b: usize,
    ) -> Self {
        Self::new(
            SolverBodyLane::resolved(body_a, index_a),
            SolverBodyLane::resolved(body_b, index_b),
        )
    }

    fn solver_indices(self, bodies: &[SolverBody]) -> Result<[usize; 2], ContactSolveFailure> {
        if self.body_a.body_id == self.body_b.body_id {
            return Err(ContactSolveFailure::UnsupportedTopology);
        }
        let indices = [
            self.body_a.solver_index(bodies)?,
            self.body_b.solver_index(bodies)?,
        ];
        if indices[0] == indices[1] {
            return Err(ContactSolveFailure::UnsupportedTopology);
        }
        Ok(indices)
    }
}

/// The source-semantic A/B/C/D lanes required by a gear joint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GearSolverLanes {
    a: SolverBodyLane,
    b: SolverBodyLane,
    c: SolverBodyLane,
    d: SolverBodyLane,
}

impl GearSolverLanes {
    pub(crate) const fn new(
        body_a: SolverBodyLane,
        body_b: SolverBodyLane,
        body_c: SolverBodyLane,
        body_d: SolverBodyLane,
    ) -> Self {
        Self {
            a: body_a,
            b: body_b,
            c: body_c,
            d: body_d,
        }
    }

    fn solver_indices(self, bodies: &[SolverBody]) -> Result<[usize; 4], ContactSolveFailure> {
        Ok([
            self.a.solver_index(bodies)?,
            self.b.solver_index(bodies)?,
            self.c.solver_index(bodies)?,
            self.d.solver_index(bodies)?,
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum JointSolverLanes {
    Ordinary(OrdinarySolverLanes),
    Gear(GearSolverLanes),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct JointConstraintInput {
    pub(crate) joint_id: JointId,
    pub(crate) lanes: JointSolverLanes,
    pub(crate) definition: JointDef,
    pub(crate) runtime: JointRuntime,
}

impl JointConstraintInput {
    pub(crate) const fn ordinary(
        joint_id: JointId,
        lanes: OrdinarySolverLanes,
        definition: JointDef,
        runtime: JointRuntime,
    ) -> Self {
        Self {
            joint_id,
            lanes: JointSolverLanes::Ordinary(lanes),
            definition,
            runtime,
        }
    }

    pub(crate) const fn gear(
        joint_id: JointId,
        lanes: &GearSolverLanes,
        definition: GearJointDef,
        runtime: GearRuntime,
    ) -> Self {
        Self {
            joint_id,
            lanes: JointSolverLanes::Gear(*lanes),
            definition: JointDef::Gear(definition),
            runtime: JointRuntime::Gear(runtime),
        }
    }
}

#[derive(Debug)]
pub(crate) struct JointImpulseSolution {
    pub(crate) joint_id: JointId,
    pub(crate) runtime: JointRuntime,
}

#[derive(Debug, Clone, Copy)]
#[allow(
    dead_code,
    reason = "family plans consume typed definitions and lanes during activation"
)]
pub(crate) struct FamilyCandidate<D, R, L> {
    joint_id: JointId,
    lanes: L,
    definition: D,
    runtime: R,
}

#[derive(Debug, Clone, Copy)]
#[allow(
    clippy::large_enum_variant,
    reason = "transactional candidates intentionally own complete typed definitions and runtimes"
)]
pub(crate) enum JointVelocityConstraint {
    Revolute(RevoluteConstraint),
    Prismatic(PrismaticConstraint),
    Distance(DistanceConstraint),
    Pulley(PulleyConstraint),
    Mouse(MouseConstraint),
    Gear(GearConstraint),
    Wheel(WheelConstraint),
    Weld(WeldConstraint),
    Friction(FrictionConstraint),
    Rope(RopeConstraint),
    Motor(MotorConstraint),
}

mod assembly;
mod body_access;
mod distance_pulley_mouse;
mod friction_rope_motor;
mod gear_constraint;
mod primary;
mod staging;
#[cfg(test)]
mod tests;
mod wheel_weld;

pub(crate) use assembly::build_constraints;
use body_access::{
    point_velocity_difference, solver_body, solver_body_pair, store_solver_body,
    store_solver_body_pair, typed_solution,
};
use gear_constraint::gear_solver_bodies;
#[cfg(test)]
use gear_constraint::store_gear_velocity_deltas;
use staging::{map_joint_error, normalized_pulley_segment, point_angle_mass};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RevoluteConstraint {
    candidate: FamilyCandidate<RevoluteJointDef, RevoluteRuntime, OrdinarySolverLanes>,
    body_a: usize,
    body_b: usize,
    r_a: Vec2,
    r_b: Vec2,
    mass: Mat33,
    motor_mass: f32,
    fixed_rotation: bool,
    time_step: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PrismaticConstraint {
    candidate: FamilyCandidate<PrismaticJointDef, PrismaticRuntime, OrdinarySolverLanes>,
    body_a: usize,
    body_b: usize,
    r_a: Vec2,
    r_b: Vec2,
    axis: Vec2,
    perpendicular: Vec2,
    a1: f32,
    a2: f32,
    s1: f32,
    s2: f32,
    mass: Mat33,
    motor_mass: f32,
    time_step: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DistanceConstraint {
    candidate: FamilyCandidate<DistanceJointDef, DistanceRuntime, OrdinarySolverLanes>,
    body_a: usize,
    body_b: usize,
    r_a: Vec2,
    r_b: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PulleyConstraint {
    candidate: FamilyCandidate<PulleyJointDef, PulleyRuntime, OrdinarySolverLanes>,
    body_a: usize,
    body_b: usize,
    r_a: Vec2,
    r_b: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MouseConstraint {
    candidate: FamilyCandidate<MouseJointDef, MouseRuntime, OrdinarySolverLanes>,
    body_b: usize,
    r_b: Vec2,
    angular_damping: f32,
    time_step: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GearConstraint {
    candidate: FamilyCandidate<GearJointDef, GearRuntime, GearSolverLanes>,
    body_indices: [usize; 4],
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct WheelConstraint {
    candidate: FamilyCandidate<WheelJointDef, WheelRuntime, OrdinarySolverLanes>,
    body_a: usize,
    body_b: usize,
    r_a: Vec2,
    r_b: Vec2,
    axis: Vec2,
    perpendicular: Vec2,
    spring_lever_a: f32,
    spring_lever_b: f32,
    line_lever_a: f32,
    line_lever_b: f32,
    time_step: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct WeldConstraint {
    candidate: FamilyCandidate<WeldJointDef, WeldRuntime, OrdinarySolverLanes>,
    body_a: usize,
    body_b: usize,
    r_a: Vec2,
    r_b: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FrictionConstraint {
    candidate: FamilyCandidate<FrictionJointDef, FrictionRuntime, OrdinarySolverLanes>,
    body_a: usize,
    body_b: usize,
    r_a: Vec2,
    r_b: Vec2,
    time_step: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RopeConstraint {
    candidate: FamilyCandidate<RopeJointDef, RopeJointRuntime, OrdinarySolverLanes>,
    body_a: usize,
    body_b: usize,
    r_a: Vec2,
    r_b: Vec2,
    mass: f32,
    inverse_time_step: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MotorConstraint {
    candidate: FamilyCandidate<MotorJointDef, MotorRuntime, OrdinarySolverLanes>,
    body_a: usize,
    body_b: usize,
    r_a: Vec2,
    r_b: Vec2,
    time_step: f32,
    inverse_time_step: f32,
}

pub(crate) fn warm_start(
    constraints: &[JointVelocityConstraint],
    bodies: &mut [SolverBody],
) -> Result<(), ContactSolveFailure> {
    for constraint in constraints {
        match constraint {
            JointVelocityConstraint::Revolute(stage) => stage.warm_start(bodies)?,
            JointVelocityConstraint::Prismatic(stage) => stage.warm_start(bodies)?,
            JointVelocityConstraint::Distance(stage) => stage.warm_start(bodies)?,
            JointVelocityConstraint::Pulley(stage) => stage.warm_start(bodies)?,
            JointVelocityConstraint::Mouse(stage) => stage.warm_start(bodies)?,
            JointVelocityConstraint::Gear(stage) => stage.warm_start(bodies)?,
            JointVelocityConstraint::Wheel(stage) => stage.warm_start(bodies)?,
            JointVelocityConstraint::Weld(stage) => stage.warm_start(bodies)?,
            JointVelocityConstraint::Friction(stage) => stage.warm_start(bodies)?,
            JointVelocityConstraint::Rope(stage) => stage.warm_start(bodies)?,
            JointVelocityConstraint::Motor(stage) => stage.warm_start(bodies)?,
        }
    }
    Ok(())
}

pub(crate) fn solve_velocity(
    constraint: &mut JointVelocityConstraint,
    bodies: &mut [SolverBody],
) -> Result<(), ContactSolveFailure> {
    match constraint {
        JointVelocityConstraint::Revolute(stage) => stage.solve_velocity(bodies),
        JointVelocityConstraint::Prismatic(stage) => stage.solve_velocity(bodies),
        JointVelocityConstraint::Distance(stage) => stage.solve_velocity(bodies),
        JointVelocityConstraint::Pulley(stage) => stage.solve_velocity(bodies),
        JointVelocityConstraint::Mouse(stage) => stage.solve_velocity(bodies),
        JointVelocityConstraint::Gear(stage) => stage.solve_velocity(bodies),
        JointVelocityConstraint::Wheel(stage) => stage.solve_velocity(bodies),
        JointVelocityConstraint::Weld(stage) => stage.solve_velocity(bodies),
        JointVelocityConstraint::Friction(stage) => stage.solve_velocity(bodies),
        JointVelocityConstraint::Rope(stage) => stage.solve_velocity(bodies),
        JointVelocityConstraint::Motor(stage) => stage.solve_velocity(bodies),
    }
}

pub(crate) fn solve_position(
    constraint: &mut JointVelocityConstraint,
    bodies: &mut [SolverBody],
) -> Result<bool, ContactSolveFailure> {
    match constraint {
        JointVelocityConstraint::Revolute(stage) => stage.solve_position(bodies),
        JointVelocityConstraint::Prismatic(stage) => stage.solve_position(bodies),
        JointVelocityConstraint::Distance(stage) => stage.solve_position(bodies),
        JointVelocityConstraint::Pulley(stage) => stage.solve_position(bodies),
        JointVelocityConstraint::Mouse(_stage) => Ok(true),
        JointVelocityConstraint::Gear(stage) => stage.solve_position(bodies),
        JointVelocityConstraint::Wheel(stage) => stage.solve_position(bodies),
        JointVelocityConstraint::Weld(stage) => stage.solve_position(bodies),
        JointVelocityConstraint::Friction(_) | JointVelocityConstraint::Motor(_) => Ok(true),
        JointVelocityConstraint::Rope(stage) => stage.solve_position(bodies),
    }
}

pub(crate) fn transient_impulses(
    constraints: &[JointVelocityConstraint],
) -> Vec<JointImpulseSolution> {
    constraints
        .iter()
        .map(|constraint| match constraint {
            JointVelocityConstraint::Revolute(stage) => stage.finalize(),
            JointVelocityConstraint::Prismatic(stage) => stage.finalize(),
            JointVelocityConstraint::Distance(stage) => stage.finalize(),
            JointVelocityConstraint::Pulley(stage) => stage.finalize(),
            JointVelocityConstraint::Mouse(stage) => stage.finalize(),
            JointVelocityConstraint::Gear(stage) => stage.finalize(),
            JointVelocityConstraint::Wheel(stage) => stage.finalize(),
            JointVelocityConstraint::Weld(stage) => stage.finalize(),
            JointVelocityConstraint::Friction(stage) => stage.finalize(),
            JointVelocityConstraint::Rope(stage) => stage.finalize(),
            JointVelocityConstraint::Motor(stage) => stage.finalize(),
        })
        .collect()
}

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
    pub(crate) legacy_linear_impulse: Vec2,
    pub(crate) legacy_angular_impulse: f32,
}

impl JointConstraintInput {
    pub(crate) const fn ordinary(
        joint_id: JointId,
        lanes: OrdinarySolverLanes,
        definition: JointDef,
        runtime: JointRuntime,
        legacy_linear_impulse: Vec2,
        legacy_angular_impulse: f32,
    ) -> Self {
        Self {
            joint_id,
            lanes: JointSolverLanes::Ordinary(lanes),
            definition,
            runtime,
            legacy_linear_impulse,
            legacy_angular_impulse,
        }
    }

    pub(crate) const fn gear(
        joint_id: JointId,
        lanes: &GearSolverLanes,
        definition: GearJointDef,
        runtime: GearRuntime,
        legacy_linear_impulse: Vec2,
        legacy_angular_impulse: f32,
    ) -> Self {
        Self {
            joint_id,
            lanes: JointSolverLanes::Gear(*lanes),
            definition: JointDef::Gear(definition),
            runtime: JointRuntime::Gear(runtime),
            legacy_linear_impulse,
            legacy_angular_impulse,
        }
    }
}

#[derive(Debug)]
pub(crate) struct JointImpulseSolution {
    pub(crate) joint_id: JointId,
    pub(crate) linear_impulse: Vec2,
    pub(crate) angular_impulse: f32,
    pub(crate) maybe_runtime: Option<JointRuntime>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CommonConstraint {
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
pub(crate) struct LegacyUnmigrated<C> {
    candidate: C,
    constraint: CommonConstraint,
}

/// Closed activation state. Only the legacy variant owns the compatibility solver.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code, reason = "family plans activate variants incrementally")]
pub(crate) enum FamilyActivation<C> {
    Activated(C),
    LegacyUnmigrated(LegacyUnmigrated<C>),
}

pub(crate) trait CompleteRuntimeCandidate: Copy {
    fn joint_id(self) -> JointId;
    fn complete_runtime(self) -> JointRuntime;
}

macro_rules! runtime_candidate {
    ($definition:ty, $runtime:ty, $lanes:ty, $variant:ident) => {
        impl CompleteRuntimeCandidate for FamilyCandidate<$definition, $runtime, $lanes> {
            fn joint_id(self) -> JointId {
                self.joint_id
            }

            fn complete_runtime(self) -> JointRuntime {
                JointRuntime::$variant(self.runtime)
            }
        }
    };
}

runtime_candidate!(
    RevoluteJointDef,
    RevoluteRuntime,
    OrdinarySolverLanes,
    Revolute
);
runtime_candidate!(
    PrismaticJointDef,
    PrismaticRuntime,
    OrdinarySolverLanes,
    Prismatic
);
runtime_candidate!(
    DistanceJointDef,
    DistanceRuntime,
    OrdinarySolverLanes,
    Distance
);
runtime_candidate!(PulleyJointDef, PulleyRuntime, OrdinarySolverLanes, Pulley);
runtime_candidate!(MouseJointDef, MouseRuntime, OrdinarySolverLanes, Mouse);
runtime_candidate!(GearJointDef, GearRuntime, GearSolverLanes, Gear);
runtime_candidate!(WheelJointDef, WheelRuntime, OrdinarySolverLanes, Wheel);
runtime_candidate!(WeldJointDef, WeldRuntime, OrdinarySolverLanes, Weld);
runtime_candidate!(
    FrictionJointDef,
    FrictionRuntime,
    OrdinarySolverLanes,
    Friction
);
runtime_candidate!(RopeJointDef, RopeJointRuntime, OrdinarySolverLanes, Rope);
runtime_candidate!(MotorJointDef, MotorRuntime, OrdinarySolverLanes, Motor);

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
    Gear(FamilyActivation<FamilyCandidate<GearJointDef, GearRuntime, GearSolverLanes>>),
    Wheel(FamilyActivation<FamilyCandidate<WheelJointDef, WheelRuntime, OrdinarySolverLanes>>),
    Weld(FamilyActivation<FamilyCandidate<WeldJointDef, WeldRuntime, OrdinarySolverLanes>>),
    Friction(
        FamilyActivation<FamilyCandidate<FrictionJointDef, FrictionRuntime, OrdinarySolverLanes>>,
    ),
    Rope(FamilyActivation<FamilyCandidate<RopeJointDef, RopeJointRuntime, OrdinarySolverLanes>>),
    Motor(FamilyActivation<FamilyCandidate<MotorJointDef, MotorRuntime, OrdinarySolverLanes>>),
}

#[allow(
    clippy::too_many_lines,
    reason = "one exhaustive construction match keeps all eleven typed family pairings auditable"
)]
pub(crate) fn build_constraints(
    inputs: &[JointConstraintInput],
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
        constraints.push(match (input.definition, input.runtime, input.lanes) {
            (
                JointDef::Revolute(definition),
                JointRuntime::Revolute(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Revolute(stage_revolute(
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Prismatic(definition),
                JointRuntime::Prismatic(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Prismatic(stage_prismatic(
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Distance(definition),
                JointRuntime::Distance(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Distance(stage_distance(
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Pulley(definition),
                JointRuntime::Pulley(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Pulley(stage_pulley(
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Mouse(definition),
                JointRuntime::Mouse(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Mouse(stage_mouse(
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Gear(definition),
                JointRuntime::Gear(runtime),
                JointSolverLanes::Gear(lanes),
            ) => JointVelocityConstraint::Gear(stage_legacy_unmigrated(
                input,
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Wheel(definition),
                JointRuntime::Wheel(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Wheel(stage_legacy_unmigrated(
                input,
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Weld(definition),
                JointRuntime::Weld(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Weld(stage_legacy_unmigrated(
                input,
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Friction(definition),
                JointRuntime::Friction(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Friction(stage_legacy_unmigrated(
                input,
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Rope(definition),
                JointRuntime::Rope(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Rope(stage_legacy_unmigrated(
                input,
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Motor(definition),
                JointRuntime::Motor(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Motor(stage_legacy_unmigrated(
                input,
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            _ => return Err(ContactSolveFailure::UnsupportedTopology),
        });
    }
    Ok(constraints)
}

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

fn stage_revolute(
    mut candidate: FamilyCandidate<RevoluteJointDef, RevoluteRuntime, OrdinarySolverLanes>,
    bodies: &[SolverBody],
    time_step: f32,
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<RevoluteConstraint, ContactSolveFailure> {
    let [first_index, second_index] = candidate.lanes.solver_indices(bodies)?;
    let body_a = bodies[first_index];
    let body_b = bodies[second_index];
    let q_a = Rotation::from_angle(body_a.angle);
    let q_b = Rotation::from_angle(body_b.angle);
    let r_a = q_a.apply(candidate.definition.local_anchor_a() - body_a.local_center);
    let r_b = q_b.apply(candidate.definition.local_anchor_b() - body_b.local_center);
    let m_a = body_a.inverse_mass;
    let m_b = body_b.inverse_mass;
    let i_a = body_a.inverse_inertia;
    let i_b = body_b.inverse_inertia;
    let fixed_rotation = i_a + i_b == 0.0;
    let mass = Mat33::from_columns(
        Vec3::new(
            m_a + m_b + r_a.y * r_a.y * i_a + r_b.y * r_b.y * i_b,
            -r_a.y * r_a.x * i_a - r_b.y * r_b.x * i_b,
            -r_a.y * i_a - r_b.y * i_b,
        ),
        Vec3::new(
            -r_a.y * r_a.x * i_a - r_b.y * r_b.x * i_b,
            m_a + m_b + r_a.x * r_a.x * i_a + r_b.x * r_b.x * i_b,
            r_a.x * i_a + r_b.x * i_b,
        ),
        Vec3::new(
            -r_a.y * i_a - r_b.y * i_b,
            r_a.x * i_a + r_b.x * i_b,
            i_a + i_b,
        ),
    );
    let motor_inverse_mass = i_a + i_b;
    let motor_mass = if motor_inverse_mass > 0.0 {
        1.0 / motor_inverse_mass
    } else {
        0.0
    };
    let angle = body_b.angle - body_a.angle - candidate.definition.reference_angle();
    candidate
        .runtime
        .initialize(
            candidate.definition,
            angle,
            warm_starting.then_some(time_step_ratio),
            fixed_rotation,
        )
        .map_err(map_joint_error)?;
    let constraint = RevoluteConstraint {
        candidate,
        body_a: first_index,
        body_b: second_index,
        r_a,
        r_b,
        mass,
        motor_mass,
        fixed_rotation,
        time_step,
    };
    if !constraint.is_finite() {
        return Err(ContactSolveFailure::NonFinite);
    }
    Ok(constraint)
}

fn stage_prismatic(
    mut candidate: FamilyCandidate<PrismaticJointDef, PrismaticRuntime, OrdinarySolverLanes>,
    bodies: &[SolverBody],
    time_step: f32,
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<PrismaticConstraint, ContactSolveFailure> {
    let [first_index, second_index] = candidate.lanes.solver_indices(bodies)?;
    let body_a = bodies[first_index];
    let body_b = bodies[second_index];
    let q_a = Rotation::from_angle(body_a.angle);
    let q_b = Rotation::from_angle(body_b.angle);
    let r_a = q_a.apply(candidate.definition.local_anchor_a() - body_a.local_center);
    let r_b = q_b.apply(candidate.definition.local_anchor_b() - body_b.local_center);
    let d = (body_b.center - body_a.center) + r_b - r_a;
    let m_a = body_a.inverse_mass;
    let m_b = body_b.inverse_mass;
    let i_a = body_a.inverse_inertia;
    let i_b = body_b.inverse_inertia;
    let axis = q_a.apply(candidate.definition.local_axis_a());
    let a1 = (d + r_a).cross(axis);
    let a2 = r_b.cross(axis);
    let motor_inverse_mass = m_a + m_b + i_a * a1 * a1 + i_b * a2 * a2;
    let motor_mass = if motor_inverse_mass > 0.0 {
        1.0 / motor_inverse_mass
    } else {
        0.0
    };
    let perpendicular = q_a.apply(Vec2::scalar_cross(1.0, candidate.definition.local_axis_a()));
    let s1 = (d + r_a).cross(perpendicular);
    let s2 = r_b.cross(perpendicular);
    let k11 = m_a + m_b + i_a * s1 * s1 + i_b * s2 * s2;
    let k12 = i_a * s1 + i_b * s2;
    let k13 = i_a * s1 * a1 + i_b * s2 * a2;
    let mut k22 = i_a + i_b;
    if k22 == 0.0 {
        k22 = 1.0;
    }
    let k23 = i_a * a1 + i_b * a2;
    let k33 = m_a + m_b + i_a * a1 * a1 + i_b * a2 * a2;
    let mass = Mat33::from_columns(
        Vec3::new(k11, k12, k13),
        Vec3::new(k12, k22, k23),
        Vec3::new(k13, k23, k33),
    );
    let translation = axis.dot(d);
    candidate
        .runtime
        .initialize(
            candidate.definition,
            translation,
            axis,
            warm_starting.then_some(time_step_ratio),
        )
        .map_err(map_joint_error)?;
    let constraint = PrismaticConstraint {
        candidate,
        body_a: first_index,
        body_b: second_index,
        r_a,
        r_b,
        axis,
        perpendicular,
        a1,
        a2,
        s1,
        s2,
        mass,
        motor_mass,
        time_step,
    };
    if !constraint.is_finite() {
        return Err(ContactSolveFailure::NonFinite);
    }
    Ok(constraint)
}

fn stage_distance(
    mut candidate: FamilyCandidate<DistanceJointDef, DistanceRuntime, OrdinarySolverLanes>,
    bodies: &[SolverBody],
    time_step: f32,
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<DistanceConstraint, ContactSolveFailure> {
    let [first_index, second_index] = candidate.lanes.solver_indices(bodies)?;
    let body_a = bodies[first_index];
    let body_b = bodies[second_index];
    let q_a = Rotation::from_angle(body_a.angle);
    let q_b = Rotation::from_angle(body_b.angle);
    let r_a = q_a.apply(candidate.definition.local_anchor_a() - body_a.local_center);
    let r_b = q_b.apply(candidate.definition.local_anchor_b() - body_b.local_center);
    let displacement = body_b.center + r_b - body_a.center - r_a;
    let mut direction = displacement;
    let length = direction.length();
    if length > LINEAR_SLOP {
        direction *= 1.0 / length;
    } else {
        direction = Vec2::ZERO;
    }
    let anchor_lever_a = r_a.cross(direction);
    let anchor_lever_b = r_b.cross(direction);
    let inverse_mass = body_a.inverse_mass
        + body_a.inverse_inertia * anchor_lever_a * anchor_lever_a
        + body_b.inverse_mass
        + body_b.inverse_inertia * anchor_lever_b * anchor_lever_b;
    candidate
        .runtime
        .initialize(
            candidate.definition,
            displacement,
            inverse_mass,
            time_step,
            warm_starting.then_some(time_step_ratio),
        )
        .map_err(map_joint_error)?;
    let constraint = DistanceConstraint {
        candidate,
        body_a: first_index,
        body_b: second_index,
        r_a,
        r_b,
    };
    if !constraint.is_finite() {
        return Err(ContactSolveFailure::NonFinite);
    }
    Ok(constraint)
}

fn stage_pulley(
    mut candidate: FamilyCandidate<PulleyJointDef, PulleyRuntime, OrdinarySolverLanes>,
    bodies: &[SolverBody],
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<PulleyConstraint, ContactSolveFailure> {
    let [first_index, second_index] = candidate.lanes.solver_indices(bodies)?;
    let body_a = bodies[first_index];
    let body_b = bodies[second_index];
    let q_a = Rotation::from_angle(body_a.angle);
    let q_b = Rotation::from_angle(body_b.angle);
    let r_a = q_a.apply(candidate.definition.local_anchor_a() - body_a.local_center);
    let r_b = q_b.apply(candidate.definition.local_anchor_b() - body_b.local_center);
    let segment_a = body_a.center + r_a - candidate.definition.ground_anchor_a();
    let segment_b = body_b.center + r_b - candidate.definition.ground_anchor_b();
    let direction_a = normalized_pulley_segment(segment_a);
    let direction_b = normalized_pulley_segment(segment_b);
    let anchor_lever_a = r_a.cross(direction_a);
    let anchor_lever_b = r_b.cross(direction_b);
    let effective_mass_a =
        body_a.inverse_mass + body_a.inverse_inertia * anchor_lever_a * anchor_lever_a;
    let effective_mass_b =
        body_b.inverse_mass + body_b.inverse_inertia * anchor_lever_b * anchor_lever_b;
    candidate
        .runtime
        .initialize(
            candidate.definition,
            segment_a,
            segment_b,
            effective_mass_a,
            effective_mass_b,
            warm_starting.then_some(time_step_ratio),
        )
        .map_err(map_joint_error)?;
    let constraint = PulleyConstraint {
        candidate,
        body_a: first_index,
        body_b: second_index,
        r_a,
        r_b,
    };
    if !constraint.is_finite() {
        return Err(ContactSolveFailure::NonFinite);
    }
    Ok(constraint)
}

fn stage_mouse(
    mut candidate: FamilyCandidate<MouseJointDef, MouseRuntime, OrdinarySolverLanes>,
    bodies: &[SolverBody],
    time_step: f32,
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<MouseConstraint, ContactSolveFailure> {
    let [_first_index, second_index] = candidate.lanes.solver_indices(bodies)?;
    let body_b = bodies[second_index];
    let q_b = Rotation::from_angle(body_b.angle);
    let r_b = q_b.apply(candidate.runtime.solver_local_anchor_b() - body_b.local_center);
    let body_mass = if body_b.inverse_mass > 0.0 {
        1.0 / body_b.inverse_mass
    } else {
        0.0
    };
    let damped_angular_velocity = candidate
        .runtime
        .initialize(
            candidate.definition,
            time_step,
            body_mass,
            body_b.inverse_mass,
            body_b.inverse_inertia,
            r_b,
            body_b.center,
            warm_starting.then_some(time_step_ratio),
            body_b.angular_velocity,
        )
        .map_err(map_joint_error)?;
    if !damped_angular_velocity.is_finite() {
        return Err(ContactSolveFailure::NonFinite);
    }
    let constraint = MouseConstraint {
        candidate,
        body_b: second_index,
        r_b,
        angular_damping: 0.98,
        time_step,
    };
    if !constraint.is_finite() {
        return Err(ContactSolveFailure::NonFinite);
    }
    Ok(constraint)
}

fn normalized_pulley_segment(mut segment: Vec2) -> Vec2 {
    let length = segment.length();
    if length > 10.0 * LINEAR_SLOP {
        segment *= 1.0 / length;
        segment
    } else {
        Vec2::ZERO
    }
}

fn map_joint_error(_error: crate::JointMutationError) -> ContactSolveFailure {
    ContactSolveFailure::NonFinite
}

fn stage_legacy_unmigrated<C: Copy>(
    input: &JointConstraintInput,
    candidate: C,
    bodies: &[SolverBody],
    time_step: f32,
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<FamilyActivation<C>, ContactSolveFailure> {
    let indices = match input.lanes {
        JointSolverLanes::Ordinary(lanes) => lanes.solver_indices(bodies)?,
        JointSolverLanes::Gear(lanes) => {
            let [body_a, body_b, _body_c, _body_d] = lanes.solver_indices(bodies)?;
            [body_a, body_b]
        }
    };
    let body_a = bodies[indices[0]];
    let body_b = bodies[indices[1]];
    let (local_axis_a, maybe_max_linear_force, maybe_max_angular_torque) =
        constraint_parameters(input.definition);
    let ratio = if warm_starting { time_step_ratio } else { 0.0 };
    let common = CommonConstraint {
        body_a: indices[0],
        body_b: indices[1],
        local_axis_a,
        reference_delta: body_b.center - body_a.center,
        linear_impulse: ratio * input.legacy_linear_impulse,
        angular_impulse: ratio * input.legacy_angular_impulse,
        constrain_angular: constrains_angular_velocity(input.definition),
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
    Ok(FamilyActivation::LegacyUnmigrated(LegacyUnmigrated {
        candidate,
        constraint: common,
    }))
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
        JointVelocityConstraint::Gear(stage) => stage.solve_velocity(bodies, AxisMode::None),
        JointVelocityConstraint::Wheel(stage) => {
            stage.solve_velocity(bodies, AxisMode::Perpendicular)
        }
        JointVelocityConstraint::Weld(stage) => stage.solve_velocity(bodies, AxisMode::None),
        JointVelocityConstraint::Friction(stage) => stage.solve_velocity(bodies, AxisMode::None),
        JointVelocityConstraint::Rope(stage) => stage.solve_velocity(bodies, AxisMode::Separation),
        JointVelocityConstraint::Motor(stage) => stage.solve_velocity(bodies, AxisMode::None),
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
        JointVelocityConstraint::Gear(stage) => stage.solve_position(bodies, AxisMode::None),
        JointVelocityConstraint::Wheel(stage) => {
            stage.solve_position(bodies, AxisMode::Perpendicular)
        }
        JointVelocityConstraint::Weld(stage) => stage.solve_position(bodies, AxisMode::None),
        JointVelocityConstraint::Friction(stage) => stage.solve_position(bodies, AxisMode::None),
        JointVelocityConstraint::Rope(stage) => stage.solve_position(bodies, AxisMode::Separation),
        JointVelocityConstraint::Motor(stage) => stage.solve_position(bodies, AxisMode::None),
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

impl RevoluteConstraint {
    fn is_finite(self) -> bool {
        self.r_a.is_valid()
            && self.r_b.is_valid()
            && self.mass.first_column().is_valid()
            && self.mass.second_column().is_valid()
            && self.mass.third_column().is_valid()
            && self.motor_mass.is_finite()
            && self.time_step.is_finite()
    }

    fn warm_start(self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let (impulse, motor_impulse) = self.candidate.runtime.solver_impulses();
        let linear = Vec2::new(impulse.x, impulse.y);
        body_a.linear_velocity -= body_a.inverse_mass * linear;
        body_a.angular_velocity -=
            body_a.inverse_inertia * (self.r_a.cross(linear) + motor_impulse + impulse.z);
        body_b.linear_velocity += body_b.inverse_mass * linear;
        body_b.angular_velocity +=
            body_b.inverse_inertia * (self.r_b.cross(linear) + motor_impulse + impulse.z);
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)
    }

    fn solve_velocity(&mut self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        if self.candidate.definition.is_motor_enabled()
            && self.candidate.runtime.solver_limit_state() != crate::JointLimitState::Equal
            && !self.fixed_rotation
        {
            let relative_speed = body_b.angular_velocity - body_a.angular_velocity;
            let impulse = self
                .candidate
                .runtime
                .solve_motor(
                    self.candidate.definition,
                    self.time_step,
                    relative_speed,
                    self.motor_mass,
                )
                .map_err(map_joint_error)?;
            body_a.angular_velocity -= body_a.inverse_inertia * impulse;
            body_b.angular_velocity += body_b.inverse_inertia * impulse;
        }
        let relative_linear = body_b.linear_velocity
            + Vec2::scalar_cross(body_b.angular_velocity, self.r_b)
            - body_a.linear_velocity
            - Vec2::scalar_cross(body_a.angular_velocity, self.r_a);
        let relative_angular = body_b.angular_velocity - body_a.angular_velocity;
        let impulse = self
            .candidate
            .runtime
            .solve_constraint_velocity(
                self.mass,
                relative_linear,
                relative_angular,
                self.candidate.definition.is_limit_enabled(),
                self.fixed_rotation,
            )
            .map_err(map_joint_error)?;
        let linear = Vec2::new(impulse.x, impulse.y);
        body_a.linear_velocity -= body_a.inverse_mass * linear;
        body_a.angular_velocity -= body_a.inverse_inertia * (self.r_a.cross(linear) + impulse.z);
        body_b.linear_velocity += body_b.inverse_mass * linear;
        body_b.angular_velocity += body_b.inverse_inertia * (self.r_b.cross(linear) + impulse.z);
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)
    }

    fn solve_position(&mut self, bodies: &mut [SolverBody]) -> Result<bool, ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let mut angular_error = 0.0;
        let fixed_rotation = body_a.inverse_inertia + body_b.inverse_inertia == 0.0;
        let limit_state = self.candidate.runtime.solver_limit_state();
        if self.candidate.definition.is_limit_enabled()
            && limit_state != crate::JointLimitState::Inactive
            && !fixed_rotation
        {
            let angle = body_b.angle - body_a.angle - self.candidate.definition.reference_angle();
            let limit_impulse = match limit_state {
                crate::JointLimitState::Equal => {
                    let correction = (angle - self.candidate.definition.lower_angle())
                        .clamp(-MAX_ANGULAR_CORRECTION, MAX_ANGULAR_CORRECTION);
                    angular_error = correction.abs();
                    -self.motor_mass * correction
                }
                crate::JointLimitState::AtLower => {
                    let mut correction = angle - self.candidate.definition.lower_angle();
                    angular_error = -correction;
                    correction = (correction + ANGULAR_SLOP).clamp(-MAX_ANGULAR_CORRECTION, 0.0);
                    -self.motor_mass * correction
                }
                crate::JointLimitState::AtUpper => {
                    let mut correction = angle - self.candidate.definition.upper_angle();
                    angular_error = correction;
                    correction = (correction - ANGULAR_SLOP).clamp(0.0, MAX_ANGULAR_CORRECTION);
                    -self.motor_mass * correction
                }
                crate::JointLimitState::Inactive => 0.0,
            };
            body_a.angle -= body_a.inverse_inertia * limit_impulse;
            body_b.angle += body_b.inverse_inertia * limit_impulse;
        }

        let q_a = Rotation::from_angle(body_a.angle);
        let q_b = Rotation::from_angle(body_b.angle);
        let r_a = q_a.apply(self.candidate.definition.local_anchor_a() - body_a.local_center);
        let r_b = q_b.apply(self.candidate.definition.local_anchor_b() - body_b.local_center);
        let error = body_b.center + r_b - body_a.center - r_a;
        let position_error = error.length();
        let k = Mat22::from_columns(
            Vec2::new(
                body_a.inverse_mass
                    + body_b.inverse_mass
                    + body_a.inverse_inertia * r_a.y * r_a.y
                    + body_b.inverse_inertia * r_b.y * r_b.y,
                -body_a.inverse_inertia * r_a.x * r_a.y - body_b.inverse_inertia * r_b.x * r_b.y,
            ),
            Vec2::new(
                -body_a.inverse_inertia * r_a.x * r_a.y - body_b.inverse_inertia * r_b.x * r_b.y,
                body_a.inverse_mass
                    + body_b.inverse_mass
                    + body_a.inverse_inertia * r_a.x * r_a.x
                    + body_b.inverse_inertia * r_b.x * r_b.x,
            ),
        );
        let impulse = -k.solve(error);
        body_a.center -= body_a.inverse_mass * impulse;
        body_a.angle -= body_a.inverse_inertia * r_a.cross(impulse);
        body_b.center += body_b.inverse_mass * impulse;
        body_b.angle += body_b.inverse_inertia * r_b.cross(impulse);
        body_a.synchronize_transform();
        body_b.synchronize_transform();
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)?;
        Ok(position_error <= LINEAR_SLOP && angular_error <= ANGULAR_SLOP)
    }

    fn finalize(self) -> JointImpulseSolution {
        JointImpulseSolution {
            joint_id: self.candidate.joint_id,
            linear_impulse: Vec2::ZERO,
            angular_impulse: 0.0,
            maybe_runtime: Some(JointRuntime::Revolute(self.candidate.runtime)),
        }
    }
}

impl PrismaticConstraint {
    fn is_finite(self) -> bool {
        self.r_a.is_valid()
            && self.r_b.is_valid()
            && self.axis.is_valid()
            && self.perpendicular.is_valid()
            && [
                self.a1,
                self.a2,
                self.s1,
                self.s2,
                self.motor_mass,
                self.time_step,
            ]
            .into_iter()
            .all(f32::is_finite)
            && self.mass.first_column().is_valid()
            && self.mass.second_column().is_valid()
            && self.mass.third_column().is_valid()
    }

    fn warm_start(self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let (impulse, motor_impulse) = self.candidate.runtime.solver_impulses();
        let axial_impulse = motor_impulse + impulse.z;
        let linear = impulse.x * self.perpendicular + axial_impulse * self.axis;
        let angular_a = impulse.x * self.s1 + impulse.y + axial_impulse * self.a1;
        let angular_b = impulse.x * self.s2 + impulse.y + axial_impulse * self.a2;
        body_a.linear_velocity -= body_a.inverse_mass * linear;
        body_a.angular_velocity -= body_a.inverse_inertia * angular_a;
        body_b.linear_velocity += body_b.inverse_mass * linear;
        body_b.angular_velocity += body_b.inverse_inertia * angular_b;
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)
    }

    fn solve_velocity(&mut self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        if self.candidate.definition.is_motor_enabled()
            && self.candidate.runtime.solver_limit_state() != crate::JointLimitState::Equal
        {
            let relative_speed = self
                .axis
                .dot(body_b.linear_velocity - body_a.linear_velocity)
                + self.a2 * body_b.angular_velocity
                - self.a1 * body_a.angular_velocity;
            let impulse = self
                .candidate
                .runtime
                .solve_motor(
                    self.candidate.definition,
                    self.time_step,
                    relative_speed,
                    self.motor_mass,
                )
                .map_err(map_joint_error)?;
            let linear = impulse * self.axis;
            body_a.linear_velocity -= body_a.inverse_mass * linear;
            body_a.angular_velocity -= body_a.inverse_inertia * impulse * self.a1;
            body_b.linear_velocity += body_b.inverse_mass * linear;
            body_b.angular_velocity += body_b.inverse_inertia * impulse * self.a2;
        }
        let perpendicular_error = self
            .perpendicular
            .dot(body_b.linear_velocity - body_a.linear_velocity)
            + self.s2 * body_b.angular_velocity
            - self.s1 * body_a.angular_velocity;
        let angular_error = body_b.angular_velocity - body_a.angular_velocity;
        let axial_error = self
            .axis
            .dot(body_b.linear_velocity - body_a.linear_velocity)
            + self.a2 * body_b.angular_velocity
            - self.a1 * body_a.angular_velocity;
        let impulse = self
            .candidate
            .runtime
            .solve_constraint_velocity(
                self.mass,
                perpendicular_error,
                angular_error,
                axial_error,
                self.candidate.definition.is_limit_enabled(),
            )
            .map_err(map_joint_error)?;
        let linear = impulse.x * self.perpendicular + impulse.z * self.axis;
        let angular_a = impulse.x * self.s1 + impulse.y + impulse.z * self.a1;
        let angular_b = impulse.x * self.s2 + impulse.y + impulse.z * self.a2;
        body_a.linear_velocity -= body_a.inverse_mass * linear;
        body_a.angular_velocity -= body_a.inverse_inertia * angular_a;
        body_b.linear_velocity += body_b.inverse_mass * linear;
        body_b.angular_velocity += body_b.inverse_inertia * angular_b;
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)
    }

    fn solve_position(&mut self, bodies: &mut [SolverBody]) -> Result<bool, ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let q_a = Rotation::from_angle(body_a.angle);
        let q_b = Rotation::from_angle(body_b.angle);
        let r_a = q_a.apply(self.candidate.definition.local_anchor_a() - body_a.local_center);
        let r_b = q_b.apply(self.candidate.definition.local_anchor_b() - body_b.local_center);
        let d = body_b.center + r_b - body_a.center - r_a;
        let axis = q_a.apply(self.candidate.definition.local_axis_a());
        let a1 = (d + r_a).cross(axis);
        let a2 = r_b.cross(axis);
        let perpendicular = q_a.apply(Vec2::scalar_cross(
            1.0,
            self.candidate.definition.local_axis_a(),
        ));
        let s1 = (d + r_a).cross(perpendicular);
        let s2 = r_b.cross(perpendicular);
        let c1 = Vec2::new(
            perpendicular.dot(d),
            body_b.angle - body_a.angle - self.candidate.definition.reference_angle(),
        );
        let mut linear_error = c1.x.abs();
        let angular_error = c1.y.abs();
        let mut active = false;
        let mut c2 = 0.0;
        if self.candidate.definition.is_limit_enabled() {
            let translation = axis.dot(d);
            if (self.candidate.definition.upper_translation()
                - self.candidate.definition.lower_translation())
            .abs()
                < 2.0 * LINEAR_SLOP
            {
                c2 = translation.clamp(-MAX_LINEAR_CORRECTION, MAX_LINEAR_CORRECTION);
                linear_error = linear_error.max(translation.abs());
                active = true;
            } else if translation <= self.candidate.definition.lower_translation() {
                c2 = (translation - self.candidate.definition.lower_translation() + LINEAR_SLOP)
                    .clamp(-MAX_LINEAR_CORRECTION, 0.0);
                linear_error =
                    linear_error.max(self.candidate.definition.lower_translation() - translation);
                active = true;
            } else if translation >= self.candidate.definition.upper_translation() {
                c2 = (translation - self.candidate.definition.upper_translation() - LINEAR_SLOP)
                    .clamp(0.0, MAX_LINEAR_CORRECTION);
                linear_error =
                    linear_error.max(translation - self.candidate.definition.upper_translation());
                active = true;
            }
        }
        let m_a = body_a.inverse_mass;
        let m_b = body_b.inverse_mass;
        let i_a = body_a.inverse_inertia;
        let i_b = body_b.inverse_inertia;
        let k11 = m_a + m_b + i_a * s1 * s1 + i_b * s2 * s2;
        let k12 = i_a * s1 + i_b * s2;
        let mut k22 = i_a + i_b;
        if k22 == 0.0 {
            k22 = 1.0;
        }
        let impulse = if active {
            let k13 = i_a * s1 * a1 + i_b * s2 * a2;
            let k23 = i_a * a1 + i_b * a2;
            let k33 = m_a + m_b + i_a * a1 * a1 + i_b * a2 * a2;
            Mat33::from_columns(
                Vec3::new(k11, k12, k13),
                Vec3::new(k12, k22, k23),
                Vec3::new(k13, k23, k33),
            )
            .solve33(-Vec3::new(c1.x, c1.y, c2))
        } else {
            let impulse2 = Mat22::from_columns(Vec2::new(k11, k12), Vec2::new(k12, k22)).solve(-c1);
            Vec3::new(impulse2.x, impulse2.y, 0.0)
        };
        let linear = impulse.x * perpendicular + impulse.z * axis;
        let angular_a = impulse.x * s1 + impulse.y + impulse.z * a1;
        let angular_b = impulse.x * s2 + impulse.y + impulse.z * a2;
        body_a.center -= m_a * linear;
        body_a.angle -= i_a * angular_a;
        body_b.center += m_b * linear;
        body_b.angle += i_b * angular_b;
        body_a.synchronize_transform();
        body_b.synchronize_transform();
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)?;
        Ok(linear_error <= LINEAR_SLOP && angular_error <= ANGULAR_SLOP)
    }

    fn finalize(self) -> JointImpulseSolution {
        JointImpulseSolution {
            joint_id: self.candidate.joint_id,
            linear_impulse: Vec2::ZERO,
            angular_impulse: 0.0,
            maybe_runtime: Some(JointRuntime::Prismatic(self.candidate.runtime)),
        }
    }
}

impl DistanceConstraint {
    fn is_finite(self) -> bool {
        self.r_a.is_valid()
            && self.r_b.is_valid()
            && self.candidate.runtime.solver_direction().is_valid()
            && self.candidate.runtime.solver_impulse().is_finite()
    }

    fn warm_start(self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let impulse = self.candidate.runtime.solver_impulse();
        let direction = self.candidate.runtime.solver_direction();
        let linear = impulse * direction;
        body_a.linear_velocity -= body_a.inverse_mass * linear;
        body_a.angular_velocity -= body_a.inverse_inertia * self.r_a.cross(linear);
        body_b.linear_velocity += body_b.inverse_mass * linear;
        body_b.angular_velocity += body_b.inverse_inertia * self.r_b.cross(linear);
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)
    }

    fn solve_velocity(&mut self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let point_velocity_a =
            body_a.linear_velocity + Vec2::scalar_cross(body_a.angular_velocity, self.r_a);
        let point_velocity_b =
            body_b.linear_velocity + Vec2::scalar_cross(body_b.angular_velocity, self.r_b);
        let applied = self
            .candidate
            .runtime
            .solve_velocity(point_velocity_b - point_velocity_a)
            .map_err(map_joint_error)?;
        let linear = applied * self.candidate.runtime.solver_direction();
        body_a.linear_velocity -= body_a.inverse_mass * linear;
        body_a.angular_velocity -= body_a.inverse_inertia * self.r_a.cross(linear);
        body_b.linear_velocity += body_b.inverse_mass * linear;
        body_b.angular_velocity += body_b.inverse_inertia * self.r_b.cross(linear);
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)
    }

    fn solve_position(&mut self, bodies: &mut [SolverBody]) -> Result<bool, ContactSolveFailure> {
        if self.candidate.definition.frequency() > 0.0 {
            return Ok(true);
        }
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let q_a = Rotation::from_angle(body_a.angle);
        let q_b = Rotation::from_angle(body_b.angle);
        let r_a = q_a.apply(self.candidate.definition.local_anchor_a() - body_a.local_center);
        let r_b = q_b.apply(self.candidate.definition.local_anchor_b() - body_b.local_center);
        let mut direction = body_b.center + r_b - body_a.center - r_a;
        let length = direction.normalize();
        let correction = (length - self.candidate.definition.length())
            .clamp(-MAX_LINEAR_CORRECTION, MAX_LINEAR_CORRECTION);
        let Some(applied) = self
            .candidate
            .runtime
            .position_impulse(self.candidate.definition, length)
            .map_err(map_joint_error)?
        else {
            return Ok(true);
        };
        let linear = applied * direction;
        body_a.center -= body_a.inverse_mass * linear;
        body_a.angle -= body_a.inverse_inertia * r_a.cross(linear);
        body_b.center += body_b.inverse_mass * linear;
        body_b.angle += body_b.inverse_inertia * r_b.cross(linear);
        body_a.synchronize_transform();
        body_b.synchronize_transform();
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)?;
        Ok(correction.abs() < LINEAR_SLOP)
    }

    fn finalize(self) -> JointImpulseSolution {
        JointImpulseSolution {
            joint_id: self.candidate.joint_id,
            linear_impulse: Vec2::ZERO,
            angular_impulse: 0.0,
            maybe_runtime: Some(JointRuntime::Distance(self.candidate.runtime)),
        }
    }
}

impl PulleyConstraint {
    fn is_finite(self) -> bool {
        let directions = self.candidate.runtime.solver_directions();
        self.r_a.is_valid()
            && self.r_b.is_valid()
            && directions[0].is_valid()
            && directions[1].is_valid()
            && self.candidate.runtime.solver_impulse().is_finite()
    }

    fn warm_start(self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let impulse = self.candidate.runtime.solver_impulse();
        let [direction_a, direction_b] = self.candidate.runtime.solver_directions();
        let linear_a = -impulse * direction_a;
        let linear_b = (-self.candidate.definition.ratio() * impulse) * direction_b;
        body_a.linear_velocity += body_a.inverse_mass * linear_a;
        body_a.angular_velocity += body_a.inverse_inertia * self.r_a.cross(linear_a);
        body_b.linear_velocity += body_b.inverse_mass * linear_b;
        body_b.angular_velocity += body_b.inverse_inertia * self.r_b.cross(linear_b);
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)
    }

    fn solve_velocity(&mut self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let point_velocity_a =
            body_a.linear_velocity + Vec2::scalar_cross(body_a.angular_velocity, self.r_a);
        let point_velocity_b =
            body_b.linear_velocity + Vec2::scalar_cross(body_b.angular_velocity, self.r_b);
        let applied = self
            .candidate
            .runtime
            .solve_velocity(
                self.candidate.definition,
                point_velocity_a,
                point_velocity_b,
            )
            .map_err(map_joint_error)?;
        let [direction_a, direction_b] = self.candidate.runtime.solver_directions();
        let linear_a = -applied * direction_a;
        let linear_b = (-self.candidate.definition.ratio() * applied) * direction_b;
        body_a.linear_velocity += body_a.inverse_mass * linear_a;
        body_a.angular_velocity += body_a.inverse_inertia * self.r_a.cross(linear_a);
        body_b.linear_velocity += body_b.inverse_mass * linear_b;
        body_b.angular_velocity += body_b.inverse_inertia * self.r_b.cross(linear_b);
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)
    }

    fn solve_position(&mut self, bodies: &mut [SolverBody]) -> Result<bool, ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let q_a = Rotation::from_angle(body_a.angle);
        let q_b = Rotation::from_angle(body_b.angle);
        let r_a = q_a.apply(self.candidate.definition.local_anchor_a() - body_a.local_center);
        let r_b = q_b.apply(self.candidate.definition.local_anchor_b() - body_b.local_center);
        let segment_a = body_a.center + r_a - self.candidate.definition.ground_anchor_a();
        let segment_b = body_b.center + r_b - self.candidate.definition.ground_anchor_b();
        let length_a = segment_a.length();
        let length_b = segment_b.length();
        let direction_a = normalized_pulley_segment(segment_a);
        let direction_b = normalized_pulley_segment(segment_b);
        let anchor_lever_a = r_a.cross(direction_a);
        let anchor_lever_b = r_b.cross(direction_b);
        let mass_a = body_a.inverse_mass + body_a.inverse_inertia * anchor_lever_a * anchor_lever_a;
        let mass_b = body_b.inverse_mass + body_b.inverse_inertia * anchor_lever_b * anchor_lever_b;
        let ratio = self.candidate.definition.ratio();
        let inverse_mass = mass_a + ratio * ratio * mass_b;
        let mass = if inverse_mass > 0.0 {
            1.0 / inverse_mass
        } else {
            0.0
        };
        let constraint = self.candidate.definition.constant() - length_a - ratio * length_b;
        let applied = -mass * constraint;
        let linear_a = -applied * direction_a;
        let linear_b = (-ratio * applied) * direction_b;
        body_a.center += body_a.inverse_mass * linear_a;
        body_a.angle += body_a.inverse_inertia * r_a.cross(linear_a);
        body_b.center += body_b.inverse_mass * linear_b;
        body_b.angle += body_b.inverse_inertia * r_b.cross(linear_b);
        body_a.synchronize_transform();
        body_b.synchronize_transform();
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)?;
        Ok(constraint.abs() < LINEAR_SLOP)
    }

    fn finalize(self) -> JointImpulseSolution {
        JointImpulseSolution {
            joint_id: self.candidate.joint_id,
            linear_impulse: Vec2::ZERO,
            angular_impulse: 0.0,
            maybe_runtime: Some(JointRuntime::Pulley(self.candidate.runtime)),
        }
    }
}

impl MouseConstraint {
    fn is_finite(self) -> bool {
        self.r_b.is_valid()
            && self.angular_damping.is_finite()
            && self.time_step.is_finite()
            && self.candidate.runtime.solver_impulse().is_valid()
    }

    fn warm_start(self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        let mut body_b = solver_body(self.body_b, bodies)?;
        body_b.angular_velocity *= self.angular_damping;
        let impulse = self.candidate.runtime.solver_impulse();
        body_b.linear_velocity += body_b.inverse_mass * impulse;
        body_b.angular_velocity += body_b.inverse_inertia * self.r_b.cross(impulse);
        store_solver_body(self.body_b, bodies, body_b)
    }

    fn solve_velocity(&mut self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        let mut body_b = solver_body(self.body_b, bodies)?;
        let point_velocity =
            body_b.linear_velocity + Vec2::scalar_cross(body_b.angular_velocity, self.r_b);
        let applied = self
            .candidate
            .runtime
            .solve_velocity(self.candidate.definition, self.time_step, point_velocity)
            .map_err(map_joint_error)?;
        body_b.linear_velocity += body_b.inverse_mass * applied;
        body_b.angular_velocity += body_b.inverse_inertia * self.r_b.cross(applied);
        store_solver_body(self.body_b, bodies, body_b)
    }

    fn finalize(self) -> JointImpulseSolution {
        JointImpulseSolution {
            joint_id: self.candidate.joint_id,
            linear_impulse: Vec2::ZERO,
            angular_impulse: 0.0,
            maybe_runtime: Some(JointRuntime::Mouse(self.candidate.runtime)),
        }
    }
}

fn solver_body(index: usize, bodies: &[SolverBody]) -> Result<SolverBody, ContactSolveFailure> {
    bodies
        .get(index)
        .copied()
        .ok_or(ContactSolveFailure::UnsupportedTopology)
}

fn store_solver_body(
    index: usize,
    bodies: &mut [SolverBody],
    body: SolverBody,
) -> Result<(), ContactSolveFailure> {
    if !body.is_finite() {
        return Err(ContactSolveFailure::NonFinite);
    }
    let Some(slot) = bodies.get_mut(index) else {
        return Err(ContactSolveFailure::UnsupportedTopology);
    };
    *slot = body;
    Ok(())
}

fn solver_body_pair(
    body_a: usize,
    body_b: usize,
    bodies: &[SolverBody],
) -> Result<(SolverBody, SolverBody), ContactSolveFailure> {
    if body_a == body_b {
        return Err(ContactSolveFailure::UnsupportedTopology);
    }
    let first = bodies
        .get(body_a)
        .copied()
        .ok_or(ContactSolveFailure::UnsupportedTopology)?;
    let second = bodies
        .get(body_b)
        .copied()
        .ok_or(ContactSolveFailure::UnsupportedTopology)?;
    Ok((first, second))
}

fn store_solver_body_pair(
    first_index: usize,
    second_index: usize,
    bodies: &mut [SolverBody],
    body_a: SolverBody,
    body_b: SolverBody,
) -> Result<(), ContactSolveFailure> {
    if !body_a.is_finite() || !body_b.is_finite() {
        return Err(ContactSolveFailure::NonFinite);
    }
    if first_index == second_index {
        return Err(ContactSolveFailure::UnsupportedTopology);
    }
    let Some(first) = bodies.get_mut(first_index) else {
        return Err(ContactSolveFailure::UnsupportedTopology);
    };
    *first = body_a;
    let Some(second) = bodies.get_mut(second_index) else {
        return Err(ContactSolveFailure::UnsupportedTopology);
    };
    *second = body_b;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum AxisMode {
    None,
    Perpendicular,
    Separation,
}

impl<C: CompleteRuntimeCandidate> FamilyActivation<C> {
    fn warm_start(self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        match self {
            Self::Activated(_candidate) => Ok(()),
            Self::LegacyUnmigrated(stage) => stage.legacy_unmigrated_warm_start(bodies),
        }
    }

    fn solve_velocity(
        &mut self,
        bodies: &mut [SolverBody],
        axis_mode: AxisMode,
    ) -> Result<(), ContactSolveFailure> {
        match self {
            Self::Activated(_candidate) => Ok(()),
            Self::LegacyUnmigrated(stage) => {
                stage.legacy_unmigrated_solve_velocity(bodies, axis_mode)
            }
        }
    }

    fn solve_position(
        &mut self,
        bodies: &mut [SolverBody],
        axis_mode: AxisMode,
    ) -> Result<bool, ContactSolveFailure> {
        match self {
            Self::Activated(_candidate) => Ok(true),
            Self::LegacyUnmigrated(stage) => {
                stage.legacy_unmigrated_solve_position(bodies, axis_mode)
            }
        }
    }
}

impl<C: CompleteRuntimeCandidate> FamilyActivation<C> {
    fn finalize(&self) -> JointImpulseSolution {
        match *self {
            Self::Activated(candidate) => JointImpulseSolution {
                joint_id: candidate.joint_id(),
                linear_impulse: Vec2::ZERO,
                angular_impulse: 0.0,
                maybe_runtime: Some(candidate.complete_runtime()),
            },
            Self::LegacyUnmigrated(stage) => stage.legacy_unmigrated_solution(),
        }
    }
}

impl<C: CompleteRuntimeCandidate> LegacyUnmigrated<C> {
    fn legacy_unmigrated_warm_start(
        self,
        bodies: &mut [SolverBody],
    ) -> Result<(), ContactSolveFailure> {
        apply_velocity_impulse(
            bodies,
            self.constraint,
            self.constraint.linear_impulse,
            self.constraint.angular_impulse,
        )
    }

    fn legacy_unmigrated_solve_velocity(
        &mut self,
        bodies: &mut [SolverBody],
        axis_mode: AxisMode,
    ) -> Result<(), ContactSolveFailure> {
        let common = self.constraint;
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
        if !matches!(axis_mode, AxisMode::None) {
            let axis = Rotation::from_angle(body_a.angle).apply(common.local_axis_a);
            let direction = legacy_axis_direction(axis_mode, axis, body_a, body_b);
            linear_impulse = linear_impulse.dot(direction) * direction;
        }
        let angular_impulse = if common.constrain_angular && inverse_inertia > 0.0 {
            -relative_angular / inverse_inertia
        } else {
            0.0
        };
        let previous_linear = common.linear_impulse;
        let previous_angular = common.angular_impulse;
        let common_mut = &mut self.constraint;
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

    fn legacy_unmigrated_solve_position(
        &mut self,
        bodies: &mut [SolverBody],
        axis_mode: AxisMode,
    ) -> Result<bool, ContactSolveFailure> {
        let common = self.constraint;
        let (body_a, body_b) = constraint_bodies(common, bodies)?;
        let inverse_mass = body_a.inverse_mass + body_b.inverse_mass;
        if inverse_mass == 0.0 {
            return Ok(true);
        }
        let delta = body_b.center - body_a.center;
        let mut error = delta - common.reference_delta;
        if !matches!(axis_mode, AxisMode::None) {
            let axis = Rotation::from_angle(body_a.angle).apply(common.local_axis_a);
            let direction = legacy_axis_direction(axis_mode, axis, body_a, body_b);
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

    fn legacy_unmigrated_solution(self) -> JointImpulseSolution {
        JointImpulseSolution {
            joint_id: self.candidate.joint_id(),
            linear_impulse: self.constraint.linear_impulse,
            angular_impulse: self.constraint.angular_impulse,
            maybe_runtime: None,
        }
    }
}

fn legacy_axis_direction(
    axis_mode: AxisMode,
    axis: Vec2,
    body_a: SolverBody,
    body_b: SolverBody,
) -> Vec2 {
    match axis_mode {
        AxisMode::None => Vec2::ZERO,
        AxisMode::Perpendicular => Vec2::scalar_cross(1.0, axis),
        AxisMode::Separation => {
            let mut separation = body_b.center - body_a.center;
            separation.normalize();
            separation
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::joint::JointRuntime;
    use crate::{BodyDef, JointDef, RevoluteJointDef, World};

    #[test]
    fn solver_body_lane_retains_semantic_identity_when_resolved() {
        // Arrange
        let mut world = World::new().expect("test world should be available");
        let body = world
            .create_body(&BodyDef::default())
            .expect("body should fit");

        // Act
        let lane = SolverBodyLane::resolved(body, 3);

        // Assert
        assert_eq!(lane.body_id(), body);
        assert_eq!(lane.maybe_solver_index(), Some(3));
    }

    #[test]
    fn staged_revolute_uses_the_live_typed_constraint() {
        // Arrange
        let mut world = World::new().expect("test world should be available");
        let body_a = world
            .create_body(&BodyDef::default())
            .expect("body A should fit");
        let body_b = world
            .create_body(&BodyDef::default())
            .expect("body B should fit");
        let definition = RevoluteJointDef::new(body_a, body_b).expect("valid definition");
        let joint_id = world
            .create_joint(JointDef::from(definition))
            .expect("joint should fit");
        let record = world.joints.get(joint_id).expect("joint remains live");
        let JointRuntime::Revolute(runtime) = record.runtime else {
            panic!("revolute runtime should match its definition");
        };
        let input = JointConstraintInput::ordinary(
            joint_id,
            OrdinarySolverLanes::resolved(body_a, 0, body_b, 1),
            record.definition,
            record.runtime,
            record.solver_linear_impulse,
            record.solver_angular_impulse,
        );
        let bodies = [test_solver_body(), test_solver_body()];

        // Act
        let constraints = build_constraints(&[input], &bodies, 1.0 / 60.0, 1.0, true)
            .expect("live revolute staging should remain available");

        // Assert
        assert_eq!(constraints.len(), 1);
        let JointVelocityConstraint::Revolute(stage) = constraints[0] else {
            panic!("revolute must use the typed live constraint");
        };
        assert_eq!(stage.candidate.joint_id, joint_id);
        assert_eq!(stage.candidate.definition, definition);
        assert_eq!(
            stage.candidate.runtime.reaction_force(1.0),
            runtime.reaction_force(1.0)
        );
    }

    #[test]
    fn unresolved_lane_is_rejected_before_legacy_dispatch() {
        // Arrange
        let mut world = World::new().expect("test world should be available");
        let body_a = world
            .create_body(&BodyDef::default())
            .expect("body A should fit");
        let body_b = world
            .create_body(&BodyDef::default())
            .expect("body B should fit");
        let definition = RevoluteJointDef::new(body_a, body_b).expect("valid definition");
        let joint_id = world
            .create_joint(definition.into())
            .expect("joint should fit");
        let record = world.joints.get(joint_id).expect("joint remains live");
        let input = JointConstraintInput::ordinary(
            joint_id,
            OrdinarySolverLanes::new(
                SolverBodyLane::resolved(body_a, 0),
                SolverBodyLane::unresolved(body_b),
            ),
            record.definition,
            record.runtime,
            Vec2::ZERO,
            0.0,
        );

        // Act
        let result = build_constraints(
            &[input],
            &[test_solver_body(), test_solver_body()],
            1.0 / 60.0,
            1.0,
            true,
        );

        // Assert
        assert!(matches!(
            result,
            Err(ContactSolveFailure::UnsupportedTopology)
        ));
    }

    #[test]
    fn activated_finalize_stages_complete_runtime_without_legacy_cache() {
        // Arrange
        let mut world = World::new().expect("test world should be available");
        let body_a = world
            .create_body(&BodyDef::default())
            .expect("body A should fit");
        let body_b = world
            .create_body(&BodyDef::default())
            .expect("body B should fit");
        let definition = RevoluteJointDef::new(body_a, body_b).expect("valid definition");
        let joint_id = world
            .create_joint(definition.into())
            .expect("joint should fit");
        let record = world.joints.get(joint_id).expect("joint remains live");
        let JointRuntime::Revolute(runtime) = record.runtime else {
            panic!("revolute runtime should match its definition");
        };
        let activated = FamilyActivation::Activated(FamilyCandidate {
            joint_id,
            lanes: OrdinarySolverLanes::resolved(body_a, 0, body_b, 1),
            definition,
            runtime,
        });

        // Act
        let solution = activated.finalize();

        // Assert
        assert_eq!(solution.joint_id, joint_id);
        assert_eq!(solution.linear_impulse, Vec2::ZERO);
        assert_eq!(solution.angular_impulse.to_bits(), 0.0_f32.to_bits());
        assert!(matches!(
            solution.maybe_runtime,
            Some(JointRuntime::Revolute(candidate_runtime))
                if candidate_runtime.reaction_force(1.0) == runtime.reaction_force(1.0)
        ));
    }

    #[test]
    fn staged_constraints_preserve_source_input_order() {
        // Arrange
        let mut world = World::new().expect("test world should be available");
        let body_a = world
            .create_body(&BodyDef::default())
            .expect("body A should fit");
        let body_b = world
            .create_body(&BodyDef::default())
            .expect("body B should fit");
        let first = world
            .create_joint(
                RevoluteJointDef::new(body_a, body_b)
                    .expect("valid first joint")
                    .into(),
            )
            .expect("first joint should fit");
        let second = world
            .create_joint(
                RevoluteJointDef::new(body_a, body_b)
                    .expect("valid second joint")
                    .into(),
            )
            .expect("second joint should fit");
        let inputs = [second, first].map(|joint_id| {
            let record = world.joints.get(joint_id).expect("joint remains live");
            JointConstraintInput::ordinary(
                joint_id,
                OrdinarySolverLanes::resolved(body_a, 0, body_b, 1),
                record.definition,
                record.runtime,
                Vec2::ZERO,
                0.0,
            )
        });
        let bodies = [test_solver_body(), test_solver_body()];

        // Act
        let constraints = build_constraints(&inputs, &bodies, 1.0 / 60.0, 1.0, true)
            .expect("source inputs should stage");

        // Assert
        let joint_ids = constraints
            .iter()
            .map(|constraint| {
                let JointVelocityConstraint::Revolute(stage) = constraint else {
                    panic!("both inputs should be live revolute candidates");
                };
                stage.candidate.joint_id
            })
            .collect::<Vec<_>>();
        assert_eq!(joint_ids, vec![second, first]);
    }

    #[test]
    fn gear_lanes_resolve_abcd_in_semantic_order_and_reject_absence() {
        // Arrange
        let mut world = World::new().expect("test world should be available");
        let [body_a, body_b, body_c, body_d] = std::array::from_fn(|_| {
            world
                .create_body(&BodyDef::default())
                .expect("body should fit")
        });
        let resolved = GearSolverLanes::new(
            SolverBodyLane::resolved(body_a, 3),
            SolverBodyLane::resolved(body_b, 1),
            SolverBodyLane::resolved(body_c, 0),
            SolverBodyLane::resolved(body_d, 2),
        );
        let missing = GearSolverLanes::new(
            SolverBodyLane::resolved(body_a, 3),
            SolverBodyLane::resolved(body_b, 1),
            SolverBodyLane::unresolved(body_c),
            SolverBodyLane::resolved(body_d, 2),
        );
        let bodies = [
            test_solver_body(),
            test_solver_body(),
            test_solver_body(),
            test_solver_body(),
        ];

        // Act
        let indices = resolved.solver_indices(&bodies);
        let missing_result = missing.solver_indices(&bodies);

        // Assert
        assert_eq!(indices, Ok([3, 1, 0, 2]));
        assert_eq!(
            missing_result,
            Err(ContactSolveFailure::UnsupportedTopology)
        );
    }

    fn test_solver_body() -> SolverBody {
        SolverBody {
            center: Vec2::ZERO,
            local_center: Vec2::ZERO,
            angle: 0.0,
            transform: crate::math::Transform::IDENTITY,
            linear_velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            inverse_mass: 1.0,
            inverse_inertia: 1.0,
        }
    }
}

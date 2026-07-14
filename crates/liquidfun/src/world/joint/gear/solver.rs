//! Pinned four-body Jacobian and impulse application for gear constraints.

use crate::JointMutationError;
use crate::math::{Rotation, Vec2};

use super::{GearBodyGeometry, GearSource};

#[derive(Debug, Clone, Copy)]
pub(crate) struct GearSolverBody {
    pub(crate) center: Vec2,
    pub(crate) angle: f32,
    pub(crate) linear_velocity: Vec2,
    pub(crate) angular_velocity: f32,
    pub(crate) local_center: Vec2,
    pub(crate) inverse_mass: f32,
    pub(crate) inverse_inertia: f32,
}

impl GearSolverBody {
    pub(super) fn geometry(self) -> GearBodyGeometry {
        let rotation = Rotation::from_angle(self.angle);
        GearBodyGeometry {
            position: self.center - rotation.apply(self.local_center),
            angle: self.angle,
        }
    }

    pub(super) fn is_valid(self) -> bool {
        self.center.is_valid()
            && self.angle.is_finite()
            && self.linear_velocity.is_valid()
            && self.angular_velocity.is_finite()
            && self.local_center.is_valid()
            && self.inverse_mass.is_finite()
            && self.inverse_mass >= 0.0
            && self.inverse_inertia.is_finite()
            && self.inverse_inertia >= 0.0
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct GearJacobian {
    pub(super) linear_ac: Vec2,
    pub(super) linear_bd: Vec2,
    pub(super) angular_a: f32,
    pub(super) angular_b: f32,
    pub(super) angular_c: f32,
    pub(super) angular_d: f32,
    pub(super) mass: f32,
}

impl GearJacobian {
    pub(super) const ZERO: Self = Self {
        linear_ac: Vec2::ZERO,
        linear_bd: Vec2::ZERO,
        angular_a: 0.0,
        angular_b: 0.0,
        angular_c: 0.0,
        angular_d: 0.0,
        mass: 0.0,
    };

    pub(super) fn is_valid(self) -> bool {
        self.linear_ac.is_valid()
            && self.linear_bd.is_valid()
            && self.angular_a.is_finite()
            && self.angular_b.is_finite()
            && self.angular_c.is_finite()
            && self.angular_d.is_finite()
            && self.mass.is_finite()
            && self.mass >= 0.0
    }
}

pub(super) fn build_jacobian(
    source_a: GearSource,
    source_b: GearSource,
    ratio: f32,
    bodies: &[GearSolverBody; 4],
) -> Result<GearJacobian, JointMutationError> {
    let [body_a, body_b, body_c, body_d] = *bodies;
    let mut jacobian = GearJacobian::ZERO;
    let mut inverse_effective_mass = 0.0;

    match source_a {
        GearSource::Revolute { .. } => {
            jacobian.angular_a = 1.0;
            jacobian.angular_c = 1.0;
            inverse_effective_mass += body_a.inverse_inertia + body_c.inverse_inertia;
        }
        GearSource::Prismatic {
            local_anchor_base,
            local_anchor_moving,
            local_axis_base,
        } => {
            let rotation_c = Rotation::from_angle(body_c.angle);
            let rotation_a = Rotation::from_angle(body_a.angle);
            let axis = rotation_c.apply(local_axis_base);
            let radius_c = rotation_c.apply(local_anchor_base - body_c.local_center);
            let radius_a = rotation_a.apply(local_anchor_moving - body_a.local_center);
            jacobian.linear_ac = axis;
            jacobian.angular_c = radius_c.cross(axis);
            jacobian.angular_a = radius_a.cross(axis);
            inverse_effective_mass += body_c.inverse_mass
                + body_a.inverse_mass
                + body_c.inverse_inertia * jacobian.angular_c * jacobian.angular_c
                + body_a.inverse_inertia * jacobian.angular_a * jacobian.angular_a;
        }
    }

    match source_b {
        GearSource::Revolute { .. } => {
            jacobian.angular_b = ratio;
            jacobian.angular_d = ratio;
            inverse_effective_mass +=
                ratio * ratio * (body_b.inverse_inertia + body_d.inverse_inertia);
        }
        GearSource::Prismatic {
            local_anchor_base,
            local_anchor_moving,
            local_axis_base,
        } => {
            let rotation_d = Rotation::from_angle(body_d.angle);
            let rotation_b = Rotation::from_angle(body_b.angle);
            let axis = rotation_d.apply(local_axis_base);
            let radius_d = rotation_d.apply(local_anchor_base - body_d.local_center);
            let radius_b = rotation_b.apply(local_anchor_moving - body_b.local_center);
            jacobian.linear_bd = ratio * axis;
            jacobian.angular_d = ratio * radius_d.cross(axis);
            jacobian.angular_b = ratio * radius_b.cross(axis);
            inverse_effective_mass += ratio * ratio * (body_d.inverse_mass + body_b.inverse_mass)
                + body_d.inverse_inertia * jacobian.angular_d * jacobian.angular_d
                + body_b.inverse_inertia * jacobian.angular_b * jacobian.angular_b;
        }
    }

    jacobian.mass = if inverse_effective_mass > 0.0 {
        1.0 / inverse_effective_mass
    } else {
        0.0
    };
    if !jacobian.is_valid() {
        return Err(JointMutationError::NonFiniteDerivedState);
    }
    Ok(jacobian)
}

pub(super) fn apply_impulse(
    bodies: &mut [GearSolverBody; 4],
    jacobian: GearJacobian,
    impulse: f32,
) {
    bodies[0].linear_velocity += bodies[0].inverse_mass * impulse * jacobian.linear_ac;
    bodies[0].angular_velocity += bodies[0].inverse_inertia * impulse * jacobian.angular_a;
    bodies[1].linear_velocity += bodies[1].inverse_mass * impulse * jacobian.linear_bd;
    bodies[1].angular_velocity += bodies[1].inverse_inertia * impulse * jacobian.angular_b;
    bodies[2].linear_velocity -= bodies[2].inverse_mass * impulse * jacobian.linear_ac;
    bodies[2].angular_velocity -= bodies[2].inverse_inertia * impulse * jacobian.angular_c;
    bodies[3].linear_velocity -= bodies[3].inverse_mass * impulse * jacobian.linear_bd;
    bodies[3].angular_velocity -= bodies[3].inverse_inertia * impulse * jacobian.angular_d;
}

pub(super) fn apply_position_impulse(
    bodies: &mut [GearSolverBody; 4],
    jacobian: GearJacobian,
    impulse: f32,
) {
    bodies[0].center += bodies[0].inverse_mass * impulse * jacobian.linear_ac;
    bodies[0].angle += bodies[0].inverse_inertia * impulse * jacobian.angular_a;
    bodies[1].center += bodies[1].inverse_mass * impulse * jacobian.linear_bd;
    bodies[1].angle += bodies[1].inverse_inertia * impulse * jacobian.angular_b;
    bodies[2].center -= bodies[2].inverse_mass * impulse * jacobian.linear_ac;
    bodies[2].angle -= bodies[2].inverse_inertia * impulse * jacobian.angular_c;
    bodies[3].center -= bodies[3].inverse_mass * impulse * jacobian.linear_bd;
    bodies[3].angle -= bodies[3].inverse_inertia * impulse * jacobian.angular_d;
}

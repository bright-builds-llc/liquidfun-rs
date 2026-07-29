use super::{
    ContactSolveFailure, DistanceConstraint, JointImpulseSolution, JointRuntime, LINEAR_SLOP,
    MAX_LINEAR_CORRECTION, MouseConstraint, PulleyConstraint, Rotation, SolverBody, Vec2,
    map_joint_error, normalized_pulley_segment, solver_body, solver_body_pair, store_solver_body,
    store_solver_body_pair,
};

impl DistanceConstraint {
    pub(super) fn is_finite(self) -> bool {
        self.r_a.is_valid()
            && self.r_b.is_valid()
            && self.candidate.runtime.solver_direction().is_valid()
            && self.candidate.runtime.solver_impulse().is_finite()
    }

    pub(super) fn warm_start(self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
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

    pub(super) fn solve_velocity(
        &mut self,
        bodies: &mut [SolverBody],
    ) -> Result<(), ContactSolveFailure> {
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

    pub(super) fn solve_position(
        &mut self,
        bodies: &mut [SolverBody],
    ) -> Result<bool, ContactSolveFailure> {
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

    pub(super) fn finalize(self) -> JointImpulseSolution {
        JointImpulseSolution {
            joint_id: self.candidate.joint_id,
            runtime: JointRuntime::Distance(self.candidate.runtime),
        }
    }
}

impl PulleyConstraint {
    pub(super) fn is_finite(self) -> bool {
        let directions = self.candidate.runtime.solver_directions();
        self.r_a.is_valid()
            && self.r_b.is_valid()
            && directions[0].is_valid()
            && directions[1].is_valid()
            && self.candidate.runtime.solver_impulse().is_finite()
    }

    pub(super) fn warm_start(self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
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

    pub(super) fn solve_velocity(
        &mut self,
        bodies: &mut [SolverBody],
    ) -> Result<(), ContactSolveFailure> {
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

    pub(super) fn solve_position(
        &mut self,
        bodies: &mut [SolverBody],
    ) -> Result<bool, ContactSolveFailure> {
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

    pub(super) fn finalize(self) -> JointImpulseSolution {
        JointImpulseSolution {
            joint_id: self.candidate.joint_id,
            runtime: JointRuntime::Pulley(self.candidate.runtime),
        }
    }
}

impl MouseConstraint {
    pub(super) fn is_finite(self) -> bool {
        self.r_b.is_valid()
            && self.angular_damping.is_finite()
            && self.time_step.is_finite()
            && self.candidate.runtime.solver_impulse().is_valid()
    }

    pub(super) fn warm_start(self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        let mut body_b = solver_body(self.body_b, bodies)?;
        body_b.angular_velocity *= self.angular_damping;
        let impulse = self.candidate.runtime.solver_impulse();
        body_b.linear_velocity += body_b.inverse_mass * impulse;
        body_b.angular_velocity += body_b.inverse_inertia * self.r_b.cross(impulse);
        store_solver_body(self.body_b, bodies, body_b)
    }

    pub(super) fn solve_velocity(
        &mut self,
        bodies: &mut [SolverBody],
    ) -> Result<(), ContactSolveFailure> {
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

    pub(super) fn finalize(self) -> JointImpulseSolution {
        JointImpulseSolution {
            joint_id: self.candidate.joint_id,
            runtime: JointRuntime::Mouse(self.candidate.runtime),
        }
    }
}

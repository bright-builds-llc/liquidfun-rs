use super::{
    ContactSolveFailure, FrictionConstraint, JointImpulseSolution, JointRuntime, MotorConstraint,
    RopeConstraint, RopeJointRuntime, Rotation, SolverBody, Vec2, map_joint_error,
    solver_body_pair, store_solver_body_pair, typed_solution,
};

impl FrictionConstraint {
    pub(super) fn is_finite(self) -> bool {
        self.r_a.is_valid() && self.r_b.is_valid() && self.time_step.is_finite()
    }

    pub(super) fn warm_start(self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let (linear, angular) = self.candidate.runtime.solver_impulses();
        body_a.linear_velocity -= body_a.inverse_mass * linear;
        body_a.angular_velocity -= body_a.inverse_inertia * (self.r_a.cross(linear) + angular);
        body_b.linear_velocity += body_b.inverse_mass * linear;
        body_b.angular_velocity += body_b.inverse_inertia * (self.r_b.cross(linear) + angular);
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)
    }

    pub(super) fn solve_velocity(
        &mut self,
        bodies: &mut [SolverBody],
    ) -> Result<(), ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let angular = self
            .candidate
            .runtime
            .solve_angular(
                self.candidate.definition,
                self.time_step,
                body_b.angular_velocity - body_a.angular_velocity,
            )
            .map_err(map_joint_error)?;
        body_a.angular_velocity -= body_a.inverse_inertia * angular;
        body_b.angular_velocity += body_b.inverse_inertia * angular;
        let relative_linear = body_b.linear_velocity
            + Vec2::scalar_cross(body_b.angular_velocity, self.r_b)
            - body_a.linear_velocity
            - Vec2::scalar_cross(body_a.angular_velocity, self.r_a);
        let linear = self
            .candidate
            .runtime
            .solve_linear(self.candidate.definition, self.time_step, relative_linear)
            .map_err(map_joint_error)?;
        body_a.linear_velocity -= body_a.inverse_mass * linear;
        body_a.angular_velocity -= body_a.inverse_inertia * self.r_a.cross(linear);
        body_b.linear_velocity += body_b.inverse_mass * linear;
        body_b.angular_velocity += body_b.inverse_inertia * self.r_b.cross(linear);
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)
    }

    pub(super) fn finalize(self) -> JointImpulseSolution {
        typed_solution(
            self.candidate.joint_id,
            JointRuntime::Friction(self.candidate.runtime),
        )
    }
}

impl RopeConstraint {
    pub(super) fn is_finite(self) -> bool {
        self.r_a.is_valid()
            && self.r_b.is_valid()
            && self.mass.is_finite()
            && self.inverse_time_step.is_finite()
            && self.candidate.runtime.solver_direction().is_valid()
    }

    pub(super) fn warm_start(self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let impulse = self.candidate.runtime.solver_impulse();
        let linear = impulse * self.candidate.runtime.solver_direction();
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
        let point_a =
            body_a.linear_velocity + Vec2::scalar_cross(body_a.angular_velocity, self.r_a);
        let point_b =
            body_b.linear_velocity + Vec2::scalar_cross(body_b.angular_velocity, self.r_b);
        let relative_speed = self
            .candidate
            .runtime
            .solver_direction()
            .dot(point_b - point_a);
        let impulse = self
            .candidate
            .runtime
            .solve_velocity(
                self.candidate.definition,
                self.inverse_time_step,
                relative_speed,
            )
            .map_err(map_joint_error)?;
        let linear = impulse * self.candidate.runtime.solver_direction();
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
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let q_a = Rotation::from_angle(body_a.angle);
        let q_b = Rotation::from_angle(body_b.angle);
        let r_a = q_a.apply(self.candidate.definition.local_anchor_a() - body_a.local_center);
        let r_b = q_b.apply(self.candidate.definition.local_anchor_b() - body_b.local_center);
        let mut direction = body_b.center + r_b - body_a.center - r_a;
        let length = direction.normalize();
        let (impulse, solved) =
            RopeJointRuntime::solve_position(self.candidate.definition, length, self.mass)
                .map_err(map_joint_error)?;
        let linear = impulse * direction;
        body_a.center -= body_a.inverse_mass * linear;
        body_a.angle -= body_a.inverse_inertia * r_a.cross(linear);
        body_b.center += body_b.inverse_mass * linear;
        body_b.angle += body_b.inverse_inertia * r_b.cross(linear);
        body_a.synchronize_transform();
        body_b.synchronize_transform();
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)?;
        Ok(solved)
    }

    pub(super) fn finalize(self) -> JointImpulseSolution {
        typed_solution(
            self.candidate.joint_id,
            JointRuntime::Rope(self.candidate.runtime),
        )
    }
}

impl MotorConstraint {
    pub(super) fn is_finite(self) -> bool {
        self.r_a.is_valid()
            && self.r_b.is_valid()
            && self.time_step.is_finite()
            && self.inverse_time_step.is_finite()
    }

    pub(super) fn warm_start(self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let (linear, angular) = self.candidate.runtime.solver_impulses();
        body_a.linear_velocity -= body_a.inverse_mass * linear;
        body_a.angular_velocity -= body_a.inverse_inertia * (self.r_a.cross(linear) + angular);
        body_b.linear_velocity += body_b.inverse_mass * linear;
        body_b.angular_velocity += body_b.inverse_inertia * (self.r_b.cross(linear) + angular);
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)
    }

    pub(super) fn solve_velocity(
        &mut self,
        bodies: &mut [SolverBody],
    ) -> Result<(), ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let angular = self
            .candidate
            .runtime
            .solve_angular(
                self.candidate.definition,
                self.time_step,
                self.inverse_time_step,
                body_b.angular_velocity - body_a.angular_velocity,
            )
            .map_err(map_joint_error)?;
        body_a.angular_velocity -= body_a.inverse_inertia * angular;
        body_b.angular_velocity += body_b.inverse_inertia * angular;
        let relative_linear = body_b.linear_velocity
            + Vec2::scalar_cross(body_b.angular_velocity, self.r_b)
            - body_a.linear_velocity
            - Vec2::scalar_cross(body_a.angular_velocity, self.r_a);
        let linear = self
            .candidate
            .runtime
            .solve_linear(
                self.candidate.definition,
                self.time_step,
                self.inverse_time_step,
                relative_linear,
            )
            .map_err(map_joint_error)?;
        body_a.linear_velocity -= body_a.inverse_mass * linear;
        body_a.angular_velocity -= body_a.inverse_inertia * self.r_a.cross(linear);
        body_b.linear_velocity += body_b.inverse_mass * linear;
        body_b.angular_velocity += body_b.inverse_inertia * self.r_b.cross(linear);
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)
    }

    pub(super) fn finalize(self) -> JointImpulseSolution {
        typed_solution(
            self.candidate.joint_id,
            JointRuntime::Motor(self.candidate.runtime),
        )
    }
}

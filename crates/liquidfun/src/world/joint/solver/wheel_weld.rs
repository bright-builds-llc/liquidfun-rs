use super::{
    ContactSolveFailure, JointImpulseSolution, JointRuntime, Rotation, SolverBody, Vec2,
    WeldConstraint, WeldRuntime, WheelConstraint, WheelRuntime, map_joint_error, point_angle_mass,
    point_velocity_difference, solver_body_pair, store_solver_body_pair, typed_solution,
};

impl WheelConstraint {
    pub(super) fn is_finite(self) -> bool {
        self.r_a.is_valid()
            && self.r_b.is_valid()
            && self.axis.is_valid()
            && self.perpendicular.is_valid()
            && [
                self.spring_lever_a,
                self.spring_lever_b,
                self.line_lever_a,
                self.line_lever_b,
                self.time_step,
            ]
            .into_iter()
            .all(f32::is_finite)
    }

    pub(super) fn warm_start(self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let [line, spring, motor] = self.candidate.runtime.solver_impulses();
        let linear = line * self.perpendicular + spring * self.axis;
        let angular_a = line * self.line_lever_a + spring * self.spring_lever_a + motor;
        let angular_b = line * self.line_lever_b + spring * self.spring_lever_b + motor;
        body_a.linear_velocity -= body_a.inverse_mass * linear;
        body_a.angular_velocity -= body_a.inverse_inertia * angular_a;
        body_b.linear_velocity += body_b.inverse_mass * linear;
        body_b.angular_velocity += body_b.inverse_inertia * angular_b;
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)
    }

    pub(super) fn solve_velocity(
        &mut self,
        bodies: &mut [SolverBody],
    ) -> Result<(), ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;

        let spring_speed = self
            .axis
            .dot(body_b.linear_velocity - body_a.linear_velocity)
            + self.spring_lever_b * body_b.angular_velocity
            - self.spring_lever_a * body_a.angular_velocity;
        let spring = self
            .candidate
            .runtime
            .solve_spring(spring_speed)
            .map_err(map_joint_error)?;
        let spring_linear = spring * self.axis;
        body_a.linear_velocity -= body_a.inverse_mass * spring_linear;
        body_a.angular_velocity -= body_a.inverse_inertia * spring * self.spring_lever_a;
        body_b.linear_velocity += body_b.inverse_mass * spring_linear;
        body_b.angular_velocity += body_b.inverse_inertia * spring * self.spring_lever_b;

        let motor = self
            .candidate
            .runtime
            .solve_motor(
                self.candidate.definition,
                self.time_step,
                body_b.angular_velocity - body_a.angular_velocity,
            )
            .map_err(map_joint_error)?;
        body_a.angular_velocity -= body_a.inverse_inertia * motor;
        body_b.angular_velocity += body_b.inverse_inertia * motor;

        let line_speed = self
            .perpendicular
            .dot(body_b.linear_velocity - body_a.linear_velocity)
            + self.line_lever_b * body_b.angular_velocity
            - self.line_lever_a * body_a.angular_velocity;
        let line = self
            .candidate
            .runtime
            .solve_line(line_speed)
            .map_err(map_joint_error)?;
        let line_linear = line * self.perpendicular;
        body_a.linear_velocity -= body_a.inverse_mass * line_linear;
        body_a.angular_velocity -= body_a.inverse_inertia * line * self.line_lever_a;
        body_b.linear_velocity += body_b.inverse_mass * line_linear;
        body_b.angular_velocity += body_b.inverse_inertia * line * self.line_lever_b;
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
        let d = body_b.center - body_a.center + r_b - r_a;
        let perpendicular = q_a.apply(Vec2::scalar_cross(
            1.0,
            self.candidate.definition.local_axis_a(),
        ));
        let lever_a = (d + r_a).cross(perpendicular);
        let lever_b = r_b.cross(perpendicular);
        let inverse_mass = body_a.inverse_mass
            + body_b.inverse_mass
            + body_a.inverse_inertia * self.line_lever_a * self.line_lever_a
            + body_b.inverse_inertia * self.line_lever_b * self.line_lever_b;
        let error = d.dot(perpendicular);
        let (impulse, solved) =
            WheelRuntime::solve_position(error, inverse_mass).map_err(map_joint_error)?;
        let linear = impulse * perpendicular;
        body_a.center -= body_a.inverse_mass * linear;
        body_a.angle -= body_a.inverse_inertia * impulse * lever_a;
        body_b.center += body_b.inverse_mass * linear;
        body_b.angle += body_b.inverse_inertia * impulse * lever_b;
        body_a.synchronize_transform();
        body_b.synchronize_transform();
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)?;
        Ok(solved)
    }

    pub(super) fn finalize(self) -> JointImpulseSolution {
        typed_solution(
            self.candidate.joint_id,
            JointRuntime::Wheel(self.candidate.runtime),
        )
    }
}

impl WeldConstraint {
    pub(super) fn is_finite(self) -> bool {
        self.r_a.is_valid() && self.r_b.is_valid()
    }

    pub(super) fn warm_start(self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        let impulse = self.candidate.runtime.solver_impulse();
        let linear = Vec2::new(impulse.x, impulse.y);
        body_a.linear_velocity -= body_a.inverse_mass * linear;
        body_a.angular_velocity -= body_a.inverse_inertia * (self.r_a.cross(linear) + impulse.z);
        body_b.linear_velocity += body_b.inverse_mass * linear;
        body_b.angular_velocity += body_b.inverse_inertia * (self.r_b.cross(linear) + impulse.z);
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)
    }

    pub(super) fn solve_velocity(
        &mut self,
        bodies: &mut [SolverBody],
    ) -> Result<(), ContactSolveFailure> {
        let (mut body_a, mut body_b) = solver_body_pair(self.body_a, self.body_b, bodies)?;
        if self.candidate.definition.frequency() > 0.0 {
            let angular = self
                .candidate
                .runtime
                .solve_soft_angular(body_b.angular_velocity - body_a.angular_velocity)
                .map_err(map_joint_error)?;
            body_a.angular_velocity -= body_a.inverse_inertia * angular;
            body_b.angular_velocity += body_b.inverse_inertia * angular;
            let relative_linear = point_velocity_difference(body_a, body_b, self.r_a, self.r_b);
            let linear = self
                .candidate
                .runtime
                .solve_soft_linear(relative_linear)
                .map_err(map_joint_error)?;
            body_a.linear_velocity -= body_a.inverse_mass * linear;
            body_a.angular_velocity -= body_a.inverse_inertia * self.r_a.cross(linear);
            body_b.linear_velocity += body_b.inverse_mass * linear;
            body_b.angular_velocity += body_b.inverse_inertia * self.r_b.cross(linear);
        } else {
            let relative_linear = point_velocity_difference(body_a, body_b, self.r_a, self.r_b);
            let relative_angular = body_b.angular_velocity - body_a.angular_velocity;
            let impulse = self
                .candidate
                .runtime
                .solve_velocity(self.candidate.definition, relative_linear, relative_angular)
                .map_err(map_joint_error)?;
            let linear = Vec2::new(impulse.x, impulse.y);
            body_a.linear_velocity -= body_a.inverse_mass * linear;
            body_a.angular_velocity -=
                body_a.inverse_inertia * (self.r_a.cross(linear) + impulse.z);
            body_b.linear_velocity += body_b.inverse_mass * linear;
            body_b.angular_velocity +=
                body_b.inverse_inertia * (self.r_b.cross(linear) + impulse.z);
        }
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
        let linear_error = body_b.center + r_b - body_a.center - r_a;
        let angular_error =
            body_b.angle - body_a.angle - self.candidate.definition.reference_angle();
        let mass = point_angle_mass(body_a, body_b, r_a, r_b);
        let (impulse, solved) = WeldRuntime::solve_position(
            self.candidate.definition,
            mass,
            linear_error,
            angular_error,
        )
        .map_err(map_joint_error)?;
        let linear = Vec2::new(impulse.x, impulse.y);
        body_a.center -= body_a.inverse_mass * linear;
        body_a.angle -= body_a.inverse_inertia * (r_a.cross(linear) + impulse.z);
        body_b.center += body_b.inverse_mass * linear;
        body_b.angle += body_b.inverse_inertia * (r_b.cross(linear) + impulse.z);
        body_a.synchronize_transform();
        body_b.synchronize_transform();
        store_solver_body_pair(self.body_a, self.body_b, bodies, body_a, body_b)?;
        Ok(solved)
    }

    pub(super) fn finalize(self) -> JointImpulseSolution {
        typed_solution(
            self.candidate.joint_id,
            JointRuntime::Weld(self.candidate.runtime),
        )
    }
}

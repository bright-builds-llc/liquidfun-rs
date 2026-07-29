use super::{
    ANGULAR_SLOP, ContactSolveFailure, JointImpulseSolution, JointRuntime, LINEAR_SLOP,
    MAX_ANGULAR_CORRECTION, MAX_LINEAR_CORRECTION, Mat22, Mat33, PrismaticConstraint,
    RevoluteConstraint, Rotation, SolverBody, Vec2, Vec3, map_joint_error, solver_body_pair,
    store_solver_body_pair,
};

impl RevoluteConstraint {
    pub(super) fn is_finite(self) -> bool {
        self.r_a.is_valid()
            && self.r_b.is_valid()
            && self.mass.first_column().is_valid()
            && self.mass.second_column().is_valid()
            && self.mass.third_column().is_valid()
            && self.motor_mass.is_finite()
            && self.time_step.is_finite()
    }

    pub(super) fn warm_start(self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
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

    pub(super) fn solve_velocity(
        &mut self,
        bodies: &mut [SolverBody],
    ) -> Result<(), ContactSolveFailure> {
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

    pub(super) fn solve_position(
        &mut self,
        bodies: &mut [SolverBody],
    ) -> Result<bool, ContactSolveFailure> {
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

    pub(super) fn finalize(self) -> JointImpulseSolution {
        JointImpulseSolution {
            joint_id: self.candidate.joint_id,
            runtime: JointRuntime::Revolute(self.candidate.runtime),
        }
    }
}

impl PrismaticConstraint {
    pub(super) fn is_finite(self) -> bool {
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

    pub(super) fn warm_start(self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
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

    pub(super) fn solve_velocity(
        &mut self,
        bodies: &mut [SolverBody],
    ) -> Result<(), ContactSolveFailure> {
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

    pub(super) fn solve_position(
        &mut self,
        bodies: &mut [SolverBody],
    ) -> Result<bool, ContactSolveFailure> {
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

    pub(super) fn finalize(self) -> JointImpulseSolution {
        JointImpulseSolution {
            joint_id: self.candidate.joint_id,
            runtime: JointRuntime::Prismatic(self.candidate.runtime),
        }
    }
}

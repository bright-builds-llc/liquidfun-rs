use super::{
    ContactSolveFailure, DistanceConstraint, DistanceJointDef, DistanceRuntime, FamilyCandidate,
    FrictionConstraint, FrictionJointDef, FrictionRuntime, GearConstraint, GearJointDef,
    GearRuntime, GearSolverLanes, LINEAR_SLOP, Mat22, Mat33, MotorConstraint, MotorJointDef,
    MotorRuntime, MouseConstraint, MouseJointDef, MouseRuntime, OrdinarySolverLanes,
    PrismaticConstraint, PrismaticJointDef, PrismaticRuntime, PulleyConstraint, PulleyJointDef,
    PulleyRuntime, RevoluteConstraint, RevoluteJointDef, RevoluteRuntime, RopeConstraint,
    RopeJointDef, RopeJointRuntime, Rotation, SolverBody, Vec2, Vec3, WeldConstraint, WeldJointDef,
    WeldRuntime, WheelConstraint, WheelJointDef, WheelRuntime, gear_solver_bodies,
};

pub(super) fn stage_revolute(
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

pub(super) fn stage_prismatic(
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

pub(super) fn stage_distance(
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

pub(super) fn stage_pulley(
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

pub(super) fn stage_mouse(
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

pub(super) fn stage_wheel(
    mut candidate: FamilyCandidate<WheelJointDef, WheelRuntime, OrdinarySolverLanes>,
    bodies: &[SolverBody],
    time_step: f32,
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<WheelConstraint, ContactSolveFailure> {
    let [first_index, second_index] = candidate.lanes.solver_indices(bodies)?;
    let body_a = bodies[first_index];
    let body_b = bodies[second_index];
    let q_a = Rotation::from_angle(body_a.angle);
    let q_b = Rotation::from_angle(body_b.angle);
    let r_a = q_a.apply(candidate.definition.local_anchor_a() - body_a.local_center);
    let r_b = q_b.apply(candidate.definition.local_anchor_b() - body_b.local_center);
    let d = body_b.center + r_b - body_a.center - r_a;
    let axis = q_a.apply(candidate.definition.local_axis_a());
    let perpendicular = q_a.apply(Vec2::scalar_cross(1.0, candidate.definition.local_axis_a()));
    let line_lever_a = (d + r_a).cross(perpendicular);
    let line_lever_b = r_b.cross(perpendicular);
    let spring_lever_a = (d + r_a).cross(axis);
    let spring_lever_b = r_b.cross(axis);
    candidate
        .runtime
        .initialize(
            candidate.definition,
            time_step,
            body_a.inverse_mass,
            body_b.inverse_mass,
            body_a.inverse_inertia,
            body_b.inverse_inertia,
            line_lever_a,
            line_lever_b,
            spring_lever_a,
            spring_lever_b,
            d.dot(axis),
            axis,
            warm_starting.then_some(time_step_ratio),
        )
        .map_err(map_joint_error)?;
    let constraint = WheelConstraint {
        candidate,
        body_a: first_index,
        body_b: second_index,
        r_a,
        r_b,
        axis,
        perpendicular,
        spring_lever_a,
        spring_lever_b,
        line_lever_a,
        line_lever_b,
        time_step,
    };
    if !constraint.is_finite() {
        return Err(ContactSolveFailure::NonFinite);
    }
    Ok(constraint)
}

pub(super) fn stage_weld(
    mut candidate: FamilyCandidate<WeldJointDef, WeldRuntime, OrdinarySolverLanes>,
    bodies: &[SolverBody],
    time_step: f32,
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<WeldConstraint, ContactSolveFailure> {
    let [first_index, second_index] = candidate.lanes.solver_indices(bodies)?;
    let body_a = bodies[first_index];
    let body_b = bodies[second_index];
    let q_a = Rotation::from_angle(body_a.angle);
    let q_b = Rotation::from_angle(body_b.angle);
    let r_a = q_a.apply(candidate.definition.local_anchor_a() - body_a.local_center);
    let r_b = q_b.apply(candidate.definition.local_anchor_b() - body_b.local_center);
    let mass = point_angle_mass(body_a, body_b, r_a, r_b);
    candidate
        .runtime
        .initialize(
            candidate.definition,
            time_step,
            body_b.angle - body_a.angle - candidate.definition.reference_angle(),
            body_a.inverse_inertia + body_b.inverse_inertia,
            mass,
            warm_starting.then_some(time_step_ratio),
        )
        .map_err(map_joint_error)?;
    let constraint = WeldConstraint {
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

pub(super) fn stage_friction(
    mut candidate: FamilyCandidate<FrictionJointDef, FrictionRuntime, OrdinarySolverLanes>,
    bodies: &[SolverBody],
    time_step: f32,
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<FrictionConstraint, ContactSolveFailure> {
    let [first_index, second_index] = candidate.lanes.solver_indices(bodies)?;
    let body_a = bodies[first_index];
    let body_b = bodies[second_index];
    let q_a = Rotation::from_angle(body_a.angle);
    let q_b = Rotation::from_angle(body_b.angle);
    let r_a = q_a.apply(candidate.definition.local_anchor_a() - body_a.local_center);
    let r_b = q_b.apply(candidate.definition.local_anchor_b() - body_b.local_center);
    candidate
        .runtime
        .initialize(
            point_mass(body_a, body_b, r_a, r_b),
            body_a.inverse_inertia + body_b.inverse_inertia,
            warm_starting.then_some(time_step_ratio),
        )
        .map_err(map_joint_error)?;
    let constraint = FrictionConstraint {
        candidate,
        body_a: first_index,
        body_b: second_index,
        r_a,
        r_b,
        time_step,
    };
    if !constraint.is_finite() {
        return Err(ContactSolveFailure::NonFinite);
    }
    Ok(constraint)
}

pub(super) fn stage_rope(
    mut candidate: FamilyCandidate<RopeJointDef, RopeJointRuntime, OrdinarySolverLanes>,
    bodies: &[SolverBody],
    time_step: f32,
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<RopeConstraint, ContactSolveFailure> {
    let [first_index, second_index] = candidate.lanes.solver_indices(bodies)?;
    let body_a = bodies[first_index];
    let body_b = bodies[second_index];
    let q_a = Rotation::from_angle(body_a.angle);
    let q_b = Rotation::from_angle(body_b.angle);
    let r_a = q_a.apply(candidate.definition.local_anchor_a() - body_a.local_center);
    let r_b = q_b.apply(candidate.definition.local_anchor_b() - body_b.local_center);
    let separation = body_b.center + r_b - body_a.center - r_a;
    let mut direction = separation;
    let length = direction.normalize();
    let lever_a = r_a.cross(direction);
    let lever_b = r_b.cross(direction);
    let inverse_mass = body_a.inverse_mass
        + body_a.inverse_inertia * lever_a * lever_a
        + body_b.inverse_mass
        + body_b.inverse_inertia * lever_b * lever_b;
    candidate
        .runtime
        .initialize(
            candidate.definition,
            separation,
            inverse_mass,
            warm_starting.then_some(time_step_ratio),
        )
        .map_err(map_joint_error)?;
    let constraint = RopeConstraint {
        candidate,
        body_a: first_index,
        body_b: second_index,
        r_a,
        r_b,
        mass: if length > LINEAR_SLOP {
            invert_positive(inverse_mass)
        } else {
            0.0
        },
        inverse_time_step: inverse_time_step(time_step),
    };
    if !constraint.is_finite() {
        return Err(ContactSolveFailure::NonFinite);
    }
    Ok(constraint)
}

pub(super) fn stage_motor(
    mut candidate: FamilyCandidate<MotorJointDef, MotorRuntime, OrdinarySolverLanes>,
    bodies: &[SolverBody],
    time_step: f32,
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<MotorConstraint, ContactSolveFailure> {
    let [first_index, second_index] = candidate.lanes.solver_indices(bodies)?;
    let body_a = bodies[first_index];
    let body_b = bodies[second_index];
    let q_a = Rotation::from_angle(body_a.angle);
    let q_b = Rotation::from_angle(body_b.angle);
    let r_a = q_a.apply(-body_a.local_center);
    let r_b = q_b.apply(-body_b.local_center);
    let linear_error =
        body_b.center + r_b - body_a.center - r_a - q_a.apply(candidate.definition.linear_offset());
    let angular_error = body_b.angle - body_a.angle - candidate.definition.angular_offset();
    candidate
        .runtime
        .initialize(
            point_mass(body_a, body_b, r_a, r_b),
            body_a.inverse_inertia + body_b.inverse_inertia,
            linear_error,
            angular_error,
            warm_starting.then_some(time_step_ratio),
        )
        .map_err(map_joint_error)?;
    let constraint = MotorConstraint {
        candidate,
        body_a: first_index,
        body_b: second_index,
        r_a,
        r_b,
        time_step,
        inverse_time_step: inverse_time_step(time_step),
    };
    if !constraint.is_finite() {
        return Err(ContactSolveFailure::NonFinite);
    }
    Ok(constraint)
}

pub(super) fn point_mass(body_a: SolverBody, body_b: SolverBody, r_a: Vec2, r_b: Vec2) -> Mat22 {
    let m_a = body_a.inverse_mass;
    let m_b = body_b.inverse_mass;
    let i_a = body_a.inverse_inertia;
    let i_b = body_b.inverse_inertia;
    Mat22::from_columns(
        Vec2::new(
            m_a + m_b + i_a * r_a.y * r_a.y + i_b * r_b.y * r_b.y,
            -i_a * r_a.x * r_a.y - i_b * r_b.x * r_b.y,
        ),
        Vec2::new(
            -i_a * r_a.x * r_a.y - i_b * r_b.x * r_b.y,
            m_a + m_b + i_a * r_a.x * r_a.x + i_b * r_b.x * r_b.x,
        ),
    )
}

pub(super) fn point_angle_mass(
    body_a: SolverBody,
    body_b: SolverBody,
    r_a: Vec2,
    r_b: Vec2,
) -> Mat33 {
    let linear = point_mass(body_a, body_b, r_a, r_b);
    let i_a = body_a.inverse_inertia;
    let i_b = body_b.inverse_inertia;
    Mat33::from_columns(
        Vec3::new(
            linear.first_column().x,
            linear.first_column().y,
            -r_a.y * i_a - r_b.y * i_b,
        ),
        Vec3::new(
            linear.second_column().x,
            linear.second_column().y,
            r_a.x * i_a + r_b.x * i_b,
        ),
        Vec3::new(
            -r_a.y * i_a - r_b.y * i_b,
            r_a.x * i_a + r_b.x * i_b,
            i_a + i_b,
        ),
    )
}

pub(super) fn invert_positive(value: f32) -> f32 {
    if value > 0.0 { 1.0 / value } else { 0.0 }
}

pub(super) fn inverse_time_step(time_step: f32) -> f32 {
    if time_step > 0.0 {
        1.0 / time_step
    } else {
        0.0
    }
}

pub(super) fn normalized_pulley_segment(mut segment: Vec2) -> Vec2 {
    let length = segment.length();
    if length > 10.0 * LINEAR_SLOP {
        segment *= 1.0 / length;
        segment
    } else {
        Vec2::ZERO
    }
}

pub(super) fn map_joint_error(_error: crate::JointMutationError) -> ContactSolveFailure {
    ContactSolveFailure::NonFinite
}

pub(super) fn stage_gear(
    mut candidate: FamilyCandidate<GearJointDef, GearRuntime, GearSolverLanes>,
    bodies: &[SolverBody],
    warm_starting: bool,
) -> Result<GearConstraint, ContactSolveFailure> {
    let body_indices = candidate.lanes.solver_indices(bodies)?;
    let solver_bodies = gear_solver_bodies(body_indices, bodies)?;
    candidate
        .runtime
        .initialize_velocity(&solver_bodies, warm_starting)
        .map_err(map_joint_error)?;
    Ok(GearConstraint {
        candidate,
        body_indices,
    })
}

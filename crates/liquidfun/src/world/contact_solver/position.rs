use super::{
    BAUMGARTE, ConstraintPoint, ContactConstraintInput, ContactPoint, ContactSolveFailure,
    LINEAR_SLOP, MAX_CONDITION_NUMBER, MAX_LINEAR_CORRECTION, SolverBody, Transform,
    VELOCITY_THRESHOLD, Vec2, VelocityConstraint, clamp, constraint_bodies, shape_radius,
    store_constraint_bodies, world_manifold,
};

pub(super) fn solve_position_constraints(
    constraint: &VelocityConstraint,
    bodies: &mut [SolverBody],
) -> Result<bool, ContactSolveFailure> {
    let (mut first, mut second) = constraint_bodies(constraint, bodies)?;
    let mut minimum_separation = 0.0_f32;
    for point_index in 0..constraint.manifold.points().len() {
        let maybe_world = world_manifold(
            &constraint.manifold,
            first.transform,
            constraint.first_radius,
            second.transform,
            constraint.second_radius,
        )
        .map_err(|_error| ContactSolveFailure::NonFinite)?;
        let Some(world) = maybe_world else {
            return Err(ContactSolveFailure::UnsupportedTopology);
        };
        let Some(point) = world.points().get(point_index).copied() else {
            return Err(ContactSolveFailure::UnsupportedTopology);
        };
        let normal = world.normal();
        let r_a = point.point() - first.center;
        let r_b = point.point() - second.center;
        minimum_separation = minimum_separation.min(point.separation());
        let correction = clamp(
            BAUMGARTE * (point.separation() + LINEAR_SLOP),
            -MAX_LINEAR_CORRECTION,
            0.0,
        );
        let normal_arm_a = r_a.cross(normal);
        let normal_arm_b = r_b.cross(normal);
        let effective_mass = first.inverse_mass
            + second.inverse_mass
            + first.inverse_inertia * normal_arm_a * normal_arm_a
            + second.inverse_inertia * normal_arm_b * normal_arm_b;
        let impulse = if effective_mass > 0.0 {
            -correction / effective_mass
        } else {
            0.0
        };
        let position_impulse = impulse * normal;
        first.center -= first.inverse_mass * position_impulse;
        first.angle -= first.inverse_inertia * r_a.cross(position_impulse);
        second.center += second.inverse_mass * position_impulse;
        second.angle += second.inverse_inertia * r_b.cross(position_impulse);
        first.synchronize_transform();
        second.synchronize_transform();
    }
    store_constraint_bodies(constraint, bodies, first, second)?;
    Ok(minimum_separation >= -3.0 * LINEAR_SLOP)
}

impl SolverBody {
    pub(in crate::world) fn synchronize_transform(&mut self) {
        let rotation = crate::math::Rotation::from_angle(self.angle);
        self.transform = Transform::from_position_angle(
            self.center - rotation.apply(self.local_center),
            self.angle,
        );
    }
}

pub(super) fn build_constraint(
    input: &ContactConstraintInput<'_>,
    bodies: &[SolverBody],
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<VelocityConstraint, ContactSolveFailure> {
    if input.first_body_index == input.second_body_index {
        return Err(ContactSolveFailure::UnsupportedTopology);
    }
    let first = *bodies
        .get(input.first_body_index)
        .ok_or(ContactSolveFailure::UnsupportedTopology)?;
    let second = *bodies
        .get(input.second_body_index)
        .ok_or(ContactSolveFailure::UnsupportedTopology)?;
    let contact = input.contact;
    let Some(manifold) = contact.maybe_manifold.as_ref() else {
        return Err(ContactSolveFailure::UnsupportedTopology);
    };
    if contact.points.is_empty() || contact.points.len() > 2 {
        return Err(ContactSolveFailure::UnsupportedTopology);
    }
    let maybe_world_manifold = world_manifold(
        manifold,
        first.transform,
        shape_radius(input.first_shape),
        second.transform,
        shape_radius(input.second_shape),
    )
    .map_err(|_error| ContactSolveFailure::NonFinite)?;
    let Some(world) = maybe_world_manifold else {
        return Err(ContactSolveFailure::UnsupportedTopology);
    };
    if world.points().len() != contact.points.len() {
        return Err(ContactSolveFailure::UnsupportedTopology);
    }

    let placeholder = ConstraintPoint::cold(contact.points[0].feature_id());
    let mut constraint = VelocityConstraint {
        contact_index: input.contact_index,
        first_body_index: input.first_body_index,
        second_body_index: input.second_body_index,
        manifold: manifold.clone(),
        first_radius: shape_radius(input.first_shape),
        second_radius: shape_radius(input.second_shape),
        points: [placeholder; 2],
        point_count: contact.points.len(),
        normal: world.normal(),
        friction: contact.friction,
        tangent_speed: contact.tangent_speed,
        k: [[0.0; 2]; 2],
        normal_mass: [[0.0; 2]; 2],
    };
    let tangent = constraint.normal.cross_scalar(1.0);
    for (index, (contact_point, world_point)) in
        contact.points.iter().zip(world.points()).enumerate()
    {
        constraint.points[index] = build_constraint_point(
            *contact_point,
            world_point.point(),
            constraint.normal,
            tangent,
            first,
            second,
            contact.restitution,
            time_step_ratio,
            warm_starting,
        );
    }
    prepare_two_point_block(&mut constraint, first, second);
    if !constraint.normal.is_valid()
        || !constraint.friction.is_finite()
        || !constraint.tangent_speed.is_finite()
        || constraint.friction < 0.0
        || constraint.points[..constraint.point_count]
            .iter()
            .any(|point| !point.is_finite())
    {
        return Err(ContactSolveFailure::NonFinite);
    }
    Ok(constraint)
}

#[allow(clippy::too_many_arguments)]
fn build_constraint_point(
    contact_point: ContactPoint,
    world_point: Vec2,
    normal: Vec2,
    tangent: Vec2,
    first: SolverBody,
    second: SolverBody,
    restitution: f32,
    time_step_ratio: f32,
    warm_starting: bool,
) -> ConstraintPoint {
    let r_a = world_point - first.center;
    let r_b = world_point - second.center;
    let normal_arm_a = r_a.cross(normal);
    let normal_arm_b = r_b.cross(normal);
    let normal_k = first.inverse_mass
        + second.inverse_mass
        + first.inverse_inertia * normal_arm_a * normal_arm_a
        + second.inverse_inertia * normal_arm_b * normal_arm_b;
    let tangent_arm_a = r_a.cross(tangent);
    let tangent_arm_b = r_b.cross(tangent);
    let tangent_k = first.inverse_mass
        + second.inverse_mass
        + first.inverse_inertia * tangent_arm_a * tangent_arm_a
        + second.inverse_inertia * tangent_arm_b * tangent_arm_b;
    let relative = second.linear_velocity + Vec2::scalar_cross(second.angular_velocity, r_b)
        - first.linear_velocity
        - Vec2::scalar_cross(first.angular_velocity, r_a);
    let relative_normal_velocity = normal.dot(relative);
    ConstraintPoint {
        feature_id: contact_point.feature_id(),
        r_a,
        r_b,
        normal_impulse: if warm_starting {
            time_step_ratio * contact_point.normal_impulse()
        } else {
            0.0
        },
        tangent_impulse: if warm_starting {
            time_step_ratio * contact_point.tangent_impulse()
        } else {
            0.0
        },
        normal_mass: if normal_k > 0.0 { 1.0 / normal_k } else { 0.0 },
        tangent_mass: if tangent_k > 0.0 {
            1.0 / tangent_k
        } else {
            0.0
        },
        velocity_bias: if relative_normal_velocity < -VELOCITY_THRESHOLD {
            -restitution * relative_normal_velocity
        } else {
            0.0
        },
    }
}

fn prepare_two_point_block(
    constraint: &mut VelocityConstraint,
    first: SolverBody,
    second: SolverBody,
) {
    if constraint.point_count != 2 {
        return;
    }
    let first_point = constraint.points[0];
    let second_point = constraint.points[1];
    let normal_arms = [
        [
            first_point.r_a.cross(constraint.normal),
            first_point.r_b.cross(constraint.normal),
        ],
        [
            second_point.r_a.cross(constraint.normal),
            second_point.r_b.cross(constraint.normal),
        ],
    ];
    let k11 = first.inverse_mass
        + second.inverse_mass
        + first.inverse_inertia * normal_arms[0][0] * normal_arms[0][0]
        + second.inverse_inertia * normal_arms[0][1] * normal_arms[0][1];
    let k22 = first.inverse_mass
        + second.inverse_mass
        + first.inverse_inertia * normal_arms[1][0] * normal_arms[1][0]
        + second.inverse_inertia * normal_arms[1][1] * normal_arms[1][1];
    let k12 = first.inverse_mass
        + second.inverse_mass
        + first.inverse_inertia * normal_arms[0][0] * normal_arms[1][0]
        + second.inverse_inertia * normal_arms[0][1] * normal_arms[1][1];
    let determinant = k11 * k22 - k12 * k12;
    if k11 * k11 >= MAX_CONDITION_NUMBER * determinant || determinant == 0.0 {
        constraint.point_count = 1;
        return;
    }
    constraint.k = [[k11, k12], [k12, k22]];
    let inverse = 1.0 / determinant;
    constraint.normal_mass = [
        [inverse * k22, -inverse * k12],
        [-inverse * k12, inverse * k11],
    ];
}

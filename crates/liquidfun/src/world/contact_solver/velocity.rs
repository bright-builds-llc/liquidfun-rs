use super::{
    ConstraintPoint, ContactSolveFailure, Shape, SolverBody, Vec2, VelocityConstraint, max,
};

pub(super) fn warm_start(
    constraint: &VelocityConstraint,
    bodies: &mut [SolverBody],
) -> Result<(), ContactSolveFailure> {
    let (mut first, mut second) = constraint_bodies(constraint, bodies)?;
    let tangent = constraint.normal.cross_scalar(1.0);
    for point in &constraint.points[..constraint.point_count] {
        let impulse = point.normal_impulse * constraint.normal + point.tangent_impulse * tangent;
        first.angular_velocity -= first.inverse_inertia * point.r_a.cross(impulse);
        first.linear_velocity -= first.inverse_mass * impulse;
        second.angular_velocity += second.inverse_inertia * point.r_b.cross(impulse);
        second.linear_velocity += second.inverse_mass * impulse;
    }
    store_constraint_bodies(constraint, bodies, first, second)
}

pub(super) fn solve_velocity_constraints(
    constraint: &mut VelocityConstraint,
    bodies: &mut [SolverBody],
) -> Result<(), ContactSolveFailure> {
    let (mut first, mut second) = constraint_bodies(constraint, bodies)?;
    solve_tangent_constraints(constraint, &mut first, &mut second);
    if constraint.point_count == 1 {
        solve_one_normal_constraint(constraint, &mut first, &mut second);
    } else {
        solve_two_normal_constraints(constraint, &mut first, &mut second);
    }
    store_constraint_bodies(constraint, bodies, first, second)
}

fn solve_tangent_constraints(
    constraint: &mut VelocityConstraint,
    first: &mut SolverBody,
    second: &mut SolverBody,
) {
    let tangent = constraint.normal.cross_scalar(1.0);
    for point in &mut constraint.points[..constraint.point_count] {
        let relative = second.linear_velocity
            + Vec2::scalar_cross(second.angular_velocity, point.r_b)
            - first.linear_velocity
            - Vec2::scalar_cross(first.angular_velocity, point.r_a);
        let lambda = point.tangent_mass * (constraint.tangent_speed - relative.dot(tangent));
        let maximum_friction = constraint.friction * point.normal_impulse;
        let new_impulse =
            (point.tangent_impulse + lambda).clamp(-maximum_friction, maximum_friction);
        let applied = new_impulse - point.tangent_impulse;
        point.tangent_impulse = new_impulse;
        apply_impulse(applied * tangent, *point, first, second);
    }
}

fn solve_one_normal_constraint(
    constraint: &mut VelocityConstraint,
    first: &mut SolverBody,
    second: &mut SolverBody,
) {
    let point = &mut constraint.points[0];
    let relative = second.linear_velocity + Vec2::scalar_cross(second.angular_velocity, point.r_b)
        - first.linear_velocity
        - Vec2::scalar_cross(first.angular_velocity, point.r_a);
    let lambda = -point.normal_mass * (relative.dot(constraint.normal) - point.velocity_bias);
    let new_impulse = max(point.normal_impulse + lambda, 0.0);
    let applied = new_impulse - point.normal_impulse;
    point.normal_impulse = new_impulse;
    apply_impulse(applied * constraint.normal, *point, first, second);
}

fn solve_two_normal_constraints(
    constraint: &mut VelocityConstraint,
    first: &mut SolverBody,
    second: &mut SolverBody,
) {
    let first_point = constraint.points[0];
    let second_point = constraint.points[1];
    let accumulated = [first_point.normal_impulse, second_point.normal_impulse];
    let relative_first = second.linear_velocity
        + Vec2::scalar_cross(second.angular_velocity, first_point.r_b)
        - first.linear_velocity
        - Vec2::scalar_cross(first.angular_velocity, first_point.r_a);
    let relative_second = second.linear_velocity
        + Vec2::scalar_cross(second.angular_velocity, second_point.r_b)
        - first.linear_velocity
        - Vec2::scalar_cross(first.angular_velocity, second_point.r_a);
    let mut b = [
        relative_first.dot(constraint.normal) - first_point.velocity_bias,
        relative_second.dot(constraint.normal) - second_point.velocity_bias,
    ];
    b[0] -= constraint.k[0][0] * accumulated[0] + constraint.k[0][1] * accumulated[1];
    b[1] -= constraint.k[1][0] * accumulated[0] + constraint.k[1][1] * accumulated[1];

    let both = [
        -(constraint.normal_mass[0][0] * b[0] + constraint.normal_mass[0][1] * b[1]),
        -(constraint.normal_mass[1][0] * b[0] + constraint.normal_mass[1][1] * b[1]),
    ];
    let maybe_solution = if both[0] >= 0.0 && both[1] >= 0.0 {
        Some(both)
    } else {
        let first_only = [-first_point.normal_mass * b[0], 0.0];
        let second_velocity = constraint.k[1][0] * first_only[0] + b[1];
        if first_only[0] >= 0.0 && second_velocity >= 0.0 {
            Some(first_only)
        } else {
            let second_only = [0.0, -second_point.normal_mass * b[1]];
            let first_velocity = constraint.k[0][1] * second_only[1] + b[0];
            if second_only[1] >= 0.0 && first_velocity >= 0.0 {
                Some(second_only)
            } else if b[0] >= 0.0 && b[1] >= 0.0 {
                Some([0.0, 0.0])
            } else {
                None
            }
        }
    };
    let Some(solution) = maybe_solution else {
        return;
    };
    let delta = [solution[0] - accumulated[0], solution[1] - accumulated[1]];
    apply_two_impulses(
        delta,
        [first_point, second_point],
        constraint.normal,
        first,
        second,
    );
    constraint.points[0].normal_impulse = solution[0];
    constraint.points[1].normal_impulse = solution[1];
}

fn apply_impulse(
    impulse: Vec2,
    point: ConstraintPoint,
    first: &mut SolverBody,
    second: &mut SolverBody,
) {
    first.linear_velocity -= first.inverse_mass * impulse;
    first.angular_velocity -= first.inverse_inertia * point.r_a.cross(impulse);
    second.linear_velocity += second.inverse_mass * impulse;
    second.angular_velocity += second.inverse_inertia * point.r_b.cross(impulse);
}

fn apply_two_impulses(
    delta: [f32; 2],
    points: [ConstraintPoint; 2],
    normal: Vec2,
    first: &mut SolverBody,
    second: &mut SolverBody,
) {
    let first_impulse = delta[0] * normal;
    let second_impulse = delta[1] * normal;
    let total = first_impulse + second_impulse;
    first.linear_velocity -= first.inverse_mass * total;
    first.angular_velocity -= first.inverse_inertia
        * (points[0].r_a.cross(first_impulse) + points[1].r_a.cross(second_impulse));
    second.linear_velocity += second.inverse_mass * total;
    second.angular_velocity += second.inverse_inertia
        * (points[0].r_b.cross(first_impulse) + points[1].r_b.cross(second_impulse));
}

pub(super) fn validate_solution(
    constraint: &VelocityConstraint,
    bodies: &[SolverBody],
) -> Result<(), ContactSolveFailure> {
    let (first, second) = constraint_bodies(constraint, bodies)?;
    if !first.is_finite()
        || !second.is_finite()
        || constraint.points[..constraint.point_count]
            .iter()
            .any(|point| !point.is_finite())
    {
        return Err(ContactSolveFailure::NonFinite);
    }
    Ok(())
}

pub(super) fn constraint_bodies(
    constraint: &VelocityConstraint,
    bodies: &[SolverBody],
) -> Result<(SolverBody, SolverBody), ContactSolveFailure> {
    let first = bodies
        .get(constraint.first_body_index)
        .copied()
        .ok_or(ContactSolveFailure::UnsupportedTopology)?;
    let second = bodies
        .get(constraint.second_body_index)
        .copied()
        .ok_or(ContactSolveFailure::UnsupportedTopology)?;
    Ok((first, second))
}

pub(super) fn store_constraint_bodies(
    constraint: &VelocityConstraint,
    bodies: &mut [SolverBody],
    first: SolverBody,
    second: SolverBody,
) -> Result<(), ContactSolveFailure> {
    if constraint.first_body_index == constraint.second_body_index {
        return Err(ContactSolveFailure::UnsupportedTopology);
    }
    let Some(first_lane) = bodies.get_mut(constraint.first_body_index) else {
        return Err(ContactSolveFailure::UnsupportedTopology);
    };
    *first_lane = first;
    let Some(second_lane) = bodies.get_mut(constraint.second_body_index) else {
        return Err(ContactSolveFailure::UnsupportedTopology);
    };
    *second_lane = second;
    Ok(())
}

pub(super) const fn shape_radius(shape: &Shape) -> f32 {
    match shape {
        Shape::Circle(circle) => circle.radius(),
        Shape::Edge(edge) => edge.radius(),
        Shape::Polygon(polygon) => polygon.radius(),
        Shape::Chain(chain) => chain.radius(),
    }
}

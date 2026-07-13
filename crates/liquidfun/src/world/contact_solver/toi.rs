use super::{
    BodyState, ContactConstraintInput, ContactImpulseSolution, ContactSolveFailure, LINEAR_SLOP,
    MAX_LINEAR_CORRECTION, SolvedBodyMotion, SolverBody, StepConfiguration, TOI_BAUMGARTE, Vec2,
    VelocityConstraint, build_constraints, clamp, constraint_bodies, integrate_position,
    solve_velocity_constraints, store_constraint_bodies, transient_impulses, validate_solution,
    world_manifold,
};

#[derive(Debug)]
pub(in crate::world) struct ToiConstraintSolution {
    pub(in crate::world) motions: Vec<SolvedBodyMotion>,
    pub(in crate::world) initial_centers: Vec<Vec2>,
    pub(in crate::world) initial_angles: Vec<f32>,
    pub(in crate::world) contact_impulses: Vec<ContactImpulseSolution>,
}

pub(in crate::world) fn solve_toi_constraints(
    body_states: &[BodyState],
    inputs: &[ContactConstraintInput<'_>],
    configuration: StepConfiguration,
    alpha: f32,
    seed_body_indices: [usize; 2],
) -> Result<ToiConstraintSolution, ContactSolveFailure> {
    if !(0.0..1.0).contains(&alpha) || seed_body_indices[0] == seed_body_indices[1] {
        return Err(ContactSolveFailure::UnsupportedTopology);
    }
    let mut bodies = Vec::new();
    bodies.try_reserve_exact(body_states.len()).map_err(|_| {
        ContactSolveFailure::CapacityExceeded {
            resource: "TOI solver bodies",
            limit: body_states.len(),
        }
    })?;
    for state in body_states {
        bodies.push(SolverBody::from_state(*state)?);
    }
    for seed in seed_body_indices {
        if seed >= bodies.len() {
            return Err(ContactSolveFailure::UnsupportedTopology);
        }
    }

    let mut position_constraints = build_constraints(inputs, &bodies, 1.0, false)?;
    for _iteration in 0..20 {
        let mut solved = true;
        for constraint in &position_constraints {
            solved = solve_toi_position_constraints(constraint, &mut bodies, seed_body_indices)?
                && solved;
        }
        if solved {
            break;
        }
    }
    let initial_centers = bodies.iter().map(|body| body.center).collect::<Vec<_>>();
    let initial_angles = bodies.iter().map(|body| body.angle).collect::<Vec<_>>();

    position_constraints.clear();
    let mut constraints = build_constraints(inputs, &bodies, 1.0, false)?;
    for _iteration in 0..configuration.velocity_iterations() {
        for constraint in &mut constraints {
            solve_velocity_constraints(constraint, &mut bodies)?;
        }
    }
    let contact_impulses = transient_impulses(&constraints)?;

    let sub_time_step = (1.0 - alpha) * configuration.time_step();
    if !sub_time_step.is_finite() || sub_time_step <= 0.0 {
        return Err(ContactSolveFailure::NonFinite);
    }
    for body in &mut bodies {
        integrate_position(body, sub_time_step);
        if !body.is_finite() {
            return Err(ContactSolveFailure::NonFinite);
        }
    }
    for constraint in &constraints {
        validate_solution(constraint, &bodies)?;
    }

    Ok(ToiConstraintSolution {
        motions: bodies
            .into_iter()
            .map(|body| SolvedBodyMotion {
                position: body.transform.position(),
                angle: body.angle,
                linear: body.linear_velocity,
                angular: body.angular_velocity,
            })
            .collect(),
        initial_centers,
        initial_angles,
        contact_impulses,
    })
}

fn solve_toi_position_constraints(
    constraint: &VelocityConstraint,
    bodies: &mut [SolverBody],
    seed_body_indices: [usize; 2],
) -> Result<bool, ContactSolveFailure> {
    let (mut first, mut second) = constraint_bodies(constraint, bodies)?;
    let first_moves = seed_body_indices.contains(&constraint.first_body_index);
    let second_moves = seed_body_indices.contains(&constraint.second_body_index);
    let first_inverse_mass = if first_moves { first.inverse_mass } else { 0.0 };
    let first_inverse_inertia = if first_moves {
        first.inverse_inertia
    } else {
        0.0
    };
    let second_inverse_mass = if second_moves {
        second.inverse_mass
    } else {
        0.0
    };
    let second_inverse_inertia = if second_moves {
        second.inverse_inertia
    } else {
        0.0
    };
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
            TOI_BAUMGARTE * (point.separation() + LINEAR_SLOP),
            -MAX_LINEAR_CORRECTION,
            0.0,
        );
        let normal_arm_a = r_a.cross(normal);
        let normal_arm_b = r_b.cross(normal);
        let effective_mass = first_inverse_mass
            + second_inverse_mass
            + first_inverse_inertia * normal_arm_a * normal_arm_a
            + second_inverse_inertia * normal_arm_b * normal_arm_b;
        let impulse = if effective_mass > 0.0 {
            -correction / effective_mass
        } else {
            0.0
        };
        let position_impulse = impulse * normal;
        first.center -= first_inverse_mass * position_impulse;
        first.angle -= first_inverse_inertia * r_a.cross(position_impulse);
        second.center += second_inverse_mass * position_impulse;
        second.angle += second_inverse_inertia * r_b.cross(position_impulse);
        first.synchronize_transform();
        second.synchronize_transform();
    }
    store_constraint_bodies(constraint, bodies, first, second)?;
    Ok(minimum_separation >= -1.5 * LINEAR_SLOP)
}

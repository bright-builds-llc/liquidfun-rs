use super::{ContactSolveFailure, JointId, JointImpulseSolution, JointRuntime, SolverBody, Vec2};

pub(super) fn typed_solution(joint_id: JointId, runtime: JointRuntime) -> JointImpulseSolution {
    JointImpulseSolution { joint_id, runtime }
}

pub(super) fn point_velocity_difference(
    body_a: SolverBody,
    body_b: SolverBody,
    r_a: Vec2,
    r_b: Vec2,
) -> Vec2 {
    body_b.linear_velocity + Vec2::scalar_cross(body_b.angular_velocity, r_b)
        - body_a.linear_velocity
        - Vec2::scalar_cross(body_a.angular_velocity, r_a)
}

pub(super) fn solver_body(
    index: usize,
    bodies: &[SolverBody],
) -> Result<SolverBody, ContactSolveFailure> {
    bodies
        .get(index)
        .copied()
        .ok_or(ContactSolveFailure::UnsupportedTopology)
}

pub(super) fn store_solver_body(
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

pub(super) fn solver_body_pair(
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

pub(super) fn store_solver_body_pair(
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

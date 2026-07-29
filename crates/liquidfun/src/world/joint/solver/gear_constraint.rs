use super::{
    ContactSolveFailure, GearConstraint, JointImpulseSolution, JointRuntime, SolverBody, Vec2,
    map_joint_error, typed_solution,
};

impl GearConstraint {
    pub(super) fn warm_start(&self, bodies: &mut [SolverBody]) -> Result<(), ContactSolveFailure> {
        let before = gear_solver_bodies(self.body_indices, bodies)?;
        let mut solved = before;
        self.candidate
            .runtime
            .warm_start(&mut solved)
            .map_err(map_joint_error)?;
        store_gear_velocity_deltas(self.body_indices, bodies, before, solved)
    }

    pub(super) fn solve_velocity(
        &mut self,
        bodies: &mut [SolverBody],
    ) -> Result<(), ContactSolveFailure> {
        let before = gear_solver_bodies(self.body_indices, bodies)?;
        let mut solved = before;
        self.candidate
            .runtime
            .solve_velocity(&mut solved)
            .map_err(map_joint_error)?;
        store_gear_velocity_deltas(self.body_indices, bodies, before, solved)
    }

    pub(super) fn solve_position(
        &mut self,
        bodies: &mut [SolverBody],
    ) -> Result<bool, ContactSolveFailure> {
        let before = gear_solver_bodies(self.body_indices, bodies)?;
        let mut solved = before;
        let position_solved = self
            .candidate
            .runtime
            .solve_position(&mut solved)
            .map_err(map_joint_error)?;
        store_gear_position_deltas(self.body_indices, bodies, before, solved)?;
        Ok(position_solved)
    }

    pub(super) fn finalize(self) -> JointImpulseSolution {
        typed_solution(
            self.candidate.joint_id,
            JointRuntime::Gear(self.candidate.runtime),
        )
    }
}

pub(super) fn gear_solver_bodies(
    indices: [usize; 4],
    bodies: &[SolverBody],
) -> Result<[crate::world::joint::gear::GearSolverBody; 4], ContactSolveFailure> {
    Ok([
        gear_solver_body(indices[0], bodies)?,
        gear_solver_body(indices[1], bodies)?,
        gear_solver_body(indices[2], bodies)?,
        gear_solver_body(indices[3], bodies)?,
    ])
}

pub(super) fn gear_solver_body(
    index: usize,
    bodies: &[SolverBody],
) -> Result<crate::world::joint::gear::GearSolverBody, ContactSolveFailure> {
    let body = bodies
        .get(index)
        .copied()
        .ok_or(ContactSolveFailure::UnsupportedTopology)?;
    Ok(crate::world::joint::gear::GearSolverBody {
        center: body.center,
        angle: body.angle,
        linear_velocity: body.linear_velocity,
        angular_velocity: body.angular_velocity,
        local_center: body.local_center,
        inverse_mass: body.inverse_mass,
        inverse_inertia: body.inverse_inertia,
    })
}

pub(super) fn store_gear_velocity_deltas(
    indices: [usize; 4],
    bodies: &mut [SolverBody],
    before: [crate::world::joint::gear::GearSolverBody; 4],
    solved: [crate::world::joint::gear::GearSolverBody; 4],
) -> Result<(), ContactSolveFailure> {
    let mut candidates = [None; 4];
    for lane in 0..4 {
        if indices[..lane].contains(&indices[lane]) {
            continue;
        }
        let mut linear_delta = Vec2::ZERO;
        let mut angular_delta = 0.0;
        for matching in 0..4 {
            if indices[matching] == indices[lane] {
                linear_delta += solved[matching].linear_velocity - before[matching].linear_velocity;
                angular_delta +=
                    solved[matching].angular_velocity - before[matching].angular_velocity;
            }
        }
        let mut candidate = bodies
            .get(indices[lane])
            .copied()
            .ok_or(ContactSolveFailure::UnsupportedTopology)?;
        candidate.linear_velocity += linear_delta;
        candidate.angular_velocity += angular_delta;
        if !candidate.is_finite() {
            return Err(ContactSolveFailure::NonFinite);
        }
        candidates[lane] = Some(candidate);
    }
    commit_gear_candidates(indices, bodies, candidates)
}

pub(super) fn store_gear_position_deltas(
    indices: [usize; 4],
    bodies: &mut [SolverBody],
    before: [crate::world::joint::gear::GearSolverBody; 4],
    solved: [crate::world::joint::gear::GearSolverBody; 4],
) -> Result<(), ContactSolveFailure> {
    let mut candidates = [None; 4];
    for lane in 0..4 {
        if indices[..lane].contains(&indices[lane]) {
            continue;
        }
        let mut center_delta = Vec2::ZERO;
        let mut angle_delta = 0.0;
        for matching in 0..4 {
            if indices[matching] == indices[lane] {
                center_delta += solved[matching].center - before[matching].center;
                angle_delta += solved[matching].angle - before[matching].angle;
            }
        }
        let mut candidate = bodies
            .get(indices[lane])
            .copied()
            .ok_or(ContactSolveFailure::UnsupportedTopology)?;
        candidate.center += center_delta;
        candidate.angle += angle_delta;
        candidate.synchronize_transform();
        if !candidate.is_finite() {
            return Err(ContactSolveFailure::NonFinite);
        }
        candidates[lane] = Some(candidate);
    }
    commit_gear_candidates(indices, bodies, candidates)
}

pub(super) fn commit_gear_candidates(
    indices: [usize; 4],
    bodies: &mut [SolverBody],
    candidates: [Option<SolverBody>; 4],
) -> Result<(), ContactSolveFailure> {
    for (lane, maybe_candidate) in candidates.into_iter().enumerate() {
        let Some(candidate) = maybe_candidate else {
            continue;
        };
        let body = bodies
            .get_mut(indices[lane])
            .ok_or(ContactSolveFailure::UnsupportedTopology)?;
        *body = candidate;
    }
    Ok(())
}

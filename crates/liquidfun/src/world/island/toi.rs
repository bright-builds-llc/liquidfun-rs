use super::{
    Arena, BodyId, BodyState, ContactConstraintInput, ContactImpulseSolution, ContactManager,
    ContactSolveFailure, Fixture, FixtureId, IslandBuildError, StepConfiguration, reserve_one,
    solve_toi_constraints,
};

#[derive(Debug, Clone, Copy)]
pub(in crate::world) struct ToiIslandLimits {
    pub(in crate::world) max_bodies: usize,
    pub(in crate::world) max_contacts: usize,
}

impl ToiIslandLimits {
    pub(in crate::world) const REVIEWED: Self = Self {
        max_bodies: 2 * crate::math::settings::MAX_TOI_CONTACTS,
        max_contacts: crate::math::settings::MAX_TOI_CONTACTS,
    };
}

#[derive(Debug)]
pub(in crate::world) struct ToiIsland {
    pub(in crate::world) body_ids: Vec<BodyId>,
    pub(in crate::world) body_states: Vec<BodyState>,
    pub(in crate::world) contact_indices: Vec<usize>,
    pub(in crate::world) seed_body_indices: [usize; 2],
    limits: ToiIslandLimits,
}

impl ToiIsland {
    pub(in crate::world) fn new(
        bodies: &[(BodyId, BodyState); 2],
        seed_contact_index: usize,
        limits: ToiIslandLimits,
    ) -> Result<Self, IslandBuildError> {
        if bodies[0].0 == bodies[1].0 || limits.max_bodies < 2 || limits.max_contacts < 1 {
            return Err(IslandBuildError::CapacityExceeded {
                resource: "TOI island seed",
                limit: limits.max_bodies.min(limits.max_contacts),
            });
        }
        let mut island = Self {
            body_ids: Vec::new(),
            body_states: Vec::new(),
            contact_indices: Vec::new(),
            seed_body_indices: [0, 1],
            limits,
        };
        island.add_body(bodies[0].0, bodies[0].1)?;
        island.add_body(bodies[1].0, bodies[1].1)?;
        island.add_contact(seed_contact_index)?;
        Ok(island)
    }

    pub(in crate::world) fn contains_body(&self, body: BodyId) -> bool {
        self.body_ids.contains(&body)
    }

    pub(in crate::world) fn contains_contact(&self, contact_index: usize) -> bool {
        self.contact_indices.contains(&contact_index)
    }

    pub(in crate::world) fn has_body_capacity(&self) -> bool {
        self.body_ids.len() < self.limits.max_bodies
    }

    pub(in crate::world) fn has_contact_capacity(&self) -> bool {
        self.contact_indices.len() < self.limits.max_contacts
    }

    pub(in crate::world) fn add_body(
        &mut self,
        body: BodyId,
        state: BodyState,
    ) -> Result<(), IslandBuildError> {
        if !self.has_body_capacity() {
            return Err(IslandBuildError::CapacityExceeded {
                resource: "TOI island bodies",
                limit: self.limits.max_bodies,
            });
        }
        reserve_one(
            &mut self.body_ids,
            "TOI island bodies",
            self.limits.max_bodies,
        )?;
        reserve_one(
            &mut self.body_states,
            "TOI island body states",
            self.limits.max_bodies,
        )?;
        self.body_ids.push(body);
        self.body_states.push(state);
        Ok(())
    }

    pub(in crate::world) fn add_contact(
        &mut self,
        contact_index: usize,
    ) -> Result<(), IslandBuildError> {
        if !self.has_contact_capacity() {
            return Err(IslandBuildError::CapacityExceeded {
                resource: "TOI island contacts",
                limit: self.limits.max_contacts,
            });
        }
        reserve_one(
            &mut self.contact_indices,
            "TOI island contacts",
            self.limits.max_contacts,
        )?;
        self.contact_indices.push(contact_index);
        Ok(())
    }
}

#[derive(Debug)]
pub(in crate::world) struct ToiIslandSolution {
    pub(in crate::world) body_ids: Vec<BodyId>,
    pub(in crate::world) body_states: Vec<BodyState>,
    pub(in crate::world) contact_impulses: Vec<ContactImpulseSolution>,
}

pub(in crate::world) fn solve_toi_island(
    island: &ToiIsland,
    contact_manager: &ContactManager,
    fixtures: &Arena<Fixture, FixtureId>,
    configuration: StepConfiguration,
    alpha: f32,
) -> Result<ToiIslandSolution, ContactSolveFailure> {
    let mut inputs = Vec::new();
    inputs
        .try_reserve_exact(island.contact_indices.len())
        .map_err(|_| ContactSolveFailure::CapacityExceeded {
            resource: "TOI island contact inputs",
            limit: island.contact_indices.len(),
        })?;
    for contact_index in &island.contact_indices {
        let contact = contact_manager
            .contacts()
            .get(*contact_index)
            .ok_or(ContactSolveFailure::UnsupportedTopology)?;
        let first_body_index = island
            .body_ids
            .iter()
            .position(|body| *body == contact.key.first.body)
            .ok_or(ContactSolveFailure::UnsupportedTopology)?;
        let second_body_index = island
            .body_ids
            .iter()
            .position(|body| *body == contact.key.second.body)
            .ok_or(ContactSolveFailure::UnsupportedTopology)?;
        let first_shape = fixtures
            .get(contact.key.first.fixture)
            .map_err(|_| ContactSolveFailure::UnsupportedTopology)?
            .definition
            .shape();
        let second_shape = fixtures
            .get(contact.key.second.fixture)
            .map_err(|_| ContactSolveFailure::UnsupportedTopology)?
            .definition
            .shape();
        inputs.push(ContactConstraintInput {
            contact_index: *contact_index,
            first_body_index,
            second_body_index,
            contact,
            first_shape,
            second_shape,
        });
    }

    let solved = solve_toi_constraints(
        &island.body_states,
        &inputs,
        configuration,
        alpha,
        island.seed_body_indices,
    )?;
    if solved.motions.len() != island.body_states.len()
        || solved.initial_centers.len() != island.body_states.len()
        || solved.initial_angles.len() != island.body_states.len()
    {
        return Err(ContactSolveFailure::UnsupportedTopology);
    }
    let mut body_states = island.body_states.clone();
    for (index, state) in body_states.iter_mut().enumerate() {
        let motion = solved.motions[index];
        *state = state
            .candidate_set_toi_solver_state(
                solved.initial_centers[index],
                solved.initial_angles[index],
                motion.position,
                motion.angle,
                motion.linear,
                motion.angular,
            )
            .map_err(|_error| ContactSolveFailure::NonFinite)?;
    }
    Ok(ToiIslandSolution {
        body_ids: island.body_ids.clone(),
        body_states,
        contact_impulses: solved.contact_impulses,
    })
}

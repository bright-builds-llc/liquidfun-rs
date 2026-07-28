#[cfg(test)]
use super::Vec2;
use super::{
    Body, BodyId, BodyType, CollisionDecisionHook, ContactHookRun, ContactSolve,
    ContactSolveFailure, FixtureBoundsError, FixtureId, FixtureProxies, IslandLimits,
    IslandSolveParameters, ParticleSystem, ParticleSystemId, PreparedFixtureBounds,
    PreparedSynchronization, SolveFailureInjection, StepError, StepTiming, World,
    WorldStepCandidate, build_islands, contact_solve_build_error, maybe_solution_body_state,
    solve_islands,
};
#[cfg(test)]
use crate::world::step::{NoDecisionHook, StepLimits};

impl World {
    #[cfg(test)]
    pub(in crate::world) fn find_new_contacts(&mut self) {
        let mut hook = NoDecisionHook;
        let mut hook_run = ContactHookRun::new(&mut hook, StepLimits::default());
        self.find_new_contacts_with_hook(&mut hook_run)
            .expect("default internal contact discovery remains within reviewed limits");
    }

    pub(in crate::world) fn find_new_contacts_with_hook<H: CollisionDecisionHook>(
        &mut self,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        self.resolve_pending_body_wakes();
        self.contact_manager.find_new_contacts(
            &mut self.broad_phase,
            &mut self.bodies,
            &mut self.fixtures,
            &self.joints,
            hook_run,
        )
    }

    pub(super) fn resolve_pending_body_wakes(&mut self) {
        for body_id in self.body_order.iter().copied() {
            let record = self
                .bodies
                .get_mut(body_id)
                .expect("source-ordered body remains live while resolving wake markers");
            if !record.pending_wake {
                continue;
            }
            record.state = record.state.candidate_set_awake(true);
            record.pending_wake = false;
        }
    }

    #[cfg(test)]
    pub(in crate::world) fn update_contacts(&mut self) {
        let mut hook = NoDecisionHook;
        let mut hook_run = ContactHookRun::new(&mut hook, StepLimits::default());
        self.update_contacts_with_hook(&mut hook_run)
            .expect("default contact hook remains within reviewed limits");
    }

    pub(in crate::world) fn update_contacts_with_hook<H: CollisionDecisionHook>(
        &mut self,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        self.contact_manager.update_contacts(
            &self.broad_phase,
            &mut self.bodies,
            &mut self.fixtures,
            &self.joints,
            hook_run,
        )
    }

    pub(in crate::world) fn solve_contacts<H: CollisionDecisionHook>(
        &mut self,
        configuration: crate::world::config::StepConfiguration,
        timing: StepTiming,
        maybe_failure_injection: Option<SolveFailureInjection>,
        contact_transitions: &[crate::world::contact::ContactTransition],
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<Vec<ContactSolve>, StepError> {
        let candidate = self
            .prepare_world_step_candidate(configuration, timing, maybe_failure_injection)
            .map_err(|error| crate::world::step::solver_step_error(error, contact_transitions))?;
        self.commit_world_step_candidate(candidate, hook_run)
    }

    #[allow(
        clippy::too_many_lines,
        reason = "the complete world step candidate must remain one prepare-before-commit transaction"
    )]
    pub(super) fn prepare_world_step_candidate(
        &self,
        configuration: crate::world::config::StepConfiguration,
        timing: StepTiming,
        maybe_failure_injection: Option<SolveFailureInjection>,
    ) -> Result<WorldStepCandidate, ContactSolveFailure> {
        let islands = build_islands(
            &self.body_order,
            &self.bodies,
            &self.joints,
            &self.contact_manager,
            IslandLimits::REVIEWED,
        )
        .map_err(contact_solve_build_error)?;
        let solutions = solve_islands(
            &islands,
            &self.contact_manager,
            &self.fixtures,
            &self.joints,
            IslandSolveParameters::new(
                self.gravity(),
                configuration,
                timing.time_step_ratio(),
                self.is_warm_starting_enabled(),
                maybe_failure_injection,
            ),
        )?;

        let mut body_states = Vec::new();
        body_states
            .try_reserve_exact(self.body_order.len())
            .map_err(|_| ContactSolveFailure::CapacityExceeded {
                resource: "world step body candidates",
                limit: self.body_order.len(),
            })?;
        for body_id in &self.body_order {
            if let Some(state) = maybe_solution_body_state(&solutions, *body_id) {
                body_states.push((*body_id, state));
            }
        }

        let contact_count = solutions
            .iter()
            .map(|solution| solution.contact_impulses.len())
            .sum();
        let joint_count = solutions
            .iter()
            .map(|solution| solution.joint_impulses.len())
            .sum();
        let mut contact_impulses = Vec::new();
        contact_impulses
            .try_reserve_exact(contact_count)
            .map_err(|_| ContactSolveFailure::CapacityExceeded {
                resource: "world step contact impulses",
                limit: contact_count,
            })?;
        let mut joint_impulses = Vec::new();
        joint_impulses.try_reserve_exact(joint_count).map_err(|_| {
            ContactSolveFailure::CapacityExceeded {
                resource: "world step joint impulses",
                limit: joint_count,
            }
        })?;
        for solution in solutions {
            contact_impulses.extend(solution.contact_impulses);
            joint_impulses.extend(solution.joint_impulses);
        }

        let mut contact_solves = Vec::new();
        contact_solves
            .try_reserve_exact(contact_impulses.len())
            .map_err(|_| ContactSolveFailure::CapacityExceeded {
                resource: "world step contact reports",
                limit: contact_impulses.len(),
            })?;
        for contact in &contact_impulses {
            let maybe_solve = self
                .contact_manager
                .maybe_staged_solve(contact.contact_index, &contact.impulses);
            let Some(solve) = maybe_solve else {
                return Err(ContactSolveFailure::UnsupportedTopology);
            };
            contact_solves.push(solve);
        }

        let mut synchronizations = Vec::new();
        for (body_id, state) in &body_states {
            let body = self
                .bodies
                .get(*body_id)
                .map_err(|_| ContactSolveFailure::UnsupportedTopology)?;
            if state.snapshot().body_type() == BodyType::Static || !state.snapshot().is_active() {
                continue;
            }
            #[cfg(feature = "differential-internals")]
            if let Some(SolveFailureInjection::ProxyBounds { fixture }) = maybe_failure_injection
                && body.fixtures.contains(&fixture)
            {
                return Err(ContactSolveFailure::InvalidProxyBounds);
            }
            let prepared = self
                .prepare_body_synchronizations(
                    *body_id,
                    &body.fixtures,
                    body.state.transform(),
                    state.transform(),
                )
                .map_err(|_error| ContactSolveFailure::InvalidProxyBounds)?;
            synchronizations.extend(prepared);
        }

        Ok(WorldStepCandidate {
            body_states,
            contact_impulses,
            joint_impulses,
            contact_solves,
            synchronizations,
            timing,
        })
    }

    pub(super) fn commit_world_step_candidate<H: CollisionDecisionHook>(
        &mut self,
        candidate: WorldStepCandidate,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<Vec<ContactSolve>, StepError> {
        hook_run.ensure_lifecycle_capacity(candidate.contact_solves.len())?;
        for (body_id, state) in candidate.body_states {
            self.bodies
                .get_mut(body_id)
                .expect("staged island body remains live during commit")
                .state = state;
        }
        for contact in candidate.contact_impulses {
            self.contact_manager
                .commit_impulses(contact.contact_index, &contact.impulses);
        }
        for joint in candidate.joint_impulses {
            let record = self
                .joints
                .get_mut(joint.joint_id)
                .expect("staged island joint remains live during commit");
            record.runtime = joint.runtime;
        }
        for solve in &candidate.contact_solves {
            hook_run.record_discrete_solve(solve.clone())?;
        }
        self.apply_body_synchronizations(candidate.synchronizations);
        self.find_new_contacts_with_hook(hook_run)?;
        self.commit_step_timing(candidate.timing);
        Ok(candidate.contact_solves)
    }

    pub(in crate::world) fn preflight_contact_solver(&self) -> Result<(), ContactSolveFailure> {
        build_islands(
            &self.body_order,
            &self.bodies,
            &self.joints,
            &self.contact_manager,
            IslandLimits::REVIEWED,
        )
        .map(|_islands| ())
        .map_err(contact_solve_build_error)
    }

    #[cfg(test)]
    pub(in crate::world) fn set_body_solver_velocity_for_test(
        &mut self,
        body: BodyId,
        linear: Vec2,
        angular: f32,
    ) {
        self.bodies
            .get_mut(body)
            .expect("test body should remain live")
            .state
            .set_solver_motion(linear, angular);
    }

    #[cfg(test)]
    pub(in crate::world) fn body_solver_velocity_for_test(&self, body: BodyId) -> (Vec2, f32) {
        let state = self
            .bodies
            .get(body)
            .expect("test body should remain live")
            .state;
        (state.solver_linear(), state.solver_angular())
    }

    #[cfg(test)]
    pub(in crate::world) fn seed_first_contact_impulses_for_test(
        &mut self,
        normal: f32,
        tangent: f32,
    ) {
        self.contact_manager
            .seed_first_impulses_for_test(normal, tangent);
    }

    pub(in crate::world) fn destroy_contacts_for_body(&mut self, body: BodyId) {
        self.contact_manager
            .destroy_for_body(body, &mut self.bodies, &mut self.fixtures);
    }

    pub(in crate::world) fn destroy_contacts_for_fixture(&mut self, fixture: FixtureId) {
        self.contact_manager
            .destroy_for_fixture(fixture, &mut self.bodies, &mut self.fixtures);
    }

    pub(in crate::world) fn prepare_body_synchronizations(
        &self,
        body: BodyId,
        fixtures: &[FixtureId],
        previous: crate::math::Transform,
        current: crate::math::Transform,
    ) -> Result<Vec<(FixtureId, PreparedSynchronization)>, FixtureBoundsError> {
        fixtures
            .iter()
            .map(|fixture| {
                let record = self
                    .fixtures
                    .get(*fixture)
                    .expect("body fixture adjacency contains a live fixture");
                record
                    .proxies
                    .prepare_synchronization(
                        &self.broad_phase,
                        *fixture,
                        body,
                        record.definition.shape(),
                        previous,
                        current,
                    )
                    .map(|prepared| (*fixture, prepared))
            })
            .collect()
    }

    pub(in crate::world) fn apply_body_synchronizations(
        &mut self,
        synchronizations: Vec<(FixtureId, PreparedSynchronization)>,
    ) {
        for (fixture, prepared) in synchronizations {
            self.fixtures
                .get_mut(fixture)
                .expect("prepared fixture remains live during transform commit")
                .proxies
                .synchronize(&mut self.broad_phase, prepared);
        }
    }

    pub(super) fn prepare_body_fixture_creations(
        &self,
        fixtures: &[FixtureId],
        transform: crate::math::Transform,
    ) -> Result<Vec<(FixtureId, PreparedFixtureBounds)>, FixtureBoundsError> {
        fixtures
            .iter()
            .map(|fixture| {
                let record = self
                    .fixtures
                    .get(*fixture)
                    .expect("body fixture adjacency contains a live fixture");
                FixtureProxies::prepare_creation(record.definition.shape(), transform)
                    .map(|prepared| (*fixture, prepared))
            })
            .collect()
    }

    pub(super) fn create_body_fixture_entries(
        &mut self,
        body: BodyId,
        creations: Vec<(FixtureId, PreparedFixtureBounds)>,
    ) {
        for (fixture, prepared) in creations {
            self.create_fixture_entries(fixture, body, prepared);
        }
    }

    pub(super) fn create_fixture_entries(
        &mut self,
        fixture: FixtureId,
        body: BodyId,
        prepared: PreparedFixtureBounds,
    ) {
        let record = self
            .fixtures
            .get_mut(fixture)
            .expect("prepared fixture remains live during entry creation");
        record.proxies.create(
            &mut self.broad_phase,
            fixture,
            body,
            record.definition.filter_data(),
            prepared,
        );
    }

    pub(super) fn destroy_body_fixture_entries(&mut self, body: BodyId, fixtures: Vec<FixtureId>) {
        for fixture in fixtures {
            self.fixtures
                .get_mut(fixture)
                .expect("body fixture adjacency contains a live fixture")
                .proxies
                .destroy(&mut self.broad_phase, fixture, body);
        }
    }

    pub(in crate::world) fn body_mut_after_validation(&mut self, body: BodyId) -> &mut Body {
        self.bodies
            .get_mut(body)
            .expect("validated body remains live during one operation")
    }

    pub(super) fn debug_assert_body_order_invariant(&self) {
        debug_assert_eq!(self.body_order.len(), self.bodies.iter().count());
        debug_assert!(
            self.body_order
                .iter()
                .all(|body| self.bodies.get(*body).is_ok())
        );
        debug_assert!(self.body_order.iter().enumerate().all(|(index, body)| {
            self.body_order[index + 1..]
                .iter()
                .all(|candidate| candidate != body)
        }));
    }

    pub(in crate::world) fn system_mut_after_validation(
        &mut self,
        system: ParticleSystemId,
    ) -> &mut ParticleSystem {
        self.particle_systems
            .get_mut(system)
            .expect("validated particle system remains live during one operation")
    }
}

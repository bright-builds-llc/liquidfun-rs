use super::{
    BodyId, ContactSolveFailure, ContinuousCandidate, ContinuousEvent, ContinuousEventError,
    ContinuousWorldBackup, IslandBuildError, PreparedSynchronization, StepConfiguration, ToiIsland,
    ToiIslandLimits, ToiIslandSolution, World, solve_toi_island,
};

impl World {
    pub(in crate::world) fn solve_next_continuous_event(
        &mut self,
        configuration: StepConfiguration,
        limits: ToiIslandLimits,
        inject_after_solve: bool,
    ) -> Result<Option<ContinuousEvent>, ContinuousEventError> {
        let backup = self.backup_continuous_world()?;
        let result =
            self.prepare_and_commit_continuous_event(configuration, limits, inject_after_solve);
        if result.is_err() {
            self.restore_continuous_world(backup);
        }
        result
    }

    fn backup_continuous_world(&self) -> Result<ContinuousWorldBackup, ContinuousEventError> {
        const REVIEWED_MAX_TRANSACTION_BODIES: usize = 4_096;
        let body_count = self.bodies.iter().count();
        if body_count > REVIEWED_MAX_TRANSACTION_BODIES {
            return Err(ContinuousEventError::Island(
                IslandBuildError::CapacityExceeded {
                    resource: "continuous transaction bodies",
                    limit: REVIEWED_MAX_TRANSACTION_BODIES,
                },
            ));
        }
        let mut bodies = Vec::new();
        bodies.try_reserve_exact(body_count).map_err(|_| {
            ContinuousEventError::Island(IslandBuildError::CapacityExceeded {
                resource: "continuous transaction bodies",
                limit: REVIEWED_MAX_TRANSACTION_BODIES,
            })
        })?;
        for (body_id, body) in self.bodies.iter() {
            bodies.push((
                body_id,
                body.state,
                body.pending_contact_destruction,
                body.pending_wake,
            ));
        }
        Ok(ContinuousWorldBackup {
            bodies,
            contact_manager: self.contact_manager.clone(),
        })
    }

    fn restore_continuous_world(&mut self, backup: ContinuousWorldBackup) {
        for (body_id, state, pending_contact_destruction, pending_wake) in backup.bodies {
            let body = self
                .bodies
                .get_mut(body_id)
                .expect("continuous transaction body remains live before commit");
            body.state = state;
            body.pending_contact_destruction = pending_contact_destruction;
            body.pending_wake = pending_wake;
        }
        self.contact_manager = backup.contact_manager;
    }

    fn prepare_and_commit_continuous_event(
        &mut self,
        configuration: StepConfiguration,
        limits: ToiIslandLimits,
        inject_after_solve: bool,
    ) -> Result<Option<ContinuousEvent>, ContinuousEventError> {
        let Some(candidate) = self.select_continuous_candidate()? else {
            return Ok(None);
        };
        let island = self.build_toi_island(candidate, limits)?;
        let solution = solve_toi_island(
            &island,
            &self.contact_manager,
            &self.fixtures,
            configuration,
            candidate.alpha(),
        )?;
        let synchronizations = self.prepare_toi_synchronizations(&solution)?;
        if inject_after_solve {
            return Err(ContinuousEventError::InjectedFailure);
        }

        let contact_occurrences = island
            .contact_indices
            .iter()
            .map(|index| {
                self.contact_manager
                    .contacts()
                    .get(*index)
                    .map(|contact| contact.ordinal + 1)
                    .ok_or(ContinuousEventError::Island(IslandBuildError::InvalidGraph))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let transient_normal_impulse_sum = solution
            .contact_impulses
            .iter()
            .flat_map(|contact| contact.impulses.iter())
            .map(|(_feature, normal, _tangent)| *normal)
            .sum::<f32>();
        if !transient_normal_impulse_sum.is_finite() {
            return Err(ContinuousEventError::Solve(ContactSolveFailure::NonFinite));
        }
        let mut contact_solves = Vec::new();
        contact_solves
            .try_reserve_exact(solution.contact_impulses.len())
            .map_err(|_| {
                ContinuousEventError::Solve(ContactSolveFailure::CapacityExceeded {
                    resource: "continuous contact reports",
                    limit: solution.contact_impulses.len(),
                })
            })?;
        for contact in &solution.contact_impulses {
            let maybe_solve = self
                .contact_manager
                .maybe_staged_solve(contact.contact_index, &contact.impulses);
            let Some(solve) = maybe_solve else {
                return Err(ContinuousEventError::Solve(
                    ContactSolveFailure::UnsupportedTopology,
                ));
            };
            contact_solves.push(solve);
        }

        for (body_id, state) in solution
            .body_ids
            .iter()
            .copied()
            .zip(solution.body_states.iter().copied())
        {
            self.bodies
                .get_mut(body_id)
                .map_err(|_| ContinuousEventError::Island(IslandBuildError::InvalidGraph))?
                .state = state;
        }
        self.apply_body_synchronizations(synchronizations);
        for body_id in solution.body_ids.iter().copied() {
            let body = self
                .bodies
                .get(body_id)
                .map_err(|_| ContinuousEventError::Island(IslandBuildError::InvalidGraph))?;
            if body.state.snapshot().body_type() == super::super::body::BodyType::Dynamic {
                self.contact_manager.invalidate_toi_for_body(body_id);
            }
        }
        self.find_new_contacts();

        Ok(Some(ContinuousEvent {
            body_ids: solution.body_ids,
            contact_occurrences,
            transient_normal_impulse_sum,
            contact_solves,
        }))
    }

    fn build_toi_island(
        &mut self,
        candidate: ContinuousCandidate,
        limits: ToiIslandLimits,
    ) -> Result<ToiIsland, ContinuousEventError> {
        let [body_a, body_b] = candidate.bodies();
        let state_a = self
            .bodies
            .get(body_a)
            .map_err(|_| ContinuousEventError::Island(IslandBuildError::InvalidGraph))?
            .state;
        let state_b = self
            .bodies
            .get(body_b)
            .map_err(|_| ContinuousEventError::Island(IslandBuildError::InvalidGraph))?
            .state;
        let mut island = ToiIsland::new(
            &[(body_a, state_a), (body_b, state_b)],
            candidate.contact_index(),
            limits,
        )?;

        for body_id in [body_a, body_b] {
            let body = self
                .bodies
                .get(body_id)
                .map_err(|_| ContinuousEventError::Island(IslandBuildError::InvalidGraph))?;
            if body.state.snapshot().body_type() != super::super::body::BodyType::Dynamic {
                continue;
            }
            let adjacency = body.contacts.clone();
            let body_is_bullet = body.state.snapshot().is_bullet();
            for ordinal in adjacency {
                if !island.has_body_capacity() || !island.has_contact_capacity() {
                    break;
                }
                self.try_add_toi_contact(&mut island, candidate, body_id, body_is_bullet, ordinal)?;
            }
        }
        Ok(island)
    }

    fn try_add_toi_contact(
        &mut self,
        island: &mut ToiIsland,
        candidate: ContinuousCandidate,
        body_id: BodyId,
        body_is_bullet: bool,
        ordinal: u64,
    ) -> Result<(), ContinuousEventError> {
        let contact_index = self
            .contact_manager
            .contact_index_for_ordinal(ordinal)
            .ok_or(ContinuousEventError::Island(IslandBuildError::InvalidGraph))?;
        if island.contains_contact(contact_index) {
            return Ok(());
        }
        let contact = self
            .contact_manager
            .contacts()
            .get(contact_index)
            .ok_or(ContinuousEventError::Island(IslandBuildError::InvalidGraph))?;
        let other_id = contact
            .other_body(body_id)
            .ok_or(ContinuousEventError::Island(IslandBuildError::InvalidGraph))?;
        let other = self
            .bodies
            .get(other_id)
            .map_err(|_| ContinuousEventError::Island(IslandBuildError::InvalidGraph))?;
        if other.state.snapshot().body_type() == super::super::body::BodyType::Dynamic
            && !body_is_bullet
            && !other.state.snapshot().is_bullet()
        {
            return Ok(());
        }
        let fixture_a = self
            .fixtures
            .get(contact.key.first.fixture)
            .map_err(|_| ContinuousEventError::Island(IslandBuildError::InvalidGraph))?;
        let fixture_b = self
            .fixtures
            .get(contact.key.second.fixture)
            .map_err(|_| ContinuousEventError::Island(IslandBuildError::InvalidGraph))?;
        if fixture_a.definition.is_sensor() || fixture_b.definition.is_sensor() {
            return Ok(());
        }

        let other_was_present = island.contains_body(other_id);
        let maybe_backup = (!other_was_present).then_some(other.state);
        if let Some(backup) = maybe_backup {
            self.bodies
                .get_mut(other_id)
                .map_err(|_| ContinuousEventError::Island(IslandBuildError::InvalidGraph))?
                .state = backup.candidate_advance_to(candidate.alpha())?;
        }
        self.contact_manager
            .refresh_continuous_contact(contact_index, &mut self.bodies, &self.fixtures)
            .ok_or(ContinuousEventError::Island(IslandBuildError::InvalidGraph))?;
        let refreshed = self
            .contact_manager
            .contacts()
            .get(contact_index)
            .ok_or(ContinuousEventError::Island(IslandBuildError::InvalidGraph))?;
        if !refreshed.is_enabled() || !refreshed.is_touching() {
            if let Some(backup) = maybe_backup {
                self.bodies
                    .get_mut(other_id)
                    .map_err(|_| ContinuousEventError::Island(IslandBuildError::InvalidGraph))?
                    .state = backup;
            }
            return Ok(());
        }

        island.add_contact(contact_index)?;
        if other_was_present {
            return Ok(());
        }
        let other = self
            .bodies
            .get_mut(other_id)
            .map_err(|_| ContinuousEventError::Island(IslandBuildError::InvalidGraph))?;
        if other.state.snapshot().body_type() != super::super::body::BodyType::Static {
            other.state = other.state.candidate_set_awake(true);
            other.pending_wake = false;
        }
        island.add_body(other_id, other.state)?;
        Ok(())
    }

    fn prepare_toi_synchronizations(
        &self,
        solution: &ToiIslandSolution,
    ) -> Result<Vec<(crate::FixtureId, PreparedSynchronization)>, ContinuousEventError> {
        let mut synchronizations = Vec::new();
        for (body_id, state) in solution
            .body_ids
            .iter()
            .copied()
            .zip(solution.body_states.iter().copied())
        {
            let body = self
                .bodies
                .get(body_id)
                .map_err(|_| ContinuousEventError::Island(IslandBuildError::InvalidGraph))?;
            if state.snapshot().body_type() != super::super::body::BodyType::Dynamic {
                continue;
            }
            let previous = state.sweep().transform_at(0.0)?;
            synchronizations.extend(self.prepare_body_synchronizations(
                body_id,
                &body.fixtures,
                previous,
                state.transform(),
            )?);
        }
        Ok(synchronizations)
    }
}

use super::{
    BodyId, BodyMassData, BodyMassMutationError, BodyMassResetError, ChildIndex, CollisionError,
    CreateObjectError, FilterData, Fixture, FixtureDef, FixtureId, FixtureMutationError,
    FixtureProxies, HandleError, RayCastHit, RayCastInput, World, WorldFixtureSnapshot,
};
#[cfg(feature = "differential-internals")]
use super::{ContactTransition, IslandLimits, build_islands};
#[cfg(feature = "differential-internals")]
use crate::world::island::IslandBuildError;

impl World {
    /// Creates a fixture attached to `body` by cloning a checked definition.
    ///
    /// # Errors
    ///
    /// Returns an error if `body` is invalid or fixture storage is exhausted.
    pub fn create_fixture(
        &mut self,
        body: BodyId,
        definition: &FixtureDef,
    ) -> Result<FixtureId, CreateObjectError> {
        self.ensure_not_poisoned_for_handle()?;
        let body_record = self.bodies.get(body)?;
        let maybe_prepared = if body_record.state.snapshot().is_active() {
            Some(FixtureProxies::prepare_creation(
                definition.shape(),
                body_record.state.transform(),
            )?)
        } else {
            None
        };
        let maybe_mass_state = if definition.density() > 0.0 {
            let fixture_mass = definition
                .shape()
                .compute_mass(definition.density())
                .map_err(|_error| CreateObjectError::InvalidFixtureMass)?;
            Some(self.prepare_body_mass_state(body, Some(fixture_mass), None)?)
        } else {
            None
        };
        let diagnostic_id = self.allocate_diagnostic_id()?;
        let fixture = self.fixtures.insert(Fixture {
            diagnostic_id,
            body,
            definition: definition.clone(),
            proxies: FixtureProxies::new(),
            contacts: Vec::new(),
            pending_refilter: false,
        })?;
        if let Some(prepared) = maybe_prepared {
            self.create_fixture_entries(fixture, body, prepared);
        }
        self.body_mut_after_validation(body)
            .fixtures
            .insert(0, fixture);
        if let Some(mass_state) = maybe_mass_state {
            self.body_mut_after_validation(body).state = mass_state;
        }
        self.invalidate_continuous_for_body(body);
        Ok(fixture)
    }

    /// Returns owned semantic state for a live fixture.
    ///
    /// # Errors
    ///
    /// Returns a handle error when `fixture` is foreign, stale, or destroyed.
    pub fn fixture_snapshot(
        &self,
        fixture: FixtureId,
    ) -> Result<WorldFixtureSnapshot, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        self.fixtures.get(fixture).map(|record| {
            WorldFixtureSnapshot::from_definition(
                record.body,
                &record.definition,
                record.proxies.len(),
            )
        })
    }

    pub(in crate::world) fn ray_cast_fixture_child(
        &self,
        fixture: FixtureId,
        child_index: ChildIndex,
        input: RayCastInput,
    ) -> Result<Option<RayCastHit>, CollisionError> {
        let fixture_record = self
            .fixtures
            .get(fixture)
            .expect("broad-phase fixture identities must remain live");
        let body = self
            .bodies
            .get(fixture_record.body)
            .expect("live fixture owners must remain live");
        fixture_record
            .definition
            .shape()
            .ray_cast(input, body.state.transform(), child_index)
    }

    /// Returns the number of shape children currently stored for broad-phase discovery.
    #[must_use]
    pub fn broad_phase_entry_count(&self) -> usize {
        self.broad_phase.proxy_count()
    }

    /// Returns the number of private automatic contact occurrences.
    #[must_use]
    pub fn contact_count(&self) -> usize {
        self.contact_manager.len()
    }

    /// Copies one body's complete Phase 6 diagnostic state.
    #[cfg(feature = "differential-internals")]
    #[doc(hidden)]
    pub fn rigid_body_diagnostic(
        &self,
        body: BodyId,
    ) -> Result<crate::rigid_differential::RigidBodyDiagnostic, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        self.bodies.get(body).map(|record| {
            crate::rigid_differential::RigidBodyDiagnostic::new(
                record.state.snapshot(),
                record.state.solver_linear(),
                record.state.solver_angular(),
            )
        })
    }

    /// Copies current manager occurrences in exact manager order.
    #[cfg(feature = "differential-internals")]
    #[doc(hidden)]
    #[must_use]
    pub fn rigid_contact_diagnostics(
        &self,
    ) -> Vec<crate::rigid_differential::RigidContactDiagnostic> {
        self.contact_manager.rigid_diagnostics()
    }

    /// Copies body identities in exact pinned newest-first world-list order.
    #[cfg(feature = "differential-internals")]
    #[doc(hidden)]
    #[must_use]
    pub fn rigid_body_order_diagnostic(&self) -> Vec<BodyId> {
        self.body_order.clone()
    }

    /// Builds owned evidence from the reviewed bounded ephemeral island graph.
    #[cfg(feature = "differential-internals")]
    #[doc(hidden)]
    pub fn rigid_island_diagnostics(
        &self,
    ) -> Result<
        Vec<crate::rigid_differential::RigidIslandDiagnostic>,
        crate::rigid_differential::RigidIslandBuildError,
    > {
        self.rigid_island_diagnostics_for_limits(IslandLimits::REVIEWED)
    }

    /// Builds owned island evidence with smaller diagnostic-only capacity limits.
    #[cfg(feature = "differential-internals")]
    #[doc(hidden)]
    pub fn rigid_island_diagnostics_with_limits(
        &self,
        max_bodies: usize,
        max_contacts: usize,
    ) -> Result<
        Vec<crate::rigid_differential::RigidIslandDiagnostic>,
        crate::rigid_differential::RigidIslandBuildError,
    > {
        self.rigid_island_diagnostics_for_limits(IslandLimits::diagnostic(max_bodies, max_contacts))
    }

    #[cfg(feature = "differential-internals")]
    pub(super) fn rigid_island_diagnostics_for_limits(
        &self,
        limits: IslandLimits,
    ) -> Result<
        Vec<crate::rigid_differential::RigidIslandDiagnostic>,
        crate::rigid_differential::RigidIslandBuildError,
    > {
        let islands = build_islands(
            &self.body_order,
            &self.bodies,
            &self.joints,
            &self.contact_manager,
            limits,
        )
        .map_err(|error| match error {
            IslandBuildError::CapacityExceeded { resource, limit } => {
                crate::rigid_differential::RigidIslandBuildError::CapacityExceeded {
                    resource,
                    limit,
                }
            }
            IslandBuildError::InvalidGraph => {
                crate::rigid_differential::RigidIslandBuildError::InvalidGraph
            }
        })?;
        Ok(islands
            .into_iter()
            .map(|island| {
                let snapshots = island
                    .body_states
                    .iter()
                    .map(|state| state.snapshot())
                    .collect();
                let occurrences = island
                    .contact_indices
                    .iter()
                    .map(|index| self.contact_manager.contacts()[*index].ordinal + 1)
                    .collect();
                crate::rigid_differential::RigidIslandDiagnostic::new(
                    island.body_ids,
                    snapshots,
                    occurrences,
                    island.positions.len(),
                    island.velocities.len(),
                    island.joint_ids.len(),
                )
            })
            .collect())
    }

    /// Drains owned contact transitions produced outside [`World::step`].
    #[cfg(feature = "differential-internals")]
    #[doc(hidden)]
    pub fn rigid_drain_contact_transitions(&mut self) -> Vec<ContactTransition> {
        self.contact_manager.drain_transitions()
    }

    /// Recomputes a body's mass properties from its current fixtures in source list order.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `body` is foreign, stale, or destroyed.
    pub fn reset_body_mass_data(&mut self, body: BodyId) -> Result<(), BodyMassResetError> {
        self.ensure_not_poisoned_for_handle()?;
        self.bodies.get(body)?;
        let mass_state = self.prepare_body_mass_state(body, None, None)?;
        self.body_mut_after_validation(body).state = mass_state;
        self.invalidate_continuous_for_body(body);
        Ok(())
    }

    /// Replaces current mass properties on a dynamic body.
    ///
    /// Static and kinematic bodies accept this operation as a source-compatible no-op.
    ///
    /// # Errors
    ///
    /// Returns a typed handle or derived-mass error without mutation.
    pub fn set_body_mass_data(
        &mut self,
        body: BodyId,
        data: BodyMassData,
    ) -> Result<(), BodyMassMutationError> {
        self.ensure_not_poisoned_for_handle()?;
        let prepared = self.bodies.get(body)?.state.with_custom_mass_data(data)?;
        self.body_mut_after_validation(body).state = prepared;
        self.invalidate_continuous_for_body(body);
        Ok(())
    }

    /// Changes fixture density without implicitly recomputing its body's mass.
    ///
    /// # Errors
    ///
    /// Returns a typed handle or checked material error without mutation.
    pub fn set_fixture_density(
        &mut self,
        fixture: FixtureId,
        density: f32,
    ) -> Result<(), FixtureMutationError> {
        self.ensure_not_poisoned_for_handle()?;
        let record = self.fixtures.get(fixture)?;
        if density > 0.0 && record.definition.shape().compute_mass(density).is_err() {
            return Err(FixtureMutationError::InvalidDerivedMass);
        }
        self.fixtures
            .get_mut(fixture)?
            .definition
            .set_density(density)?;
        Ok(())
    }

    /// Changes the friction used when future contacts are created.
    ///
    /// # Errors
    ///
    /// Returns a typed handle or checked material error without mutation.
    pub fn set_fixture_friction(
        &mut self,
        fixture: FixtureId,
        friction: f32,
    ) -> Result<(), FixtureMutationError> {
        self.ensure_not_poisoned_for_handle()?;
        self.fixtures
            .get_mut(fixture)?
            .definition
            .set_friction(friction)?;
        Ok(())
    }

    /// Changes the restitution used when future contacts are created.
    ///
    /// # Errors
    ///
    /// Returns a typed handle or checked material error without mutation.
    pub fn set_fixture_restitution(
        &mut self,
        fixture: FixtureId,
        restitution: f32,
    ) -> Result<(), FixtureMutationError> {
        self.ensure_not_poisoned_for_handle()?;
        self.fixtures
            .get_mut(fixture)?
            .definition
            .set_restitution(restitution)?;
        Ok(())
    }

    /// Changes whether a fixture reports overlap without collision response.
    ///
    /// A changed sensor state records the owning body's pending wake side effect.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `fixture` is foreign, stale, or destroyed.
    pub fn set_fixture_sensor(
        &mut self,
        fixture: FixtureId,
        sensor: bool,
    ) -> Result<(), HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let record = self.fixtures.get(fixture)?;
        if record.definition.is_sensor() == sensor {
            return Ok(());
        }
        let body = record.body;
        self.fixtures
            .get_mut(fixture)?
            .definition
            .set_sensor(sensor);
        self.body_mut_after_validation(body).pending_wake = true;
        self.invalidate_continuous_for_body(body);
        Ok(())
    }

    /// Replaces collision filtering and touches every active broad-phase child.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `fixture` is foreign, stale, or destroyed.
    pub fn set_fixture_filter(
        &mut self,
        fixture: FixtureId,
        filter: FilterData,
    ) -> Result<(), HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let body = self.fixtures.get(fixture)?.body;
        self.set_fixture_filter_after_validation(fixture, filter);
        self.invalidate_continuous_for_body(body);
        Ok(())
    }
}

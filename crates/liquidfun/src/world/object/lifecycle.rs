use super::{
    AggregateMassError, ArenaInsertError, BodyControlError, BodyId, BodyState, CreateObjectError,
    DestroyedId, DestructionCause, DestructionRecord, DestructionReport, FilterData,
    FixtureDestructionError, FixtureId, HandleError, JointId, LifecycleEvent, MassData,
    MutationReport, ObjectSnapshot, ParticleBufferMode, ParticleBufferTeardown, ParticleGroupId,
    ParticleId, ParticleSystem, ParticleSystemDef, ParticleSystemDestructionTransaction,
    ParticleSystemId, World, WorldFixtureSnapshot, remove_occurrence,
};

impl World {
    /// Creates a particle system.
    ///
    /// # Errors
    ///
    /// Returns an arena error if particle-system storage is exhausted.
    pub fn create_particle_system(&mut self) -> Result<ParticleSystemId, ArenaInsertError> {
        self.create_particle_system_with_def(&ParticleSystemDef::default())
    }

    /// Creates a stable particle identity in `system` and optionally associates it with `group`.
    ///
    /// # Errors
    ///
    /// Returns an error if an owner is invalid, the group belongs to another system, or particle
    /// storage is exhausted.
    pub fn create_particle(
        &mut self,
        system: ParticleSystemId,
        maybe_group: Option<ParticleGroupId>,
    ) -> Result<crate::world::particle_object::ParticleCreationReceipt, CreateObjectError> {
        self.create_particle_with_def(system, maybe_group, &crate::ParticleDef::default())
    }

    /// Returns whether a body handle resolves in this world.
    #[must_use]
    pub fn contains_body(&self, body: BodyId) -> bool {
        self.bodies.get(body).is_ok()
    }

    /// Returns whether a fixture handle resolves in this world.
    #[must_use]
    pub fn contains_fixture(&self, fixture: FixtureId) -> bool {
        self.fixtures.get(fixture).is_ok()
    }

    /// Returns whether a joint handle resolves in this world.
    #[must_use]
    pub fn contains_joint(&self, joint: JointId) -> bool {
        self.joints.get(joint).is_ok()
    }

    /// Returns the number of live joints, including gear joints.
    #[must_use]
    pub fn joint_count(&self) -> usize {
        self.joints.iter().count()
    }

    /// Returns whether a particle-system handle resolves in this world.
    #[must_use]
    pub fn contains_particle_system(&self, system: ParticleSystemId) -> bool {
        self.particle_systems.get(system).is_ok()
    }

    /// Returns whether a particle-group handle resolves in this world.
    #[must_use]
    pub fn contains_particle_group(&self, group: ParticleGroupId) -> bool {
        self.particle_groups.get(group).is_ok()
    }

    /// Returns whether a particle handle resolves in this world.
    #[must_use]
    pub fn contains_particle(&self, particle: ParticleId) -> bool {
        self.particle_snapshot(particle).is_ok()
    }

    /// Destroys a body and all attached joints and fixtures.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `body` is foreign, stale, or destroyed.
    pub fn destroy_body(&mut self, body: BodyId) -> Result<DestructionReport, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let transition_checkpoint = self.contact_manager.transition_checkpoint();
        let root = self.bodies.get(body)?;
        let joints = root.joints.clone();
        let fixtures = root.fixtures.clone();
        let root_snapshot = ObjectSnapshot::Body {
            state: root.state.snapshot(),
            fixtures: fixtures.clone(),
            joints: joints.clone(),
        };
        let dependent_gears = self.collect_body_gear_dependents(&joints);
        let mut records =
            Vec::with_capacity(dependent_gears.len() + joints.len() + fixtures.len() + 1);
        let mut lifecycle = Vec::with_capacity(records.capacity() + root.contacts.len());

        for (gear, source) in &dependent_gears {
            let record = self.remove_joint(
                *gear,
                DestructionCause::GearDependencyCascade { source: *source },
            );
            lifecycle.push(LifecycleEvent::JointGoodbye(record.clone()));
            records.push(record);
        }

        for joint in joints {
            if dependent_gears.iter().any(|(gear, _source)| *gear == joint) {
                continue;
            }
            let record = self.remove_joint(joint, DestructionCause::BodyCascade { body });
            lifecycle.push(LifecycleEvent::JointGoodbye(record.clone()));
            records.push(record);
        }
        self.destroy_contacts_for_body(body);
        lifecycle.extend(
            self.contact_manager
                .drain_transitions_since(transition_checkpoint)
                .into_iter()
                .map(LifecycleEvent::ContactDestruction),
        );
        for fixture in fixtures {
            let record = self.remove_fixture(fixture, DestructionCause::BodyCascade { body }, None);
            lifecycle.extend(
                self.contact_manager
                    .drain_transitions()
                    .into_iter()
                    .map(LifecycleEvent::ContactDestruction),
            );
            lifecycle.push(LifecycleEvent::FixtureGoodbye(record.clone()));
            records.push(record);
        }
        let root = self.remove_body(body, DestructionCause::Explicit, root_snapshot);
        lifecycle.push(LifecycleEvent::Destruction(root.clone()));
        records.push(root);
        Ok(MutationReport::new(records, lifecycle))
    }

    /// Destroys one fixture after validating it before mutation.
    ///
    /// # Errors
    ///
    /// Returns [`FixtureDestructionError::InvalidHandle`] when `fixture` is foreign, stale, or
    /// destroyed, or [`FixtureDestructionError::InvalidAggregateMass`] when the complete
    /// remaining-fixture aggregate is invalid. Either failure leaves contacts, proxies, fixture
    /// storage, adjacency, and body mass state unchanged.
    pub fn destroy_fixture(
        &mut self,
        fixture: FixtureId,
    ) -> Result<DestructionReport, FixtureDestructionError> {
        self.ensure_not_poisoned_for_handle()?;
        let transition_checkpoint = self.contact_manager.transition_checkpoint();
        let body = self.fixtures.get(fixture)?.body;
        let candidate = self.prepare_body_mass_state(body, None, Some(fixture))?;
        let record = self.remove_fixture(fixture, DestructionCause::Explicit, Some(candidate));
        let mut lifecycle = self
            .contact_manager
            .drain_transitions_since(transition_checkpoint)
            .into_iter()
            .map(LifecycleEvent::ContactDestruction)
            .collect::<Vec<_>>();
        lifecycle.push(LifecycleEvent::Destruction(record.clone()));
        Ok(MutationReport::new(vec![record], lifecycle))
    }

    /// Destroys a particle system and all its groups and particles.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `system` is foreign, stale, or destroyed.
    pub fn destroy_particle_system(
        &mut self,
        system: ParticleSystemId,
    ) -> Result<DestructionReport, HandleError> {
        self.destroy_particle_system_owned(system)
            .map(|(records, _removed)| records)
    }

    /// Destroys a particle system and returns its complete owned lane bundle.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `system` is foreign, stale, or destroyed.
    pub fn destroy_particle_system_with_buffers(
        &mut self,
        system: ParticleSystemId,
    ) -> Result<ParticleBufferTeardown, HandleError> {
        let (report, removed) = self.destroy_particle_system_owned(system)?;
        let records = report.into_value();
        let capacity = removed.definition.capacity();
        let mode = if capacity.is_fixed() {
            ParticleBufferMode::Fixed {
                capacity: capacity.count(),
            }
        } else {
            ParticleBufferMode::Growable {
                initial_capacity: capacity.count(),
            }
        };
        let bundle = removed.storage.into_buffer_bundle(mode);
        Ok(ParticleBufferTeardown::new(records, bundle))
    }

    pub(super) fn destroy_particle_system_owned(
        &mut self,
        system: ParticleSystemId,
    ) -> Result<(DestructionReport, ParticleSystem), HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let transaction = self.capture_particle_system_destruction(system)?;
        let mut records =
            Vec::with_capacity(transaction.groups.len() + transaction.particles.len() + 1);
        let mut lifecycle = Vec::new();

        for group in transaction.groups {
            records.push(
                self.remove_particle_group(
                    group,
                    DestructionCause::ParticleSystemCascade { system },
                ),
            );
        }
        for snapshot in transaction.particles {
            let requested = snapshot
                .input
                .flags
                .contains(crate::ParticleFlags::DESTRUCTION_LISTENER);
            let record = Self::particle_destruction_record(
                snapshot,
                DestructionCause::ParticleSystemCascade { system },
            );
            if requested {
                lifecycle.push(LifecycleEvent::ParticleDestruction(record.clone()));
            }
            records.push(record);
        }
        let (root_record, removed) = self.remove_particle_system(
            system,
            DestructionCause::Explicit,
            transaction.root_snapshot,
        );
        lifecycle.push(LifecycleEvent::Destruction(root_record.clone()));
        records.push(root_record);
        Ok((MutationReport::new(records, lifecycle), removed))
    }

    /// Destroys a particle group without destroying its particles.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `group` is foreign, stale, or destroyed.
    pub fn destroy_particle_group(
        &mut self,
        group: ParticleGroupId,
    ) -> Result<DestructionRecord, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        self.particle_groups.get(group)?;
        Ok(self.remove_particle_group(group, DestructionCause::Explicit))
    }

    /// Destroys one stable particle identity.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `particle` is foreign, stale, or destroyed.
    pub fn destroy_particle(
        &mut self,
        particle: ParticleId,
    ) -> Result<DestructionRecord, HandleError> {
        self.destroy_particle_now(particle)
    }

    pub(super) fn capture_particle_system_destruction(
        &self,
        system: ParticleSystemId,
    ) -> Result<ParticleSystemDestructionTransaction, HandleError> {
        let root = self.particle_systems.get(system)?;
        let groups = root.groups.clone();
        let particles = root.storage.snapshots();
        let particle_ids = particles.iter().map(|snapshot| snapshot.id).collect();
        let root_snapshot = ObjectSnapshot::ParticleSystem {
            groups: groups.clone(),
            particles: particle_ids,
        };

        Ok(ParticleSystemDestructionTransaction {
            groups,
            particles,
            root_snapshot,
        })
    }

    pub(in crate::world) fn ensure_not_poisoned_for_handle(&self) -> Result<(), HandleError> {
        if self.step_state.is_poisoned() {
            return Err(HandleError::WorldPoisoned);
        }
        Ok(())
    }

    pub(super) fn update_body_state(
        &mut self,
        body: BodyId,
        prepare: impl FnOnce(BodyState) -> Result<BodyState, BodyControlError>,
    ) -> Result<(), BodyControlError> {
        self.ensure_not_poisoned_for_handle()?;
        let state = self.bodies.get(body)?.state;
        let candidate = prepare(state)?;
        self.body_mut_after_validation(body).state = candidate;
        self.invalidate_continuous_for_body(body);
        Ok(())
    }

    pub(super) fn invalidate_continuous_for_body(&mut self, body: BodyId) {
        self.continuous_step_state.invalidate();
        self.contact_manager.invalidate_toi_for_body(body);
    }

    pub(in crate::world) fn clear_force_accumulators(&mut self) {
        for body in self.bodies.values_mut() {
            body.state.clear_accumulated_forces();
        }
    }

    pub(in crate::world) fn ensure_not_poisoned_for_insert(&self) -> Result<(), ArenaInsertError> {
        if self.step_state.is_poisoned() {
            return Err(ArenaInsertError::WorldPoisoned);
        }
        Ok(())
    }

    pub(super) fn remove_body(
        &mut self,
        body: BodyId,
        cause: DestructionCause,
        snapshot: ObjectSnapshot,
    ) -> DestructionRecord {
        let removed = self
            .bodies
            .remove(body)
            .expect("validated destruction root and adjacency remain live");
        remove_occurrence(&mut self.body_order, &body);
        self.debug_assert_body_order_invariant();
        DestructionRecord {
            destroyed: DestroyedId::Body(body),
            diagnostic_id: removed.diagnostic_id,
            cause,
            snapshot,
        }
    }

    pub(super) fn remove_fixture(
        &mut self,
        fixture: FixtureId,
        cause: DestructionCause,
        maybe_mass_state: Option<BodyState>,
    ) -> DestructionRecord {
        self.destroy_contacts_for_fixture(fixture);
        let record = self
            .fixtures
            .get_mut(fixture)
            .expect("validated fixture adjacency remains live");
        let broad_phase_entry_count = record.proxies.len();
        record
            .proxies
            .destroy(&mut self.broad_phase, fixture, record.body);
        let removed = self
            .fixtures
            .remove(fixture)
            .expect("validated fixture adjacency remains live");
        remove_occurrence(
            &mut self.body_mut_after_validation(removed.body).fixtures,
            &fixture,
        );
        if let Some(mass_state) = maybe_mass_state {
            self.body_mut_after_validation(removed.body).state = mass_state;
        }
        self.invalidate_continuous_for_body(removed.body);
        DestructionRecord {
            destroyed: DestroyedId::Fixture(fixture),
            diagnostic_id: removed.diagnostic_id,
            cause,
            snapshot: ObjectSnapshot::Fixture {
                body: removed.body,
                state: WorldFixtureSnapshot::from_definition(
                    removed.body,
                    &removed.definition,
                    broad_phase_entry_count,
                ),
            },
        }
    }

    pub(in crate::world) fn remove_joint(
        &mut self,
        joint: JointId,
        cause: DestructionCause,
    ) -> DestructionRecord {
        let removed = self
            .joints
            .remove(joint)
            .expect("validated joint adjacency remains live");
        if let crate::JointDef::Gear(definition) = removed.definition {
            for dependency in definition.source_joints() {
                remove_occurrence(
                    &mut self
                        .joints
                        .get_mut(dependency)
                        .expect("live gear sources remain valid until dependent removal")
                        .reverse_gear_dependents,
                    &joint,
                );
            }
        }
        remove_occurrence(
            &mut self.body_mut_after_validation(removed.bodies[0]).joints,
            &joint,
        );
        if removed.bodies[1] != removed.bodies[0] {
            remove_occurrence(
                &mut self.body_mut_after_validation(removed.bodies[1]).joints,
                &joint,
            );
        }
        DestructionRecord {
            destroyed: DestroyedId::Joint(joint),
            diagnostic_id: removed.diagnostic_id,
            cause,
            snapshot: ObjectSnapshot::Joint {
                bodies: removed.bodies,
                maybe_gear_dependencies: match removed.definition {
                    crate::JointDef::Gear(definition) => Some(definition.source_joints()),
                    _ => None,
                },
            },
        }
    }

    pub(super) fn collect_body_gear_dependents(
        &self,
        joints: &[JointId],
    ) -> Vec<(JointId, JointId)> {
        let mut dependents = Vec::new();
        for source in joints {
            let record = self
                .joints
                .get(*source)
                .expect("body joint adjacency contains only live joints");
            for gear in &record.reverse_gear_dependents {
                if dependents
                    .iter()
                    .any(|(existing, _source)| existing == gear)
                {
                    continue;
                }
                dependents.push((*gear, *source));
            }
        }
        dependents
    }

    pub(super) fn remove_particle_system(
        &mut self,
        system: ParticleSystemId,
        cause: DestructionCause,
        snapshot: ObjectSnapshot,
    ) -> (DestructionRecord, ParticleSystem) {
        let removed = self
            .particle_systems
            .remove(system)
            .expect("validated destruction root and adjacency remain live");
        remove_occurrence(&mut self.particle_system_order, &system);
        (
            DestructionRecord {
                destroyed: DestroyedId::ParticleSystem(system),
                diagnostic_id: removed.diagnostic_id,
                cause,
                snapshot,
            },
            removed,
        )
    }

    pub(super) fn remove_particle_group(
        &mut self,
        group: ParticleGroupId,
        cause: DestructionCause,
    ) -> DestructionRecord {
        let system = self
            .particle_groups
            .get(group)
            .expect("validated particle-group adjacency remains live")
            .system;
        let particles = self
            .system_mut_after_validation(system)
            .storage
            .clear_group(group)
            .expect("validated particle storage remains coherent during group removal");
        let removed = self
            .particle_groups
            .remove(group)
            .expect("validated particle-group adjacency remains live");
        remove_occurrence(
            &mut self.system_mut_after_validation(removed.system).groups,
            &group,
        );
        DestructionRecord {
            destroyed: DestroyedId::ParticleGroup(group),
            diagnostic_id: removed.diagnostic_id,
            cause,
            snapshot: ObjectSnapshot::ParticleGroup {
                system: removed.system,
                particles,
            },
        }
    }

    pub(super) fn touch_body_fixture_entries(&mut self, body: BodyId, fixtures: &[FixtureId]) {
        for fixture in fixtures {
            let record = self
                .fixtures
                .get(*fixture)
                .expect("body fixture adjacency contains a live fixture");
            record.proxies.touch(&mut self.broad_phase, *fixture, body);
        }
    }

    pub(super) fn prepare_body_mass_state(
        &self,
        body: BodyId,
        maybe_candidate: Option<MassData>,
        maybe_excluded_fixture: Option<FixtureId>,
    ) -> Result<BodyState, AggregateMassError> {
        let fixture_mass_data =
            self.collect_fixture_mass_data(body, maybe_candidate, maybe_excluded_fixture);
        self.bodies
            .get(body)
            .expect("validated body remains live during mass reset")
            .state
            .with_reset_mass_data(&fixture_mass_data)
    }

    pub(super) fn collect_fixture_mass_data(
        &self,
        body: BodyId,
        maybe_candidate: Option<MassData>,
        maybe_excluded_fixture: Option<FixtureId>,
    ) -> Vec<MassData> {
        let fixture_ids = &self
            .bodies
            .get(body)
            .expect("validated body remains live during mass collection")
            .fixtures;
        let mut fixture_mass_data =
            Vec::with_capacity(fixture_ids.len() + usize::from(maybe_candidate.is_some()));
        if let Some(candidate) = maybe_candidate {
            fixture_mass_data.push(candidate);
        }
        fixture_mass_data.extend(fixture_ids.iter().filter_map(|fixture| {
            if Some(*fixture) == maybe_excluded_fixture {
                return None;
            }
            let definition = &self
                .fixtures
                .get(*fixture)
                .expect("body fixture adjacency contains a live fixture")
                .definition;
            if definition.density() == 0.0 {
                return None;
            }
            Some(
                definition
                    .shape()
                    .compute_mass(definition.density())
                    .expect("checked fixture shape and density produce valid mass data"),
            )
        }));
        fixture_mass_data
    }

    pub(super) fn set_fixture_filter_after_validation(
        &mut self,
        fixture: FixtureId,
        filter: FilterData,
    ) {
        {
            let record = self
                .fixtures
                .get_mut(fixture)
                .expect("validated fixture remains live during refilter");
            record.definition.set_filter_data(filter);
            record.pending_refilter = true;
            record
                .proxies
                .set_filter(&mut self.broad_phase, fixture, record.body, filter);
        }
        self.contact_manager.flag_fixture_for_filtering(fixture);
    }
}

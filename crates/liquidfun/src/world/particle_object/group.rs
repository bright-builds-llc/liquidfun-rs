use super::{
    ArenaInsertError, AssociationMap, CreateObjectError, DestructionRecord, GROUP_SAMPLE_LIMIT,
    GROUP_SAMPLING_WORK_LIMIT, GroupPlan, GroupPlanInput, HandleError, MutationReport,
    ParticleGroup, ParticleGroupCreationPlan, ParticleGroupDestination, ParticleGroupId,
    ParticleGroupMutationError, ParticleGroupRecipe, ParticleGroupView, ParticleSystemId,
    SamplingLimits, World, append_group_particle, group_mutation, group_plan_creation_error,
    group_sampling_creation_error, group_topology_limits, plan_samples, refresh_candidate_contacts,
    sampling_capacity, settings, storage_handle_error,
};

impl World {
    /// Creates a complete particle group from one checked recipe.
    ///
    /// Sampling, particle identities, lifecycle capacity, contacts, and topology
    /// are prepared in owned storage before the world publishes the group shell.
    ///
    /// # Errors
    ///
    /// Returns a typed no-effect error for a locked or poisoned world, an
    /// invalid owner or append target, exhausted capacity or identity space,
    /// invalid sampling output, or invalid topology.
    pub fn create_particle_group(
        &mut self,
        system: ParticleSystemId,
        recipe: &ParticleGroupRecipe<()>,
    ) -> Result<ParticleGroupId, CreateObjectError> {
        let plan = self.plan_particle_group(system, recipe)?;
        Ok(self.commit_particle_group(plan))
    }

    /// Creates a complete particle group and atomically installs its application association.
    ///
    /// A `New` recipe installs its carried association under the returned group
    /// identity. In pinned `AppendTo` semantics, the temporary group's
    /// association is discarded when that hidden group joins the target.
    ///
    /// # Errors
    ///
    /// Returns the same no-effect errors as [`Self::create_particle_group`],
    /// plus association-table reservation failure.
    pub fn create_particle_group_with_association<UserAssociation>(
        &mut self,
        system: ParticleSystemId,
        recipe: ParticleGroupRecipe<UserAssociation>,
        associations: &mut AssociationMap<ParticleGroupId, UserAssociation>,
    ) -> Result<ParticleGroupId, CreateObjectError> {
        let installs_association = matches!(recipe.destination(), ParticleGroupDestination::New)
            && recipe.maybe_user_association().is_some();
        if installs_association {
            associations
                .try_reserve_one()
                .map_err(|()| CreateObjectError::AssociationCapacityExceeded)?;
        }
        let plan = self.plan_particle_group(system, &recipe)?;
        let group = self.commit_particle_group(plan);
        let maybe_association = recipe.into_user_association();
        if installs_association && let Some(association) = maybe_association {
            let replaced = associations.insert(group, association);
            debug_assert!(replaced.is_none());
        }
        Ok(group)
    }

    /// Borrows complete semantic state for one live particle group.
    ///
    /// # Errors
    ///
    /// Returns a scoped error when the group or its owning system is foreign,
    /// stale, destroyed, or internally inconsistent.
    pub fn particle_group_view(
        &self,
        group: ParticleGroupId,
    ) -> Result<ParticleGroupView<'_>, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let shell = self.particle_groups.get(group)?;
        let system = self.particle_systems.get(shell.system)?;
        let diameter = 2.0 * system.definition.radius();
        let particle_mass =
            system.definition.density() * (settings::PARTICLE_STRIDE * diameter).powi(2);
        system
            .storage
            .group_view(group, particle_mass)
            .map_err(storage_handle_error)
    }

    /// Joins two live groups from the same particle system.
    ///
    /// The first identity survives and is returned. The second identity becomes
    /// stale only after the exact storage candidate and shell removal have both
    /// been preflighted.
    ///
    /// # Errors
    ///
    /// Returns a typed no-effect error for locked or poisoned worlds, invalid
    /// handles, cross-system groups, capacity exhaustion, or invalid topology.
    pub fn join_particle_groups(
        &mut self,
        group_a: ParticleGroupId,
        group_b: ParticleGroupId,
    ) -> Result<MutationReport<ParticleGroupId>, ParticleGroupMutationError> {
        group_mutation::plan_join(self, group_a, group_b)
    }

    /// Splits one group into its source-ordered connected components.
    ///
    /// The original identity is always first. Later component identities are
    /// allocated and returned in source component order.
    ///
    /// # Errors
    ///
    /// Returns a typed no-effect error for locked or poisoned worlds, an
    /// invalid handle, exhausted identity capacity, or invalid connectivity.
    pub fn split_particle_group(
        &mut self,
        group: ParticleGroupId,
    ) -> Result<Vec<ParticleGroupId>, ParticleGroupMutationError> {
        group_mutation::plan_split(self, group)
    }

    /// Splits one group and copies its application association to every new component.
    ///
    /// The source association, when present, remains under the original group.
    /// All value clones and side-table capacity are prepared before the world
    /// publishes any new group identity.
    ///
    /// # Errors
    ///
    /// Returns the same no-effect errors as [`Self::split_particle_group`],
    /// plus [`ParticleGroupMutationError::AssociationCapacityExceeded`] when
    /// the side table cannot reserve every new entry.
    pub fn split_particle_group_with_association<UserAssociation: Clone>(
        &mut self,
        group: ParticleGroupId,
        associations: &mut AssociationMap<ParticleGroupId, UserAssociation>,
    ) -> Result<Vec<ParticleGroupId>, ParticleGroupMutationError> {
        group_mutation::plan_split_with_association(self, group, associations)
    }

    /// Replaces the public behavior flags for one live particle group.
    ///
    /// [`crate::particle::ParticleGroupFlags`] removes upstream-private bits at construction;
    /// this operation accepts only that invariant-bearing public value.
    ///
    /// # Errors
    ///
    /// Returns a typed no-effect error for locked or poisoned worlds, an
    /// invalid handle, or an invalid storage candidate.
    pub fn set_particle_group_flags(
        &mut self,
        group: ParticleGroupId,
        flags: crate::particle::ParticleGroupFlags,
    ) -> Result<(), ParticleGroupMutationError> {
        group_mutation::set_flags(self, group, flags)
    }

    /// Explicitly destroys a retained empty particle-group shell.
    ///
    /// Groups with members must use the particle lifecycle or an explicit join;
    /// this method never silently ungroups live particles.
    ///
    /// # Errors
    ///
    /// Returns [`ParticleGroupMutationError::GroupNotEmpty`] without effects
    /// when the group still owns particles, plus the ordinary locked, poisoned,
    /// and handle errors.
    pub fn destroy_empty_particle_group(
        &mut self,
        group: ParticleGroupId,
    ) -> Result<DestructionRecord, ParticleGroupMutationError> {
        group_mutation::destroy_empty(self, group)
    }

    pub(super) fn plan_particle_group<UserAssociation>(
        &self,
        system: ParticleSystemId,
        recipe: &ParticleGroupRecipe<UserAssociation>,
    ) -> Result<ParticleGroupCreationPlan, CreateObjectError> {
        self.ensure_not_poisoned_for_handle()?;
        if self.step_state.is_locked() {
            return Err(CreateObjectError::WorldLocked);
        }
        let source_system = self.particle_systems.get(system)?;
        let maybe_append_target = match recipe.destination() {
            ParticleGroupDestination::New => None,
            ParticleGroupDestination::AppendTo(target) => {
                let target_shell = self.particle_groups.get(target)?;
                if target_shell.system != system {
                    return Err(CreateObjectError::InvalidHandle(
                        HandleError::WrongParticleSystem,
                    ));
                }
                Some(target)
            }
        };
        let temporary_group = self.particle_groups.next_handle()?;
        let maximum_samples = sampling_capacity(source_system).min(GROUP_SAMPLE_LIMIT);
        let samples = plan_samples(
            recipe,
            settings::PARTICLE_STRIDE * 2.0 * source_system.definition.radius(),
            SamplingLimits::new(GROUP_SAMPLING_WORK_LIMIT, maximum_samples),
        )
        .map_err(group_sampling_creation_error)?
        .into_samples();
        let creates_shell = maybe_append_target.is_none();
        let diagnostic_count = samples
            .len()
            .checked_add(usize::from(creates_shell))
            .ok_or(ArenaInsertError::DiagnosticIdExhausted)?;
        let (first_diagnostic_id, next_diagnostic_id) =
            self.preflight_diagnostic_ids(diagnostic_count)?;
        let particle_diagnostic_start = first_diagnostic_id + u64::from(creates_shell);

        let mut system_candidate = source_system.clone();
        for (ordinal, sample) in samples.iter().copied().enumerate() {
            append_group_particle(
                &mut system_candidate,
                recipe,
                temporary_group,
                sample.position(),
                sample.velocity(),
                particle_diagnostic_start
                    + u64::try_from(ordinal)
                        .map_err(|_error| ArenaInsertError::DiagnosticIdExhausted)?,
            )?;
        }
        refresh_candidate_contacts(&mut system_candidate)?;
        let topology: GroupPlan = system_candidate
            .storage
            .plan_group(GroupPlanInput {
                group: temporary_group,
                maybe_append_target,
                flags: recipe.group_flags(),
                strength: recipe.strength(),
                transform: recipe.transform(),
                particle_diameter: 2.0 * system_candidate.definition.radius(),
                voronoi_limits: group_topology_limits(),
            })
            .map_err(group_plan_creation_error)?;
        let result_group = topology.result_group();
        topology.commit_group(&mut system_candidate.storage);
        let maybe_shell = creates_shell.then_some((temporary_group, first_diagnostic_id));
        if creates_shell {
            system_candidate.groups.push(temporary_group);
        }
        Ok(ParticleGroupCreationPlan {
            system,
            system_candidate,
            result_group,
            maybe_shell,
            next_diagnostic_id,
        })
    }

    pub(super) fn commit_particle_group(
        &mut self,
        plan: ParticleGroupCreationPlan,
    ) -> ParticleGroupId {
        if let Some((group, diagnostic_id)) = plan.maybe_shell {
            let inserted = self
                .particle_groups
                .insert(ParticleGroup {
                    diagnostic_id,
                    system: plan.system,
                })
                .expect("preflighted particle-group shell remains available until commit");
            debug_assert_eq!(inserted, group);
        }
        *self
            .particle_systems
            .get_mut(plan.system)
            .expect("validated particle system remains live until immediate commit") =
            plan.system_candidate;
        self.commit_next_diagnostic_id(plan.next_diagnostic_id);
        plan.result_group
    }
}
